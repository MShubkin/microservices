use std::error::Error;

use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, ResponseError};
use broker::BrokerError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use asez2_shared_db::result::SharedDbError;

use crate::presentation::dto::{
    error::{AsezErrorComplete, ErrorLevel, Level},
    response_request::{ApiResponse, Message, ResponseMessage},
    AsezError, AsezErrorDict,
};

pub type SchedulerResult<T> = Result<T, SchedulerError>;

/// Basic `Error` enum for Scheduler service
#[derive(Error, Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum SchedulerError {
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
    #[error("Ошибка взаимодействия с брокером: {0}")]
    Broker(String),
    #[error("Ошибка сериализации: {0}")]
    SerdeError(String),
    #[error("Ошибка БД: {0}")]
    DBError(String),
    #[error("Данные не найдены: {0}")]
    DataNotFound(String),
    #[error("Тело сообщения не валидное: `{0}`")]
    InvalidBody(String),
    #[error("Ошибка производственного календаря: {0}")]
    ProductionCalender(String),
}

impl AsezErrorComplete for SchedulerError {}

impl ResponseMessage for SchedulerError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            SchedulerError::Broker(err)
            | SchedulerError::SerdeError(err)
            | SchedulerError::DataNotFound(err)
            | SchedulerError::DBError(err) => {
                vec![Message::stop(err)]
            }

            SchedulerError::Internal(err)
            | SchedulerError::InvalidBody(err)
            | SchedulerError::ProductionCalender(err) => {
                vec![Message::error(err)]
            }
        }
    }
}

impl ResponseError for SchedulerError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for SchedulerError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<SchedulerError> for AsezErrorDict {
    fn from(r: SchedulerError) -> Self {
        Self::Scheduler(r)
    }
}

impl From<sqlx::Error> for SchedulerError {
    fn from(e: sqlx::Error) -> Self {
        SchedulerError::DBError(format!("Ошибка Sqlx: {}", e))
    }
}

impl From<serde_json::Error> for SchedulerError {
    fn from(e: serde_json::Error) -> Self {
        SchedulerError::SerdeError(e.to_string())
    }
}

impl From<SharedDbError> for SchedulerError {
    fn from(e: SharedDbError) -> Self {
        SchedulerError::Internal(format!("Ошибка обращения к базе данных {}", e))
    }
}
impl From<BrokerError> for SchedulerError {
    fn from(e: BrokerError) -> Self {
        SchedulerError::Broker(format!(
            "Ошибка работы Брокера {:?}: {}",
            e.source(),
            e
        ))
    }
}

impl From<AsezError> for SchedulerError {
    fn from(e: AsezError) -> Self {
        SchedulerError::Internal(format!(
            "Внутренняя ошибка сервиса {}: {}",
            e.source(),
            e
        ))
    }
}
impl From<SchedulerError> for AsezError {
    fn from(e: SchedulerError) -> Self {
        AsezError::new(e)
    }
}
