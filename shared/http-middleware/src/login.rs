//! Тут живёт middleware который проверяет куку/логин
use std::sync::Arc;

use actix_http::{
    body::{BoxBody, EitherBody, MessageBody},
    HttpMessage,
};
use actix_session::SessionExt;

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::ContentType,
    web::Query,
    FromRequest, HttpRequest, HttpResponse, ResponseError,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use igg_tracing::tracing_fields::{
    AsezTracingFieldsCollection, USER_ID, USER_NAME,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, field::display};
use tracing_actix_web::RootSpan;
use uuid::Uuid;

use shared_essential::presentation::dto::response_request::{
    ApiResponse, Message, Messages,
};
use shared_essential::presentation::dto::view_storage::AuthCheckRes;

use rabbit_services::{
    properties::AsezRabbitProperties, view_storage::ViewStorageService,
};

pub(crate) const AUTH_WITHOUT_COOKIE: &str = "AUTH_WITHOUT_COOKIE";

/// "Сырой материал для конструкции `AsezSessionTransform`.
/// Как раз это передаётся в `wrap`, когда строится сервис. Например:
/// ```ignore
/// HttpServer::new(move || {
///     App::new()
///         .wrap(AsezSessionWatcher::new(config.views_service()))
/// })
/// .bind(url)?
/// .run()
/// .await
/// ```
#[derive(Debug)]
pub struct AsezSessionWatcher;

/// Сервис (middleware), который отвечает за логин.
/// Это внутренний тип, который отвечает за осуществление.
#[derive(Debug)]
pub struct AsezSessionTransform<S: Service<ServiceRequest>> {
    /// Сам сервер
    service: Arc<S>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthQuery {
    user_id: i32,
}

impl<S, B> Transform<S, ServiceRequest> for AsezSessionWatcher
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AsezSessionTransform<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AsezSessionTransform {
            service: Arc::new(service),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for AsezSessionTransform<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<
        'static,
        Result<ServiceResponse<EitherBody<B>>, actix_web::Error>,
    >;

    actix_web::dev::forward_ready!(service);

    #[tracing::instrument(skip_all)]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        tracing::debug!(
            kind = "auth",
            "URL: {}, COOKIES: {:?}",
            req.uri(),
            req.cookies()
        );

        let s = self.service.clone();

        Box::pin(async move {
            let (http_req, _) = req.parts_mut();
            let user_id = check_query(http_req)?;

            // add user_id as a root_span field
            if let Some(root_span) = http_req.extensions().get::<RootSpan>() {
                root_span.record(USER_ID, &user_id);
            }
            // user_id should be recorded before view_storage is extracted
            if let Some(properties) =
                http_req.extensions_mut().get_mut::<AsezRabbitProperties>()
            {
                properties.set_user_id(user_id);
            }

            let view_storage = match ViewStorageService::extract(http_req).await {
                Ok(v) => v,
                Err(err) => {
                    error!(kind = "auth", error = %err, "не удается получить сервис view_storage");
                    return Err(err.into());
                }
            };

            let auth = match get_auth(&view_storage, user_id, http_req).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(
                        kind = "siem",
                        "[SIEM] Несуществующий пользователь {user_id}: {e}",
                    );
                    return Ok(req.error_response(e).map_into_right_body());
                }
            };

            if !auth.is_authorised {
                let user_id = user_id.to_string();
                tracing::warn!(
                    kind = "siem",
                    "[SIEM] Авторизация отклонена для пользователя {} (логин:{}).",
                    user_id,
                    auth.user_code,
                );
                return Ok(req
                    .error_response(SessionError::Permission(user_id))
                    .map_into_right_body());
            }

            // add user_name as a root_span field
            if let Some(root_span) = req.extensions().get::<RootSpan>() {
                root_span.record(USER_NAME, &display(&auth.user_code));
            }
            // store it as a field for futher tracing
            if let Some(fields) =
                req.extensions_mut().get_mut::<AsezTracingFieldsCollection>()
            {
                fields.set_user_name(auth.user_code);
            }

            let res = s.call(req).await?;

            Ok(res.map_into_left_body())
        })
    }
}

pub(crate) fn do_not_check_cookie_flag() -> bool {
    std::env::var(AUTH_WITHOUT_COOKIE)
        .ok()
        .and_then(|x| x.to_lowercase().parse::<bool>().ok())
        .unwrap_or_default()
}

#[tracing::instrument(skip_all)]
async fn get_auth(
    v: &ViewStorageService,
    user_id: i32,
    req: &HttpRequest,
) -> Result<AuthCheckRes, SessionError> {
    tracing::info!(kind = "auth", "get_auth: {:#?}", req);

    // If the server is not explicitly set to ignore cookies, it will not ignore them.
    if do_not_check_cookie_flag() {
        tracing::info!(kind = "auth", "Вход без аутентификации через куку");
        return v
            .check_session(user_id.to_string())
            .await
            .map_err(|x| SessionError::Other(x.to_string()));
    }

    // Изначально нам надо проверить, передал ли пользователь новую сессию или нет
    // Если не передал, то пробуем взять из стора
    let session_token = check_session_token(req)?;
    tracing::debug!(kind = "auth", "Session uuid: {}", session_token);

    v.check_session_with_token(user_id.to_string(), session_token)
        .await
        .map_err(|x| SessionError::Permission(x.to_string()))
}

pub(crate) fn check_query(req: &HttpRequest) -> Result<i32, SessionError> {
    match <Query<AuthQuery>>::from_query(req.query_string()) {
        Ok(user_id) => Ok(user_id.user_id),
        _ => Err(SessionError::InvalidFormat(String::from(
            "query параметры не содержат требуемый user_id",
        ))),
    }
}

/// Если мы используем токены сессии то мы немного по другому заходим.
pub(crate) fn check_session_token(req: &HttpRequest) -> Result<Uuid, SessionError> {
    let session = req.get_session();
    let item = session.get::<String>("session_id").map_err(|_| {
        SessionError::InvalidFormat(String::from("Не найден `session_id` в куке"))
    })?;

    match item {
        Some(i) => Uuid::parse_str(&i).map_err(|_| {
            SessionError::InvalidFormat(String::from(
                "Невалидный формат токена авторизации в id куке",
            ))
        }),
        None => Err(SessionError::InvalidFormat(String::from(
            "Отсутствует токен авторизации в id куке",
        ))),
    }
}

#[derive(Error, Debug, Serialize)]
pub enum SessionError {
    #[error("Ошибка аутентификации: {0}")]
    Other(String),
    #[error("Пользователь {0} не авторизован")]
    Permission(String),
    #[error("Неправильный формат запроса: {0}")]
    InvalidFormat(String),
}

type SessionResponse = ApiResponse<(), ()>;

impl ResponseError for SessionError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut messages = Messages::default();
        messages.add_prepared_message(Message::error(self.to_string()));

        let response: SessionResponse = ApiResponse {
            messages,
            ..Default::default()
        };

        HttpResponse::Unauthorized()
            .content_type(ContentType::json())
            .body(
                serde_json::to_string(&response)
                    .expect("Должно быть сериализовано"),
            )
            .map_into_boxed_body()
    }
}
