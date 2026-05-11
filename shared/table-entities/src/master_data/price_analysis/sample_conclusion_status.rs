use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use sqlx::Type;

use asez2_shared_db::db_item::AsezTimestamp;
use shared_db_derive::DbEnum;

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
pub enum SampleConclusionStatusId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Создан
    Created = 1,
    /// Изменен
    Changed = 2,
    /// Согласован
    Agreed = 3,
    /// Отклонен
    Rejected = 4,
}

/// Справочник "Типы заключений эксперта"
/// See https://rcportal.inlinegroup.ru/web#id=2465&cids=1&model=project.task&view_type=form notes for details
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "sample_conclusion_status"]
pub struct StatusSampleConclusion {
    /// Id статуса
    #[item_field_pkey]
    pub id: SampleConclusionStatusId,
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
