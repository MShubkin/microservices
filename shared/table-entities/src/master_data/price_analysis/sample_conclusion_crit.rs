use crate::master_data::routes::CritValue;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};

use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

/// Справочник "Критерии шаблонов заключений"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "sample_conclusion_crit"]
pub struct SampleConclusionCrit {
    /// Уникальный идентификатор
    #[item_field_pkey]
    pub sample_conclusion_uuid: Uuid,
    /// Название критерия
    #[item_field_pkey]
    pub field_name: String,
    /// Значение критерия
    pub predicate: Json<SampleConclusionPredicate>,
    /// Запись удалена
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub enum SampleConclusionPredicate {
    #[default]
    None,
    In(Vec<CritValue>),
}
