//! Отвечает а объекты с таблицы `partner_agenda_protocol`.
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use shared_db_derive::DbItemExt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "partner_agenda_protocol"]
#[item_aggr_insert]
pub struct PartnerAgendaProtocol {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    pub item_uuid: Uuid,
    pub user_id: i32,
    pub user_email: String,
    pub is_present: bool,
    pub is_removed: bool,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
