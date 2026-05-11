use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};

use crate::ExpertConclusionId;
use asez2_shared_db::db_item::AsezTimestamp;

/// Справочник "Типы заключений эксперта"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "expert_conclusion_type"]
pub struct ExpertConclusionType {
    /// Id статуса
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: ExpertConclusionId,
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
