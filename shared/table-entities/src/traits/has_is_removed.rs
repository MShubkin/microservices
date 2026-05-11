//! Тут осуществлён HasIsRemoved.

/// Для того чтобы можно было обобщать объекты с полем is_removed.
pub trait HasIsRemoved {
    fn is_removed(&self) -> bool;
    fn set_is_removed(&mut self, r: bool);
}

macro_rules! impl_has_is_removed {
    ($entity:ty, $field:ident) => {
        impl HasIsRemoved for $entity {
            fn is_removed(&self) -> bool {
                self.$field
            }
            fn set_is_removed(&mut self, r: bool) {
                self.$field = r;
            }
        }
    };
}

impl_has_is_removed!(crate::Plan, is_removed);
impl_has_is_removed!(crate::ContractAmendment, is_removed);
impl_has_is_removed!(crate::PlanItemFull, is_removed);
impl_has_is_removed!(crate::ContractAmendmentItem, is_removed);
impl_has_is_removed!(crate::PlanRetrospective, is_removed);
