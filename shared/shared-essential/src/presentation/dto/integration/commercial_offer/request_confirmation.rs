use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommercialOfferRequestConfirmationData {
    pub data: CommercialOfferRequestConfirmation,
    pub user_id: i32,
    /// Номер ЗЦИ
    pub id: i64,
}

/// Подтверждение доставки сообщения
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CommercialOfferRequestConfirmation {
    /// Идентификатор сообщения
    pub request_id: String,
    /// Cтатус обработки (success/error)
    pub status: String,
    /// Результат обработки
    pub message: String,
    /// Ошибки
    pub errors: Option<Errors>,
    /// Идентификатор объекта
    pub req_info: ReqInfo,
}

/// Ошибки
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Errors {
    pub error: Vec<Error>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Error {
    /// Код ошибки
    pub code: String,
    /// Описание ошибки
    pub message: String,
    /// Детальное описание ошибки
    pub details: Details,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Details {
    /// Описание
    pub field: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ReqInfo {
    /// Номер ЗЦИ
    pub req_number: String,
    /// UUID ЗЦИ
    #[serde(rename = "ReqUUID")]
    pub req_uuid: Option<Uuid>,
}
