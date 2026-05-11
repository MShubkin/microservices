use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::time::PlanningTimestamp;

// TODO: используется #[serde(default)] потому что монолит сам не знает,
// что хочет отдавать
#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub struct MonolithCustomer {
    pub id: i32,
    pub text: String,
    pub uuid: Uuid,
    pub budget_item_group_id: i32,
    pub iko: String,
    pub inn: String,
    pub is_under_sanctions: bool,
    pub is_1352: bool,
    pub is_ius_p: bool,
    pub is_not_in_asbu: bool,
    pub kind_id: i32,
    pub kpp: String,
    pub legal_address: String,
    pub nsi_code: String,
    pub ogrn: String,
    pub okato_id: i32,
    pub purchasing_policy_id: i32,
    pub sap_id: i32,
    pub text_short: String,
    pub is_removed: Option<bool>,
    pub created_by: Option<i32>,
    pub created_at: Option<PlanningTimestamp>,
    pub changed_by: Option<i32>,
    pub changed_at: Option<PlanningTimestamp>,
}
