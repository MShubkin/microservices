//! Серверная сторона взаимодействия с RabbitMQ: приём и обработка сообщений.
//!
//! Три паттерна потребления:
//! - [`basic`] -- просто слушает очередь и вызывает обработчик.
//! - [`rpc`] -- принимает запрос, вызывает обработчик, отправляет ответ в `reply_to`.
//! - [`confirm`] -- как rpc, но ответ -- это подтверждение факта обработки,
//!   а не результат вычисления.

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

/// Ошибки при получении и обработке AMQP-сообщений.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Ошибка транспортного уровня (соединение, канал).
    #[error("broker error: {0}")]
    Broker(#[from] BrokerError),
    /// Тело сообщения не удалось десериализовать из JSON.
    #[error("error deserializing message: {0}")]
    Deserialize(#[from] serde_json::Error),
    /// Сообщение пришло без заголовка `reply_to` -- некуда отправить ответ.
    #[error("no reply_to property")]
    NoReplyTo,
    /// Сообщение пришло без `correlation_id` -- нельзя сопоставить ответ с запросом.
    #[error("no correlation_id property")]
    NoCorrelationId,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Обёртка над бесконечным future консьюмера.
///
/// Создаётся методами `run()` каждого типа консьюмера. Используется как
/// задача tokio: `tokio::spawn(consumer_server)`. Завершается ошибкой при
/// потере соединения с брокером.
pub struct ConsumerServer {
    fut: BoxFuture<'static, Result<()>>,
}

impl Future for ConsumerServer {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut Pin::into_inner(self).fut).poll(cx)
    }
}
