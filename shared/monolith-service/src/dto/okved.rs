use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::time::PlanningTimestamp;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Okved {
    pub uuid: Uuid,
    pub id: i32,
    pub code: String,
    pub from_date: String,
    pub to_date: String,
    pub text: String,
    pub is_removed: bool,
    pub created_at: PlanningTimestamp,
    pub created_by: i32,
    pub changed_at: Option<PlanningTimestamp>,
    pub changed_by: Option<i32>,
}
