use std::{future::Future, marker::PhantomData, sync::Arc};

use amqprs::{channel::BasicPublishArguments, BasicProperties};
use broker::{
    rabbit::{RabbitAdapter, RabbitPublisher},
    BrokerAdapter,
};
use futures_util::future::BoxFuture;

use serde::Deserialize;
use shared_essential::presentation::dto::response_request::{ApiResponse, Message};

use super::{
    basic::{BasicConsumerHandler, BasicRabbitConsumer},
    ConsumerServer, Error, Result,
};
use std::result::Result as StdResult;

/// Результат работы confirm-обработчика: `ApiResponse<(), ()>` при успехе или типизированная ошибка.
///
/// `ApiResponse<(), ()>` выбран намеренно: confirm-паттерн сообщает только факт обработки,
/// а не её результат. Ошибка `E` трансформируется в `Message::stop` и тоже отправляется
/// как `ApiResponse`, чтобы отправитель мог понять, что пошло не так.
pub type HandlerResult<E> = StdResult<ApiResponse<(), ()>, E>;

/// Консьюмер с подтверждением: принимает задание, выполняет его и отправляет квитанцию.
///
/// Реализует надёжную однонаправленную доставку: отправитель (паблишер) узнаёт,
/// что получатель обработал сообщение, через ответную очередь в `reply_to`.
/// `correlation_id` позволяет паблишеру сопоставить квитанцию с конкретным запросом.
///
/// Текущее ограничение: ack к брокеру отправляется автоматически (auto-ack),
/// ручной ack не работает -- при падении обработчика между получением и ack'ом
/// сообщение будет потеряно.
pub struct RabbitConsumerWithConfirmation<H, T, E> {
    adapter: Arc<RabbitAdapter>,
    queue: String,
    consumer_tag: String,
    handler: H,
    _phantom_data: PhantomData<(T, E)>,
}

impl<H, T, E> RabbitConsumerWithConfirmation<H, T, E>
where
    H: ConfirmingHandler<T, E> + Send + Sync + 'static,
    T: for<'de> Deserialize<'de> + std::fmt::Debug + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn new(
        adapter: Arc<RabbitAdapter>,
        queue: &str,
        consumer_tag: &str,
        handler: H,
    ) -> Self {
        RabbitConsumerWithConfirmation {
            adapter,
            queue: queue.to_owned(),
            consumer_tag: consumer_tag.to_owned(),
            handler,
            _phantom_data: PhantomData::default(),
        }
    }

    /// Регистрирует паблишер для квитанций и запускает цикл потребления.
    ///
    /// Паблишер регистрируется с пустым routing_key: конкретная очередь для квитанции
    /// берётся из `reply_to` каждого входящего сообщения.
    pub async fn run(self) -> Result<ConsumerServer> {
        let RabbitConsumerWithConfirmation {
            adapter,
            queue,
            consumer_tag,
            handler,
            ..
        } = self;

        tracing::info!("registering confirmation publisher");
        let basic_props = BasicProperties::default();
        let publish_args = BasicPublishArguments::new("", "");
        let publisher =
            adapter.register_publisher(basic_props, publish_args).await?;

        let basic_handler = ConfirmationHandler {
            handler: handler.into(),
            publisher: publisher.into(),
            _phantom_data: PhantomData::default(),
        };

        let basic_consumer =
            BasicRabbitConsumer::new(adapter, &queue, &consumer_tag, basic_handler);
        basic_consumer.run().await
    }
}

/// Обработчик данных для confirm-консьюмера.
///
/// `request_id` -- это `correlation_id` из входящего сообщения; передаётся обработчику,
/// чтобы он мог использовать его для идемпотентности или логирования.
pub trait ConfirmingHandler<T, E> {
    fn handle(
        &self,
        data: T,
        request_id: String,
    ) -> BoxFuture<'static, HandlerResult<E>>;
}

impl<T, E, F, Fut> ConfirmingHandler<T, E> for F
where
    F: Fn(T, String) -> Fut,
    Fut: Future<Output = HandlerResult<E>> + Send + 'static,
{
    fn handle(
        &self,
        data: T,
        request_id: String,
    ) -> BoxFuture<'static, HandlerResult<E>> {
        Box::pin(self(data, request_id))
    }
}

/// Адаптер между [`BasicConsumerHandler`] и [`ConfirmingHandler`].
///
/// Извлекает `reply_to` и `correlation_id` из входящего сообщения, вызывает
/// пользовательский обработчик и отправляет квитанцию с тем же `correlation_id`.
struct ConfirmationHandler<H, T, E> {
    handler: Arc<H>,
    publisher: Arc<RabbitPublisher>,
    _phantom_data: PhantomData<(T, E)>,
}

impl<H, T, E> BasicConsumerHandler<T> for ConfirmationHandler<H, T, E>
where
    H: ConfirmingHandler<T, E> + Send + Sync + 'static,
    T: Send + Sync + 'static,
    E: std::error::Error,
{
    fn handle(
        &self,
        data: Result<T>,
        properties: BasicProperties,
    ) -> BoxFuture<'static, Result<()>> {
        let handler = self.handler.clone();
        let publisher = self.publisher.clone();

        Box::pin(async move {
            // confirm_queue -- имя очереди, куда отправить квитанцию об обработке.
            let Some(confirm_queue) = properties.reply_to() else {
                return Err(Error::NoReplyTo);
            };
            // request_id дублируется в ответе через correlation_id, чтобы паблишер
            // мог сопоставить квитанцию с отправленным сообщением.
            let Some(request_id) = properties.correlation_id() else {
                return Err(Error::NoCorrelationId);
            };

            let confirmation = match data {
                Ok(data) => match handler.handle(data, request_id.clone()).await {
                    Ok(response) => {
                        tracing::info!("data is handled");
                        response
                    }
                    Err(error) => {
                        tracing::error!(%error, "error handling data");
                        ApiResponse::default().with_messages(
                            Message::stop(format!(
                                "ошибка обработки данных: {error}"
                            ))
                            .into(),
                        )
                    }
                },
                Err(error) => {
                    tracing::error!(%error, "error consuming data");
                    ApiResponse::default().with_messages(
                        Message::stop(format!("ошибка обработки данных: {error}"))
                            .into(),
                    )
                }
            };

            tracing::debug!(?confirmation, "handle result message");

            tracing::info!(%confirm_queue, %request_id, "publishing confirmation");
            let basic_props =
                BasicProperties::default().with_correlation_id(request_id).finish();
            let publish_args = BasicPublishArguments::new("", confirm_queue);
            publisher
                .channel()
                .basic_publish(basic_props, publish_args, &confirmation, false)
                .await?;

            Ok(())
        })
    }
}
