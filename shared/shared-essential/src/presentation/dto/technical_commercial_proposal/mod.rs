use std::fmt::Display;

use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, ResponseError};
use monolith_service::http::error::MonolithHttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use asez2_shared_db::result::SharedDbError;
use asez2_tables::Section;
use broker::BrokerError;

use crate::application::validation::allowed_fields::ForbiddenFieldError;
use crate::presentation::dto::error::{AsezErrorComplete, ErrorLevel, Level};
use crate::presentation::dto::integration::commercial_offer::{
    request_confirmation::CommercialOfferRequestConfirmationData,
    response::CommercialOfferResponseData,
};
use crate::presentation::dto::response_request::{
    ApiResponse, Message, ResponseMessage,
};
use crate::presentation::dto::AsezErrorDict;

use super::response_request::Messages;
use super::AsezError;

pub mod create_price_information_request;

pub type TcpResult<T> = Result<T, TcpError>;

/// Basic `Error` enum for `Technical commercial proposal` service
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum TcpError {
    #[error("Внутренняя Ошибка: {0}")]
    InternalError(String),
    #[error("Ошибка валидности данных: {0}")]
    InvalidData(TcpInvalidDataError),
    #[error("Запрос на запрещённые поля: {0:?}")]
    NotAllowedFields(Vec<String>),
    #[error("Бизнес ошибки: {0:#?}")]
    Business(Messages),
    #[error("Ошибка при взаимодействии с АСЭЗ 1.0: {0}")]
    MonolithError(String),
    #[error("Не найдена запись {0} из таблицы {1}")]
    RecordNotFound(String, String),
    #[error("Ошибка секций: {0}.")]
    Section(String),
    #[error("Ошибка во время экспорта: {0}.")]
    Export(String),
}

impl TcpError {
    pub fn not_found<S: ToString>(id: S, table: &str) -> Self {
        Self::RecordNotFound(id.to_string(), table.to_string())
    }
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum TcpInvalidDataError {
    #[error("Некорректный id типа зци: {0}")]
    InvalidPriceInformationRequestType(u8),
    #[error("Некорректный id статуса зци: {0}")]
    InvalidPriceInformationRequestStatus(u8),
    #[error("Некорректный id статуса поставщика: {0}")]
    InvalidSupplierStatus(u8),
    #[error("Некорректный id типа вложения: {0}")]
    InvalidAttachmentType(u8),
    #[error("Некорректные данные для выборки {0}")]
    InvalidSelectData(String),
    #[error("Значение не задано: {0}")]
    DataNotDefined(String),
}

impl AsezErrorComplete for TcpError {}

impl ResponseMessage for TcpError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            Self::Business(msgs) => msgs.messages.to_owned(),
            _ => vec![Message::stop(format!(
                "Произошла внутренняя ошибка. Обратитесь к администратору {}",
                self
            ))],
        }
    }
}

impl ResponseError for TcpError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for TcpError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<TcpError> for AsezErrorDict {
    fn from(r: TcpError) -> Self {
        Self::Tcp(r)
    }
}

impl From<TcpError> for AsezError {
    fn from(err: TcpError) -> Self {
        AsezError::new(err)
    }
}

impl From<MonolithHttpError> for TcpError {
    fn from(value: MonolithHttpError) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<uuid::Error> for TcpError {
    fn from(e: uuid::Error) -> Self {
        TcpError::InternalError(format!("Uuid Error: {}", e))
    }
}

impl From<SharedDbError> for TcpError {
    fn from(e: SharedDbError) -> Self {
        TcpError::InternalError(format!("Shared Db Error: {}", e))
    }
}

impl From<std::num::ParseIntError> for TcpError {
    fn from(e: std::num::ParseIntError) -> Self {
        TcpError::InternalError(e.to_string())
    }
}

impl From<BrokerError> for TcpError {
    fn from(e: BrokerError) -> Self {
        TcpError::InternalError(format!("Broker Error: {}", e))
    }
}

impl From<ForbiddenFieldError> for TcpError {
    fn from(ForbiddenFieldError(e): ForbiddenFieldError) -> Self {
        TcpError::NotAllowedFields(e)
    }
}

impl From<sqlx::Error> for TcpError {
    fn from(e: sqlx::Error) -> Self {
        TcpError::InternalError(format!("Sqlx Error: {}", e))
    }
}

impl From<std::io::Error> for TcpError {
    fn from(e: std::io::Error) -> Self {
        TcpError::InternalError(format!("IO Error: {}", e))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum UiSection {
    /// Список ЗЦИ
    RequestList,
    /// Список поставщиков
    CustomerList,
    /// Позиции ЗЦИ
    RequestItem,
    /// Позиции ТКП
    TkpPosition,
}

impl Display for UiSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UiSection::RequestList => "request_list",
            UiSection::CustomerList => "customer_list",
            UiSection::TkpPosition => "tkp_position",
            UiSection::RequestItem => "request_item",
        };
        write!(f, "{}", str)
    }
}

impl From<UiSection> for Section {
    fn from(s: UiSection) -> Self {
        match s {
            UiSection::RequestList => Self::TcpPriceRequestList,
            UiSection::CustomerList => Self::TcpPriceCustomerList,
            UiSection::TkpPosition => Self::TcpPosition,
            UiSection::RequestItem => Self::TcpRequestItem,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TcpDataAction {
    CommercialOfferRequestConfirmation(CommercialOfferRequestConfirmationData),
    CommercialOfferResponse(CommercialOfferResponseData),
    CommercialOfferAddDocResponse(i32),
}
