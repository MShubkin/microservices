//! Клиентская библиотека для взаимодействия с микросервисами АСЭЗ 2.0 через RabbitMQ.
//!
//! Организована вокруг трёх концепций:
//! - **Маршрутизация** (`routing`) -- константы и enum очередей всех сервисов.
//! - **Сервисы** (`services`) -- типизированные RPC-клиенты, реализующие [`AsezRabbitService`].
//!   Каждый сервис встраивается в Actix-хэндлер через `FromRequest` (см. `from_request!`).
//! - **Консьюмеры** (`consume`) -- серверная сторона: базовый, RPC и confirm-паттерн.

pub mod callbacks;

pub mod properties;

pub mod routing;
pub use routing::*;

pub mod services;
pub use services::*;

pub mod consume;
pub mod publish;

// Content-Type для всех AMQP-сообщений в системе -- JSON без исключений.
const CONTENT_TYPE: &str = "application/json";
