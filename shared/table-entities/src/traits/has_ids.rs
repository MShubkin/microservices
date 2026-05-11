/// Для того чтобы можно было обобщать объекты со статусами.
pub trait HasId {
    fn id(&self) -> i64;
    fn set_id(&mut self, status: i64);
}

macro_rules! impl_has_id {
    ($entity:ty, $field:ident) => {
        impl HasId for $entity {
            fn id(&self) -> i64 {
                self.$field
            }
            fn set_id(&mut self, x: i64) {
                self.$field = x;
            }
        }
    };
}

impl_has_id!(crate::Plan, id);
impl_has_id!(crate::ContractAmendment, id);
