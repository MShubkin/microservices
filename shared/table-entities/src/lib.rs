#![allow(ambiguous_glob_reexports)]

pub mod helper_entities;
pub mod legacy;
pub mod master_data;
pub mod maths;
pub mod processing;
pub mod scheduler;
pub mod view_storage;

pub mod test_setup;
#[cfg(test)]
mod tests;
pub mod traits;

pub mod tcp;

pub use helper_entities::*;
pub use legacy::*;

pub use master_data::organizational_structure::*;
pub use master_data::organizational_user_assignment::*;
pub use processing::agenda::*;
pub use processing::agenda_item::*;
pub use processing::attachment::*;
pub use processing::ca_item::*;
pub use processing::commission_result::*;
pub use processing::contract_amendment::*;
pub use processing::document_approver::*;
pub use processing::estimated_commission_partner::*;
pub use processing::estimated_commission_settings::*;
pub use processing::executor_method::*;
pub use processing::favourite_plan::*;
pub use processing::field_histories::*;
pub use processing::object_route::*;
pub use processing::object_type::*;
pub use processing::partner_agenda_protocol::*;
pub use processing::partner_type_commission::*;
pub use processing::plan::*;
pub use processing::plan_item::*;
pub use processing::plan_retrospective::*;
pub use processing::price_analysis_user::*;
pub use processing::protocol::*;
pub use processing::protocol_item::*;
pub use processing::rel_agenda_protocol::*;
pub use processing::rel_agenda_protocol_item::*;
pub use processing::route_addep::*;
pub use processing::status_history::*;
pub use processing::status_object::*;

use asez2_shared_db::result::Result as SharedDbRes;
use asez2_shared_db::value::Value;
use shared_db_derive::DbEnum;

mod cached_tables;
pub mod plan_amendment;

pub use plan_amendment::{
    PlanOrAmendment, PlanOrAmendmentRep, WidePlanOrAmendmentRep,
};

use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub fn parse_string<T: std::str::FromStr>(x: String) -> SharedDbRes<T> {
    x.parse::<T>().map_err(|_| format!("Cannot parse {:?}", x).into())
}

pub fn num_to_string<T: Display>(x: T) -> String {
    x.to_string()
}

pub fn activate_option<T: Default>() -> T {
    <T>::default()
}

pub fn val_from_json(s: Option<String>) -> SharedDbRes<Option<Value>> {
    s.map(|s| serde_json::from_str(&s)).transpose().map_err(Into::into)
}

pub fn val_to_json(s: Option<Value>) -> Option<String> {
    s.map(|s| serde_json::to_string(&s).expect("Serializes"))
}

/// Вид секции или же модуль системы АСЭЗ 2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SectionKind {
    EstimatedCommission,
    PriceAnalysis,
    SpecializedDepartments,
    TechnicalCommercialProposal,
    None,
}

/// Все возможные секции для взаимодействия с BE
/// в системе АСЭЗ 2.0
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    sqlx::Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[repr(i16)]
pub enum Section {
    EstimatedCommissionInPerson = 1,
    AllPlansForEstimatedCommission = 2,
    EstimatedCommissionCorrespondence = 3,
    EstimatedCommissionNotRequired = 4,
    EstimatedCommissionProcurements = 5,
    EstimatedCommissionInPersonPreparation = 13,
    EstimatedCommissionSummingUpInPerson = 14,
    EstimatedCommissionSummingUpCorrespondence = 15,
    EstimatedCommissionKggProcurements = 23,
    EstimatedCommissionReports = 24,
    EstimatedCommissionCalendar = 25,

    PriceAnalysisAssignExpert = 6,
    PriceAnalysisDeterminePrice = 7,
    PriceAnalysisApprovePrice = 8,
    PriceAnalysisGgp = 9,
    PriceAnalysisPrimaryExpertControl = 10,
    PriceAnalysisLottingMTP = 17,
    PriceAnalysisReportingPrice = 26,
    PriceAnalysisConclusionTemplates = 27,
    PriceAnalysisOffersByRequest = 28,
    PriceAnalysisAutomaticAssignExpert = 29,

    AssignExpertDepartment = 11,
    InWorkByExpertDepartment = 12,
    ProcurementsReviewedByDepartment = 21,
    AutoAssignDepartment = 22,

    TcpPriceRequestList = 16,
    TcpPriceCustomerList = 18,
    TcpPosition = 19,
    TcpRequestItem = 20,

    #[db_default]
    None = 0,
}

impl Section {
    pub fn kind(&self) -> SectionKind {
        if self.is_estimated_commission() {
            SectionKind::EstimatedCommission
        } else if self.is_price_analysis() {
            SectionKind::PriceAnalysis
        } else if self.is_specialized_departments() {
            SectionKind::SpecializedDepartments
        } else if self.is_technical_commercial_proposal() {
            SectionKind::TechnicalCommercialProposal
        } else {
            SectionKind::None
        }
    }

    pub fn is_estimated_commission(&self) -> bool {
        use Section::*;
        matches!(
            self,
            EstimatedCommissionInPerson
                | AllPlansForEstimatedCommission
                | EstimatedCommissionCorrespondence
                | EstimatedCommissionNotRequired
                | EstimatedCommissionProcurements
                | EstimatedCommissionInPersonPreparation
                | EstimatedCommissionSummingUpInPerson
                | EstimatedCommissionSummingUpCorrespondence
                | EstimatedCommissionKggProcurements
                | EstimatedCommissionReports
                | EstimatedCommissionCalendar
        )
    }

    pub fn is_price_analysis(&self) -> bool {
        use Section::*;
        matches!(
            self,
            PriceAnalysisAssignExpert
                | PriceAnalysisDeterminePrice
                | PriceAnalysisApprovePrice
                | PriceAnalysisGgp
                | PriceAnalysisPrimaryExpertControl
                | PriceAnalysisLottingMTP
                | PriceAnalysisReportingPrice
                | PriceAnalysisConclusionTemplates
                | PriceAnalysisOffersByRequest
                | PriceAnalysisAutomaticAssignExpert
        )
    }

    pub fn is_specialized_departments(&self) -> bool {
        use Section::*;
        matches!(
            self,
            InWorkByExpertDepartment
                | AssignExpertDepartment
                | ProcurementsReviewedByDepartment
        )
    }

    pub fn is_technical_commercial_proposal(&self) -> bool {
        matches!(self, Section::TcpPriceRequestList)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Section::None)
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Section::EstimatedCommissionInPerson => "Очная СК",
            Section::EstimatedCommissionCorrespondence => "Заочная СК",
            Section::EstimatedCommissionNotRequired => "СК не требуется",
            Section::EstimatedCommissionProcurements => "Закупки ЕИ",
            Section::EstimatedCommissionInPersonPreparation => {
                "Проведение очной СК"
            }
            Section::EstimatedCommissionSummingUpInPerson => {
                "Подведение итогов очной СК"
            }
            Section::EstimatedCommissionSummingUpCorrespondence => {
                "Подведение итогов заочной СК"
            }
            Section::EstimatedCommissionKggProcurements => "Закупки Газпром",
            Section::EstimatedCommissionReports => "Отчет СК",
            Section::EstimatedCommissionCalendar => "Календарь СК",
            // TODO: еще вод вопросом существования
            Section::AllPlansForEstimatedCommission => "Все ППЗ/ДС для СК",

            Section::PriceAnalysisAssignExpert => "Назначить Эксперта АЦ",
            Section::PriceAnalysisDeterminePrice => "Определить цену",
            Section::PriceAnalysisApprovePrice => "Утвердить цену",
            Section::PriceAnalysisGgp => "Закупки Группы Газпром",
            Section::PriceAnalysisPrimaryExpertControl => {
                "Первичный контроль Эксепрта АЦ"
            }
            Section::PriceAnalysisLottingMTP => "Лотирование МТР",
            Section::PriceAnalysisReportingPrice => "Сообщение о поступлении цены",
            Section::PriceAnalysisConclusionTemplates => "Шаблоны заключения",
            Section::PriceAnalysisOffersByRequest => "Предложения по запросу",
            Section::PriceAnalysisAutomaticAssignExpert => {
                "Автоназначение эксперта АЦ"
            }

            Section::AssignExpertDepartment => "Назначить Эксперта ПД",
            Section::InWorkByExpertDepartment => "В работе у Эксперта ПД",
            Section::ProcurementsReviewedByDepartment => {
                "Закупки, рассмотренные ПД"
            }
            Section::AutoAssignDepartment => "Автоназначение департамента ПД",

            Section::TcpPriceRequestList => "Список ЗЦИ",
            Section::TcpPriceCustomerList => "Список поставщиков",
            Section::TcpPosition => "Позиции ТКП",
            Section::TcpRequestItem => "Позиции ЗЦИ",

            Section::None => "Без секции",
        };
        write!(f, "{}", s)
    }
}
