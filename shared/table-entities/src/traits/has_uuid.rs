use uuid::Uuid;

/// Для того чтобы можно было обобщать объекты со статусами.
pub trait HasUuid {
    fn uuid(&self) -> Uuid;
    fn set_uuid(&mut self, status: Uuid);
}

macro_rules! impl_has_uuid {
    ($entity:ty, $field:ident) => {
        impl HasUuid for $entity {
            fn uuid(&self) -> Uuid {
                self.$field
            }
            fn set_uuid(&mut self, x: Uuid) {
                self.$field = x;
            }
        }
    };
}

impl_has_uuid!(crate::Plan, uuid);
impl_has_uuid!(crate::PlanItemFull, uuid);
impl_has_uuid!(crate::PlanVersion, uuid);
impl_has_uuid!(crate::ContractAmendment, uuid);
impl_has_uuid!(crate::ContractAmendmentItem, uuid);
impl_has_uuid!(crate::ContractAmendmentVersion, uuid);
