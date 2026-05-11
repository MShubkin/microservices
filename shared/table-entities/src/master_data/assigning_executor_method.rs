use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;
use uuid::Uuid;

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
pub enum AssigningExecutorMethodId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Выполнено автоматическое назначение исполнителя
    Auto = 1,
    /// Выполнено ручное назначение исполнителя
    Manual = 2,
    /// Автоматически назначенный исполнитель изменен вручную
    ManualCorrection = 3,
}

/// Справочник "Способ назначения исполнителя"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "assigning_executor_method"]
pub struct AssigningExecutorMethod {
    /// Идентификатор записи в таблице
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: AssigningExecutorMethodId,
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
