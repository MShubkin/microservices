use std::{sync::Arc, time::Duration};

use amqprs::{
    channel::{BasicConsumeArguments, BasicPublishArguments},
    BasicProperties,
};
use anyhow::Result;
use broker::{
    rabbit::{RabbitAdapter, RabbitConsumer},
    BrokerAdapter, Consumer, Publisher,
};
use rabbit_services::publish::RabbitPublisherWithConfirmation;
use serde::{Deserialize, Serialize};
use shared_essential::presentation::dto::response_request::{ApiResponse, Message};
use tokio::sync::mpsc::unbounded_channel;
use tracing::Instrument;
use uuid::Uuid;

mod common;

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

    fn into_api_response(self) -> ApiResponse<(), ()> {
        let message = match self.handle_result {
            HandleResult::Ok => Message::success(self.message),
            HandleResult::MessageError => Message::error(self.message),
            HandleResult::ResultError => Message::stop(self.message),
        };
        ApiResponse::default().with_messages(message.into())
    }
}

#[tokio::test]
async fn test() -> Result<()> {
    igg_tracing::setup_dev_logger();

    const PUBLISH_QUEUE: &str = "test_publisher_publish";
    const CONFIRM_QUEUE: &str = "test_publisher_confirm";
    const CONSUMER_TAG: &str = "test_publisher_consumer";
    const PUBLISHER_CONSUMER_TAG: &str = "test_publisher_publisher";

    let (test_tx, mut test_rx) = unbounded_channel::<String>();

    let rabbit_adapter = Arc::new(rabbit_adapter().await?);
    let (tx, rx) = unbounded_channel();
    let publisher = RabbitPublisherWithConfirmation::new(
        PUBLISHER_CONSUMER_TAG,
        PUBLISH_QUEUE,
        CONFIRM_QUEUE,
        rx,
        rabbit_adapter.clone(),
        move |conf: ApiResponse<(), ()>, request_id: &str| {
            tracing::info!(?conf.messages, "confirmation received");
            test_tx.send(request_id.to_string()).unwrap();
        },
    )
    .run()
    .await?;

    let consume_args =
        BasicConsumeArguments::new(PUBLISH_QUEUE, CONSUMER_TAG).finish();
    let mut consumer = rabbit_adapter.register_consumer(consume_args).await?;

    let test = async move {
        let id = format!("success-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        tx.send((Data::ok("success"), id.clone()))?;

        let (data, reply_to, request_id) = consume_data(&mut consumer).await?;
        assert_eq!(request_id.as_ref(), Some(&id));
        assert!(reply_to.is_some());
        publish_confirmation(
            &rabbit_adapter,
            data,
            &reply_to.unwrap(),
            &request_id.unwrap(),
        )
        .await?;

        let confirm_id =
            tokio::time::timeout(Duration::from_millis(250), test_rx.recv())
                .await
                .unwrap();
        assert!(matches!(confirm_id.clone(), Some(x) if x == id), "{confirm_id:?}");

        let id = format!("message-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        tx.send((Data::message_error("error as message"), id.clone()))?;

        let (data, reply_to, request_id) = consume_data(&mut consumer).await?;
        assert_eq!(request_id, Some(id.clone()));
        assert!(reply_to.is_some());
        publish_confirmation(
            &rabbit_adapter,
            data,
            &reply_to.unwrap(),
            &request_id.unwrap(),
        )
        .await?;

        let confirm_id =
            tokio::time::timeout(Duration::from_millis(250), test_rx.recv())
                .await
                .unwrap();
        assert!(matches!(confirm_id.clone(), Some(x) if x == id), "{confirm_id:?}");

        let id = format!("error-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        tx.send((Data::result_error("error as result"), id.clone()))?;

        let (data, reply_to, request_id) = consume_data(&mut consumer).await?;
        assert_eq!(request_id, Some(id.clone()));
        assert!(reply_to.is_some());
        publish_confirmation(
            &rabbit_adapter,
            data,
            &reply_to.unwrap(),
            &request_id.unwrap(),
        )
        .await?;

        let confirm_id =
            tokio::time::timeout(Duration::from_millis(250), test_rx.recv())
                .await
                .unwrap();
        assert!(matches!(confirm_id.clone(), Some(x) if x == id), "{confirm_id:?}");

        // to give publisher time to ack confirmation
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok::<_, anyhow::Error>(())
    }
    .instrument(tracing::info_span!("test"));

    tokio::select! {
        res = publisher => {
            panic!("should not finish: {res:?}");
        },
        res = test => { res?; }
    }

    Ok(())
}

async fn consume_data(
    consumer: &mut RabbitConsumer,
) -> Result<(Data, Option<String>, Option<String>)> {
    let message = consumer.consume_with_timeout(Duration::from_millis(250)).await?;
    let reply_to = message.properties.reply_to().cloned();
    let correlation_id = message.properties.correlation_id().cloned();
    Ok((message.content, reply_to, correlation_id))
}

async fn publish_confirmation(
    rabbit_adapter: &RabbitAdapter,
    data: Data,
    reply_to: &str,
    request_id: &str,
) -> Result<()> {
    let basic_props =
        BasicProperties::default().with_correlation_id(request_id).finish();
    let publish_args = BasicPublishArguments::new("", reply_to);
    let publisher =
        rabbit_adapter.register_publisher(basic_props, publish_args).await?;

    Publisher::<ApiResponse<(), ()>>::publish(
        &publisher,
        &data.into_api_response(),
    )
    .await?;

    Ok(())
}

pub async fn rabbit_adapter() -> Result<RabbitAdapter> {
    Ok(common::connect(common::get_config()?).await?)
}
