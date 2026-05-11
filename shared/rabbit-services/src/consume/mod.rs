use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use broker::BrokerError;
use futures_util::future::BoxFuture;

pub mod basic;
pub mod confirm;
pub mod rpc;

pub use confirm::*;
pub use rpc::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("broker error: {0}")]
    Broker(#[from] BrokerError),
    #[error("error deserializing message: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("no reply_to property")]
    NoReplyTo,
    #[error("no correlation_id property")]
    NoCorrelationId,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ConsumerServer {
    fut: BoxFuture<'static, Result<()>>,
}

impl Future for ConsumerServer {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut Pin::into_inner(self).fut).poll(cx)
    }
}
