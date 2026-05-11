use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbItemExt;
use uuid::Uuid;

#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, DbItemExt)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "status_history"]
#[item_aggr_insert]
pub struct StatusHistory {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub object_uuid: Uuid,
    pub status_id: i16,
    pub comment: String,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
}

impl StatusHistory {
    pub fn new<T: Into<i16>>(
        object_uuid: Uuid,
        status_id: T,
        comment: &str,
        created_by: i32,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            object_uuid,
            status_id: status_id.into(),
            comment: comment.to_owned(),
            created_at: AsezTimestamp::now(),
            created_by,
        }
    }
}
