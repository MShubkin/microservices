use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbItem;
use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, DbItem, Serialize, Deserialize)]
pub struct BudgetItem {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub id: i16,
    pub parent_id: i16,
    pub code: String,
    pub from_date: AsezDate,
    pub to_date: AsezDate,
    pub text: String,
    pub is_removed: bool,
    pub group_id: i16,
    pub is_actual: bool,
    pub is_selectable: bool,
    pub sap_node_id: i16,
    pub sap_parent_node_id: i16,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
    pub created_by: i32,
}
