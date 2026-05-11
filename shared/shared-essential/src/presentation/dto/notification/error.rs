use actix_web::http::header::ContentType;
use actix_web::{error::ResponseError, HttpResponse};
use broker::BrokerError;
use monolith_service::http::error::MonolithHttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::super::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    response_request::{ApiResponse, Message, ResponseMessage},
    AsezError,
};

pub type NotificationResult<T> = Result<T, NotificationError>;

/// Ошибка при обращении к `Notification` сервису
#[derive(Error, Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum NotificationError {
    #[error("Внутренняя ошибка: `{0}`")]
    Internal(String),
    #[error("Invalid body was passed: `{0}`")]
    InvalidBody(String),
    #[error("Не удалось отправить уведомление: {0}")]
    CannotSendNotification(String),
    #[error("Ошибка при взаимодействии с брокером сообщений: {0}")]
    BrokerError(#[from] BrokerError),
}

impl AsezErrorComplete for NotificationError {}

impl ResponseMessage for NotificationError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            NotificationError::InvalidBody(field) => {
                vec![Message::error(format!("Неверно введено поле {}", field))]
            }
            NotificationError::CannotSendNotification(e) => {
                vec![Message::error(format!(
                    "Ошибка при отправлении уведомления: {e}"
                ))]
            }
            NotificationError::Internal(err) => {
                vec![Message::error(format!(
                    "Внутренняя ошибка сервиса уведомлений: {}",
                    err
                ))]
            }
            NotificationError::BrokerError(e) => {
                vec![Message::stop(format!("Внутренняя ошибка: {e}"))]
            }
        }
    }
}

impl ResponseError for NotificationError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for NotificationError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<NotificationError> for AsezErrorDict {
    fn from(r: NotificationError) -> Self {
        Self::Notification(r)
    }
}

impl From<NotificationError> for AsezError {
    fn from(err: NotificationError) -> Self {
        AsezError::new(err)
    }
}

impl From<MonolithHttpError> for NotificationError {
    fn from(err: MonolithHttpError) -> Self {
        Self::Internal(format!("Ошибка при взаимодействии с АСЭЗ 1.0: {}", err))
    }
}
