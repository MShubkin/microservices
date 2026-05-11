use shared_essential::presentation::dto::processing::legacy_interaction::*;

use serde::{Deserialize, Serialize};

/// Запрос который приходит чисто от легаси-серви
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "request_kind", content = "request_data")]
pub enum LegacyReq {
    /// Обновить планы из монолита
    InsertUpdateLegacyPlans(InsertUpdateSrmPlansReq),
    InsertUpdateLegacyAmendments(InsertUpdateSrmAmendmentsReq),
}

/// Запрос который приходит чисто от легаси-серви
#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd)]
pub enum ProcessingToLegacyReq {
    /// Обновить планы из монолита
    UpdatePlans(InsertUpdateSrmPlansReq),
    UpdateAmendments(InsertUpdateSrmAmendmentsReq),
}

pub(crate) const SEND_TO_MONOLITH_QUEUE: &str = "plans_source";
