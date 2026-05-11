use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::time::PlanningTimestamp;

/// Направления закупки
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct PurchasingTrend {
    pub uuid: Uuid,
    pub id: i32,
    pub text: String,
    pub is_removed: bool,
    pub created_at: PlanningTimestamp,
    pub created_by: i32,
    pub changed_at: Option<PlanningTimestamp>,
    pub changed_by: Option<i32>,
}
