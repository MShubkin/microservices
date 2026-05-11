use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecDepsAction {
    GetApproversForPlans(GetApproversForPlansReq),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetApproversForPlansReq {
    pub plan_ids: Vec<i64>,
    pub is_actual: Option<bool>,
}
