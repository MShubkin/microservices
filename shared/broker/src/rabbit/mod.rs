//! Конкретная реализация [`crate::BrokerAdapter`] поверх RabbitMQ через библиотеку `amqprs`.
//!
//! Структура модулей:
//! - [`adapter`]   — реализация `BrokerAdapter` и `DirectReply` (RPC поверх AMQP)
//! - [`channel`]   — обёртка над `amqprs::Channel` с ретраем, ack/nack и коллбэками
//! - [`consumer`]  — `RabbitConsumer` и `RabbitMessage`
//! - [`publisher`] — `RabbitPublisher` с поддержкой TTL для `DirectReply`
//!
//! Реэкспорты сделаны для удобства: пользователи крейта импортируют из `rabbit::*`,
//! не привязываясь к внутренней структуре модулей.

pub mod adapter;
pub use adapter::RabbitAdapter;

pub mod consumer;
pub use consumer::{RabbitConsumer, RabbitMessage};

pub mod publisher;
pub use publisher::RabbitPublisher;

pub mod channel;
pub use channel::RabbitChannel;
