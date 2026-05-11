use std::{sync::Arc, time::Duration};

use anyhow::Result;
use broker::rabbit::RabbitAdapter;
use rabbit_services::{
    consume::RabbitConsumerWithConfirmation,
    publish::RabbitPublisherWithConfirmation,
};
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

#[derive(Debug, thiserror::Error, Serialize, Deserialize, PartialEq)]
#[error("my precious error: {0}")]
struct Error(String);

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

    fn into_result(self) -> Result<ApiResponse<(), ()>, Error> {
        match self.handle_result {
            HandleResult::Ok => Ok(ApiResponse::default()
                .with_messages(Message::success(self.message).into())),
            HandleResult::MessageError => Ok(ApiResponse::default()
                .with_messages(Message::error(self.message).into())),
            HandleResult::ResultError => Err(Error(self.message)),
        }
    }
}

#[tokio::test]
async fn test() -> Result<()> {
    igg_tracing::setup_dev_logger();

    const PUBLISH_QUEUE: &str = "test_pubcons_publish";
    const CONFIRM_QUEUE: &str = "test_pubcons_confirm";
    const CONSUMER_TAG: &str = "test_pubcons_consumer";
    const PUBLISHER_CONSUMER_TAG: &str = "test_pubcons_publisher";

    let rabbit_adapter = Arc::new(rabbit_adapter().await?);
    let (test_tx, mut test_rx) = unbounded_channel::<String>();
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

    let consumer = RabbitConsumerWithConfirmation::new(
        rabbit_adapter,
        PUBLISH_QUEUE,
        CONSUMER_TAG,
        |data: Data, _request_id: String| Box::pin(async { data.into_result() }),
    )
    .run()
    .await?;

    let test = async move {
        let id = format!("success-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        tx.send((Data::ok("success"), id.clone()))?;

        let confirm_id =
            tokio::time::timeout(Duration::from_millis(250), test_rx.recv())
                .await
                .unwrap();
        assert!(matches!(confirm_id.clone(), Some(x) if x == id), "{confirm_id:?}");

        let id = format!("message-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        tx.send((Data::message_error("error as message"), id.clone()))?;

        let confirm_id =
            tokio::time::timeout(Duration::from_millis(250), test_rx.recv())
                .await
                .unwrap();
        assert!(matches!(confirm_id.clone(), Some(x) if x == id), "{confirm_id:?}");

        let id = format!("error-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");
        tx.send((Data::result_error("error as result"), id.clone()))?;

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
        res = consumer => {
            panic!("should not finish: {res:?}");
        },
        res = test => { res?; }
    }

    Ok(())
}

pub async fn rabbit_adapter() -> Result<RabbitAdapter> {
    Ok(common::connect(common::get_config()?).await?)
}
