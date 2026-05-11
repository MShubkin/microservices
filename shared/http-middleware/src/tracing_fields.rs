use std::{
    future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use actix_http::HttpMessage;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use igg_tracing::tracing_fields::AsezTracingFieldsCollection;

/// Middleware that injects AsezTracingFieldsCollection
/// for collecting common request fields for tracing in other services.
pub struct AsezTracingFields;

impl<S, B> Transform<S, ServiceRequest> for AsezTracingFields
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = S::Response;

    type Error = S::Error;
    type InitError = ();

    type Transform = AsezTracingFieldsService<S>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ready(Ok(AsezTracingFieldsService { service }))
    }
}

/// Service that injects instance of fields collection into the request.
pub struct AsezTracingFieldsService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AsezTracingFieldsService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = S::Response;

    type Error = Error;

    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let uri = req.uri();
        let user_agent = req
            .headers()
            .get("User-Agent")
            .map(|h| h.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_string();
        let source = req
            .connection_info()
            .realip_remote_addr()
            .and_then(|s| s.parse::<SocketAddr>().ok())
            .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0));
        let collection =
            AsezTracingFieldsCollection::new(uri.to_string(), source, user_agent);
        req.extensions_mut().insert(collection);
        let fut = self.service.call(req);
        Box::pin(fut)
    }
}
