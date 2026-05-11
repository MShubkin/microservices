use crate::presentation::dto::response_request::ApiResponseData;
use asez2_tables::master_data::plan_reasons_cancel::PlanReasonCancelHeaderRep;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanReasonCancel {
    #[serde(flatten)]
    pub header: PlanReasonCancelHeaderRep,
    #[serde(rename = "customer_id")]
    pub customers: Vec<PlanReasonCancelCustomer>,
}

impl ApiResponseData for PlanReasonCancel {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "operator")]
#[serde(rename_all = "lowercase")]
pub enum PlanReasonCancelCustomer {
    All,
    In { filter_values: Vec<i32> },
}
