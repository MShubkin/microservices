//! Отвечает а объекты с таблицы `estimated_commission_partner`.

use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem, DbUpsert};
use asez2_shared_db::impl_join_on;
use shared_db_derive::DbItemExt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::partner_type_commission::PartnerTypeCommission;

impl_join_on!(EcPartner:user_id => PartnerTypeCommission:user_id, aggr);

/// TODO: Investigate array in array to be able to use:
#[derive(
    Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, DbUpsert, PartialEq,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "estimated_commission_partner"]
#[item_aggr_insert]
pub struct EcPartner {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub protocol_agenda_uuid: Uuid,
    pub user_id: i32,
    pub e_mail: Option<String>,
    pub is_checked_in: bool,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
    // Доп. поля добавляются снизу пока мы не переделаем Joined в shared-db.
    #[adaptor_rename = "commission_role_id"]
    pub role_id: i16,
}
