//! Тут осуществлён HasSourceUuid.
use uuid::Uuid;

/// Для того чтобы можно было обобщать объекты со статусами.
pub trait HasSourceUuid {
    fn source_uuid(&self) -> Uuid;
    fn set_source_uuid(&mut self, is_actual: Uuid);
}

macro_rules! impl_has_source_uuid {
    ($entity:ty, $field:ident) => {
        impl HasSourceUuid for $entity {
            #[allow(clippy::misnamed_getters)]
            fn source_uuid(&self) -> Uuid {
                self.$field
            }
            #[allow(clippy::misnamed_getters)]
            fn set_source_uuid(&mut self, uuid: Uuid) {
                self.$field = uuid;
            }
        }
    };
}
impl_has_source_uuid!(crate::PlanItem, plan_uuid);
impl_has_source_uuid!(crate::PlanItemFull, plan_uuid);
impl_has_source_uuid!(crate::ContractAmendmentItem, header_uuid);
