//! Интеграция с Actix-Web: корневой span для каждого HTTP-запроса.
//!
//! `ServiceRootSpanBuilder` реализует трейт `RootSpanBuilder` из `tracing-actix-web`.
//! Он вызывается `TracingLogger` middleware при старте и завершении каждого запроса.
//!
//! ## Порядок middleware важен
//!
//! `TracingLogger::<ServiceRootSpanBuilder>` должен быть зарегистрирован **после**
//! middleware, которые наполняют `AsezTracingFieldsCollection` в `req.extensions()`:
//! - `AsezSessionWatcher` — декодирует cookie, устанавливает `user_id`/`user_name`
//! - `DomainIDsTransform` — устанавливает `object_ids`/`object_uuids`
//!
//! В коде Actix middleware применяются в обратном порядке относительно `.wrap()`,
//! поэтому последний зарегистрированный через `.wrap()` выполняется первым.

use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    HttpMessage,
};
use tracing::{field::display, Span};
use tracing_actix_web::{root_span, DefaultRootSpanBuilder, RootSpanBuilder};

use crate::tracing_fields::*;

/// Построитель корневого span-а для входящих HTTP-запросов АСЭЗ.
///
/// Если в `req.extensions()` есть `AsezTracingFieldsCollection` — создаёт span
/// с полными данными запроса (user_id, uri, source_ip и т.д.).
/// Если коллекции нет — делегирует `root_span!(request)` из `tracing-actix-web`,
/// который создаёт минимальный span с базовыми HTTP-полями.
///
/// Поля `user_id`, `user_name`, `object_ids`, `object_uuids` объявляются как
/// `tracing::field::Empty` при создании span-а и дозаписываются позже методом
/// `span.record(...)`. Это необходимо, потому что tracing не позволяет добавлять
/// новые поля после создания span-а — только обновлять уже объявленные.
#[derive(Debug, Clone, Copy)]
pub struct ServiceRootSpanBuilder;

impl ServiceRootSpanBuilder {
    fn with_tracing_fields(
        request: &ServiceRequest,
        fields: AsezTracingFieldsCollection,
    ) -> tracing::Span {
        let span = root_span!(
            request,
            // TODO: use constants for field names
            // when uplifting tracing dependency to >= 0.1.39
            "asez.request_id" = %fields.request_id,
            "asez.uri" = %fields.uri,
            "asez.user_agent" = %fields.user_agent,
            "asez.source_ip" = %fields.source.ip(),
            "asez.source_port" = %fields.source.port(),
            "asez.timestamp" = %fields.timestamp,
            // the following fields will be recorded by other middlewares
            "asez.user_id" = tracing::field::Empty,
            "asez.user_name" = tracing::field::Empty,
            "asez.object_ids" = tracing::field::Empty,
            "asez.object_uuids" = tracing::field::Empty,
        );

        // record optional fields individually, in case they are already set
        if let Some(user_id) = &fields.user_id {
            span.record("asez.user_id", user_id);
        }
        if let Some(user_name) = &fields.user_name {
            span.record("asez.user_name", &display(user_name));
        }
        if !fields.object_ids.is_empty() {
            span.record("asez.object_ids", &display(&fields.object_ids));
        }
        if !fields.object_uuids.is_empty() {
            span.record("asez.object_uuids", &display(&fields.object_uuids));
        }

        span
    }
}

impl RootSpanBuilder for ServiceRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> tracing::Span {
        let fields = {
            let mut extensions = request.extensions_mut();
            // `tracing_actix_web` генерирует собственный RequestId — синхронизируем
            // его с нашим `request_id` в коллекции для единого идентификатора в логах.
            let request_id =
                extensions.get::<tracing_actix_web::RequestId>().cloned();
            extensions.get_mut::<AsezTracingFieldsCollection>().map(|fields| {
                if let Some(request_id) = request_id {
                    fields.set_request_id(*request_id)
                }
                fields.clone()
            })
        };
        if let Some(fields) = fields {
            Self::with_tracing_fields(request, fields)
        } else {
            // AsezTracingFieldsCollection не установлена — используем стандартный
            // span tracing-actix-web с базовыми HTTP-полями (method, path, status).
            root_span!(request)
        }
    }

    fn on_request_end<B>(
        span: Span,
        outcome: &Result<ServiceResponse<B>, actix_web::Error>,
    ) {
        // Делегируем стандартной реализации — она записывает HTTP status code и
        // помечает span как ошибочный при 5xx/ошибках actix.
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
