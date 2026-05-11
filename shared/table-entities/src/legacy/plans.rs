//! This module handles the implementation of `DbItem` and `DbAdaptor` for plans.
//! In this case, since we only ever need a small subset of the fields, the macros
//! are probably not optimal (we will construct a structure with a hundred fields)
//! but only ever send 10-30 of them. It may help to use a simplified structure
//! if performance is not satisfactory.
use std::fmt::Display;

use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::db_item::DbItemExt;
use asez2_shared_db::{DbAdaptor, DbItem};
use monolith_service::dto::time::PlanningTimestamp;

use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::types::Type;
use uuid::Uuid;

use crate::maths::*;
use crate::{ExpertConclusionId, SavingsAccountingId, TypeOfPurchaseId};

use super::{
    CommissionKind, CompletePlanRep, PlanItemFullRep, PlanItemLegacyRep, PlanRep,
};

/// A PlanLegacy with all of its items.
/// TODO: Make this a queryable db thing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompletePlanLegacyRep {
    pub plan: PlanLegacyRep,
    pub items: Vec<PlanItemLegacyRep>,
}

impl TryFrom<CompletePlanLegacyRep> for CompletePlanRep {
    type Error = String;
    fn try_from(x: CompletePlanLegacyRep) -> Result<Self, Self::Error> {
        let plan = x.plan.try_into()?;
        let items = x
            .items
            .into_iter()
            .map(PlanItemFullRep::from)
            .map(Into::into)
            .collect::<Vec<_>>();

        Ok(Self { plan, items })
    }
}

// Currently `PlanLegacy` is `1048` bytes and has 134 fields. It may be worth creating
// a core structure for the commonly used and accessible fields.
#[derive(Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, PartialEq)]
#[adaptor_derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    PartialOrd,
    Deserialize,
    Serialize
)]
#[item_table = "plans_legacy"]
pub struct PlanLegacy {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub id: String,
    pub year: i16,
    pub customer_id: i32,
    pub declarant_id: i32,
    pub agent_id: i32,
    pub initiator_user_id: i32,
    pub tender_user_id: i32,
    pub organizer_id: i32,
    pub section_id: i16,
    pub branch: String,
    pub contract_subject: String,
    pub minimal_requirements: String,
    pub number_customer: String,
    pub number_cgg: String,
    pub currency_id: i32,
    //-------------------------
    pub kod_st_buda: Option<String>,
    pub okdp2: Option<String>,
    pub category_id: Option<String>,
    pub code_type: Option<TypeOfPurchaseId>,
    //-------------------------
    pub vat_id: VatId,
    pub sum_excluded_vat: CurrencyValue,
    pub sum_vat: CurrencyValue,
    pub sum_included_vat: CurrencyValue,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_vat_rub: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
    pub purchasing_method_id: i16,
    pub purchasing_type_id: i16,
    pub purchasing_kind_id: i16,
    pub regulation_document_id: i16,
    pub publication_type_id: i16,
    pub master_system_id: i16,
    pub funding_source_id: i16,
    pub purchasing_trend_id: i16,
    pub budget_item_id: i16,
    pub payment_balance_item_id: i16,
    //-------------------------
    pub is_smb: bool,
    pub is_smb_sub: bool,
    pub smb_exception_id: i16,
    pub smb_sub_percent: i64,
    pub smb_sub_sum: CurrencyValue,
    pub customer_note: String,
    pub documentation_date: AsezDate,
    pub publication_date: AsezDate,
    pub publication_start_date: AsezDate,
    pub publication_end_date: AsezDate,
    pub bid_opening_date: AsezDate,
    pub summing_up_date: AsezDate,
    pub contract_sing_date: Option<AsezDate>,
    pub contract_sign_date: AsezDate,
    pub delivery_start_date: AsezDate,
    pub delivery_end_date: AsezDate,
    pub organizer_note: String,
    pub competitive_note_for_expert: String,
    //-------------------------
    pub single_supplier_reason_id: i16,
    pub single_supplier_reason_code: String,
    pub single_supplier_expert_id: i32,
    pub single_supplier_decision_id: i16,
    pub single_supplier_decision_resume: String,
    pub single_supplier_note_for_expert: String,
    pub supplier_id: i32,
    pub supplier_text: String,
    pub is_affiliated: bool,
    pub is_supplier_smb: bool,
    pub is_competitive_now: bool,
    pub management_order_number: String,
    pub management_order_date: AsezDate,
    pub reason_document: String,
    pub is_approved_by_d646: bool,
    pub is_price_analysis_by_d646: bool,
    pub is_approved_by_d647: bool,
    pub is_pricing_by_complectation: bool,
    pub is_pricing: bool,
    pub commission_kind_id: CommissionKind,
    pub commission_date: AsezDate,
    //-------------------------
    pub status_scheme_id: i16,
    pub status_id: PlanStatus,
    pub status_note: String,
    pub is_priority_project: bool,
    pub priority_project_document: String,
    pub is_priority_introductory: bool,
    pub priority_introductory_date: AsezDate,
    pub priority_introductory_document: String,
    pub is_priority_repair: bool,
    pub priority_repair_document: String,
    pub is_priority_ozp: bool,
    pub priority_ozp_document: String,
    pub is_priority_income_contract: bool,
    pub priority_income_contract_document: String,
    pub priority_income_contract_partner_id: i32,
    pub is_priority_other: bool,
    pub is_priority_far_eastern: bool,
    pub is_priority_far_nonprofit: bool,
    pub is_headquarters: bool,
    pub is_first_time: bool,
    pub is_nko: bool,
    pub is_priority_nonprofit: bool,
    //----------------------
    pub is_list_price: bool,
    pub is_lease_supplier_selection: bool,
    pub is_performance_indicator: bool,
    pub is_need_for_departments: bool,
    pub is_sanctioned: bool,
    pub pricing_method_id: i16,
    pub pricing_expert_id: i32,
    pub pricing_resume: String,
    pub general_contract_date: AsezDate,
    pub general_contract_number: String,
    pub general_contract_stages: String,
    pub is_material_registry: bool,
    pub start_price_uuid: Uuid,
    pub start_price_type: i16,
    //---------------------
    pub is_no_qualification: bool,
    pub is_other_qualification: bool,
    pub other_qualification_rationale: String,
    //-----------------------
    pub mery_prin: bool,
    pub control_pp_2013: i16,
    pub is_actual: bool,
    //-----------------------
    pub expert_conclusion_id: Option<ExpertConclusionId>,
    pub is_check_documentation: bool,
    pub check_documentation_date: Option<PlanningTimestamp>,
    //------------------------
    pub savings_accounting_id: SavingsAccountingId,
    pub savings_sum_excluded_vat: Option<CurrencyValue>,
    pub savings_sum_excluded_vat_rub: Option<CurrencyValue>,
    pub savings_sum_included_vat: Option<CurrencyValue>,
    pub savings_sum_included_vat_rub: Option<CurrencyValue>,
    //--------------------------
    pub pricing_vat_id: VatId,
    pub pricing_currency_id: Option<i16>,
    pub pricing_currency_rate: Option<CurrencyRate>,
    pub pricing_sum_excluded_vat: CurrencyValue,
    pub pricing_sum_excluded_vat_rub: Option<CurrencyValue>,
    pub pricing_sum_included_vat: Option<CurrencyValue>,
    pub pricing_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_sum_vat: Option<CurrencyValue>,
    pub pricing_sum_vat_rub: Option<CurrencyValue>,
    pub pricing_transportation_vat_id: VatId,
    pub pricing_transportation_price: Option<CurrencyValue>,
    pub pricing_transportation_price_rub: Option<CurrencyValue>,
    pub pricing_transportation_sum_vat: Option<CurrencyValue>,
    pub pricing_transportation_sum_vat_rub: Option<CurrencyValue>,
    pub pricing_transportation_sum_included_vat: Option<CurrencyValue>,
    pub pricing_transportation_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_total_sum: Option<CurrencyValue>,
    pub pricing_total_sum_rub: Option<CurrencyValue>,
    // pricing_started_at new on 2024.11.25
    pub pricing_started_at: PlanningTimestamp,
    //-----------------------
    // The below fields are deprecated as of 2024-10-02
    pub version_type: i16,
    pub posting_date: AsezDate,
    // pub contract_subject: String,
    pub currency_rate: CurrencyRate,
    pub is_commission: bool,
    pub is_onm: bool,
    pub is_agent_fee: bool,
    pub agent_contract_number: String,
    pub is_design_stage: bool,
    pub repair_stage_id: i16,
    pub is_gas_supply: bool,
    pub is_little_cost: bool,
    pub is_banking_support: bool,
    pub is_innovative: bool,
    pub is_to_publish: bool,
    pub rationale_for_not_publication: String,
    pub rationale_for_publication: String,
    pub is_cooperative: bool,
    pub is_not_purchase: bool,
    pub is_under_control: bool,
    pub rationale_is_under_control: String,
    pub technical_developer: String,
    pub limit_on_construction: i64,
    pub limit_on_works: i64,
    pub priority: i16,
    pub priority_income_contract_partner_text: String,
    // pub qualification_id: i32,
    pub extract_number_d646: String,
    pub extract_date_d646: AsezDate,
    pub extract_sum_included_vat_rub_d646: CurrencyValue,
    pub extract_number_d647: String,
    pub extract_date_d647: AsezDate,
    pub extract_sum_included_vat_rub_d647: CurrencyValue,
    pub product_type_id: i16,
    pub description: String,
    pub is_removed: bool,
    pub created_at: PlanningTimestamp,
    pub created_by: i32,
    pub changed_at: PlanningTimestamp,
    pub changed_by: i32,
    pub version_number: i16,
    pub active_uuid: Option<Uuid>,
    pub items_number: i16,
    pub reason_cancel_id: Option<i32>,
    pub replaced_id: Option<i64>,
}

/// Статус ППЗ/ДС
///
/// Перечень значений приведен в таблице - Справочник «Статусы объекта»
#[derive(
    Clone,
    Copy,
    Debug,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    Deserialize,
    Type,
    Hash,
    Eq,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum PlanStatus {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    /// Формирование Заказчиком. Формирование ППЗ/ДС
    FormationInitiator = 111,
    /// Формирование Заказчиком. Согласование тендерного подразделения
    FormationTenderReview = 112,
    /// Формирование Заказчиком. Рассмотрение Профильным Департаментом
    FormationSpecDepartmentReview = 115,
    /// Формирование Заказчиком. Утверждение тендерным подразделением
    FormationTenderApproval = 116,
    /// Запрос документации у Заказчика
    RequestClientDocumentation = 120,
    /// ППЗ/ДС Доработка Заказчиком.
    ReturnToClientRework = 131, //
    /// ППЗ/ДС Доработка Заказчиком. Изменение сроков закупки
    ReturnToClientReworkDates = 133,
    /// ППЗ/ДС Утверждена
    PriceConfirmed = 140,
    /// ППЗ/ДС аннулирован(а)
    PlanCancelled = 150, //
    /// Цена определена
    PriceDetermined = 160, //

    /// Проверка Д646
    Approving = 210,
    /// Проверка Д646. Первичный контроль
    ApprovingInitialControl = 211,
    /// Проверка Д646. Формирование/ корректировка закупки
    ApprovingCooperative = 212,
    /// Проверка Д646. Принятие решения
    ApprovingMakingDecision = 213,
    /// Проверка Д646. Обработка замечаний
    ApprovingCorrections = 214,
    /// Проверка Д646. Переформирование
    ApprovingReforming = 215,
    /// Проверка Д646. Контрольная проверка
    ApprovingControl = 216,

    /// Анализ цены Д646. Назначение исполнителя
    ExecutorAppointmentD646 = 221,
    /// Анализ цены Д646. Исполнитель назначен
    ExecutorAppointedD646 = 222,
    /// Анализ цены Д646. Анализ проведен
    AnalysisPerformedD646 = 223,
    /// Анализ цены Д646. Анализ завершен
    AnalysisCompletedD646 = 225, //
    /// Сметная комиссия. Очная СК
    EstimatedCommissionInPerson = 251,
    /// Сметная комиссия. Заочная СК
    EstimatedCommissionCorrespondence = 252,
    /// Сметная комиссия. Не требуется
    EstimatedCommissionNo = 253,

    /// Согласование ДС Д646
    ContractAmendmentApproving = 260,
    /// Согласование ДС Д646. Назначение исполнителя
    ContractAmendmentApprovingAssignment = 261,
    /// Согласование ДС Д646. Исполнитель назначен
    ContractAmendmentApprovingAssigned = 262,
    /// Согласование ДС Д646. Подготовка решения
    ContractAmendmentApprovingConfirmation = 263,
    /// Согласование ДС Д646. Предварительное решение
    ContractAmendmentApprovingVoting = 264,
    /// Согласование ДС Д646. Обработка замечаний
    ContractAmendmentApprovingCorrection = 265,

    /// Анализ цены Д647. Назначение исполнителя
    ExecutorAppointmentD647 = 341,
    /// Анализ цены Д647. Исполнитель назначен
    ExecutorAppointedD647 = 342,
    /// Анализ цены Д647. Анализ проведен
    AnalysisPerformedD647 = 343,
    /// Анализ цены Д647. Анализ завершен
    AnalysisCompletedD647 = 345, //
    /// Анализ цены МТР. Назначение исполнителя
    ExecutorAppointmentMTP = 351,
    // Анализ цены МТР. Исполнитель назначен
    ExecutorAppointedMTP = 352,
    // Анализ цены МТР. Анализ проведен
    AnalysisPerformedMTP = 353,
    // Анализ цены МТР. Анализ завершен
    AnalysisCompletedMTP = 355, //
    /// Lotting for MTR.
    LottingMTP = 356,

    /// Анализ цены Д645. Назначение исполнителя
    ExecutorAppointmentD645 = 371,
    /// Анализ цены Д645. Исполнитель назначен
    ExecutorAppointedD645 = 372,
    /// Анализ цены Д645. Анализ проведен
    AnalysisPerformedD645 = 373,
    /// Анализ цены Д645. Анализ завершен
    AnalysisCompletedD645 = 375,
}

impl PlanStatus {
    /// incoming status is EstimatedCommission{whatever}, aka (251, 252, 253)
    pub fn is_ec(self) -> bool {
        use PlanStatus::*;
        matches!(
            self,
            EstimatedCommissionInPerson
                | EstimatedCommissionCorrespondence
                | EstimatedCommissionNo
        )
    }
}

impl Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            PlanStatus::Undefined => "Не установлено",
            PlanStatus::FormationInitiator => {
                "Формирование Заказчиком. Формирование ППЗ/ДС"
            }
            PlanStatus::FormationTenderReview => {
                "Формирование Заказчиком. Согласование тендерного подразделения"
            }
            PlanStatus::FormationSpecDepartmentReview => {
                "Формирование Заказчиком. Рассмотрение Профильным Департаментом"
            }
            PlanStatus::FormationTenderApproval => {
                "Формирование Заказчиком. Утверждение тендерным подразделением"
            }
            PlanStatus::RequestClientDocumentation => {
                "Запрос документации у Заказчика"
            }
            PlanStatus::ReturnToClientRework => "ППЗ/ДС Доработка Заказчиком.",
            PlanStatus::ReturnToClientReworkDates => {
                "ППЗ/ДС Доработка Заказчиком. Изменение сроков закупки"
            }
            PlanStatus::PriceConfirmed => "ППЗ/ДС Утверждена",
            PlanStatus::PlanCancelled => "ППЗ/ДС аннулирован(а)",
            PlanStatus::PriceDetermined => "Цена определена",
            PlanStatus::ExecutorAppointmentD646 => {
                "Анализ цены Д646. Назначение исполнителя"
            }
            PlanStatus::ExecutorAppointedD646 => {
                "Анализ цены Д646. Исполнитель назначен"
            }
            PlanStatus::AnalysisPerformedD646 => {
                "Анализ цены Д646. Анализ проведен"
            }
            PlanStatus::AnalysisCompletedD646 => {
                "Анализ цены Д646. Анализ завершен"
            }
            PlanStatus::EstimatedCommissionInPerson => "Сметная комиссия. Очная СК",
            PlanStatus::EstimatedCommissionCorrespondence => {
                "Сметная комиссия. Заочная СК"
            }
            PlanStatus::EstimatedCommissionNo => "Сметная комиссия. Не требуется",
            PlanStatus::ExecutorAppointmentD647 => {
                "Анализ цены Д647. Назначение исполнителя"
            }
            PlanStatus::ExecutorAppointedD647 => {
                "Анализ цены Д647. Исполнитель назначен"
            }
            PlanStatus::AnalysisPerformedD647 => {
                "Анализ цены Д647. Анализ проведен"
            }
            PlanStatus::AnalysisCompletedD647 => {
                "Анализ цены Д647. Анализ завершен"
            }
            PlanStatus::ExecutorAppointmentMTP => {
                "Анализ цены МТР. Назначение исполнителя"
            }
            PlanStatus::ExecutorAppointedMTP => {
                "Анализ цены МТР. Исполнитель назначен"
            }
            PlanStatus::AnalysisPerformedMTP => "Анализ цены МТР. Анализ проведен",
            PlanStatus::AnalysisCompletedMTP => "Анализ цены МТР. Анализ завершен",
            PlanStatus::LottingMTP => "Лотирование МТР",
            PlanStatus::ExecutorAppointmentD645 => {
                "Анализ цены Д645. Назначение исполнителя"
            }
            PlanStatus::ExecutorAppointedD645 => {
                "Анализ цены Д645. Исполнитель назначен"
            }
            PlanStatus::AnalysisPerformedD645 => {
                "Анализ цены Д645. Анализ проведен"
            }
            PlanStatus::AnalysisCompletedD645 => {
                "Анализ цены Д645. Анализ завершен"
            }
            PlanStatus::Approving => "Проверка Д646",
            PlanStatus::ApprovingInitialControl => {
                "Проверка Д646. Первичный контроль"
            }
            PlanStatus::ApprovingCooperative => {
                "Проверка Д646. Формирование/ корректировка закупки"
            }
            PlanStatus::ApprovingMakingDecision => {
                "Проверка Д646. Принятие решения"
            }
            PlanStatus::ApprovingCorrections => {
                "Проверка Д646. Обработка замечаний"
            }
            PlanStatus::ApprovingReforming => "Проверка Д646. Переформирование",
            PlanStatus::ApprovingControl => "Проверка Д646. Контрольная проверка",
            PlanStatus::ContractAmendmentApproving => "Согласование ДС Д646",
            PlanStatus::ContractAmendmentApprovingAssignment => {
                "Согласование ДС Д646. Назначение исполнителя"
            }
            PlanStatus::ContractAmendmentApprovingAssigned => {
                "Согласование ДС Д646. Исполнитель назначен"
            }
            PlanStatus::ContractAmendmentApprovingConfirmation => {
                "Согласование ДС Д646. Подготовка решения"
            }
            PlanStatus::ContractAmendmentApprovingVoting => {
                "Согласование ДС Д646. Предварительное решение"
            }
            PlanStatus::ContractAmendmentApprovingCorrection => {
                "Согласование ДС Д646. Обработка замечаний"
            }
        };
        write!(f, "{}", msg)
    }
}

impl PlanStatus {
    pub fn is_pricing(&self) -> bool {
        matches!(
            self,
            Self::ExecutorAppointmentD646
                | Self::ExecutorAppointedD646
                | Self::AnalysisPerformedD646
                | Self::AnalysisCompletedD646
                | Self::ExecutorAppointmentD647
                | Self::ExecutorAppointedD647
                | Self::AnalysisPerformedD647
                | Self::AnalysisCompletedD647
                | Self::ExecutorAppointmentMTP
                | Self::ExecutorAppointedMTP
                | Self::AnalysisPerformedMTP
                | Self::AnalysisCompletedMTP
                | Self::ExecutorAppointmentD645
                | Self::ExecutorAppointedD645
                | Self::AnalysisPerformedD645
                | Self::AnalysisCompletedD645
        )
    }

    pub fn is_pricing_assignment(&self) -> bool {
        matches!(
            self,
            Self::ExecutorAppointmentD646
                | Self::ExecutorAppointmentD647
                | Self::ExecutorAppointmentMTP
                | Self::ExecutorAppointmentD645
        )
    }

    pub fn is_pricing_assigned(&self) -> bool {
        matches!(
            self,
            Self::ExecutorAppointedD646
                | Self::ExecutorAppointedD647
                | Self::ExecutorAppointedMTP
                | Self::ExecutorAppointedD645
        )
    }

    pub fn is_pricing_finished(&self) -> bool {
        matches!(
            self,
            Self::AnalysisCompletedD646
                | Self::AnalysisCompletedD647
                | Self::AnalysisCompletedMTP
                | Self::AnalysisCompletedD645
        )
    }

    pub fn is_pricing_in_progress(&self) -> bool {
        matches!(
            self,
            Self::ExecutorAppointedD646
                | Self::AnalysisPerformedD646
                | Self::ExecutorAppointedD647
                | Self::AnalysisPerformedD647
                | Self::ExecutorAppointedMTP
                | Self::AnalysisPerformedMTP
                | Self::ExecutorAppointedD645
                | Self::AnalysisPerformedD645
        )
    }

    pub fn is_commission(&self) -> bool {
        matches!(
            self,
            Self::EstimatedCommissionInPerson
                | Self::EstimatedCommissionCorrespondence
                | Self::EstimatedCommissionNo
        )
    }
}

impl From<PlanRep> for PlanLegacyRep {
    fn from(x: PlanRep) -> Self {
        Self {
            uuid: x.uuid.map(Into::into),
            id: x.id.map(|x| x.to_string()),
            is_actual: x.is_actual,
            year: x.year.map(Into::into),
            customer_id: x.customer_id.map(Into::into),
            declarant_id: x.declarant_id.map(Into::into),
            agent_id: x.agent_id.map(Into::into),
            initiator_user_id: x.initiator_user_id.map(Into::into),
            tender_user_id: x.tender_user_id.map(Into::into),
            organizer_id: x.organizer_id.map(|x| x.unwrap_or_default()),
            section_id: x.section_id.map(Into::into),
            // branch: x.branch.map(Into::into),
            contract_subject: x.contract_subject.map(Into::into),
            minimal_requirements: x.minimal_requirements.map(Into::into),
            number_customer: x.number_customer.map(Into::into),
            number_cgg: x.number_cgg.map(|x| x.unwrap_or_default()),
            currency_id: x.currency_id.map(Into::into),
            commission_date: x.commission_date.map(Option::unwrap_or_default),
            //------------------------
            kod_st_buda: x.kod_st_buda,
            okdp2: x.okdp2,
            category_id: x.category_id,
            code_type: x.code_type,
            //-------------------------
            vat_id: x.vat_id,
            sum_excluded_vat: x.sum_excluded_vat.map(Into::into),
            sum_vat: x.sum_vat.map(Into::into),
            sum_included_vat: x.sum_included_vat.map(Into::into),
            sum_excluded_vat_rub: x.sum_excluded_vat_rub.map(Into::into),
            sum_vat_rub: x.sum_vat_rub.map(Into::into),
            sum_included_vat_rub: x.sum_included_vat_rub.map(Into::into),
            purchasing_method_id: x.purchasing_method_id.map(Into::into),
            purchasing_type_id: x.purchasing_type_id.map(Into::into),
            purchasing_kind_id: x.purchasing_kind_id.map(Into::into),
            regulation_document_id: x.regulation_document_id.map(|x| x as i16),
            publication_type_id: x.publication_type_id.map(Into::into),
            master_system_id: x.master_system_id.map(|x| x.unwrap_or_default()),
            funding_source_id: x.funding_source_id.map(Into::into),
            purchasing_trend_id: x.purchasing_trend_id.map(Into::into),
            budget_item_id: x.budget_item_id.map(Into::into),
            payment_balance_item_id: x.payment_balance_item_id.map(Into::into),
            //-------------------------
            is_smb: x.is_smb,
            is_smb_sub: x.is_smb_sub,
            smb_exception_id: x.smb_exception_id.map(|x| x.unwrap_or_default()),
            smb_sub_percent: x.smb_sub_percent.map(|x| x.unwrap_or_default()),
            smb_sub_sum: x.smb_sub_sum.map(|x| x.unwrap_or_default()),
            customer_note: x.customer_note.map(|x| x.unwrap_or_default()),
            documentation_date: x.documentation_date.map(|x| x.unwrap_or_default()),
            publication_date: x.publication_date.map(|x| x.unwrap_or_default()),
            publication_start_date: x.publication_start_date.map(Into::into),
            // publication_end_date: x.publication_end_date.map(Into::into),
            bid_opening_date: x.bid_opening_date.map(Into::into),
            summing_up_date: x.summing_up_date.map(|x| x.unwrap_or_default()),
            contract_sign_date: x.contract_sign_date.map(Into::into),
            contract_sing_date: x.contract_sing_date.map(Into::into),
            delivery_start_date: x.delivery_start_date.map(Into::into),
            delivery_end_date: x.delivery_end_date.map(Into::into),
            organizer_note: x.organizer_note.map(Into::into),
            competitive_note_for_expert: x
                .competitive_note_for_expert
                .map(|x| x.unwrap_or_default()),
            //-------------------------
            single_supplier_reason_id: x.single_supplier_reason_id.map(Into::into),
            // single_supplier_reason_code: x
            //     .single_supplier_reason_code
            //     .map(Into::into),
            single_supplier_expert_id: x
                .single_supplier_expert_id
                .map(|x| x.unwrap_or_default()),
            single_supplier_decision_id: x
                .single_supplier_decision_id
                .map(|x| x.unwrap_or_default() as i16),
            single_supplier_decision_resume: x
                .single_supplier_decision_resume
                .map(|x| x.unwrap_or_default()),
            single_supplier_note_for_expert: x
                .single_supplier_note_for_expert
                .map(Into::into),
            supplier_id: x.supplier_id.map(Into::into),
            supplier_text: x.supplier_text.map(|x| x.unwrap_or_default()),
            is_affiliated: x.is_affiliated,
            is_supplier_smb: x.is_supplier_smb,
            is_competitive_now: x.is_competitive_now,
            // TODO: Decide how to handle management order number.
            management_order_date: x
                .management_order_date
                .map(|x| x.unwrap_or_default()),
            reason_document: x.reason_document.map(|x| x.unwrap_or_default()),
            is_approved_by_d646: x.is_approved_by_d646.map(Into::into),
            is_price_analysis_by_d646: x.is_price_analysis_by_d646,
            is_approved_by_d647: x.is_approver_by_d647,
            is_pricing_by_complectation: x.is_pricing_by_complectation,
            // is_pricing: x.is_pricing.map(Into::into),
            commission_kind_id: x.commission_kind_id.map(Into::into),
            //-------------------------
            status_scheme_id: x.status_scheme_id.map(Into::into),
            status_id: x.status_id.map(Into::into),
            // status_note: x.status_note.map(Into::into),
            is_priority_project: x.is_priority_project,
            priority_project_document: x
                .priority_project_document
                .map(|x| x.unwrap_or_default()),
            is_priority_introductory: x.is_priority_introductory,
            priority_introductory_date: x
                .priority_introductory_date
                .map(|x| x.unwrap_or_default()),
            priority_introductory_document: x
                .priority_introductory_document
                .map(|x| x.unwrap_or_default()),
            is_priority_repair: x.is_priority_repair,
            priority_repair_document: x
                .priority_repair_document
                .map(|x| x.unwrap_or_default()),
            is_priority_ozp: x.is_priority_ozp,
            priority_ozp_document: x
                .priority_ozp_document
                .map(|x| x.unwrap_or_default()),
            is_priority_income_contract: x.is_priority_income_contract,
            priority_income_contract_document: x
                .priority_income_contract_document
                .map(|x| x.unwrap_or_default()),
            priority_income_contract_partner_id: x
                .priority_income_contract_partner_id
                .map(|x| x.unwrap_or_default()),
            is_priority_other: x.is_priority_other,
            is_priority_far_eastern: x.is_priority_far_eastern.map(Into::into),
            // is_priority_far_nonprofit: x.is_priority_far_nonprofit.map(Into::into),
            is_headquarters: x.is_headquarters,
            is_first_time: x.is_first_time,
            //----------------------
            is_list_price: x.is_list_price,
            is_lease_supplier_selection: x.is_lease_supplier_selection,
            is_performance_indicator: x.is_performance_indicator,
            // is_need_for_departments: x.is_need_for_departments.map(Into::into),
            // is_sanctioned: x.is_sanctioned.map(Into::into),
            pricing_method_id: x.pricing_method_id.map(Into::into),
            pricing_expert_id: x.pricing_expert_id.map(|x| x.unwrap_or_default()),
            pricing_resume: x.pricing_resume.map(|x| x.unwrap_or_default()),
            general_contract_date: x
                .general_contract_date
                .map(|x| x.unwrap_or_default()),
            general_contract_number: x
                .general_contract_number
                .map(|x| x.unwrap_or_default()),
            general_contract_stages: x
                .general_contract_stages
                .map(|x| x.unwrap_or_default()),
            //-----------------------
            is_onm: x.is_onm,
            is_agent_fee: x.is_agent_fee,
            agent_contract_number: x
                .agent_contract_number
                .map(Option::unwrap_or_default),
            is_design_stage: x.is_design_stage,
            repair_stage_id: x.repair_stage_id.map(Option::unwrap_or_default),
            is_gas_supply: x.is_gas_supply,
            is_little_cost: x.is_little_cost,
            is_banking_support: x.is_banking_support,
            is_innovative: x.is_innovative,
            is_to_publish: x.is_to_publish,
            rationale_for_not_publication: x
                .rationale_for_not_publication
                .map(Option::unwrap_or_default),
            rationale_for_publication: x
                .rationale_for_publication
                .map(Option::unwrap_or_default),
            is_cooperative: x.is_cooperative,
            is_not_purchase: x.is_not_purchase,
            is_under_control: x.is_under_control,
            rationale_is_under_control: x
                .rationale_is_under_control
                .map(Option::unwrap_or_default),
            technical_developer: x
                .technical_developer
                .map(Option::unwrap_or_default),
            //-----------------------
            expert_conclusion_id: x.expert_conclusion_id,
            is_check_documentation: x.is_check_documentation,
            check_documentation_date: x
                .check_documentation_date
                .map(|t| t.map(Into::into)),
            //------------------------
            savings_accounting_id: x.savings_accounting_id,
            savings_sum_excluded_vat: x.savings_sum_excluded_vat,
            savings_sum_excluded_vat_rub: x.savings_sum_excluded_vat_rub,
            savings_sum_included_vat: x.savings_sum_included_vat,
            savings_sum_included_vat_rub: x.savings_sum_included_vat_rub,
            //------------------------
            pricing_vat_id: x.pricing_vat_id,
            pricing_currency_id: x.pricing_currency_id,
            pricing_currency_rate: x.pricing_currency_rate,
            pricing_sum_excluded_vat: x.pricing_sum_excluded_vat,
            pricing_sum_excluded_vat_rub: x.pricing_sum_excluded_vat_rub,
            pricing_sum_included_vat: x.pricing_sum_included_vat,
            pricing_sum_included_vat_rub: x.pricing_sum_included_vat_rub,
            pricing_sum_vat: x.pricing_sum_vat,
            pricing_sum_vat_rub: x.pricing_sum_vat_rub,
            pricing_transportation_vat_id: x.pricing_transportation_vat_id,
            pricing_transportation_price: x.pricing_transportation_price,
            pricing_transportation_price_rub: x.pricing_transportation_price_rub,
            pricing_transportation_sum_vat: x.pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub: x
                .pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat: x
                .pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub: x
                .pricing_transportation_sum_included_vat_rub,
            pricing_total_sum: x.pricing_total_sum,
            pricing_total_sum_rub: x.pricing_total_sum_rub,
            pricing_started_at: x.pricing_started_at.map(Into::into),
            limit_on_construction: x.limit_on_construction,
            limit_on_works: x.limit_on_works,
            priority: x.priority,
            priority_income_contract_partner_text: x
                .priority_income_contract_partner_text,
            extract_number_d646: x.extract_number_d646,
            extract_date_d646: x.extract_date_d646,
            extract_sum_included_vat_rub_d646: x.extract_sum_included_vat_rub_d646,
            extract_number_d647: x.extract_number_d647,
            extract_date_d647: x.extract_date_d647,
            extract_sum_included_vat_rub_d647: x.extract_sum_included_vat_rub_d647,
            product_type_id: x.product_type_id,
            description: x.description,
            is_removed: x.is_removed,
            control_pp_2013: x.control_pp_2013.map(Into::into),
            is_no_qualification: x.is_no_qualification,
            is_commission: x.is_commission,
            is_nko: x.is_nko,
            is_priority_nonprofit: x.is_priority_nonprofit,
            created_at: x.created_at.map(Into::into),
            created_by: x.created_by,
            changed_at: x.changed_at.map(Into::into),
            changed_by: x.changed_by,
            version_type: x.version_type,
            version_number: x.version_number,
            active_uuid: x.active_uuid,
            items_number: x.items_number,
            currency_rate: x.currency_rate,
            posting_date: x.posting_date,
            reason_cancel_id: x.reason_cancel_id,
            replaced_id: x.replaced_id,
            ..Default::default()
        }
    }
}

impl TryFrom<PlanLegacyRep> for PlanRep {
    type Error = String;
    fn try_from(x: PlanLegacyRep) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: x.uuid.map(Into::into),
            id: x
                .id
                .map(|x| x.parse::<i64>())
                .transpose()
                .map_err(|x| x.to_string())?,
            is_actual: x.is_actual,
            year: x.year.map(Into::into),
            customer_id: x.customer_id.map(Into::into),
            declarant_id: x.declarant_id.map(Into::into),
            agent_id: x.agent_id.map(Into::into),
            initiator_user_id: x.initiator_user_id.map(Into::into),
            tender_user_id: x.tender_user_id.map(Into::into),
            organizer_id: x.organizer_id.map(Into::into),
            section_id: x.section_id.map(Into::into),
            // branch: x.branch.map(Into::into),
            contract_subject: x.contract_subject.map(Into::into),
            minimal_requirements: x.minimal_requirements.map(Into::into),
            number_customer: x.number_customer.map(Into::into),
            number_cgg: x.number_cgg.map(Into::into),
            currency_id: x.currency_id.map(|x| x as i16),
            commission_date: x.commission_date.map(Some),
            //------------------------
            kod_st_buda: x.kod_st_buda,
            okdp2: x.okdp2,
            category_id: x.category_id,
            code_type: x.code_type,
            //-------------------------
            vat_id: x.vat_id.map(Into::into),
            sum_excluded_vat: x.sum_excluded_vat.map(Into::into),
            sum_vat: x.sum_vat.map(Into::into),
            sum_included_vat: x.sum_included_vat.map(Into::into),
            sum_excluded_vat_rub: x.sum_excluded_vat_rub.map(Into::into),
            sum_vat_rub: x.sum_vat_rub.map(Into::into),
            sum_included_vat_rub: x.sum_included_vat_rub.map(Into::into),
            purchasing_method_id: x.purchasing_method_id.map(Into::into),
            purchasing_type_id: x.purchasing_type_id.map(Into::into),
            purchasing_kind_id: x.purchasing_kind_id.map(Into::into),
            regulation_document_id: x.regulation_document_id.map(Into::into),
            publication_type_id: x.publication_type_id.map(Into::into),
            master_system_id: x.master_system_id.map(Into::into),
            funding_source_id: x.funding_source_id.map(Into::into),
            purchasing_trend_id: x.purchasing_trend_id.map(Into::into),
            budget_item_id: x.budget_item_id.map(Into::into),
            payment_balance_item_id: x.payment_balance_item_id.map(Into::into),
            //-------------------------
            is_smb: x.is_smb.map(Into::into),
            is_smb_sub: x.is_smb_sub.map(Into::into),
            smb_exception_id: x.smb_exception_id.map(Into::into),
            smb_sub_percent: x.smb_sub_percent.map(|x| match x {
                0 => None,
                x => Some(x),
            }),
            smb_sub_sum: x.smb_sub_sum.map(Into::into),
            customer_note: x.customer_note.map(Into::into),
            documentation_date: x.documentation_date.map(Into::into),
            publication_date: x.publication_date.map(Into::into),
            publication_start_date: x.publication_start_date.map(Into::into),
            // publication_end_date: x.publication_end_date.map(Into::into),
            bid_opening_date: x.bid_opening_date.map(Into::into),
            summing_up_date: x.summing_up_date.map(Into::into),
            contract_sign_date: x.contract_sign_date.map(Into::into),
            contract_sing_date: x.contract_sing_date.map(Into::into),
            delivery_start_date: x.delivery_start_date.map(Into::into),
            delivery_end_date: x.delivery_end_date.map(Into::into),
            organizer_note: x.organizer_note.map(Into::into),
            competitive_note_for_expert: x
                .competitive_note_for_expert
                .map(Into::into),
            //-------------------------
            single_supplier_reason_id: x.single_supplier_reason_id.map(Into::into),
            // single_supplier_reason_code: x
            //     .single_supplier_reason_code
            //     .map(Into::into),
            single_supplier_expert_id: x.single_supplier_expert_id.map(Into::into),
            single_supplier_decision_id: x.single_supplier_decision_id.map(|x| {
                match x {
                    0 => None,
                    x => Some(x as i32),
                }
            }),
            single_supplier_decision_resume: x
                .single_supplier_decision_resume
                .map(Into::into),
            single_supplier_note_for_expert: x
                .single_supplier_note_for_expert
                .map(Into::into),
            supplier_id: x.supplier_id.map(Into::into),
            supplier_text: x.supplier_text.map(Into::into),
            is_affiliated: x.is_affiliated.map(Into::into),
            is_supplier_smb: x.is_supplier_smb.map(Into::into),
            is_competitive_now: x.is_competitive_now.map(Into::into),
            // TODO: Decide how to handle management order number.
            management_order_date: x.management_order_date.map(Into::into),
            reason_document: x.reason_document.map(Into::into),
            is_approved_by_d646: x.is_approved_by_d646.map(Into::into),
            is_price_analysis_by_d646: x.is_price_analysis_by_d646.map(Into::into),
            is_approver_by_d647: x.is_approved_by_d647.map(Into::into),
            is_pricing_by_complectation: x
                .is_pricing_by_complectation
                .map(Into::into),
            // is_pricing: x.is_pricing.map(Into::into),
            commission_kind_id: x.commission_kind_id.map(Into::into),
            //-------------------------
            status_scheme_id: x.status_scheme_id.map(Into::into),
            status_id: x.status_id.map(Into::into),
            // status_note: x.status_note.map(Into::into),
            is_priority_project: x.is_priority_project.map(Into::into),
            priority_project_document: x.priority_project_document.map(Into::into),
            is_priority_introductory: x.is_priority_introductory.map(Into::into),
            priority_introductory_date: x
                .priority_introductory_date
                .map(Into::into),
            priority_introductory_document: x
                .priority_introductory_document
                .map(Into::into),
            is_priority_repair: x.is_priority_repair.map(Into::into),
            priority_repair_document: x.priority_repair_document.map(Into::into),
            is_priority_ozp: x.is_priority_ozp.map(Into::into),
            priority_ozp_document: x.priority_ozp_document.map(Into::into),
            is_priority_income_contract: x
                .is_priority_income_contract
                .map(Into::into),
            priority_income_contract_document: x
                .priority_income_contract_document
                .map(Into::into),
            priority_income_contract_partner_id: x
                .priority_income_contract_partner_id
                .map(Into::into),
            is_priority_other: x.is_priority_other.map(Into::into),
            // is_priority_far_nonprofit: x.is_priority_far_nonprofit.map(Into::into),
            is_headquarters: x.is_headquarters.map(Into::into),
            is_first_time: x.is_first_time.map(Into::into),
            //----------------------
            is_list_price: x.is_list_price.map(Into::into),
            is_lease_supplier_selection: x
                .is_lease_supplier_selection
                .map(Into::into),
            is_performance_indicator: x.is_performance_indicator.map(Into::into),
            // is_need_for_departments: x.is_need_for_departments.map(Into::into),
            // is_sanctioned: x.is_sanctioned.map(Into::into),
            pricing_method_id: x.pricing_method_id.map(Into::into),
            pricing_expert_id: x.pricing_expert_id.map(|x| match x {
                0 => None,
                x => Some(x),
            }),
            pricing_resume: x.pricing_resume.map(Into::into),
            general_contract_date: x.general_contract_date.map(Into::into),
            general_contract_number: x.general_contract_number.map(Into::into),
            general_contract_stages: x.general_contract_stages.map(Into::into),
            // is_material_registry: x.is_material_registry.map(Into::into),
            // start_price_uuid: x.start_price_uuid.map(Into::into),
            // start_price_type: x.start_price_type.map(Into::into),
            // ---------------------
            // is_other_qualification: x.is_other_qualification.map(Into::into),
            // other_qualification_rationale: x
            //     .other_qualification_rationale
            //     .map(Into::into),
            //-----------------------
            // mery_prin: x.mery_prin.map(Into::into),
            control_pp_2013: x.control_pp_2013.map(Into::into),
            //-----------------------
            is_onm: x.is_onm,
            is_agent_fee: x.is_agent_fee,
            agent_contract_number: x.agent_contract_number.map(Some),
            is_design_stage: x.is_design_stage,
            repair_stage_id: x.repair_stage_id.map(Some),
            is_gas_supply: x.is_gas_supply,
            is_little_cost: x.is_little_cost,
            is_banking_support: x.is_banking_support,
            is_innovative: x.is_innovative,
            is_to_publish: x.is_to_publish,
            rationale_for_not_publication: x
                .rationale_for_not_publication
                .map(Some),
            rationale_for_publication: x.rationale_for_publication.map(Some),
            is_cooperative: x.is_cooperative,
            is_not_purchase: x.is_not_purchase,
            is_under_control: x.is_under_control,
            rationale_is_under_control: x.rationale_is_under_control.map(Some),
            technical_developer: x.technical_developer.map(Some),
            //-----------------------
            expert_conclusion_id: x.expert_conclusion_id,
            is_check_documentation: x.is_check_documentation,
            check_documentation_date: x
                .check_documentation_date
                .map(|t| t.map(Into::into)),
            //------------------------
            savings_accounting_id: x.savings_accounting_id,
            savings_sum_excluded_vat: x.savings_sum_excluded_vat,
            savings_sum_excluded_vat_rub: x.savings_sum_excluded_vat_rub,
            savings_sum_included_vat: x.savings_sum_included_vat,
            savings_sum_included_vat_rub: x.savings_sum_included_vat_rub,
            //------------------------
            pricing_vat_id: x.pricing_vat_id,
            pricing_currency_id: x.pricing_currency_id,
            pricing_currency_rate: x.pricing_currency_rate,
            pricing_sum_excluded_vat: x.pricing_sum_excluded_vat,
            pricing_sum_excluded_vat_rub: x.pricing_sum_excluded_vat_rub,
            pricing_sum_included_vat: x.pricing_sum_included_vat,
            pricing_sum_included_vat_rub: x.pricing_sum_included_vat_rub,
            pricing_sum_vat: x.pricing_sum_vat,
            pricing_sum_vat_rub: x.pricing_sum_vat_rub,
            pricing_transportation_vat_id: x.pricing_transportation_vat_id,
            pricing_transportation_price: x.pricing_transportation_price,
            pricing_transportation_price_rub: x.pricing_transportation_price_rub,
            pricing_transportation_sum_vat: x.pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub: x
                .pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat: x
                .pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub: x
                .pricing_transportation_sum_included_vat_rub,
            pricing_total_sum: x.pricing_total_sum,
            pricing_total_sum_rub: x.pricing_total_sum_rub,
            pricing_started_at: x.pricing_started_at.map(Into::into),
            limit_on_construction: x.limit_on_construction,
            limit_on_works: x.limit_on_works,
            priority: x.priority,
            priority_income_contract_partner_text: x
                .priority_income_contract_partner_text,
            extract_number_d646: x.extract_number_d646,
            extract_date_d646: x.extract_date_d646,
            extract_sum_included_vat_rub_d646: x.extract_sum_included_vat_rub_d646,
            extract_number_d647: x.extract_number_d647,
            extract_date_d647: x.extract_date_d647,
            extract_sum_included_vat_rub_d647: x.extract_sum_included_vat_rub_d647,
            product_type_id: x.product_type_id,
            description: x.description,
            is_removed: x.is_removed,
            is_no_qualification: x.is_no_qualification,
            is_commission: x.is_commission,
            is_priority_far_eastern: x.is_priority_far_eastern,
            is_nko: x.is_nko,
            is_priority_nonprofit: x.is_priority_nonprofit,
            created_at: x.created_at.map(Into::into),
            created_by: x.created_by,
            changed_at: x.changed_at.map(Into::into),
            changed_by: x.changed_by,
            version_type: x.version_type,
            version_number: x.version_number,
            active_uuid: x.active_uuid,
            items_number: x.items_number,
            currency_rate: x.currency_rate,
            posting_date: x.posting_date,
            reason_cancel_id: x.reason_cancel_id,
            replaced_id: x.replaced_id,
            ..Default::default()
        })
    }
}
