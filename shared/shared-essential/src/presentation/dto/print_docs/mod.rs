//! Этот модуль содержит структуры шаблонов
use super::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    response_request::{ApiResponse, Message, ResponseMessage},
    AsezError,
};
use crate::presentation::dto::general::{
    DataRecords, InternalExportReq, InternalParseReq,
};
use actix_web::http::header::ContentType;
use actix_web::{error::ResponseError, HttpResponse};
use broker::BrokerError;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Local;
use thiserror::Error;

pub mod common;
pub mod docx;
pub mod xlsx;
use crate::presentation::dto::print_docs::docx::{
    EcProtocolReq, EsConclusionReq, EsSubpoenaReq,
};
use crate::presentation::dto::print_docs::xlsx::EsBulletinReq;
use crate::presentation::dto::print_docs::xlsx::PaPlansReq;

use crate::presentation::dto::print_docs::common::TemplateFormat;

/// Basic `Error` enum for PrintDoc service
#[derive(Error, Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum PrintDocError {
    #[error("Internal Error: {0}")]
    Internal(String),
    #[error("Invalid body was passed: `{0}`")]
    InvalidBody(String),
    #[error("Invalid format was passed: `{0}`")]
    UnsupportedFormat(String),
    #[error("local io error {0}")]
    Io(String),
    #[error("Error transmission message in broker {0}")]
    BrokerT(String),
}

impl AsezErrorComplete for PrintDocError {}

impl ResponseMessage for PrintDocError {
    fn message_response(&self) -> Vec<Message> {
        use PrintDocError::*;
        match self {
            Internal(ref x) => {
                vec![Message::stop(format!("Ошибка создания документа {}", x))]
            }
            InvalidBody(ref x) => {
                vec![Message::stop(format!(
                    "Неверно введено поле {}, ошибка входных данных",
                    x
                ))]
            }
            UnsupportedFormat(ref x) => {
                vec![Message::stop(format!("Ошибка ввода/вывода данных {}", x))]
            }
            Io(ref x) => {
                vec![Message::stop(format!("Ошибка ввода/вывода данных {}", x))]
            }
            BrokerT(ref x) => {
                vec![Message::stop(format!(
                    "Ошибка передачи сообщения через брокер {}",
                    x
                ))]
            }
        }
    }
}

impl ResponseError for PrintDocError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&response).expect("It serializes"))
    }
}

impl ErrorLevel for PrintDocError {
    fn error_level(&self) -> Level {
        Level::Stop
    }
}

impl From<PrintDocError> for AsezErrorDict {
    fn from(r: PrintDocError) -> Self {
        Self::PrintDocs(r)
    }
}

impl From<PrintDocError> for AsezError {
    fn from(r: PrintDocError) -> Self {
        AsezError::new(r)
    }
}

impl From<BrokerError> for PrintDocError {
    fn from(_: BrokerError) -> Self {
        Self::Internal(String::from("Внутренняя ошибка, обратитесь в поддержку"))
    }
}

/// Сообщение из кролика, содержащее информацию о создаваемом документе
#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub extension: TemplateFormat,
    pub confidentially: bool,
    pub input_content: PrintReq,
}

impl Content {
    pub fn make_result_file_name(&self) -> String {
        match &self.input_content {
            PrintReq::General(export_record) => {
                let timestamp = Local::now().format("%Y%m%d-%H%M%S");
                let name =
                    export_record.template.clone().unwrap_or("result".to_string());
                let extension = export_record.format.unwrap_or(self.extension);
                format!("{}-{}.{}", timestamp, name, extension.file_extension())
            }
            _ => {
                format!("result.{}", self.extension.file_extension())
            }
        }
    }
}

/// Сообщение сервису хранения
#[derive(Serialize, Deserialize, Debug)]
pub struct SaveDocSevRequest {
    pub created_doc: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveDocResponse {
    pub uuid_saved_doc: Option<usize>,
}

/// Ответ сервиса print-doc
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Response {
    pub result: Option<usize>,
    pub buf: Option<Vec<u8>>,
    pub data_records: Option<DataRecords>,
}

//todo оптимизировать
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::all)]
pub enum PrintReq {
    EsSubpoena(EsSubpoenaReq),
    EsBulletin(EsBulletinReq),
    EcProtocol(EcProtocolReq),
    EsConclusion(EsConclusionReq),
    PaPlans(PaPlansReq),
    General(InternalExportReq),
    XlsxExport(InternalExportReq),
    XlsxParse(InternalParseReq),
}
