//! Тут осуществлён HasIsActual.

/// Для того чтобы можно было обобщать объекты со статусами.
pub trait HasIsActual {
    fn is_actual(&self) -> bool;
    fn set_is_actual(&mut self, is_actual: bool);
}

macro_rules! impl_has_is_actual {
    ($entity:ty, $field:ident) => {
        impl HasIsActual for $entity {
            fn is_actual(&self) -> bool {
                self.$field
            }
            fn set_is_actual(&mut self, is_actual: bool) {
                self.$field = is_actual;
            }
        }
    };
}

impl_has_is_actual!(crate::Plan, is_actual);
impl_has_is_actual!(crate::ContractAmendment, is_actual);
