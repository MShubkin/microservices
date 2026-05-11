use crate::PricingUnitId;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Справочник «Департамент (организация) АЦ»
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "pricing_organization_unit"]
pub struct PricingUnit {
    /// Код роли Сметной комиссии
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: PricingUnitId,
    /// uuid записи
    pub uuid: Uuid,
    /// Код орг. структуры SAP
    pub sap_code: i32,
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
