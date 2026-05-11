use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::time::PlanningTimestamp;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Vat {
    pub uuid: Uuid,
    pub id: i32,
    pub rate: i32,
    pub text: String,
    pub is_removed: Option<bool>,
    pub created_at: PlanningTimestamp,
    pub created_by: Option<i32>,
    pub changed_at: PlanningTimestamp,
    pub changed_by: Option<i32>,
}
