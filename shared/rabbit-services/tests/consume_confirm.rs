use std::time::Duration;

use amqprs::{
    channel::{
        BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
    },
    BasicProperties,
};
use anyhow::Result;
use broker::{
    rabbit::{RabbitAdapter, RabbitMessage},
    BrokerAdapter, Consumer, Publisher,
};
use rabbit_services::consume::{
    ConsumerServer, HandlerResult, RabbitConsumerWithConfirmation,
};
use serde::{Deserialize, Serialize};
use shared_essential::presentation::dto::response_request::{ApiResponse, Message};
use tracing::Instrument;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn consume_handle() -> Result<()> {
    igg_tracing::setup_dev_logger();

    const PUBLISH_QUEUE: &str = "publish";
    const CONFIRM_QUEUE: &str = "confirm";
    const CONSUMER_TAG: &str = "test_consumer123";
    const PUBLISHER_CONSUMER_TAG: &str = "test_publisher123";

    let adapter = rabbit_adapter().await?;

    let queue_args =
        QueueDeclareArguments::new(PUBLISH_QUEUE).durable(true).finish();
    adapter.declare_queue(queue_args).await?;

    let queue_args =
        QueueDeclareArguments::new(CONFIRM_QUEUE).durable(true).finish();
    adapter.declare_queue(queue_args).await?;

    // prepare consumer
    let consumer_fut = launch_consumer(PUBLISH_QUEUE, CONSUMER_TAG).await?;
    tracing::info!("consumer is started");

    let consume_args =
        BasicConsumeArguments::new(CONFIRM_QUEUE, PUBLISHER_CONSUMER_TAG)
            .auto_ack(true)
            .finish();
    let mut consumer = adapter.register_consumer(consume_args).await?;

    let test = async move {
        // prepare test publisher
        let id = format!("success-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        let basic_props = BasicProperties::default()
            .with_reply_to(CONFIRM_QUEUE)
            .with_correlation_id(&id)
            .finish();
        let publish_args = BasicPublishArguments::new("", PUBLISH_QUEUE);
        let publisher =
            adapter.register_publisher(basic_props, publish_args).await?;

        // publish some data
        publisher.publish(&Data::ok("successfully handled")).await?;

        // check confirmation
        let msg: RabbitMessage<ApiResponse<(), ()>> =
            consumer.consume_with_timeout(Duration::from_secs(1)).await?;
        tracing::debug!(?msg.content, "confirmation message");
        assert_eq!(msg.properties.correlation_id(), Some(&id));
        assert!(!msg.content.messages.is_error());

        // prepare test publisher
        let id = format!("message-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        let basic_props = BasicProperties::default()
            .with_reply_to(CONFIRM_QUEUE)
            .with_correlation_id(&id)
            .finish();
        let publish_args = BasicPublishArguments::new("", PUBLISH_QUEUE);
        let publisher =
            adapter.register_publisher(basic_props, publish_args).await?;

        // publish some data
        publisher
            .publish(&Data::message_error("handled with error message"))
            .await?;

        // check confirmation
        let msg: RabbitMessage<ApiResponse<(), ()>> =
            consumer.consume_with_timeout(Duration::from_secs(1)).await?;
        tracing::debug!(?msg.content, "confirmation message");
        assert_eq!(msg.properties.correlation_id(), Some(&id));
        assert!(msg.content.messages.is_error());
        assert!(!msg.content.messages.is_stop());

        // prepare test publisher
        let id = format!("error-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        let basic_props = BasicProperties::default()
            .with_reply_to(CONFIRM_QUEUE)
            .with_correlation_id(&id)
            .finish();
        let publish_args = BasicPublishArguments::new("", PUBLISH_QUEUE);
        let publisher =
            adapter.register_publisher(basic_props, publish_args).await?;

        // publish some data
        publisher.publish(&Data::result_error("handled with error")).await?;

        // check confirmation
        let msg: RabbitMessage<ApiResponse<(), ()>> =
            consumer.consume_with_timeout(Duration::from_secs(1)).await?;
        tracing::debug!(?msg.content, "confirmation message");
        assert_eq!(msg.properties.correlation_id(), Some(&id));
        assert!(msg.content.messages.is_stop());

        // prepare test publisher
        let id = format!("invalid-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        let basic_props = BasicProperties::default()
            .with_reply_to(CONFIRM_QUEUE)
            .with_correlation_id(&id)
            .finish();
        let publish_args = BasicPublishArguments::new("", PUBLISH_QUEUE);
        let publisher =
            adapter.register_publisher(basic_props, publish_args).await?;

        // publish some data

        publisher.publish(&OtherData::default()).await?;

        // check confirmation
        let msg: RabbitMessage<ApiResponse<(), ()>> =
            consumer.consume_with_timeout(Duration::from_secs(1)).await?;
        tracing::debug!(?msg.content, "confirmation message");
        assert_eq!(msg.properties.correlation_id(), Some(&id));
        assert!(msg.content.messages.is_stop());

        Ok(())
    }
    .instrument(tracing::debug_span!("test"));

    tokio::select! {
        res = consumer_fut => {
            if let Err(e) = res {
                panic!("consumer failed miserably: {e}");
            } else {
                unreachable!("should not return");
            }
        }
        res = test => {
            res
        }
    }
}

async fn rabbit_adapter() -> Result<RabbitAdapter> {
    Ok(common::connect(common::get_config()?).await?)
}

#[derive(Debug, Serialize, Deserialize)]
enum HandleResult {
    Ok,
    MessageError,
    ResultError,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    handle_result: HandleResult,
    message: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct OtherData {
    some_field: bool,
    some_other_field: i32,
}

impl Data {
    fn ok(message: &str) -> Self {
        Self {
            handle_result: HandleResult::Ok,
            message: message.to_string(),
        }
    }
    fn message_error(message: &str) -> Self {
        Self {
            handle_result: HandleResult::MessageError,
            message: message.to_string(),
        }
    }
    fn result_error(message: &str) -> Self {
        Self {
            handle_result: HandleResult::ResultError,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize, PartialEq)]
#[error("my precious error: {0}")]
struct Error(String);

async fn handler(data: Data, _: String) -> HandlerResult<Error> {
    match data.handle_result {
        HandleResult::Ok => Ok(ApiResponse::default()
            .with_messages(Message::success(data.message).into())),
        HandleResult::MessageError => Ok(ApiResponse::default()
            .with_messages(Message::error(data.message).into())),
        HandleResult::ResultError => Err(Error(data.message)),
    }
}

async fn launch_consumer(
    queue: &'static str,
    consumer_tag: &'static str,
) -> Result<ConsumerServer> {
    let adapter = rabbit_adapter().await?;
    let consumer = RabbitConsumerWithConfirmation::new(
        adapter.into(),
        queue,
        consumer_tag,
        handler,
    )
    .run()
    .await?;
    Ok(consumer)
}
