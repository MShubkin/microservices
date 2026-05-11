//! Операции, общие для элементов повестки и протокола.

use uuid::Uuid;

use crate::{EcAgendaItem, EcProtocolItem};

pub trait AgendaProtocolItem {
    fn uuid(&self) -> Uuid;
    fn source_uuid(&self) -> Uuid;
    fn is_removed(&self) -> bool;
    fn is_excluded(&self) -> bool;
    fn is_registered_by_d647(&self) -> bool;
    fn number(&self) -> i64;
}

macro_rules! item_impl {
    ($ty:ty) => {
        impl AgendaProtocolItem for $ty {
            fn uuid(&self) -> Uuid {
                self.uuid
            }
            fn source_uuid(&self) -> Uuid {
                self.source_uuid
            }
            fn is_removed(&self) -> bool {
                self.is_removed
            }
            fn is_excluded(&self) -> bool {
                self.is_excluded
            }
            fn is_registered_by_d647(&self) -> bool {
                self.is_registered_by_d647
            }
            fn number(&self) -> i64 {
                self.number
            }
        }
    };
}

item_impl!(EcProtocolItem);
item_impl!(EcAgendaItem);
