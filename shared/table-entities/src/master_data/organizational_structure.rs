//! Отвечает а объекты с таблицы `department`.
use crate::*;
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use shared_db_derive::DbItemExt;

use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

/// We must NOT use `#[item_aggr_insert]` here as it breaks the vector type.
#[derive(
    Debug,
    Default,
    Clone,
    DbItem,
    DbItemExt,
    DbAdaptor,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "organizational_structure"]
pub struct OrganizationalStructure {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    pub id: i32,
    pub text: String,
    pub text_short: String,
    pub level: DepartmentLevel,
    pub parent_id: Option<i32>,
    #[db_field_name = "type"]
    pub dep_type: DepartmentType,
    pub is_specialized_department: bool,
    pub sap_id: Option<i32>,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

/// Тип департамента.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum DepartmentType {
    /// ЮЛ
    #[db_default]
    #[display(fmt = "ЮЛ")]
    LegalEntity = 1,
    /// Департамент.
    #[display(fmt = "Департамент")]
    Department = 2,
    /// Управление.
    #[display(fmt = "Управление")]
    Division = 3,
    /// Отдел.
    #[display(fmt = "Отдел")]
    Section = 4,
}

/// Уровень департамента в иерархии.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum DepartmentLevel {
    #[db_default]
    #[display(fmt = "Неопределено")]
    Undefined,
    /// ПАО Газпром/ДО
    #[display(fmt = "ПАО Газпром/ДО")]
    GP = 1,
    /// Департамент/Управление
    #[display(fmt = "Департамент/Управление")]
    Department = 2,
    /// Управление/Отдел
    #[display(fmt = "Управление/Отдел")]
    Division = 3,
    /// Отдел
    #[display(fmt = "Отдел")]
    SubDivision = 4,
}
