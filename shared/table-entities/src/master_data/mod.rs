use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;

/// Справочник "Способ назначения исполнителя"
pub mod assigning_executor_method;
pub use assigning_executor_method::*;
/// Справочники типа вложенного Документа
pub mod attachment_type;
pub use attachment_type::*;

/// Справочник "Цветовые схемы критичности"
pub mod critical_type_color_scheme;
pub use critical_type_color_scheme::*;
/// Справочники модуля "Сметная комиссия"
pub mod estimated_commission;
pub use estimated_commission::{
    agenda_status::*, protocol_status::*, protocol_type::*, results::*, role::*,
};
/// Справочник "Типы заключений эксперта"
pub mod expert_conclusion_type;
pub use expert_conclusion_type::*;
/// Избранные записи
pub mod favorites;
pub use favorites::*;
/// Типы объектов
pub mod object_type;
pub use object_type::*;
/// Поставщики
pub mod organization;
pub use organization::*;
/// Справочник "Выходных форм"
pub mod output_form;
pub use output_form::*;

pub mod organizational_structure;
pub use organizational_structure::*;

/// Справочник Статья бюджета
pub mod budget_item;
pub use budget_item::*;
/// Справочник ВПЗ
pub mod category;
pub use category::*;
/// Справочник кодов ОКПД2
pub mod okpd2;
pub use okpd2::*;
pub mod organizational_user_assignment;
/// Справочник "Условия оплаты"
pub mod payment_conditions;
pub use payment_conditions::*;
/// Справочник "Причины аннулирования"
pub mod plan_reasons_cancel;
pub use plan_reasons_cancel::*;
/// Справочник "Тип ППЗ"
pub mod ppz_type;
pub use ppz_type::*;
/// Справочники модуля "Анализ Цены"
pub mod price_analysis;
pub use price_analysis::{
    analysis_method::*, price_analysis_method::*, pricing_unit::*,
};
/// Справочник решений в базе данных НСИ
pub mod response;
pub use response::*;
/// Маршруты согласования
pub mod routes;
pub use routes::*;
/// Сервис календарей.
pub mod scheduler_calendar;
pub use scheduler_calendar::scheduler_update_catalog_request::*;
/// Справочники модуля "ТКП"
pub mod technical_commercial_proposal;
pub use technical_commercial_proposal::{request_type::*, status_type::*};

/// Справочник "Тип объекта"
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
pub enum ObjectTypeId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Plan
    Plan = 1,
    /// ContractAmendment
    ContractAmendment = 2,
    /// Повестка
    Agenda = 3,
    /// Протокол
    Protocol = 4,
    /// ЗП
    Purchase = 5,
    /// ВП
    Quotation = 6,
    /// КД
    Contract = 7,
}
