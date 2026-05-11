use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::time::PlanningTimestamp;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Category {
    pub uuid: Uuid,
    pub parent_uuid: Uuid,
    pub id: i32,
    pub parent_id: i32,
    pub code: String,
    pub text: String,
    pub is_automatized: bool,
    pub is_non_assignable: bool,
    pub is_removed: bool,
    pub created_at: PlanningTimestamp,
    pub created_by: i32,
    pub changed_at: Option<PlanningTimestamp>,
    pub changed_by: Option<i32>,
}
