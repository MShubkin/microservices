use std::time::Duration;

use amqprs::{channel::BasicPublishArguments, BasicProperties};
use anyhow::Result;
use broker::rabbit::RabbitAdapter;
use rabbit_services::consume::{
    ConsumerServer, RabbitConsumerForRpc, RpcHandlerResult,
};
use serde::{Deserialize, Serialize};
use shared_essential::presentation::dto::response_request::{
    ApiResponse, ApiResponseDataWrapper, Message,
};
use tracing::Instrument;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn direct_reply() -> Result<()> {
    igg_tracing::setup_dev_logger();

    const QUEUE: &str = "test_rpc_queue";
    const CONSUMER_TAG: &str = "test_rpc_consumer";
    const TIMEOUT: Duration = Duration::from_millis(100);

    let adapter = rabbit_adapter().await?;

    // prepare consumer
    let consumer_fut = launch_consumer(QUEUE, CONSUMER_TAG).await?;
    tracing::info!("consumer is started");

    let test = async move {
        // prepare test publisher
        let id = format!("success-{}", Uuid::new_v4());
        tracing::debug!(%id, "correlation id");

        let basic_props = BasicProperties::default().finish();
        let publish_args = BasicPublishArguments::new("", QUEUE);
        let mut direct_reply =
            adapter.direct_reply(basic_props, publish_args, CONSUMER_TAG).await?;

        // publish some data
        let message = direct_reply
            .request::<Data, RpcHandlerResult<ApiResponse<String, ()>, Error>>(
                &Data::ok("successfully handled"),
                TIMEOUT,
            )
            .await?;

        // check result
        assert!(message.content.is_ok());

        let response = message.content.unwrap();
        assert_eq!(&response.data, "successfully handled");
        assert!(!response.messages.is_error());

        // publish some data
        let message = direct_reply
            .request::<Data, RpcHandlerResult<ApiResponse<String, ()>, Error>>(
                &Data::message_error("handled with error message"),
                TIMEOUT,
            )
            .await?;

        // check result
        assert!(message.content.is_ok());

        let response = message.content.unwrap();
        assert_eq!(&response.data, "");
        assert!(response.messages.is_error());

        // publish some data
        let message = direct_reply
            .request::<Data, RpcHandlerResult<ApiResponse<String, ()>, Error>>(
                &Data::result_error("handled with error result"),
                TIMEOUT,
            )
            .await?;

        // check result
        assert!(message.content.is_err());

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

type Response = ApiResponseDataWrapper<String>;

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

impl From<rabbit_services::consume::Error> for Error {
    fn from(error: rabbit_services::consume::Error) -> Self {
        Error(error.to_string())
    }
}

async fn handler(data: Data) -> RpcHandlerResult<ApiResponse<Response, ()>, Error> {
    match data.handle_result {
        HandleResult::Ok => Ok(ApiResponse::default()
            .with_messages(Message::success(data.message.clone()).into())
            .with_data(data.message.into())),
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
    let consumer =
        RabbitConsumerForRpc::new(adapter.into(), queue, consumer_tag, handler)
            .run()
            .await?;
    Ok(consumer)
}
