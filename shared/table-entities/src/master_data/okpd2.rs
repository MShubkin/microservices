use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbItem;

#[derive(Clone, Debug, Default, PartialEq, DbItem, Serialize, Deserialize)]
pub struct Okpd2 {
    #[item_field_pkey]
    pub id: i32,
    pub code: String,
    pub text: String,
    pub is_removed: bool,
    pub from_date: AsezDate,
    pub to_date: AsezDate,
    #[item_field_autogen]
    pub changed_at: AsezTimestamp,
}
