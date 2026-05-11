use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use sqlx::Type;

use asez2_shared_db::db_item::AsezTimestamp;
use shared_db_derive::DbEnum;

/// Id решения комиссии
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum EstimatedCommissionAgendaStatusId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Сформирована
    Formed = 100,
    /// Отправлена
    Sent = 200,
    /// Сформирован Протокол
    ProtocolFormed = 300,
    /// Удалена
    Removed = 400,
}

/// Справочник "Статусы Повестки"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "agenda_status"]
pub struct EstimatedCommissionAgendaStatus {
    /// Id статуса
    #[item_field_pkey]
    #[item_field_autogen]
    #[adaptor_field_duplicate = "status_id"]
    pub id: EstimatedCommissionAgendaStatusId,
    /// Наименование статуса
    #[serde(rename = "text")]
    pub name: String,
    /// Запись удалена
    pub is_removed: bool,
    /// Создано
    pub created_at: AsezTimestamp,
    /// Изменено
    pub changed_at: AsezTimestamp,
    /// Создатель
    pub created_by: i32,
    /// Кем изменено
    pub changed_by: i32,
}
