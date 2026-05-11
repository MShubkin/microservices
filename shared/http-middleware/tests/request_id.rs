use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, Mutex},
};

use actix_web::{web, App, HttpServer};
use http_middleware::tracing_fields::AsezTracingFields;
use igg_tracing::ServiceRootSpanBuilder;
use tracing::{field::Visit, span, Id, Subscriber};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, Layer, Registry,
};

/// tracing layer используется для того, чтобы определять принятие подтверждения паблишером
#[derive(Debug, Default)]
struct TestLayer(Arc<Mutex<TraceState>>);

#[derive(Debug, Default)]
struct TraceState {
    marker_span: Option<Id>,
    request_ids: HashMap<Id, String>,
    asez_request_ids: HashMap<Id, String>,
}

impl TraceState {
    fn get_ids(&self) -> (Option<String>, Option<String>) {
        self.marker_span
            .as_ref()
            .map(|id| {
                (
                    self.request_ids.get(id).cloned(),
                    self.asez_request_ids.get(id).cloned(),
                )
            })
            .unwrap_or_default()
    }

    fn record(
        &mut self,
        id: &Id,
        request_id: Option<String>,
        asez_request_id: Option<String>,
    ) {
        if let Some(request_id) = request_id {
            self.request_ids.entry(id.clone()).or_insert(request_id);
        }
        if let Some(asez_request_id) = asez_request_id {
            self.asez_request_ids.entry(id.clone()).or_insert(asez_request_id);
        }
    }
}

#[derive(Debug, Default)]
struct TestVisitor {
    confirmation: bool,
    request_id: Option<String>,
    asez_request_id: Option<String>,
}

impl Visit for TestVisitor {
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        match field.name() {
            "request_id" => {
                self.request_id = Some(format!("{value:?}"));
            }
            "asez.request_id" => {
                self.asez_request_id = Some(format!("{value:?}"));
            }
            "message" => {
                self.confirmation = &format!("{value:?}") == "marker";
            }
            _ => {}
        }
    }
}

impl<S: Subscriber> Layer<S> for TestLayer {
    fn on_record(
        &self,
        id: &span::Id,
        values: &span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = TestVisitor::default();
        values.record(&mut visitor);
        let mut state = self.0.lock().unwrap();
        state.record(id, visitor.request_id, visitor.asez_request_id);
    }
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = TestVisitor::default();
        attrs.record(&mut visitor);
        let mut state = self.0.lock().unwrap();
        state.record(id, visitor.request_id, visitor.asez_request_id);
    }
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = TestVisitor::default();
        event.record(&mut visitor);
        if visitor.confirmation {
            self.0.lock().unwrap().marker_span = ctx.current_span().id().cloned();
        }
    }
}

#[tokio::test]
async fn scope() -> anyhow::Result<()> {
    let trace_state = Arc::new(Mutex::new(TraceState::default()));
    let test_layer = TestLayer(trace_state.clone());
    let filtered_layer = tracing_subscriber::fmt::layer()
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());

    Registry::default().with(filtered_layer).with(test_layer).init();

    let api_scope = |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::scope("v1")
                .wrap(TracingLogger::<ServiceRootSpanBuilder>::new())
                .wrap(AsezTracingFields)
                .service(
                    web::scope("api")
                        .route("/", web::get().to(logging_get_handler)),
                ),
        );
    };
    let server = HttpServer::new(move || App::new().configure(api_scope))
        .bind("127.0.0.1:0")?;
    let addr = server.addrs().pop().expect("address");
    let server = server.run();

    let request = reqwest::get(format!("http://{addr}/v1/api/"));

    tokio::select! {
        _ = server => unreachable!("should not happen"),
        res = request => res
    }?;

    let (rid, arid) = trace_state.lock().unwrap().get_ids();
    assert!(rid.is_some());
    assert_eq!(rid, arid);

    Ok(())
}

async fn logging_get_handler() -> Result<String, Infallible> {
    tracing::info!("marker");
    Ok("Ok".to_string())
}
