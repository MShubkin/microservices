//! Тут осуществлён HasPlanStatusId.
use crate::legacy::plans::PlanStatus;

/// Для того чтобы можно было обобщать объекты со статусами.
pub trait HasPlanStatusId {
    fn plan_status(&self) -> PlanStatus;
    fn set_plan_status(&mut self, status: PlanStatus);
}

macro_rules! impl_has_plan_status_id {
    ($entity:ty, $field:ident) => {
        impl HasPlanStatusId for $entity {
            fn plan_status(&self) -> PlanStatus {
                self.$field
            }
            fn set_plan_status(&mut self, status: PlanStatus) {
                self.$field = status;
            }
        }
    };
}

impl_has_plan_status_id!(crate::Plan, status_id);
impl_has_plan_status_id!(crate::ContractAmendment, status_id);
