use std::fmt::Display;

use sqlx::Type;

use shared_db_derive::DbEnum;

/// Справочник "Статусы Повестки"
pub mod agenda_status;
/// Справочник "Статусы Протокола"
pub mod protocol_status;
/// Справочник "Тип Протокола"
pub mod protocol_type;
/// Справочник "Решения комисии СК по ППЗ/ДС"
pub mod results;
/// Справочник «Роли пользователей Сметной комиссии»
pub mod role;
