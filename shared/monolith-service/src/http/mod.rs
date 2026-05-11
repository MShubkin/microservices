pub mod error;
pub mod monolith_token;
pub mod service;

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{header::COOKIE, multipart::Form, redirect::Policy, Client, Method};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{MonolithDriver, MonolithService};

use self::error::{MonolithHttpError, MonolithHttpResult};

/// Дефолтный таймаут для запросов к монолиту по HTTP
pub const MONOLITH_DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(3);
/// Увеличенный таймаут для мультипарт запросов к монолиту по HTTP
pub const MONOLITH_MULTIPART_HTTP_TIMEOUT: Duration = Duration::from_secs(15);
/// Кука с токеном пользователя
pub const MONOLITH_TOKEN_COOKIE: &str = "id";
/// Квери с user_id пользователя
pub const MONOLITH_USER_ID_QUERY: &str = "user_id";

/// Альяс для [`MonolithService<MonolithHttpDriver>`]
pub type MonolithHttpService = MonolithService<MonolithHttpDriver>;

/// HTTP-драйвер для обращения к монолиту
#[derive(Clone, Debug)]
pub struct MonolithHttpDriver {
    base_url: Url,
    client: Client,
}

/// Свойства для обращения к монолиту с помощью [`MonolithHttpDriver`]
#[derive(Debug, Default)]
pub struct MonolithHttpProperties {
    /// Путь эндпоинта
    pub path: String,
    /// Метод обращения к эндпоинту
    pub method: Method,
    /// Токен пользователя, которые используется в куках
    pub token: String,
    /// user_id пользователя, которые передается в квери
    pub user_id: Option<i32>,
}

impl MonolithHttpDriver {
    /// Создание базового драйвера на основе базового url
    ///
    /// Может вернуть [`MonolithHttpError::ClientConfig`] при ошибке
    /// создания HTTP-клиента
    pub fn basic_driver(base_url: Url) -> MonolithHttpResult<Self> {
        Ok(Self {
            base_url,
            client: Client::builder()
                .tls_built_in_root_certs(true)
                .danger_accept_invalid_certs(true)
                .redirect(Policy::default())
                .https_only(false)
                .build()
                .map_err(MonolithHttpError::ClientConfig)?,
        })
    }
}

#[async_trait]
impl MonolithDriver for MonolithHttpDriver {
    type Properties = MonolithHttpProperties;
    type Error = error::MonolithHttpError;

    async fn request<B, R>(
        &self,
        body: &B,
        props: Self::Properties,
    ) -> Result<R, Self::Error>
    where
        B: Serialize + Send + Sync,
        R: for<'a> Deserialize<'a>,
    {
        let path = self.base_url.join(&props.path)?;

        let builder = match props.method {
            Method::POST => self.client.post(path.clone()).json(&body),
            Method::GET => self.client.get(path.clone()),
            _ => {
                return Err(MonolithHttpError::Unavailable(String::from(
                    "Недопустимый метод для запроса",
                )))
            }
        };

        let mut builder = builder.timeout(MONOLITH_DEFAULT_HTTP_TIMEOUT).header(
            COOKIE,
            format!(
                "{name}={value}",
                name = MONOLITH_TOKEN_COOKIE,
                value = props.token
            ),
        );
        if let Some(user_id) = props.user_id {
            builder = builder.query(&[(MONOLITH_USER_ID_QUERY, user_id)])
        }

        let response = builder.send().await?.error_for_status().map_err(|e| {
            tracing::error!("Ошибка при отправке на \"{}\": {}", path, e);
            MonolithHttpError::InvalidResponse(e)
        })?;

        let content = response.json::<R>().await.map_err(|e| {
            tracing::error!(
                "Ошибка при получения ответа JSON от \"{}\": {}",
                path,
                e
            );
            MonolithHttpError::InvalidResponse(e)
        })?;

        Ok(content)
    }

    async fn request_blob<B>(
        &self,
        body: &B,
        props: Self::Properties,
    ) -> Result<Vec<u8>, Self::Error>
    where
        B: Serialize + Send + Sync,
    {
        let path = self.base_url.join(&props.path)?;

        let builder = match props.method {
            Method::POST => self.client.post(path.clone()).json(&body),
            Method::GET => self.client.get(path.clone()),
            _ => {
                return Err(MonolithHttpError::Unavailable(String::from(
                    "Недопустимый метод для запроса",
                )))
            }
        };

        let mut builder = builder.timeout(MONOLITH_DEFAULT_HTTP_TIMEOUT).header(
            COOKIE,
            format!(
                "{name}={value}",
                name = MONOLITH_TOKEN_COOKIE,
                value = props.token
            ),
        );

        if let Some(user_id) = props.user_id {
            builder = builder.query(&[(MONOLITH_USER_ID_QUERY, user_id)]);
        }

        let response = builder.send().await?.error_for_status().map_err(|e| {
            tracing::error!("Ошибка при отправке на \"{}\": {}", path, e);
            MonolithHttpError::InvalidResponse(e)
        })?;

        let bytes = response.bytes().await.map_err(|e| {
            tracing::error!("Ошибка при чтении файла \"{}\": {}", path, e);
            MonolithHttpError::InvalidResponse(e)
        })?;

        Ok(bytes.to_vec())
    }

    async fn request_multipart<R>(
        &self,
        form: Form,
        props: Self::Properties,
    ) -> Result<R, Self::Error>
    where
        R: for<'a> Deserialize<'a> + Send,
    {
        if props.method != Method::POST {
            return Err(MonolithHttpError::Unavailable(
                "Multipart запрос только через POST".to_string(),
            ));
        }

        let path = self.base_url.join(&props.path)?;

        let mut builder = self.client.post(path.clone()).multipart(form);
        builder = builder
            .timeout(MONOLITH_DEFAULT_HTTP_TIMEOUT)
            .header(COOKIE, format!("{}={}", MONOLITH_TOKEN_COOKIE, props.token))
            .header("Connection", "close");

        if let Some(user_id) = props.user_id {
            builder = builder.query(&[(MONOLITH_USER_ID_QUERY, user_id)]);
        }

        let response = builder.send().await?.error_for_status().map_err(|e| {
            tracing::error!("Ошибка при отправке multipart на \"{}\": {}", path, e);
            MonolithHttpError::InvalidResponse(e)
        })?;

        let content = response.json::<R>().await.map_err(|e| {
            tracing::error!(
                "Ошибка при получении JSON-ответа от \"{}\": {}",
                path,
                e
            );
            MonolithHttpError::InvalidResponse(e)
        })?;

        Ok(content)
    }
}

impl MonolithHttpProperties {
    pub fn new(path: &str, method: Method, token: String) -> Self {
        Self {
            path: path.to_owned(),
            method,
            token,
            user_id: None,
        }
    }

    /// Определение пути эндпоинта
    pub fn with_path(self, path: &str) -> Self {
        Self {
            path: path.to_owned(),
            ..self
        }
    }

    /// Определение метода обращения к эндпоинту
    pub fn with_method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    /// Определение токена пользователя
    pub fn with_token(self, token: String) -> Self {
        Self { token, ..self }
    }

    /// Определние айди пользователя
    pub fn with_user_id(self, user_id: i32) -> Self {
        Self {
            user_id: Some(user_id),
            ..self
        }
    }
}
