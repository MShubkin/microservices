use std::{fmt::Debug, future::Future, marker::PhantomData, sync::Arc};

use amqprs::{
    channel::{BasicPublishArguments, QueueDeclareArguments},
    BasicProperties,
};
use broker::{
    rabbit::{RabbitAdapter, RabbitPublisher},
    BrokerAdapter,
};
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::{
    consume::{
        basic::{BasicConsumerHandler, BasicRabbitConsumer},
        ConsumerServer, Error, Result,
    },
    CONTENT_TYPE,
};

/// Консьюмер для обработки RPC-подобных запросов (direct reply_to).
///
/// - [ ] Принимает запрос из очереди
/// - [ ] Вызывает обработчик с полученными данными
/// - [ ] Отправляет ответ обратно
pub struct RabbitConsumerForRpc<H, I, O, E> {
    adapter: Arc<RabbitAdapter>,
    queue: String,
    consumer_tag: String,
    handler: H,
    _phantom_data: PhantomData<(I, O, E)>,
}

pub type RpcHandlerResult<O, E> = std::result::Result<O, E>;

/// Хэндлер вызова RPC через кролика. На основе входного значения возвращает
/// данные через стандартный ответ
/// ([`ApiResponse`](shared_essential::request_response::ApiResponse)) или
/// ошибку, преобразуемую к `AsezError`.
pub trait ConsumerRpcHandler<I, O, E> {
    fn handle(&self, input: I) -> BoxFuture<'static, RpcHandlerResult<O, E>>;
}

impl<I, O, E, F, Fut> ConsumerRpcHandler<I, O, E> for F
where
    F: Fn(I) -> Fut,
    Fut: Future<Output = RpcHandlerResult<O, E>> + Send + 'static,
{
    fn handle(&self, input: I) -> BoxFuture<'static, RpcHandlerResult<O, E>> {
        Box::pin(self(input))
    }
}

impl<H, I, O, E> RabbitConsumerForRpc<H, I, O, E>
where
    H: ConsumerRpcHandler<I, O, E> + Send + Sync + 'static,
    I: for<'de> Deserialize<'de> + Debug + Send + Sync + 'static,
    O: Debug + Serialize + Send + Sync + 'static,
    E: std::error::Error + Serialize + Send + Sync + 'static,
{
    pub fn new(
        adapter: Arc<RabbitAdapter>,
        queue: &str,
        consumer_tag: &str,
        handler: H,
    ) -> Self {
        RabbitConsumerForRpc {
            adapter,
            queue: queue.to_owned(),
            consumer_tag: consumer_tag.to_owned(),
            handler,
            _phantom_data: PhantomData::default(),
        }
    }

    pub async fn run(self) -> Result<ConsumerServer> {
        let RabbitConsumerForRpc {
            adapter,
            queue,
            consumer_tag,
            handler,
            ..
        } = self;

        tracing::info!(%queue, "declaring queue");
        let queue_args = QueueDeclareArguments::durable_client_named(&queue);
        adapter.declare_queue(queue_args).await?;

        tracing::info!("registering RPC response publisher");
        let basic_props = BasicProperties::default();
        let publish_args = BasicPublishArguments::new("", "");
        let publisher =
            adapter.register_publisher(basic_props, publish_args).await?;

        let basic_handler = BasicHandlerForRpc {
            publisher: publisher.into(),
            handler: handler.into(),
            _phantom_data: PhantomData::default(),
        };

        let basic_consumer =
            BasicRabbitConsumer::new(adapter, &queue, &consumer_tag, basic_handler);
        basic_consumer.run().await
    }
}

struct BasicHandlerForRpc<H, I, O, E> {
    publisher: Arc<RabbitPublisher>,
    handler: Arc<H>,
    _phantom_data: PhantomData<(I, O, E)>,
}

impl<H, I, O, E> BasicConsumerHandler<I> for BasicHandlerForRpc<H, I, O, E>
where
    H: ConsumerRpcHandler<I, O, E> + Send + Sync + 'static,
    I: for<'de> Deserialize<'de> + Debug + Send + Sync + 'static,
    O: Debug + Serialize + Send + Sync + 'static,
    E: std::error::Error + Serialize + Send + Sync + 'static,
{
    fn handle(
        &self,
        consume_result: super::Result<I>,
        rabbit_props: amqprs::BasicProperties,
    ) -> BoxFuture<'static, super::Result<()>> {
        let handler = self.handler.clone();
        let publisher = self.publisher.clone();

        Box::pin(async move {
            let Some(reply_to) = rabbit_props.reply_to() else {
                return Err(Error::NoReplyTo);
            };

            let result: RpcHandlerResult<O, E> = match consume_result {
                Ok(input) => match handler.handle(input).await {
                    Ok(response) => {
                        tracing::info!("data is handled");
                        Ok(response)
                    }
                    Err(error) => {
                        tracing::error!(%error, "error handling data");
                        Err(error)
                    }
                },
                Err(error) => {
                    tracing::error!(%error, "error consuming data");
                    return Err(error);
                }
            };

            tracing::debug!(?result, "handle RPC message");

            tracing::info!("publishing result");
            let basic_props =
                BasicProperties::default().with_content_type(CONTENT_TYPE).finish();
            let publish_args = BasicPublishArguments::new("", reply_to);
            publisher
                .channel()
                .basic_publish(basic_props, publish_args, &result, false)
                .await?;

            Ok(())
        })
    }
}
