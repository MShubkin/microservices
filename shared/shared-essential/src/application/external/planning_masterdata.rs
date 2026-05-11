// TODO: Возможно, что нужно вынести в `monolith-service`

use crate::presentation::dto::response_request::ApiResponseData;
use env_setup::PlanningRestCfg;
use monolith_service::dto::time::PlanningTimestamp;
use serde::{Deserialize, Serialize};

use super::common::RequestType;
use super::rest::PlanningRestConfig;
use super::IntegrationResult;

/// Вызов ручки /api/json/master_data/get_multiple/ в монолите
/// Выборка справочников из монолита одним запросом

pub async fn process_planning_multiple_request(
    request: MultipleRequest,
    user_id: i32,
    token: &str,
) -> IntegrationResult<MultipleResponse> {
    let client =
        PlanningRestConfig::from(PlanningRestCfg::from_env()?).get_client()?;

    let function_url = "/api/json/master_data/get_multiple/";

    let external_data = client
        .query_data::<MultipleResponse>(
            function_url,
            RequestType::Multiple(request),
            user_id,
            token,
        )
        .await?;
    Ok(external_data)
}

impl ApiResponseData for MultipleResponse {}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct MultipleRequest {
    #[serde(default)]
    pub user_ids: Vec<u32>,
    #[serde(default)]
    pub customer_ids: Vec<u32>,
    #[serde(default)]
    pub purchasing_method_ids: Vec<u8>,
    #[serde(default)]
    pub status_scheme_ids: Vec<u8>,
    #[serde(default)]
    pub unit_ids: Vec<u16>,
    #[serde(default)]
    pub currency_ids: Vec<u16>,
    #[serde(default)]
    pub vat_ids: Vec<u8>,
    #[serde(default)]
    pub document_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct MultipleResponse {
    pub users: Vec<User>,
    pub customers: Vec<Customer>,
    pub purchasing_methods: Vec<PurchasingMethod>,
    pub statuses: Vec<Status>,
    pub units: Vec<Unit>,
    pub currencies: Vec<Currency>,
    pub vats: Vec<Vat>,
    pub documents: Vec<Document>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct SingleSupplierReason {
    pub id: u8,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct BudgetItem {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct CustomerSupplier {
    pub id: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct Okpd2Code {
    pub id: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct Okved2Code {
    pub id: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct OkatoCode {
    pub id: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct Vat {
    pub id: u8,
    pub rate: u8,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct Currency {
    pub id: u16,
    pub code: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct Unit {
    pub id: u16,
    pub text: String,
    pub text_short: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct Status {
    pub scheme_id: u8,
    pub id: u8,
    pub parent_id: u8,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct PurchasingMethod {
    pub id: u8,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, derive_more::Display)]
#[display(fmt = "{text}")]
pub struct Customer {
    pub id: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct User {
    pub id: u32,
    pub initials_last_name: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Document {
    pub id: i64,
    pub status_id: u16,
    pub product_type_id: u16,
    pub items_number: u16,
    #[serde(default)]
    pub status_history: Vec<StatusHistory>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct StatusHistory {
    pub changed_at: PlanningTimestamp,
    pub status_id: u16,
    pub user_id: u32,
    pub note: String,
    pub single_supplier_decision_id: u16,
    pub voting_decision_id: u16,
}
