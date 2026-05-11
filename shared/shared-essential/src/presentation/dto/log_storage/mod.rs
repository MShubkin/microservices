use super::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    response_request::{Message, ResponseMessage},
    Source,
};
use actix_web::{
    error::ResponseError, http::header::ContentType, http::StatusCode, HttpResponse,
};
use broker::BrokerError;

use crate::presentation::dto::AsezError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogDataInsert {
    pub user_id: String, //Уникальный идентификатор пользователя
    pub event_id: i16,   //Уникальный идентификатор события в Системе
    pub request_id: Option<String>, //Для событий - запросов BE
    pub source_id: Source, //Идентификатор сервиса - источника
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum LogStorageError {
    #[error("Database error: {0}")]
    DBError(String),
    #[error("Serialization error: {0}")]
    SerdeError(String),
    #[error("Row not found")]
    RowNotFound,
    #[error("Ошибка при обращении к брокеру сообщений")]
    BrokerError(#[from] BrokerError),
}

impl ResponseError for LogStorageError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            LogStorageError::RowNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl AsezErrorComplete for LogStorageError {}

impl ResponseMessage for LogStorageError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            LogStorageError::RowNotFound => {
                vec![Message::stop("Запись не найдена")]
            }
            _ => vec![Message::stop("Ошибка сервиса хранения логов")],
        }
    }
}

impl ErrorLevel for LogStorageError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<LogStorageError> for AsezErrorDict {
    fn from(r: LogStorageError) -> Self {
        Self::LogStorage(r)
    }
}

impl From<serde_json::Error> for LogStorageError {
    fn from(e: serde_json::Error) -> Self {
        LogStorageError::SerdeError(e.to_string())
    }
}

impl From<sqlx::Error> for LogStorageError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => LogStorageError::RowNotFound,
            _ => LogStorageError::DBError(e.to_string()),
        }
    }
}

impl From<LogStorageError> for AsezError {
    fn from(x: LogStorageError) -> Self {
        AsezError::new(x)
    }
}
