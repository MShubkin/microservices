use std::future;

use actix_http::HttpMessage;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use rabbit_services::properties::AsezRabbitProperties;

/// Middleware, которая кладёт `AsezRabbitProperties` в extensions каждого запроса.
///
/// `AsezRabbitProperties` — контейнер для метаданных, которые уйдут в AMQP-заголовки
/// при вызове других сервисов через RabbitMQ. Middleware создаёт пустой экземпляр,
/// а последующие middleware (в первую очередь `AsezSessionWatcher`) дополняют его
/// `user_id`, `user_name` и другими полями по мере прохождения запроса.
///
/// Должна быть зарегистрирована раньше всех остальных через `.wrap()`,
/// потому что в Actix wrap-и выполняются в обратном порядке.
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

/// Фактическая реализация middleware — клонирует дефолтные `AsezRabbitProperties`
/// в extensions запроса. `Clone` дешёвый: структура содержит только `Option<String>`
/// и числа, без Arc или аллокаций.
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
