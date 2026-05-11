use crate::application::external::IntegrationError;
use crate::application::records;
use crate::application::validation::allowed_fields::ForbiddenFieldError;
use crate::presentation::dto::response_request::{ApiResponse, Messages};
use crate::presentation::dto::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    response_request::{Message, ResponseMessage},
    AsezError,
};
use actix_web::{
    error::ResponseError, http::header::ContentType, http::StatusCode, HttpResponse,
};
use asez2_shared_db::result::SharedDbError;
use asez2_tables::master_data::routes::TryFromRouteDataContentError;
use broker::BrokerError;
use monolith_service::http::error::MonolithHttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type SdResult<T> = std::result::Result<T, SpecDepsError>;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SpecDepsError {
    #[error("Ошибка сериализации: {0}")]
    SerdeError(String),
    #[error("Ошибка базы данных: {0}")]
    DbError(String),
    #[error("Ошибка обновления элементов базы данных: {0}")]
    RecordError(String),
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
    #[error("Ошибка при взаимодействии с АСЭЗ 1.0: {0}")]
    Monolith(String),
    #[error("Бизнес ошибки ПД: {0:?}")]
    Business(Messages),
    #[error("Невалидные входящие данные: {0}")]
    InvalidData(String),
    #[error("Пользователь не аутентифицирован")]
    Unauthenticated,
    #[error("Запрос на запрещённые поля: {0:?}")]
    NotAllowedFields(Vec<String>),
    #[error("Не удается получить ПД для пользователя")]
    NoDepartment,
    #[error("Не удается получить Управление для пользователя")]
    NoDivision,
    #[error("Не удается получить справочник Решений")]
    NoResponses,
    #[error("Ошибка маршрутов автоназначения: {0}")]
    Routes(#[from] RoutesError),
    #[error("Ошибка внешнего сервиса: {0}.")]
    Integration(String),
}

impl ResponseMessage for SpecDepsError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            SpecDepsError::Business(messages) => messages.messages.clone(),

            SpecDepsError::InvalidData(_) |
            SpecDepsError::Routes(_) |
            SpecDepsError::Unauthenticated |
            SpecDepsError::NotAllowedFields(_) |
            SpecDepsError::NoDepartment |
            SpecDepsError::NoDivision |
            SpecDepsError::NoResponses |
            SpecDepsError::Integration(_) => vec![Message::error(self.to_string())],

            SpecDepsError::SerdeError(_) |
            SpecDepsError::DbError(_) |
            SpecDepsError::RecordError(_) |
            SpecDepsError::Internal(_) |
            SpecDepsError::Monolith(_) => vec![Message::error(format!("Внутренняя ошибка инфраструктуры. Доложите администратору или повторите операцию позднее: {self}"))],
        }
    }
}

impl ResponseError for SpecDepsError {
    fn error_response(&self) -> HttpResponse {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .json(response)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotAllowedFields(_) | SpecDepsError::InvalidData(_) => {
                StatusCode::BAD_REQUEST
            }
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::SerdeError(_) | SpecDepsError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            _ => StatusCode::OK,
        }
    }
}

impl AsezErrorComplete for SpecDepsError {}

impl ErrorLevel for SpecDepsError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<SpecDepsError> for AsezErrorDict {
    fn from(err: SpecDepsError) -> Self {
        Self::SpecDeps(err)
    }
}

impl From<SpecDepsError> for AsezError {
    fn from(x: SpecDepsError) -> Self {
        AsezError::new(x)
    }
}

impl From<serde_json::Error> for SpecDepsError {
    fn from(e: serde_json::Error) -> Self {
        SpecDepsError::SerdeError(e.to_string())
    }
}

impl From<ForbiddenFieldError> for SpecDepsError {
    fn from(ForbiddenFieldError(e): ForbiddenFieldError) -> Self {
        Self::NotAllowedFields(e)
    }
}

impl From<IntegrationError> for SpecDepsError {
    fn from(error: IntegrationError) -> Self {
        SpecDepsError::Integration(error.to_string())
    }
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum RoutesError {
    #[error(transparent)]
    Data(#[from] TryFromRouteDataContentError),
}

impl From<RoutesError> for AsezError {
    fn from(value: RoutesError) -> Self {
        SpecDepsError::from(value).into()
    }
}

impl From<tokio::task::JoinError> for SpecDepsError {
    fn from(e: tokio::task::JoinError) -> Self {
        SpecDepsError::Internal(format!("Join Error: {}", e))
    }
}

impl From<SharedDbError> for SpecDepsError {
    fn from(error: SharedDbError) -> Self {
        SpecDepsError::DbError(error.to_string())
    }
}

impl From<records::Error> for SpecDepsError {
    fn from(error: records::Error) -> Self {
        SpecDepsError::RecordError(error.to_string())
    }
}

impl From<BrokerError> for SpecDepsError {
    fn from(e: BrokerError) -> Self {
        SpecDepsError::Internal(format!(
            "Ошибка при взаимодействии с брокером сообщений: {}",
            e
        ))
    }
}

impl From<MonolithHttpError> for SpecDepsError {
    fn from(e: MonolithHttpError) -> Self {
        SpecDepsError::Monolith(e.to_string())
    }
}
