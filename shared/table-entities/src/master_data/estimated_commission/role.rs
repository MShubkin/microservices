use serde::{Deserialize, Serialize};
use uuid::Uuid;

use asez2_shared_db::{DbAdaptor, DbItem};
use shared_db_derive::DbEnum;

use super::*;
use asez2_shared_db::db_item::AsezTimestamp;

/// Справочник «Роли пользователей Сметной комиссии»
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "estimated_commission_role"]
pub struct EstimatedCommissionRole {
    /// Код роли Сметной комиссии
    #[item_field_pkey]
    #[item_field_autogen]
    #[adaptor_field_duplicate = "role_id"]
    pub id: EstimatedCommissionRoleId,
    /// uuid записи
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
pub enum EstimatedCommissionRoleId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Председатель Сметной комиссии
    Chairman = 1,
    /// Протокол очного заседния СК
    Member = 2,
    /// Секретарь Сметной комиссии
    Secretary = 3,
}
