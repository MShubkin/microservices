use sqlx::Type;
use uuid::Uuid;

use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;

use asez2_shared_db::db_item::AsezTimestamp;

/// Справочник "Тип ППЗ"

/// Id
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
pub enum PpzTypeId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// МТР
    MTR = 1,
    /// Работа
    Job = 2,
    /// Услуга
    Service = 3,
}

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "ppz_type"]
pub struct PpzType {
    /// Id типа документам
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: PpzTypeId,
    /// Идентификатор записи в таблице
    pub uuid: Uuid,
    /// Наименование
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
