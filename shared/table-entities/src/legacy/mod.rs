//! Модуль с таблицами из АСЕЗ-1, с которыми требуется синхронизироваться.
use super::*;

pub mod amendment;
pub mod amendment_item;
pub mod plan_items;
pub mod plans;
pub mod retrospective;
pub mod specialized_departments;

pub use amendment::*;
pub use amendment_item::*;
pub use plan_items::*;
pub use plans::*;
pub use retrospective::*;
pub use specialized_departments::*;

fn make_none_if_default<T: Default + PartialEq>(d: T) -> Option<T> {
    if d != T::default() {
        Some(d)
    } else {
        None
    }
}
