use asez2_shared_db::db_item::{int_array::AsezArray, AsezDate};
use monolith_service::dto::time::PlanningTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DocumentApproverRep;

/// Данные профильного департамента, присылаемые сервисом планирования.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct PlanningDocumentApprover {
    pub uuid: Uuid,
    pub department_id: i32,
    pub division_id: Option<i32>,
    pub planned_date: AsezDate,
    pub is_auto: bool,
    pub is_removed: bool,
    pub is_preapproved: bool,
    pub is_actual: bool,
    pub number: i32,
    pub expert_id: Option<i32>,
    pub route_id: Option<Vec<i64>>,
    pub send_date_1: Option<PlanningTimestamp>,
    pub send_users_1: Option<AsezArray<i32>>,
    pub created_by: i32,
    pub created_at: PlanningTimestamp,
}

impl PlanningDocumentApprover {
    pub fn to_document_approver_rep(
        self,
        plan_id: Option<i64>,
        document_uuid: Option<Uuid>,
    ) -> DocumentApproverRep {
        let PlanningDocumentApprover {
            uuid,
            department_id,
            division_id,
            planned_date,
            is_auto,
            is_removed,
            is_preapproved,
            is_actual,
            route_id,
            number,
            expert_id,
            send_date_1,
            send_users_1,
            created_by,
            created_at: _,
        } = self;
        DocumentApproverRep {
            uuid: Some(uuid),
            department_id: Some(department_id),
            division_id: Some(division_id),
            planned_date: Some(planned_date),
            is_auto: Some(is_auto),
            route_id: route_id.map(AsezArray),
            is_removed: Some(is_removed),
            is_preapproved: Some(is_preapproved),
            is_actual: Some(is_actual),
            number: Some(number),
            expert_id: Some(expert_id),
            created_by: Some(created_by),
            document_uuid,
            plan_id,
            send_date_1: Some(send_date_1.map(Into::into)),
            send_users_1,
            ..Default::default()
        }
    }
}
