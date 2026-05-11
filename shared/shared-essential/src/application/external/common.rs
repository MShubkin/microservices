use super::common_lookup_cfg::CommonLookupCfg;
use super::db_enum_cfg::EnumLookupCfg;
use super::enrichment::Id;
use super::id_lookup_cfg::IdLookupCfg;
use super::planning_masterdata::MultipleRequest;
use actix_web::cookie::Cookie;
use actix_web::http;
use ahash::AHashMap;

use env_setup::EnvError;
use reqwest::redirect::Policy;
use reqwest::{Client, Error, Response, Url};

use super::monolith::DictionaryKind;
use crate::presentation::dto::response_request::{ApiResponse, ApiResponseData};

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("ошибка конфигурации переменных среды: {0}")]
    Env(#[from] EnvError),
    #[error("ошибка парсинга URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("ошибка запроса к внешнему серверу: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("запрос к внешнему серверу вернул ошибку: {0}")]
    Status(String),
    #[error("запрос к внешнему серверу вернул данные в неверном формате: {0}")]
    Format(String),
    #[error("не найдено: {0}")]
    NotFound(&'static str),
}

pub type IntegrationResult<T> = Result<T, IntegrationError>;

/// Конфигурация источников справочных данных
#[derive(Debug, Default)]
pub struct MasterDataCfg {
    /// Конфигурация на основе ручки'/<dictionary>/search_by_id/`
    pub id_lookup_config: IdLookupCfg,
    /// Конфигурация на основе ручки '/get_updates/0/`
    pub common_lookup_config: CommonLookupCfg,
    /// Конфигурация на основе db-энумов
    pub enum_lookup_config: EnumLookupCfg,
}

pub type LookupDataIdMap = AHashMap<LookupRecordId, LookupRecordData>;
pub type LookupDataFieldMap = AHashMap<String, LookupDataIdMap>;
pub type LookupDataDictionaryKindMap = AHashMap<DictionaryKind, LookupDataIdMap>;

/// Идентификатор записи справочника
/// Для справочника "Статусы" используется составной идентификатор: id+scheme_id
#[derive(Debug, Hash, Eq, PartialEq, Default, Clone)]
pub struct LookupRecordId {
    pub id: i32,
    pub second_id: i32,
}
impl LookupRecordId {
    pub fn new(id: i32, second_id: i32) -> Self {
        LookupRecordId { id, second_id }
    }
    pub fn with_id(id: i32) -> Self {
        LookupRecordId {
            id,
            ..Default::default()
        }
    }
}

/// Справочные данные
#[derive(Debug, Default)]
pub struct LookupData {
    /// Справочные данные по ручке '/<dictionary>/search_by_id/`
    pub id_lookup_data: LookupDataFieldMap,
    /// Справочные данные по ручке '/get_updates/0/`
    pub common_lookup_data: LookupDataDictionaryKindMap,
    /// Справочные данные из db-энумов
    pub enum_lookup_data: LookupDataFieldMap,
}
/// Данные записи из справочника
#[derive(Debug, Default, Clone)]
pub struct LookupRecordData {
    pub dictionary_kind: DictionaryKind,
    pub id: i32,
    pub parent_id: i32,
    pub text: String,
    pub code: String,
}

pub(super) enum RequestType {
    SearchById(Vec<Id>),
    Multiple(MultipleRequest),
}

pub(super) fn make_reqwest_client() -> IntegrationResult<Client> {
    Client::builder()
        .tls_built_in_root_certs(true)
        .danger_accept_invalid_certs(true)
        .redirect(Policy::default())
        .https_only(false)
        .build()
        .map_err(IntegrationError::from)
}

/// POST запрос в монолит или НСИ (MDS - MasterDataService)
pub(super) async fn query_data<R: ApiResponseData>(
    client: &Client,
    request_url: Url,
    request_type: RequestType,
    user_id: i32,
    token: &str,
) -> IntegrationResult<R> {
    let mut builder = client
        .post(request_url.clone())
        .query(&[("user_id", user_id.to_string())])
        .header(http::header::COOKIE, Cookie::new("id", token).to_string());

    match request_type {
        RequestType::SearchById(value) => {
            builder = builder.json(&value);
        }
        RequestType::Multiple(value) => {
            builder = builder.json(&value);
        }
    }
    let result = builder.send().await;
    process_result(request_url, result).await
}

pub async fn process_result<R>(
    request_url: Url,
    result: Result<Response, Error>,
) -> IntegrationResult<R>
where
    R: ApiResponseData,
{
    let host = request_url.host_str().unwrap_or("<unknown host name>");
    let response: Response = match result {
        Ok(response) => response,
        Err(err) => {
            let message = format!(
                "Failed to send request to external API {}: {} [{}]",
                host,
                err,
                err.status().unwrap_or_default()
            );
            trace_error(message.as_str());
            return Err(err.into());
        }
    };
    if !response.status().is_success() {
        let message = format!(
            "External API return error for {}: {}",
            host,
            response.status()
        );
        trace_error(message.as_str());
        Err(IntegrationError::Status(message))
    } else {
        match response.json::<ApiResponse<R, ()>>().await {
            Ok(data) => Ok(data.data),
            Err(err) => {
                let message =
                    format!("Failed to parse response data from {}: {}", host, err);
                trace_error(message.as_str());
                Err(IntegrationError::Format(message))
            }
        }
    }
}

fn trace_error(error: &str) {
    tracing::error!(kind = "integration", error = error);
}
