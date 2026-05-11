//! Отвечает а объекты с таблицы `department`.

use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "executor_method"]
#[item_manually_activate_fields]
#[item_aggr_insert]
pub struct ExecutorMethod {
    #[item_field_pkey]
    pub id: i16,
    pub value: String,
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

impl ExecutorMethod {
    fn activate_fields_manually(&mut self) {
        if self.changed_by == i32::default() {
            self.changed_by = self.created_by;
        }
    }
}
