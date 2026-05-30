//! Middleware-слой для HTTP-сервисов АСЭЗ.
//!
//! Каждый модуль — отдельный middleware. Порядок регистрации критичен:
//! в Actix последний `.wrap()` выполняется первым, поэтому правильный порядок:
//! 1. `DefaultRabbitProperties` — готовит контейнер для RabbitMQ-параметров
//! 2. `AsezTracingFields` — создаёт коллекцию полей трассировки из заголовков
//! 3. `TracingLogger::<ServiceRootSpanBuilder>` — открывает span, читает коллекцию
//! 4. `DomainIDsTransform` — дополняет коллекцию object_ids/object_uuids
//! 5. `AsezSessionWatcher` — проверяет сессию, дополняет user_id/user_name

pub mod cookie_decoder;
pub mod domain_ids;
pub mod login;
pub mod rabbit;
pub mod tracing_fields;

pub use cookie_decoder::default_cookie_decoder;
pub use login::AsezSessionWatcher;

#[cfg(test)]
mod tests;
