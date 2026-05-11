use super::*;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Id типа протокола
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
pub enum EstimatedCommissionProtocolTypeId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Протокол очного заседания СК
    MinutesOfInPersonMeeting = 1,
    /// Протокол заочного заседания СК
    MinutesOfCorrespondenceMeeting = 2,
}

/// Справочник "Статусы Протокола"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "protocol_type"]
pub struct EstimatedCommissionProtocolType {
    /// Id типа протокола
    #[item_field_pkey]
    #[item_field_autogen]
    #[adaptor_field_duplicate = "type_id"]
    pub id: EstimatedCommissionProtocolTypeId,
    /// Идентификатор записи в таблице
    pub uuid: Uuid,
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
