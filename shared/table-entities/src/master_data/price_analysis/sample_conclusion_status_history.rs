use super::sample_conclusion_status::SampleConclusionStatusId;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use asez2_shared_db::db_item::AsezTimestamp;

/// Справочник "История изменения статуса шаблона заключения"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "sample_conclusion_status_history"]
pub struct SampleConclusion {
    /// Уникальный идентификатор записи
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Уникальный идентификатор шаблона
    pub sample_conclusion_uuid: Uuid,
    /// Статус шаблона
    pub status_id: SampleConclusionStatusId,
    /// Текст шаблона
    pub comment: Option<String>,
    /// Запись удалена
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
