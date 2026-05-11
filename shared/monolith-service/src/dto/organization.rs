use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::time::PlanningTimestamp;

/// Организация монолита которая должна приходить по дорожке
/// "/api/json/organization/search/"
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Organization {
    pub uuid: Uuid,
    pub id: i32,
    pub form_id: i16,
    pub code: String,
    pub country: String,
    pub inn: String,
    pub kpp: String,
    pub text: String,
    pub text_full: String,
    pub address_legal: String,
    pub address_fact: String,
    pub is_removed: Option<bool>,
    pub changed_by: Option<i32>,
    pub changed_at: Option<PlanningTimestamp>,
}
