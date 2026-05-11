//! This module deals with the actix extension that was in the original library.

use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    HttpMessage,
};
use tracing::{field::display, Span};
use tracing_actix_web::{root_span, DefaultRootSpanBuilder, RootSpanBuilder};

use crate::tracing_fields::*;

/// This struct allows us to implement the `ServiceRootSpanBuilder` for it.
///
/// For proper tracing of common ASEZ data associated with a specific user request,
/// the [collection](`AsezTracingFieldsCollection`) of fields containing this information is used.
/// Some fields are taken from the collection directly, others (like user_name, object_ids, etc)
/// are added later by other middlewares.
///
/// The instance of this collection can be injected into the request using e.g. `AsezTracingFields` middleware.
/// If it is not present in the request, standard `actix_web_tracing` root span is used, that contains
/// basic request information.
///
/// Note that a request should be processes by [ServiceRootSpanBuilder] before any other
/// middleware that populate root span's data. Currently, `DomainIDsTransform` and `AsezSessionWatcher` do that.
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
            root_span!(request)
        }
    }

    fn on_request_end<B>(
        span: Span,
        outcome: &Result<ServiceResponse<B>, actix_web::Error>,
    ) {
        // Capture the standard fields when the request finishes.
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
