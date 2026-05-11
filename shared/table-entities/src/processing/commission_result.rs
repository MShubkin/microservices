//! This is used in processing for Estimates Commission related work:
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};

use super::protocol_item::ResultId;

#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "estimated_commission_result"]
#[item_manually_activate_fields]
#[item_aggr_insert]
pub struct EsCommissionResult {
    #[item_field_pkey]
    /// TODO: Change to ENUM: This is done in Elisey's branch.
    pub id: ResultId,
    pub value: String,
    pub created_by: i32,
    pub changed_by: i32,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
}

impl EsCommissionResult {
    /// We set the date when we insert the item. The other fields
    /// MUST be set beforehand by the user or we MUST crash.
    fn activate_fields_manually(&mut self) {
        if self.changed_by == i32::default() {
            self.changed_by = self.created_by;
        }
    }
}
