use crate::master_data::ObjectTypeId;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Справочник "Способ назначения исполнителя"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "object_type"]
pub struct ObjectType {
    /// Идентификатор записи в таблице
    #[item_field_pkey]
    pub id: ObjectTypeId,
    /// Идентификатор записи в таблице
    pub uuid: Uuid,
    /// Наименование статуса
    #[serde(rename = "text")]
    pub name: String,
    /// Код
    pub code: String,
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
