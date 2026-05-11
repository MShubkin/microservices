use super::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    response_request::{Message, ResponseMessage},
};
use crate::presentation::dto::response_request::ApiResponse;
use actix_web::{
    error::ResponseError, http::header::ContentType, http::StatusCode, HttpResponse,
};
use ahash::AHashMap;
use broker::BrokerError;

use crate::presentation::dto::response_request::ApiResponseData;
use crate::presentation::dto::AsezError;
use asez2_tables::{
    test_setup::TestSetupError,
    view_storage::notification::{
        Notification, NotificationType, UserNotificationSettings,
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthCheckReq {
    pub user_id: String,
    pub token: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthCheckRes {
    pub user_code: String,
    pub is_authorised: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllowedFieldsRequest {
    pub workplace_id: String,
    pub section_id: String,
    pub user_id: String, // useless now but will be needed later 100%
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AllowedFieldsResponse {
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckPermissionRequest {
    pub user_id: String,
    pub workplace_id: String,
    pub section_id: String,
    pub action_id: String,
}

/// Получить права и секции
#[derive(Debug, Deserialize, Serialize)]
pub struct PricingSectionUserReq {
    pub user_id: i32,
}

/// Запрос к view-storage на получение или обновление данных
/// по уведомлениями
#[derive(Debug, Deserialize, Serialize)]
pub enum NotificationStorageReq {
    /// Получение уведомления по его типу(айди)
    ///
    /// Возвращает [`GetNotificationByTypeResponse`]
    GetByType(GetNotificationsByType),
}

/// Получение информации по уведомлению по его типу
#[derive(Debug, Deserialize, Serialize)]
pub struct GetNotificationsByType {
    /// Уведомления, по которым нужно получить информацию для генерации сообщения
    pub types: Vec<NotificationType>,
    /// Пользователи, которым отправляется уведомление
    pub user_ids: Vec<i32>,
}
/// Ответ на [`GetNotificationByType`]
#[derive(Debug, Deserialize, Serialize)]
pub struct GetNotificationsByTypeResponse {
    /// Информация для генерации уведомлений
    pub notifications: AHashMap<NotificationType, Notification>,
    /// Настройки пользователей, которым отправляется уведомление
    pub settings: AHashMap<i32, UserNotificationSettings>,
}

#[derive(
    sqlx::FromRow, Clone, Debug, Serialize, Deserialize, Default, PartialEq,
)]
pub struct SectionUserResponse {
    pub env_type_id: Vec<String>,
    pub section_id: Vec<String>,
    pub type_operation_id: Vec<String>,
    pub option_id: Vec<String>,
    pub element_operation_id: Vec<String>,
}

impl ApiResponseData for SectionUserResponse {}

pub type PricingSectionUserResponse = ApiResponse<SectionUserResponse, ()>;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ViewStorageError {
    #[error("Database error: {0}")]
    DBError(String),
    #[error("Ошибка сериализации: {0}")]
    SerdeError(String),
    #[error("Ошибка УИД: {0}")]
    UuidError(String),
    #[error("Строка в БД не найдена")]
    RowNotFound,
    #[error("Ошибка при взаимодействии с брокером сообщений: {0}")]
    BrokerError(String),
    #[error("Ошибка сервиса: {0}")]
    BusinessError(String),
    #[error("Ошибка входных данных: {0}")]
    Input(String),
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
}

impl TestSetupError for ViewStorageError {}

impl ResponseError for ViewStorageError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ViewStorageError::RowNotFound => StatusCode::NOT_FOUND,
            ViewStorageError::UuidError(_) | ViewStorageError::Input(_) => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl AsezErrorComplete for ViewStorageError {}

impl ResponseMessage for ViewStorageError {
    fn message_response(&self) -> Vec<Message> {
        use ViewStorageError::*;
        match self {
            RowNotFound => {
                vec![Message::stop("Запись не найдена")]
            }
            Input(err) => {
                vec![Message::stop(format!("Неверный запрос к сервису: {}", err))]
            }
            Internal(err) => {
                vec![Message::stop(format!("Внутренняя ошибка сервиса: {}", err))]
            }
            DBError(e) => vec![Message::stop(format!("Database error: {e}"))],
            SerdeError(e) => {
                vec![Message::stop(format!("Ошибка сериализации: {e}"))]
            }
            UuidError(e) => vec![Message::stop(format!("Ошибка УИД: {e}"))],
            BrokerError(e) => vec![Message::stop(format!(
                "Ошибка при взаимодействии с брокером сообщений: {e}"
            ))],
            BusinessError(e) => vec![Message::stop(format!("Ошибка сервиса: {e}"))],
        }
    }
}

impl ErrorLevel for ViewStorageError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<ViewStorageError> for AsezErrorDict {
    fn from(r: ViewStorageError) -> Self {
        Self::ViewStorage(r)
    }
}

impl From<sqlx::Error> for ViewStorageError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ViewStorageError::RowNotFound,
            x => ViewStorageError::DBError(x.to_string()),
        }
    }
}

impl From<ViewStorageError> for AsezError {
    fn from(x: ViewStorageError) -> Self {
        AsezError::new(x)
    }
}

impl From<BrokerError> for ViewStorageError {
    fn from(err: BrokerError) -> Self {
        Self::BrokerError(err.to_string())
    }
}

impl From<std::io::Error> for ViewStorageError {
    fn from(err: std::io::Error) -> Self {
        ViewStorageError::BusinessError(err.to_string())
    }
}

impl From<tokio::task::JoinError> for ViewStorageError {
    fn from(err: tokio::task::JoinError) -> Self {
        ViewStorageError::BusinessError(err.to_string())
    }
}

impl From<serde_json::Error> for ViewStorageError {
    fn from(e: serde_json::Error) -> Self {
        ViewStorageError::SerdeError(e.to_string())
    }
}

impl From<uuid::Error> for ViewStorageError {
    fn from(e: uuid::Error) -> Self {
        ViewStorageError::UuidError(e.to_string())
    }
}

impl From<asez2_shared_db::result::SharedDbError> for ViewStorageError {
    fn from(err: asez2_shared_db::result::SharedDbError) -> Self {
        ViewStorageError::Internal(err.to_string())
    }
}
