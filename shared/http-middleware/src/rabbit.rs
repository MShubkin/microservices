use std::future;

use actix_http::HttpMessage;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use rabbit_services::properties::AsezRabbitProperties;

/// Middleware that inject default rabbit properties
/// to be used when calling remote services via Rabbit broker.
pub struct DefaultRabbitProperties;

impl<S, B> Transform<S, ServiceRequest> for DefaultRabbitProperties
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = S::Response;

    type Error = S::Error;
    type InitError = ();

    type Transform = DefaultRabbitPropertiesService<S>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ready(Ok(DefaultRabbitPropertiesService {
            service,
            properties: AsezRabbitProperties::default(),
        }))
    }
}

/// Service that injects instance of default rabbit properties into request.
pub struct DefaultRabbitPropertiesService<S> {
    service: S,
    properties: AsezRabbitProperties,
}

impl<S, B> Service<ServiceRequest> for DefaultRabbitPropertiesService<S>
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
        req.extensions_mut().insert(self.properties.clone());
        let fut = self.service.call(req);
        Box::pin(fut)
    }
}
