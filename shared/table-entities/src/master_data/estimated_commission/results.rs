use super::*;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
pub enum EstimatedCommissionResultId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Утверждено
    Appoved = 1,
    /// Согласовано с корректировкойкой стоимости
    AgreedWithCostAdjustment = 2,
    /// Не согласовано. Вернуть Эксперту АЦ
    NotAgreedWithReturnToExpertPa = 3,
    /// Аннулировано
    Cancelled = 4,
}

/// Справочник "Решения комисии СК по ППЗ/ДС"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "estimated_commission_result"]
pub struct EstimatedCommissionResult {
    /// Id решения комиссии
    #[item_field_pkey]
    #[item_field_autogen]
    #[adaptor_field_duplicate = "result_id"]
    pub id: EstimatedCommissionResultId,
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
