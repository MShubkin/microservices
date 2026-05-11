//! Отвечает а объекты с таблицы `status_object`.
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};

use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::types::Type;

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "status_object"]
#[item_aggr_insert]
pub struct ProcessingStatusObject {
    #[item_field_pkey]
    pub id: ProcessingStatus,
    pub value: String,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Type, DbEnum)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum ProcessingStatus {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    /// Сформирован
    Created = 1,
    /// На согласовании
    InNegotiation = 2,
    /// На подписании
    InSigning = 3,
    /// Утвержден
    Confirmed = 4,
    /// Удален
    Deleted = 5,
}
