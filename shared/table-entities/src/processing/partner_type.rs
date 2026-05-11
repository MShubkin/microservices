//! Отвечает а объекты с таблицы `partner_type`.
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};

use serde::{Deserialize, Serialize};

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "partner_type"]
#[item_aggr_insert]
pub struct PartnerType {
    #[item_field_pkey]
    pub user_id: i32,
    pub id: i64,
    pub type_id: i16,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
