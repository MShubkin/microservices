use actix_web::{http::header::ContentType, HttpResponse, ResponseError};
use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
use asez2_shared_db::result::SharedDbError;
use asez2_tables::maths::CurrencyValue;
use asez2_tables::PricingUnitId;
use monolith_service::http::error::MonolithHttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::application::validation::allowed_fields::ForbiddenFieldError;
use crate::domain::{
    AttachmentRep, EcPartnerRep, EcProtocolItemRep, EcProtocolRep,
};
use crate::presentation::dto::response_request::Message;
use crate::presentation::dto::{
    error::{AsezErrorComplete, ErrorLevel, Level},
    response_request::{
        ApiResponse, MessageKind, Messages, ResponseMessage, Status,
    },
    AsezError, AsezErrorDict,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
/// Request DTO for "/rest/estimated_commission/v1/update/protocol/"
pub struct UpdateProtocolReqWithUser {
    pub user_id: i32,
    pub header: EcProtocolRep,
    pub items: Vec<EcProtocolItemRep>,
    pub items_d647: Vec<EcProtocolItemRep>,
    pub partner_list: Vec<EcPartnerRep>,
    pub attachment_list: Vec<AttachmentRep>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
/// Request DTO for "/rest/estimated_commission/v1/update/agenda/"
pub struct UpdateAgendaReqWithUser {
    pub user: i32,
    pub header: UpdateAgendaHeader,
    pub items: Vec<UpdateAgendaItem>,
    pub items_d647: Vec<UpdateAgendaItem>,
    pub partner_list: Vec<EcPartnerRep>,
    pub attachment_list: Vec<AttachmentRep>,
}

/// Request DTO for "/rest/estimated_commission/v1/update/agenda/", agenda header.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UpdateAgendaHeader {
    pub id: i64,
    pub uuid: Uuid,
    pub meeting_date: Option<AsezDate>,
    pub pricing_organization_unit_id: Option<PricingUnitId>,
}

/// Request DTO for "/rest/estimated_commission/v1/update/agenda/", items/items_d647 element.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UpdateAgendaItem {
    pub uuid: Option<Uuid>,
    pub source_uuid: Uuid,
    pub is_excluded: bool,
    pub sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_sum_excluded_vat: Option<CurrencyValue>,
    pub reviewed_at: Option<AsezTimestamp>,
    pub is_removed: Option<bool>,
}

pub type EcResult<T> = Result<T, EcError>;

/// Базовая ошибка сервисы `Estimated Commission`
#[derive(Error, Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum EcError {
    #[error("Внутренняя ошибка: `{0}`")]
    Internal(String),
    #[error("Невалидное тело запроса: `{0}`")]
    InvalidBody(String),
    #[error("Пользователь не аутентифицирован")]
    Unauthenticated,
    #[error("Отсутствует id кука c токеном")]
    MissingToken,
    #[error("Пользователь запросил недоступные ему поля: {0:?}")]
    NotAllowedFields(Vec<String>),
}

impl From<ForbiddenFieldError> for EcError {
    fn from(ForbiddenFieldError(e): ForbiddenFieldError) -> Self {
        Self::NotAllowedFields(e)
    }
}

impl AsezErrorComplete for EcError {}

impl ResponseMessage for EcError {
    fn message_response(&self) -> Vec<Message> {
        match self {
            EcError::Internal(err) => {
                vec![Message::stop(format!(
                    "Внутренняя ошибка, попробуйте позже: {}",
                    err
                ))]
            }
            EcError::InvalidBody(field) => {
                vec![Message::error(format!("Неверно введено поле {}", field))]
            }
            EcError::Unauthenticated => {
                vec![Message::error("Пользователь не аутентифицирован")]
            }
            EcError::MissingToken => {
                vec![Message::error("Отсутствует id кука c токеном")]
            }
            EcError::NotAllowedFields(not_allowed_fields) => not_allowed_fields
                .iter()
                .map(|field| {
                    Message::error(format!("Поле {} недоступно вам", field))
                })
                .collect(),
        }
    }
}

impl ResponseError for EcError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let response: ApiResponse<(), ()> = ApiResponse {
            data: (),
            status: Status::Error,
            objects: vec![()],
            messages: Messages {
                messages: self.message_response(),
                kind: MessageKind::Error,
            },
        };
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for EcError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<MonolithHttpError> for EcError {
    fn from(value: MonolithHttpError) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<SharedDbError> for EcError {
    fn from(e: SharedDbError) -> Self {
        match e {
            SharedDbError::ValueError(err) => EcError::InvalidBody(err),
            _ => EcError::Internal(e.to_string()),
        }
    }
}

impl From<EcError> for AsezErrorDict {
    fn from(err: EcError) -> Self {
        Self::EstimatedCommission(err)
    }
}

impl From<EcError> for AsezError {
    fn from(err: EcError) -> Self {
        AsezError::new(err)
    }
}
