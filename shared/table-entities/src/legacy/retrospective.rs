use crate::legacy::plans::PlanStatus;

use crate::PlanRetrospectiveRep;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct PlanRetrospectiveLegacy {
    pub active_uuid: Uuid,
    pub plan_id: String,
    pub year: i16,
    pub status_id: PlanStatus,
    pub is_removed: bool,
}

impl PlanRetrospectiveLegacy {
    pub fn to_plan_retrospective_rep(
        self,
        id: Option<i64>,
        uuid: Option<Uuid>,
    ) -> PlanRetrospectiveRep {
        PlanRetrospectiveRep {
            plan_uuid: uuid,
            plan_id: id,
            plan_year: Some(self.year),
            plan_status: Some(self.status_id),
            id_ly: Some(self.plan_id.parse::<i64>().expect("id_ly is a number")),
            uuid_ly: Some(self.active_uuid),
            is_removed: Some(self.is_removed),
            ..Default::default()
        }
    }
}
