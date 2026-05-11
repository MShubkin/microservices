use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;
use uuid::Uuid;

/// Способ анализа
#[repr(i16)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Type, Serialize, Deserialize, DbEnum,
)]
#[serde(from = "i16", into = "i16")]
pub enum AnalysisMethodId {
    /// Unknown (this is a hack to allow into)
    #[db_default]
    Undefined = 0,
    /// Без запроса цен
    WithoutPriceRequest = 1,
    /// С запросом цен
    PriceRequest = 2,
}

/// Справочник "Способ анализа"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "analysis_method"]
pub struct AnalysisMethod {
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: AnalysisMethodId,
    /// uuid записи
    pub uuid: Uuid,
    /// Наименование метода
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
