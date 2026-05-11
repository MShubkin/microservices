use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use sqlx::Type;

use crate::ColorCode;
use asez2_shared_db::db_item::AsezTimestamp;
use shared_db_derive::DbEnum;

/// Id цветовой схемы
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
pub enum CriticalTypeColorSchemeId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Зеленый цвет, нормативные сроки анализа цены не нарушены
    Normal = 1,
    /// Оранжевый цвет, нормативные сроки анализа цены истекают
    Expiring = 2,
    /// Красный цвет, нормативные сроки анализа цены нарушены
    Violated = 3,
}

/// Справочник "Цветовые схемы критичности"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "critical_type_color_scheme"]
pub struct CriticalTypeColorScheme {
    /// Id статуса
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: CriticalTypeColorSchemeId,
    /// Наименование
    #[serde(rename = "text")]
    pub name: String,
    /// Идентификатор типа критичности
    #[serde(rename = "type")]
    pub type_id: CriticalTypeId,
    ///Код цветовой схемы
    pub color_code: ColorCode,
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

/// Id типа критичности
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
pub enum CriticalTypeId {
    /// Зелёный
    #[db_default]
    Green = 1,
    /// Оранжевый
    Orange = 2,
    /// Красный
    Red = 3,
}
