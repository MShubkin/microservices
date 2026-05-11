use std::{
    fmt::Debug,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use amqprs::{
    channel::{
        BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
    },
    BasicProperties,
};
use broker::{
    rabbit::{RabbitAdapter, RabbitConsumer, RabbitPublisher},
    BrokerAdapter, BrokerError, Consumer,
};
use futures_util::future::BoxFuture;
use serde::Serialize;
use shared_essential::presentation::dto::response_request::ApiResponse;
use tokio::sync::mpsc::{error::TryRecvError, UnboundedReceiver};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("broker error: {0}")]
    Broker(#[from] BrokerError),
    #[error("receive channel closed: {0}")]
    Recv(#[from] TryRecvError),
    #[error("no reply-to property")]
    NoReplyTo,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct PublisherServer<'a> {
    fut: BoxFuture<'a, Result<()>>,
}

impl<'a> std::future::Future for PublisherServer<'a> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut Pin::into_inner(self).fut).poll(cx)
    }
}

/// Нотификация о получении подтверждения.
pub trait ConfirmationHandler {
    fn handle(&mut self, confirmation: ApiResponse<(), ()>, request_id: &str);
}

impl<F> ConfirmationHandler for F
where
    F: FnMut(ApiResponse<(), ()>, &str),
{
    fn handle(&mut self, confirmation: ApiResponse<(), ()>, request_id: &str) {
        self(confirmation, request_id);
    }
}

pub struct RabbitPublisherWithConfirmation<T, H> {
    consumer_tag: String,
    publish_queue: String,
    confirm_queue: String,
    publish_rx: UnboundedReceiver<(T, String)>,
    rabbit_adapter: Arc<RabbitAdapter>,
    handler: H,
}

struct ReliableRabbitPublisherInner<T, H> {
    publish_queue: String,
    confirm_queue: String,
    publish_rx: UnboundedReceiver<(T, String)>,
    data_publisher: RabbitPublisher,
    confirm_consumer: RabbitConsumer,
    handler: H,
}

impl<T, H> RabbitPublisherWithConfirmation<T, H> {
    pub fn new(
        consumer_tag: &str,
        publish_queue: &str,
        confirm_queue: &str,
        publish_rx: UnboundedReceiver<(T, String)>,
        rabbit_adapter: Arc<RabbitAdapter>,
        handler: H,
    ) -> Self {
        RabbitPublisherWithConfirmation {
            consumer_tag: consumer_tag.to_string(),
            publish_queue: publish_queue.to_string(),
            confirm_queue: confirm_queue.to_string(),
            publish_rx,
            rabbit_adapter,
            handler,
        }
    }
}
impl<'a, T, H> RabbitPublisherWithConfirmation<T, H>
where
    T: Debug + Serialize + Send + Sync + 'a,
    H: ConfirmationHandler + Send + Sync + 'a,
{
    pub async fn run(self) -> Result<PublisherServer<'a>> {
        let RabbitPublisherWithConfirmation {
            consumer_tag,
            publish_queue,
            confirm_queue,
            publish_rx,
            rabbit_adapter,
            handler,
        } = self;

        let queue_args =
            QueueDeclareArguments::new(&publish_queue).durable(true).finish();
        rabbit_adapter.declare_queue(queue_args).await?;

        let queue_args =
            QueueDeclareArguments::new(&confirm_queue).durable(true).finish();
        rabbit_adapter.declare_queue(queue_args).await?;

        let consume_args =
            BasicConsumeArguments::new(&confirm_queue, &consumer_tag);
        let confirm_consumer =
            rabbit_adapter.register_consumer(consume_args).await?;

        let basic_props = BasicProperties::default();
        let publish_args = BasicPublishArguments::new("", &publish_queue);
        let data_publisher =
            rabbit_adapter.register_publisher(basic_props, publish_args).await?;

        let inner = ReliableRabbitPublisherInner {
            publish_queue,
            confirm_queue,
            publish_rx,
            data_publisher,
            confirm_consumer,
            handler,
        };
        let fut = inner.publisher_main_loop();

        Ok(PublisherServer { fut: Box::pin(fut) })
    }
}

impl<T, H> ReliableRabbitPublisherInner<T, H>
where
    T: Debug + Serialize + Send + Sync,
    H: ConfirmationHandler,
{
    #[tracing::instrument(skip_all, fields(queue = %self.publish_queue))]
    async fn publisher_main_loop(self) -> Result<()> {
        tracing::info!("publisher is started");

        let ReliableRabbitPublisherInner {
            publish_queue: _,
            confirm_queue,
            mut publish_rx,
            data_publisher,
            mut confirm_consumer,
            mut handler,
        } = self;

        loop {
            match publish_rx.try_recv() {
                Ok((data, request_id)) => {
                    tracing::info!("publishing data");
                    tracing::trace!(?data, "data to publish");
                    Self::publish_data(
                        data,
                        &request_id,
                        &confirm_queue,
                        &data_publisher,
                    )
                    .await?;
                }
                Err(TryRecvError::Empty) => {
                    // нет данных
                }
                Err(error) => {
                    tracing::error!(%error, "error receiving data to publish");
                    return Err(error.into());
                }
            }

            match Consumer::<ApiResponse<(), ()>>::consume_with_timeout(
                &mut confirm_consumer,
                Duration::from_millis(100),
            )
            .await
            {
                Ok(message) => {
                    tracing::info!("received confirmation");
                    Self::process_confirmation(
                        message.content,
                        message.properties,
                        &mut handler,
                    )
                    .await?;
                }
                Err(BrokerError::WaitingTooLong) => {
                    // нет сообщений
                }
                Err(error) => {
                    tracing::error!(%error, "consume error");
                    todo!("recreate consumer?");
                }
            }
        }
    }

    async fn publish_data(
        data: T,
        request_id: &str,
        confirm_queue: &str,
        publisher: &RabbitPublisher,
    ) -> Result<()> {
        let basic_props = BasicProperties::default()
            .with_reply_to(confirm_queue)
            .with_correlation_id(request_id)
            .finish();
        publisher
            .channel()
            .basic_publish(
                basic_props,
                publisher.publish_args().clone(),
                &data,
                false,
            )
            .await?;

        Ok(())
    }

    async fn process_confirmation(
        confirm: ApiResponse<(), ()>,
        properties: BasicProperties,
        handler: &mut H,
    ) -> Result<()> {
        let Some(request_id) = properties.correlation_id() else {
            tracing::error!("no correlation id");
            return Ok(());
        };

        let messages = &confirm.messages;
        if messages.is_error() {
            tracing::warn!(%request_id, ?messages, "received confirmation with error");
        } else {
            tracing::info!(%request_id, ?messages, "received confirmation");
        }

        handler.handle(confirm, request_id);

        Ok(())
    }
}
