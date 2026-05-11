use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, ResponseError};
use broker::BrokerError;
use serde::{Deserialize, Serialize};
use std::string::FromUtf8Error;
use thiserror::Error;

use super::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    integration::commercial_offer::request::CommercialOfferData,
    response_request::{ApiResponse, Message, ResponseMessage},
    AsezError,
};
use monolith_service::http::error::MonolithHttpError;

pub mod commercial_offer;
pub mod documents;

pub type IntegResult<T> = Result<T, IntegError>;

#[derive(Error, Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum IntegError {
    #[error("Внутренняя Ошибка: {0}")]
    Internal(String),
    #[error("Invalid body was passed: `{0}`")]
    InvalidBody(String),
    #[error("Invalid path: `{0}`")]
    InvalidPath(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Network error: {0}")]
    Network(String),
    #[error("Invalid base64 content: {0}")]
    InvalidBase64(String),
    #[error("Отсутствуют приложенные документы: {0}")]
    MissingAttachments(String),
    #[error("Ошибки валидации")]
    Validation(Vec<(String, String)>),
    #[error("Ошибка: {0}")]
    Business(String),
}

impl AsezErrorComplete for IntegError {}

impl ResponseMessage for IntegError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            IntegError::Internal(err) => vec![Message::stop(format!(
                "Произошла внутренняя ошибка. Обратитесь к администратору: {}",
                err
            ))],
            IntegError::InvalidBody(err) => {
                vec![Message::error(format!("Неверно введено поле {}", err))]
            }
            IntegError::InvalidPath(err) => {
                vec![Message::error(format!("Неверно введен путь {}", err))]
            }
            IntegError::Unauthorized => {
                vec![Message::stop("Неавторизован".to_string())]
            }
            IntegError::Network(err) => {
                vec![Message::stop(format!("Ошибка соединения {}", err))]
            }
            IntegError::InvalidBase64(err) => {
                vec![Message::error(format!("Невалидный base64 {}", err))]
            }
            IntegError::MissingAttachments(msg) => vec![Message::error(format!(
                "Отсутствуют приложенные документы: {}",
                msg
            ))],
            IntegError::Validation(list) => list
                .iter()
                .map(|(field, msg)| {
                    Message::error(format!("Ошибка в поле {}: {}", field, msg))
                })
                .collect(),
            IntegError::Business(err) => vec![Message::stop(format!(
                "Произошла ошибка. Обратитесь к администратору: {}",
                err
            ))],
        }
    }
}

impl ResponseError for IntegError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for IntegError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<IntegError> for AsezErrorDict {
    fn from(r: IntegError) -> Self {
        Self::Integration(r)
    }
}

impl From<IntegError> for AsezError {
    fn from(err: IntegError) -> Self {
        AsezError::new(err)
    }
}

impl From<FromUtf8Error> for IntegError {
    fn from(err: FromUtf8Error) -> Self {
        IntegError::Internal(err.to_string())
    }
}

impl From<quick_xml::Error> for IntegError {
    fn from(err: quick_xml::Error) -> Self {
        IntegError::Internal(err.to_string())
    }
}

impl From<BrokerError> for IntegError {
    fn from(err: BrokerError) -> Self {
        IntegError::Internal(err.to_string())
    }
}

impl From<std::io::Error> for IntegError {
    fn from(err: std::io::Error) -> Self {
        IntegError::Internal(err.to_string())
    }
}

impl From<std::num::ParseIntError> for IntegError {
    fn from(err: std::num::ParseIntError) -> Self {
        IntegError::Internal(err.to_string())
    }
}

impl From<MonolithHttpError> for IntegError {
    fn from(err: MonolithHttpError) -> Self {
        Self::Internal(format!("Ошибка при взаимодействии с АСЭЗ 1.5: {}", err))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Ошибки валидации полей")]
    Fields(Vec<FieldError>),
}

#[derive(Debug, thiserror::Error)]
pub enum FieldError {
    #[error("Поле '{field}' должно быть заполнено")]
    Empty { field: &'static str },

    #[error("Поле '{field}' содержит некорректное значение: {reason}")]
    Invalid { field: String, reason: String },

    #[error("Поле '{field}' должно быть заполнено при условии {condition}")]
    Conditional {
        field: &'static str,
        condition: String,
    },

    #[error("Коллекция '{field}' не должна быть пустой")]
    EmptyCollection { field: &'static str },
}

impl FieldError {
    pub fn field(&self) -> &str {
        match self {
            FieldError::Empty { field }
            | FieldError::Conditional { field, .. }
            | FieldError::EmptyCollection { field } => field,
            FieldError::Invalid { field, .. } => field.as_str(),
        }
    }

    pub fn reason_string(&self) -> String {
        match self {
            FieldError::Empty { .. } => "Поле должно быть заполнено".into(),
            FieldError::EmptyCollection { .. } => {
                "Коллекция не должна быть пустой".into()
            }
            FieldError::Conditional { condition, .. } => {
                format!("Поле должно быть заполнено при условии {}", condition)
            }
            FieldError::Invalid { reason, .. } => reason.clone(),
        }
    }
}

impl From<ValidationError> for IntegError {
    fn from(error: ValidationError) -> Self {
        match error {
            ValidationError::Fields(errors) => IntegError::Validation(
                errors
                    .into_iter()
                    .map(|e| (e.field().to_string(), e.to_string()))
                    .collect(),
            ),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum IntegrationRequest {
    CommercialOfferRequestOut(CommercialOfferData),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommonResponse {
    pub success: bool,
    pub message: String,
}
