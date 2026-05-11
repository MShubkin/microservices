use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, ResponseError};
use asez2_tables::test_setup::TestSetupError;
use monolith_service::http::error::MonolithHttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use asez2_shared_db::result::SharedDbError;
use broker::BrokerError;

use crate::application::records;
use crate::domain::enums::master_data::DirectoryType;
use crate::presentation::dto::error::{AsezErrorComplete, ErrorLevel, Level};
use crate::presentation::dto::response_request::{
    ApiResponse, Message, Messages, ResponseMessage,
};
use crate::presentation::dto::{AsezError, AsezErrorDict};

pub type MasterDataResult<T> = std::result::Result<T, MasterDataError>;

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum MasterDataError {
    #[error("Внутренняя Ошибка: {0}")]
    InternalError(String),
    #[error("Бизнес ошибки: {0:?}")]
    Business(Messages),
    #[error("Record not found in directory: {0}. Record data: {1}")]
    RecordNotFoundInDirectory(DirectoryType, String),
    #[error(
        "Невозможно добавить избранную запись. Указан некорректный справочник (идентификатор {0})"
    )]
    FavoritesInvalidDictionary(i32),
    #[error("Ошибка при работе с маршрутами: {0}")]
    RouteError(String),
    #[error("Ошибка HTTP запроса к Сервису планирования: {0}")]
    Monolith(String),
}

impl AsezErrorComplete for MasterDataError {}

impl ResponseMessage for MasterDataError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            Self::InternalError(err) => vec![Message::stop(format!("Произошла внутренняя ошибка. Обратитесь к администратору: {}", err))],
            Self::Business(msgs) => msgs.messages.to_owned(),
            Self::RecordNotFoundInDirectory(dir_ty, record) => vec![Message::error(format!("Запись не была найдена в директории {}, данные: {}", dir_ty, record))],
            Self::FavoritesInvalidDictionary(id) => vec![Message::error(format!("Невозможно добавить избранную запись. Указан некорректный справочник (идентификатор {0})", id))],
            Self::Monolith(err) => vec![Message::stop(format!("Ошибка при взаимодействии с АСЭЗ 1.0: {}", err))],
            Self::RouteError(_) => vec![Message::stop(self.to_string())],
        }
    }
}

impl ResponseError for MasterDataError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for MasterDataError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<MasterDataError> for AsezErrorDict {
    fn from(r: MasterDataError) -> Self {
        Self::MasterData(r)
    }
}

impl From<MasterDataError> for AsezError {
    fn from(r: MasterDataError) -> Self {
        AsezError::new(r)
    }
}

impl From<uuid::Error> for MasterDataError {
    fn from(e: uuid::Error) -> Self {
        MasterDataError::InternalError(format!("Uuid Error: {}", e))
    }
}

impl From<MonolithHttpError> for MasterDataError {
    fn from(e: MonolithHttpError) -> Self {
        MasterDataError::InternalError(format!(
            "Ошибка при взаимодействии с монолитом: {}",
            e
        ))
    }
}

impl From<SharedDbError> for MasterDataError {
    fn from(e: SharedDbError) -> Self {
        MasterDataError::InternalError(format!("Shared Db Error: {}", e))
    }
}

impl From<BrokerError> for MasterDataError {
    fn from(e: BrokerError) -> Self {
        MasterDataError::InternalError(format!("Broker Error: {}", e))
    }
}

impl From<sqlx::Error> for MasterDataError {
    fn from(e: sqlx::Error) -> Self {
        MasterDataError::InternalError(format!("Sqlx Error: {}", e))
    }
}
impl From<std::io::Error> for MasterDataError {
    fn from(e: std::io::Error) -> Self {
        MasterDataError::InternalError(format!("IO Error: {}", e))
    }
}
impl From<tokio::task::JoinError> for MasterDataError {
    fn from(e: tokio::task::JoinError) -> Self {
        MasterDataError::InternalError(format!("Join Error: {}", e))
    }
}

impl From<records::Error> for MasterDataError {
    fn from(error: records::Error) -> Self {
        use records::Error::*;
        match error {
            DbError(error) => Self::InternalError(error.to_string()),
            Rules(_, messages) | UpdateFailed(_, messages) => {
                Self::Business(messages)
            }
            StatusError(error) => Self::Business(
                Messages::default().with_message(Message::error(error.to_string())),
            ),
        }
    }
}

impl TestSetupError for MasterDataError {}
