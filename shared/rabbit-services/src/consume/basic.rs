use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use amqprs::{channel::BasicConsumeArguments, BasicProperties};
use broker::{
    rabbit::{RabbitAdapter, RabbitConsumer, RabbitMessage},
    BrokerAdapter, Consumer,
};
use futures_util::future::BoxFuture;
use serde::Deserialize;
use tracing::{debug_span, Instrument, Span};

use super::{ConsumerServer, Result};

/// Обработчик входящих сообщений для [`BasicRabbitConsumer`].
///
/// Получает результат десериализации (`Ok(T)` или ошибку) и свойства сообщения.
/// Свойства присутствуют всегда -- они берутся из транспортного сообщения до десериализации.
/// Реализации могут использовать `rabbit_props` для чтения заголовков трассировки,
/// `correlation_id` или `reply_to`.
pub trait BasicConsumerHandler<T> {
    fn handle(
        &self,
        consume_result: Result<T>,
        rabbit_props: BasicProperties,
    ) -> BoxFuture<'static, Result<()>>;
}

/// Базовый консьюмер: подписывается на очередь и в цикле вызывает [`BasicConsumerHandler`].
///
/// Каждое сообщение обрабатывается в отдельной задаче `tokio::spawn`, поэтому
/// медленный обработчик не блокирует получение следующих сообщений.
/// При ошибке получения (транспортной) сообщение пропускается с логом и цикл продолжается.
pub struct BasicRabbitConsumer<H, T> {
    /// Адаптер подключения к брокеру.
    adapter: Arc<RabbitAdapter>,
    /// Имя очереди, которую слушает консьюмер.
    queue: String,
    /// Уникальный тег консьюмера (используется для идентификации в логах и при отмене).
    consumer_tag: String,
    /// Обработчик входящих данных.
    handler: H,
    phantom_data: PhantomData<T>,
}

impl<H, T> BasicRabbitConsumer<H, T> {
    pub fn new(
        adapter: Arc<RabbitAdapter>,
        queue: &str,
        consumer_tag: &str,
        handler: H,
    ) -> Self {
        BasicRabbitConsumer {
            adapter,
            queue: queue.to_owned(),
            consumer_tag: consumer_tag.to_owned(),
            handler,
            phantom_data: Default::default(),
        }
    }
}

impl<H, T> BasicRabbitConsumer<H, T>
where
    H: BasicConsumerHandler<T> + Send + Sync + 'static,
    T: for<'de> Deserialize<'de> + Debug + Send + Sync + 'static,
{
    /// Регистрирует консьюмера и возвращает [`ConsumerServer`] -- future, который надо заспаунить.
    pub async fn run(self) -> Result<ConsumerServer> {
        let consumer = self.register_consumer().await?;
        let fut = Box::pin(self.runner(consumer));
        Ok(ConsumerServer { fut })
    }

    async fn register_consumer(&self) -> Result<RabbitConsumer> {
        let BasicRabbitConsumer {
            adapter,
            queue,
            consumer_tag,
            ..
        } = self;

        let consume_args = BasicConsumeArguments::new(queue, consumer_tag).finish();
        tracing::debug!("registering consumer...");
        let consumer = adapter.register_consumer(consume_args).await?;

        Ok(consumer)
    }

    #[tracing::instrument(level = "debug", name = "rabbit_consumer", skip(self), fields(queue = %self.queue, consumer = %self.consumer_tag))]
    async fn runner(self, mut consumer: RabbitConsumer) -> Result<()> {
        let BasicRabbitConsumer { handler, .. } = self;
        // Arc нужен, чтобы клонировать ссылку на хэндлер для каждой спаунируемой задачи.
        let handler = Arc::new(handler);

        loop {
            let message = match Self::try_consume_message(&mut consumer).await {
                Ok(v) => v,
                Err(error) => {
                    // Транспортная ошибка: ack/nack отправить нечем (мы не получили
                    // delivery_tag). Логируем и продолжаем; на стороне `RabbitChannel`
                    // десериализационные ошибки уже отправляют nack самостоятельно.
                    tracing::error!(%error, "error receiving message from queue, skipping it");
                    continue;
                }
            };
            let task_span = debug_span!(
                parent: Span::current(),
                "message_handler",
                // correlation_id позволяет связать лог обработки с конкретным запросом.
                request_id =
                    message.properties.correlation_id().map_or("", String::as_str)
            );
            let task = Self::process_message(message, handler.clone());
            tokio::spawn(task.instrument(task_span));
        }
    }

    async fn try_consume_message(
        consumer: &mut RabbitConsumer,
    ) -> Result<RabbitMessage<serde_json::Value>> {
        tracing::debug!("consuming message...");
        let message = consumer.consume().await?;
        tracing::info!("data is consumed");
        tracing::trace!(?message, "consumed message");
        Ok(message)
    }

    async fn process_message(
        message: RabbitMessage<serde_json::Value>,
        handler: Arc<H>,
    ) {
        if let Err(error) = Self::handle_message(message, handler).await {
            tracing::error!(%error, "error handling message");
        } else {
            tracing::info!("message successfully handled");
        }
    }

    async fn handle_message(
        message: RabbitMessage<serde_json::Value>,
        handler: Arc<H>,
    ) -> Result<()> {
        let RabbitMessage {
            content,
            delivery: _,
            properties,
        } = message;
        // Десериализация выполняется через serde_json::Value, а не напрямую в T,
        // чтобы иметь возможность передать ошибку в хэндлер вместе со свойствами сообщения.
        let result = serde_json::from_value::<T>(content).map_err(Into::into);
        handler.handle(result, properties).await?;
        Ok(())
    }
}
