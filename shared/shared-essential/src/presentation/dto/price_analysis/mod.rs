use actix_web::http::header::ContentType;
use actix_web::{error::ResponseError, HttpResponse};
use asez2_shared_db::result::SharedDbError;
use broker::BrokerError;
use monolith_service::http::error::MonolithHttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::application::validation::allowed_fields::ForbiddenFieldError;
use crate::presentation::dto::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    processing::price_analysis::CompleteLottingData,
    response_request::{ApiResponse, Message, ResponseMessage},
    AsezError,
};

pub type PaResult<T> = Result<T, PaError>;

/// Ошибка при общении с `Price Analysis` сервисом
#[derive(Error, Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum PaError {
    #[error("Бизнес ошибка: {0}")]
    Business(String),
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
    #[error("Невалидное тело запроса: `{0}`")]
    InvalidBody(String),
    #[error("Неверно введены поля {0}, причина: {1}")]
    InvalidField(String, String),
    #[error("Пользователь не аутентифицирован")]
    Unauthenticated,
    #[error("Пользователь не имеет права на данное действие")]
    Permission,
    #[error("Пользователь запросил недоступные ему поля: {0:?}")]
    NotAllowedFields(Vec<String>),
    #[error("Ошибка при автоконфигурации эксперта: {0}")]
    ExpertAutoConfiguration(String),
}

impl From<tokio::task::JoinError> for PaError {
    fn from(t: tokio::task::JoinError) -> PaError {
        PaError::Internal(format!("Internal Task error: {t}"))
    }
}

impl From<SharedDbError> for PaError {
    fn from(e: SharedDbError) -> Self {
        match e {
            SharedDbError::ValueError(err) => PaError::InvalidBody(err),
            _ => PaError::Internal(e.to_string()),
        }
    }
}

impl From<ForbiddenFieldError> for PaError {
    fn from(ForbiddenFieldError(e): ForbiddenFieldError) -> Self {
        Self::NotAllowedFields(e)
    }
}

impl AsezErrorComplete for PaError {}

impl ResponseMessage for PaError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            PaError::Business(x) => vec![Message::error(x)],
            PaError::Internal(x) => {
                vec![Message::stop(format!("Внутренняя ошибка: {x}"))]
            }
            PaError::InvalidBody(_) => {
                vec![Message::error(self)]
            }
            PaError::InvalidField(field, reason) => {
                vec![Message::error(format!(
                    "Неверно введено поле {}, причина: {}",
                    field, reason
                ))]
            }
            PaError::Unauthenticated => {
                vec![Message::error("Пользователь не аутентифицирован")]
            }
            PaError::NotAllowedFields(not_allowed_fields) => not_allowed_fields
                .iter()
                .map(|field| {
                    Message::error(format!("Поле {} недоступно вам", field))
                })
                .collect(),
            PaError::ExpertAutoConfiguration(err) => {
                vec![Message::error(format!(
                    "Ошибка при автоконфигурации эксперта: {}",
                    err
                ))]
            }
            PaError::Permission => vec![Message::error(String::from(
                r#"Отсутствуют полномочия для работы в модуле "Определение цены""#,
            ))],
        }
    }
}

impl ResponseError for PaError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for PaError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<PaError> for AsezError {
    fn from(err: PaError) -> Self {
        AsezError::new(err)
    }
}

impl From<MonolithHttpError> for PaError {
    fn from(value: MonolithHttpError) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<PaError> for AsezErrorDict {
    fn from(r: PaError) -> Self {
        Self::PriceAnalysis(r)
    }
}

impl From<BrokerError> for PaError {
    fn from(err: BrokerError) -> Self {
        Self::Internal(err.to_string())
    }
}

/// For "POST /rest/pricing/v1/action/complete_lotting/"
pub type CompleteLottingRes = ApiResponse<CompleteLottingData, ()>;
