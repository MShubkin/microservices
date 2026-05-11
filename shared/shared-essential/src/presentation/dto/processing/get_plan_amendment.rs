use asez2_tables::maths::CurrencyValue;
use serde::{Deserialize, Serialize};

use asez2_shared_db::db_item::*;

use crate::domain::{
    AttachmentRep, ContractAmendmentItemRep, ContractAmendmentItemVersionRep,
    ContractAmendmentRep, ContractAmendmentVersionRep, ExpertConclusionId,
    PlanItemFullRep, PlanItemFullVersionRep, PlanRep, PlanVersionRep,
};
use crate::presentation::dto::response_request::{ApiResponse, PaginatedData};

/// Данные из ответа на запрос на получение полного плана
pub type GetPlanResponseData = PaginatedData<GetPlanDataRep>;
/// Ответ на запрос на получение полного плана
pub type GetPlanResponse = ApiResponse<GetPlanResponseData, ()>;

pub type GetPlanVersionResponseData = PaginatedData<GetPlanVersionDataRep>;
pub type GetPlanVersionResponse = ApiResponse<GetPlanVersionResponseData, ()>;

/// Данные из ответа на запрос на получение полного допсоглашения
pub type GetContractAmendmentResponseData =
    PaginatedData<GetContractAmendmentDataRep>;
/// Ответ на запрос на получение полного допсоглашения
pub type GetContractAmendmentResponse =
    ApiResponse<GetContractAmendmentResponseData, ()>;

pub type GetContractAmendmentVersionResponseData =
    PaginatedData<GetContractAmendmentVersionDataRep>;
pub type GetContractAmendmentVersionResponse =
    ApiResponse<GetContractAmendmentVersionResponseData, ()>;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GetPlanAmendmentData<E, I> {
    pub plan: E,
    pub items: Vec<I>,
    pub attachments: Vec<AttachmentRep>,
    pub versions: Vec<VersionInfo>,
}

/// Response to get_contract_amendment
pub type GetContractAmendmentDataRep =
    GetPlanAmendmentData<ContractAmendmentRep, ContractAmendmentItemRep>;

/// Response to get_contract_amendment_version
pub type GetContractAmendmentVersionDataRep = GetPlanAmendmentData<
    ContractAmendmentVersionRep,
    ContractAmendmentItemVersionRep,
>;

/// Response to get_plan
pub type GetPlanDataRep = GetPlanAmendmentData<PlanRep, PlanItemFullRep>;

/// Response to get_plan_version
pub type GetPlanVersionDataRep =
    GetPlanAmendmentData<PlanVersionRep, PlanItemFullVersionRep>;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub pricing_version: Option<i16>,
    pub is_active: bool,
    pub pricing_expert_id: Option<i32>,
    pub expert_conclusion_id: Option<ExpertConclusionId>,
    pub pricing_created_at: Option<AsezTimestamp>,
    pub sum_excluded_vat: CurrencyValue,
    pub sum_included_vat: CurrencyValue,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
}
