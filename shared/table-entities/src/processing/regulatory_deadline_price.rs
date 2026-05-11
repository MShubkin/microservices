use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbItemExt;
use uuid::Uuid;

#[derive(Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "regulatory_deadline_price"]
#[item_aggr_insert]
pub struct RegulatoryDeadlinePrice {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub section: i32,
    pub field_id: Option<i32>,
    pub color_scheme_id: i32,
    pub type_criticality: i32,
    pub start_day: i32,
    pub end_day: i32,
    pub created_by: i32,
    pub created_at: AsezTimestamp,
    pub changed_by: i32,
    pub changed_at: AsezTimestamp,
    pub status: Option<bool>,
}
