//! Ядро бизнес-логики и общие контракты микросервисов АСЭЗ.
//!
//! Структура по слоям Clean Architecture:
//! - `domain/` — бизнес-сущности (`Plan`, `ContractAmendment`, ...) без зависимостей;
//! - `application/` — use cases, правила валидации, форматирование сообщений;
//! - `infrastructure/` — подключение к PostgreSQL, RabbitMQ, HTTP-серверу;
//! - `presentation/` — DTO для API: `ApiResponse`, `Messages`, `ParamItem`, ...
//!
//! `presentation/dto/response_request.rs` — центральный файл: все HTTP-ответы
//! сервисов возвращают `ApiResponse<Data, Object>` с единым форматом сообщений.
//! `MessageKind` упорядочен (`Ord`) — это позволяет `Messages` автоматически
//! отслеживать "наихудший" уровень через `add_prepared_message`.
pub mod application;
pub mod common;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
