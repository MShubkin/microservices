use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbItem;
use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, DbItem, Serialize, Deserialize)]
pub struct Category {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub id: i16,
    pub parent_id: i16,
    pub code: String,
    pub text: String,
    pub gws_group: String,
    pub destination: i16,
    pub is_automatized: bool,
    pub is_non_assignable: bool,
    pub is_removed: bool,
    pub from_date: AsezDate,
    pub to_date: AsezDate,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
    pub created_by: i32,
}
