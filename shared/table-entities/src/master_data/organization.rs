use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "organization"]
pub struct Organization {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub id: i32,
    pub nsi_code: i32,
    pub inn: String,
    pub kpp: String,
    pub text: String,
    pub text_full: String,
    pub source: String,
    pub is_removed: bool,
    pub form_id: i16,
    pub ogrn: String,
    pub etp_code: i32,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
}
