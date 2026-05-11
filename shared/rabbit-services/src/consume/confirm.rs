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

/// Результат хэндлера.
pub type HandlerResult<E> = StdResult<ApiResponse<(), ()>, E>;

/// Консьюмер сообщений из заданной очереди RabbitMQ, реализующий
/// принимающую часть механизма надежной передачи данных.
///
/// Осуществляет следующие действия
/// - [x] Слушает заданную очередь
/// - [x] При принятии сообщения отправляет acknowledge серверу RabbitMQ
///       (сейчас автоматически, ручной ack почему-то не работает)
/// - [ ] Записывает событие о принятии данных
/// - [x] Вызывает обработчик данных
/// - [x] По результату выполнения обработчика данных отправляет сообщение в ответную очередь
/// - [ ] Записывает событие о завершении обработки данных
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

/// Хендлер данных для очереди с подтверждением.
///
/// Осуществляет обработку (принятие) данных `data` и возвращает результат обработки.
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

/// Структура, реализующая базовый хэндлер для отправки подтверждения по факту обработки.
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
            let Some(confirm_queue) = properties.reply_to() else {
                return Err(Error::NoReplyTo);
            };
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
