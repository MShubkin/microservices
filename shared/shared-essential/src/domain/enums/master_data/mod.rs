use crate::presentation::dto::master_data::error::MasterDataError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Тип справочника
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Hash, Eq)]
#[serde(rename = "snake_case")]
pub enum DirectoryType {
    /// Справочник "Метод ценообразования"
    AnalysisMethod,
    /// Справочник "Способ назначения исполнителя"
    AssigningExecutorMethod,
    /// Цветовые схемы критичности
    CriticalTypeColorScheme,
    /// Решения комиссии СК по ППЗ/ДС
    EstimatedCommissionResult,
    /// Статусы повестки
    EstimatedCommissionAgendaStatus,
    /// Статусы протокола
    #[serde(rename = "protocol_status")]
    EstimatedCommissionProtocolStatus,
    /// Тип протокола
    #[serde(rename = "protocol_type")]
    EstimatedCommissionProtocolType,
    /// Типы заключений эксперта
    ExpertConclusionType,
    /// Типы объектов
    ObjectType,
    /// Поставщики
    Organization,
    /// Условия оплаты
    PaymentConditions,
    /// Тип ППЗ
    PpzType,
    /// Методы ценообразования
    PriceAnalysisMethod,
    /// Департаменты (организации) АЦ
    PricingUnit,
    /// Каталог "Производственный календарь"
    SchedulerRequestUpdateCatalog,
    /// Тип запроса ЗЦИ
    PriceInformationRequestType,
    /// Статусы ТКП
    TcpStatus,
    /// Справочник решений
    DepartmentResponseStatus,
    /// Справочник ОКПД2
    Okpd2,
    /// Справочник Статья бюджета
    BudgetItem,
    /// Справочник ВПЗ
    Category,
    /// Структура организаций.
    OrganizationalStructure,
    /// Справочник "Основания аннулирования"
    PlanReasonCancelImpactArea,
    /// Справочник "Функциональность"
    PlanReasonCancelFunctionality,
    /// Справочник "Проверки для ППЗ"
    PlanReasonCancelCheckReason,
}

impl Display for DirectoryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for DirectoryType {
    type Error = MasterDataError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use DirectoryType::*;
        match value {
            "agenda_status" => Ok(EstimatedCommissionAgendaStatus),
            "analysis_method" => Ok(AnalysisMethod),
            "assigning_executor_method" => Ok(AssigningExecutorMethod),
            "critical_type_color_scheme" => Ok(CriticalTypeColorScheme),
            "estimated_commission_agenda_status" => {
                Ok(EstimatedCommissionAgendaStatus)
            }
            "estimated_commission_protocol_status" => {
                Ok(EstimatedCommissionProtocolStatus)
            }
            "estimated_commission_result" => Ok(EstimatedCommissionResult),
            "expert_conclusion_type" => Ok(ExpertConclusionType),
            "organization" => Ok(Organization),
            "payment_conditions" => Ok(PaymentConditions),
            "ppz_type" => Ok(PpzType),
            "price_information_request_type" => Ok(PriceInformationRequestType),
            "price_analysis_method" => Ok(PriceAnalysisMethod),
            "pricing_organization_unit" => Ok(PricingUnit),
            "pricing_unit" => Ok(PricingUnit),
            "protocol_status" => Ok(EstimatedCommissionProtocolStatus),
            "protocol_type" => Ok(EstimatedCommissionProtocolType),
            "scheduler_request_update_catalog" => Ok(SchedulerRequestUpdateCatalog),
            "request_type" => Ok(PriceInformationRequestType),
            "status_type" => Ok(TcpStatus),
            "okpd2" => Ok(Okpd2),
            "category" => Ok(Category),
            "budget_item" => Ok(BudgetItem),
            "organizational_structure" => Ok(OrganizationalStructure),
            "plan_reason_cancel_impact_area" => Ok(PlanReasonCancelImpactArea),
            "plan_reason_cancel_functionality" => Ok(PlanReasonCancelFunctionality),
            "plan_reason_cancel_check_reason" => Ok(PlanReasonCancelCheckReason),

            _ => Err(MasterDataError::InternalError(format!(
                "Directory with name {} not found",
                value
            ))),
        }
    }
}
