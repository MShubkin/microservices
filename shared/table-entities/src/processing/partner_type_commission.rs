//! Отвечает а объекты с таблицы `object_route`.

use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbItemExt;
use uuid::Uuid;

use crate::ProtocolType;

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "partner_type_commission"]
#[item_aggr_insert]
pub struct PartnerTypeCommission {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub user_id: i32,
    #[adaptor_field_duplicate = "commission_role_id"]
    pub role_id: i16,
    pub protocol_type_id: ProtocolType,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
