use serde::{Deserialize, Serialize};

use asez2_shared_db::db_item::AsezTimestamp;
use asez2_tables::master_data::{
    assigning_executor_method::AssigningExecutorMethod,
    attachment_type::AttachmentType,
    critical_type_color_scheme::CriticalTypeColorScheme,
    estimated_commission::role::EstimatedCommissionRole,
    expert_conclusion_type::ExpertConclusionType,
    favorites::FavoriteDictionary,
    object_type::ObjectType,
    output_form::OutputForm,
    payment_conditions::PaymentCondition,
    plan_reasons_cancel::{
        PlanReasonCancelCheckReason, PlanReasonCancelFunctionality,
        PlanReasonCancelImpactArea,
    },
    ppz_type::PpzType,
    price_analysis::{
        analysis_method::AnalysisMethod,
        price_analysis_method::PriceAnalysisMethod, pricing_unit::PricingUnit,
    },
    response::Response,
};

use crate::domain::master_data::{
    estimated_commission::{
        agenda_status::EstimatedCommissionAgendaStatus,
        protocol_status::EstimatedCommissionProtocolStatus,
        protocol_type::EstimatedCommissionProtocolType,
        results::EstimatedCommissionResult,
    },
    scheduler_calendar::scheduler_update_catalog_request::SchedulerRequestUpdateCatalog,
    technical_commercial_proposal::request_type::PriceInformationRequestType,
};
use crate::presentation::dto::response_request::ApiResponseData;

/// Данные с обновлениями справочников для фронта
#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct MasterDataUpdates {
    pub changed_at: AsezTimestamp,
    pub entity_list: Vec<MasterDataUpdate>,
}

impl ApiResponseData for MasterDataUpdates {}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct MasterDataUpdate {
    pub changed_at: AsezTimestamp,
    #[serde(flatten)]
    pub entity: MasterDataUpdateEntity,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "id", content = "item_list")]
pub enum MasterDataUpdateEntity {
    FavoriteDictionary(Vec<FavoriteDictionary>),

    AgendaStatus(Vec<EstimatedCommissionAgendaStatus>),
    AnalysisMethod(Vec<AnalysisMethod>),
    AssigningExecutorMethod(Vec<AssigningExecutorMethod>),
    AttachmentType(Vec<AttachmentType>),
    CriticalTypeColorScheme(Vec<CriticalTypeColorScheme>),
    EstimatedCommissionResult(Vec<EstimatedCommissionResult>),
    EstimatedCommissionRole(Vec<EstimatedCommissionRole>),
    ExpertConclusionType(Vec<ExpertConclusionType>),
    ObjectType(Vec<ObjectType>),
    OutputForm(Vec<OutputForm>),
    PaymentConditions(Vec<PaymentCondition>),
    PpzType(Vec<PpzType>),
    PriceInformationRequestType(Vec<PriceInformationRequestType>),
    PricingMethod(Vec<PriceAnalysisMethod>),
    PricingOrganizationUnit(Vec<PricingUnit>),
    ProtocolStatus(Vec<EstimatedCommissionProtocolStatus>),
    ProtocolType(Vec<EstimatedCommissionProtocolType>),
    DepartmentResponseStatus(Vec<Response>),
    SchedulerRequestUpdateCatalog(Vec<SchedulerRequestUpdateCatalog>),
    PlanReasonCancelImpactArea(Vec<PlanReasonCancelImpactArea>),
    PlanReasonCancelFunctionality(Vec<PlanReasonCancelFunctionality>),
    PlanReasonCancelCheckReason(Vec<PlanReasonCancelCheckReason>),
    #[default]
    Empty,
}
