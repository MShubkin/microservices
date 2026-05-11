//! Отвечает а объекты с таблицы `object_type`.
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};

use serde::{Deserialize, Serialize};

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "object_type"]
#[item_aggr_insert]
pub struct ProcessingObjectType {
    #[item_field_pkey]
    pub id: i16,
    pub sort_code: i64,
    pub value: i16,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
