//! Работает с ДС (contract_amendment) из монолита.
use asez2_shared_db::db_item::int_array::AsezArray;
use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::db_item::{DbItemExt, FieldTolerance};
use asez2_shared_db::{DbAdaptor, DbItem};
use monolith_service::dto::time::PlanningTimestamp;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{maths::*, PlanStatus};
use crate::{
    CommissionKind, ContractAmendmentRep, ExpertConclusionId, SavingsAccountingId,
    TypeOfPurchaseId,
};

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
#[item_table = "contract_amendment_legacy"]
#[item_skip_field_tolerance]
pub struct ContractAmendmentLegacy {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub id: String,
    pub version_type: i16,
    pub version_number: i16,
    pub active_uuid: Uuid,
    pub is_actual: bool,
    pub is_pur_asbu: bool,
    pub system_number: String,
    pub external_number: String,
    pub customer_id: i32,
    pub declarant_id: i32,
    pub branch: String,
    pub agent_id: i32,
    pub assignee_id: i32,
    pub project_institute_id: i32,
    pub organizer_id: i32,
    pub initiator_user_id: i32,
    pub tender_user_id: i32,
    //-----------------------------
    pub year: i16,
    pub purchasing_type_id: i16,
    pub purchasing_method_id: i16,
    pub section_id: i16,
    pub funding_source_id: i16,
    pub single_supplier_reason_id: i16,
    pub number_cgg: String,
    pub contract_system_number: String,
    pub contract_external_number: String,
    pub number_eis: String,
    pub supplier_id: i32,
    pub supplier_text: String,
    pub contract_subject: String,
    pub contract_type_id: i16,
    pub accepted_volume_included_vat_rub: i64,
    pub is_banking_support: bool,
    pub is_with_amendments: bool,
    //---------------------------
    pub is_secret_state: bool,
    pub is_secret_commercial: bool,
    pub is_material_registry: bool,
    pub is_to_publish: bool,
    pub rationale: String,
    pub funding_availability: String,
    pub is_chairman_order: bool,
    pub is_chairman_order_secret: bool,
    pub chairman_order_number: String,
    pub chairman_order_date: AsezDate,
    pub is_vice_chairman_order: bool,
    pub is_with_approval: bool,
    pub is_need_for_departments: bool,
    pub is_sum_increase_was_specified: bool,
    pub is_sum_increase_10_percent: bool,
    pub is_sum_increase_first_time: bool,
    pub is_sum_changed_via_key_rate: bool,
    //------------------------------
    pub vat_id: VatId,
    pub sum_excluded_vat: CurrencyValue,
    pub sum_vat: CurrencyValue,
    pub sum_included_vat: CurrencyValue,
    pub currency_id: i32,
    pub currency_rate: CurrencyRate,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_vat_rub: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
    pub initial_sum_excluded_vat: CurrencyValue,
    pub initial_sum_included_vat: CurrencyValue,
    pub initial_currency_id: i32,
    pub initial_currency_rate: CurrencyRate,
    pub initial_sum_excluded_vat_rub: CurrencyValue,
    pub initial_sum_included_vat_rub: CurrencyValue,
    pub initial_vat_id: VatId,
    pub initial_sum_vat: CurrencyValue,
    //----------------------------
    pub initial_sum_vat_rub: CurrencyValue,
    pub previous_sum_excluded_vat: CurrencyValue,
    pub previous_sum_vat: CurrencyValue,
    pub previous_sum_included_vat: CurrencyValue,
    pub previous_currency_id: i32,
    pub previous_currency_rate: CurrencyRate,
    pub previous_sum_excluded_vat_rub: CurrencyValue,
    pub previous_sum_vat_rub: CurrencyValue,
    pub previous_sum_included_vat_rub: CurrencyValue,
    pub delta_sum_excluded_vat: CurrencyValue,
    pub delta_sum_included_vat: CurrencyValue,
    pub delta_sum_excluded_vat_rub: CurrencyValue,
    pub delta_sum_included_vat_rub: CurrencyValue,
    pub delta_sum_vat: CurrencyValue,
    pub delta_sum_vat_rub: CurrencyValue,
    pub sign_date: AsezDate,
    pub start_date: AsezDate,
    pub end_date: AsezDate,
    pub whole_start_date: AsezDate,
    pub whole_end_date: AsezDate,
    //--------------------------------
    pub initial_start_date: AsezDate,
    pub initial_end_date: AsezDate,
    pub initial_whole_start_date: AsezDate,
    pub initial_whole_end_date: AsezDate,
    pub previous_start_date: AsezDate,
    pub previous_end_date: AsezDate,
    pub previous_whole_start_date: AsezDate,
    pub previous_whole_end_date: AsezDate,
    pub is_priority_project: bool,
    pub priority_project_document: String,
    pub is_priority_introductory: bool,
    pub priority_introductory_date: AsezDate,
    pub priority_introductory_document: String,
    pub is_priority_repair: bool,
    pub priority_repair_document: String,
    pub is_priority_ozp: bool,
    //-------------------------------
    pub priority_ozp_document: String,
    pub is_priority_income_contract: bool,
    pub priority_income_contract_document: String,
    pub priority_income_contract_partner_id: i32,
    pub priority_income_contract_partner_text: String,
    pub is_priority_far_eastern: bool,
    pub is_priority_nonprofit: bool,
    pub is_priority_other: bool,
    pub is_priority_headquarters: bool,
    pub is_performance_indicator: bool,
    pub pricing_expert_id: i32,
    pub pricing_method_id: i16,
    pub pricing_resume: String,
    pub pricing_approved_sum_included_vat: CurrencyValue,
    pub pricing_start_date: AsezDate,
    pub pricing_end_date: AsezDate,
    pub approving_expert_id: i32,
    pub approving_decision_id: i32,
    pub approving_decision_resume: String,
    //-------------------------------
    pub approving_note_for_expert: String,
    pub approving_start_sum_included_vat_rub: CurrencyValue,
    pub approving_end_sum_included_vat_rub: CurrencyValue,
    pub approving_start_date: AsezDate,
    pub approving_end_date: AsezDate,
    pub commission_kind_id: CommissionKind,
    pub commission_date: AsezDate,
    pub number_customer: String,
    pub status_scheme_id: i16,
    pub status_id: PlanStatus,
    pub status_note: String,
    // status_moves: this filed is a vector field and we do not use it.
    pub status_case: String,
    pub is_approved_by_d646: bool,
    pub is_pricing_by_d646: bool,
    pub is_pricing_by_d647: bool,
    pub is_pricing_by_complectation: bool,
    pub plan_id: String,
    pub purchase_id: String,
    pub purchase_number_eis: String,
    //----------------------------------
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

    pub pricing_delta_currency_id: Option<i64>,
    pub pricing_delta_currency_rate: Option<CurrencyRate>,
    pub pricing_delta_sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_delta_sum_excluded_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_sum_included_vat: Option<CurrencyValue>,
    pub pricing_delta_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_sum_vat: Option<CurrencyValue>,
    pub pricing_delta_sum_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_total_sum: Option<CurrencyValue>,
    pub pricing_delta_total_sum_rub: Option<CurrencyValue>,
    pub pricing_delta_transportation_price: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_included_vat: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_vat: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_vat_rub: Option<CurrencyValue>,
    // pricing_started_at new on 2024.11.25
    pub pricing_started_at: PlanningTimestamp,
    //----------------------------------
    pub quotation_id: String,
    pub contract_id: String,
    pub claim_id: i64,
    pub items_number: i32,
    pub posting_date: AsezDate,
    pub created_at: PlanningTimestamp,
    pub created_by: i32,
    pub changed_at: PlanningTimestamp,
    pub changed_by: i32,
    //-----------------------
    pub expert_conclusion_id: Option<ExpertConclusionId>,
    pub is_check_documentation: bool,
    pub check_documentation_date: Option<PlanningTimestamp>,
    //---------------------------
    pub kod_st_buda: Option<String>,
    pub okdp2: Option<String>,
    pub category_id: Option<String>,
    pub code_type: Option<TypeOfPurchaseId>,
    //--------------------------------
    #[adaptor_rename = "kinds"]
    pub contract_amendment_types: AsezArray<i32>,
    //--------------------------------
    pub savings_accounting_id: SavingsAccountingId,
    pub savings_sum_excluded_vat: Option<CurrencyValue>,
    pub savings_sum_excluded_vat_rub: Option<CurrencyValue>,
    pub savings_sum_included_vat: Option<CurrencyValue>,
    pub savings_sum_included_vat_rub: Option<CurrencyValue>,
    //-----------------------
    // It is not clear whether these fields exist in GPI.
    pub product_type_id: i16,
    pub repair_stage_id: i16,
    pub previous_vat_id: VatId,
    pub close_date: AsezDate,
    pub termination_date: AsezDate,
    pub associated_plan_id: i64,
    pub payment_balance_item_id: i16,
    pub budget_item_id: i16,
    // kinds: not used as is a list.
    // items: not used as is a a list.
    // files: not used as is a list.
    // status_history: not used as is a list.
    // approvers: not used as is a list.
    // _meta: not used as is a list.
}

fn parse(x: Option<String>) -> Result<Option<i64>, String> {
    x.map(|x| x.parse::<i64>()).transpose().map_err(|x| x.to_string())
}

impl From<ContractAmendmentRep> for ContractAmendmentLegacyRep {
    fn from(x: ContractAmendmentRep) -> Self {
        Self {
            uuid: x.uuid.map(Into::into),
            id: x.id.map(|x| x.to_string()),
            version_type: x.version_type,
            version_number: x.version_number,
            active_uuid: x.active_uuid.map(Into::into),
            is_actual: x.is_actual.map(Into::into),
            is_pur_asbu: x.is_pur_asbu.map(Into::into),
            system_number: x.system_number.map(Into::into),
            external_number: x.external_number.map(Into::into),
            customer_id: x.customer_id.map(Into::into),
            declarant_id: x.declarant_id.map(Into::into),
            branch: x.branch.map(Into::into),
            agent_id: x.agent_id.map(Into::into),
            assignee_id: x.assignee_id.map(Into::into),
            project_institute_id: x.project_institute_id.map(Into::into),
            organizer_id: x.organizer_id.map(Into::into),
            initiator_user_id: x.initiator_user_id.map(Into::into),
            tender_user_id: x.tender_user_id.map(Into::into),
            //-----------------------------
            year: x.year.map(Into::into),
            purchasing_type_id: x.purchasing_type_id,
            purchasing_method_id: x.purchasing_method_id,
            section_id: x.section_id.map(Into::into),
            funding_source_id: x.funding_source_id.map(Into::into),
            single_supplier_reason_id: x.single_supplier_reason_id.map(Into::into),
            number_cgg: x.number_cgg.map(Into::into),
            contract_system_number: x.contract_system_number.map(Into::into),
            contract_external_number: x.contract_external_number.map(Into::into),
            number_eis: x.number_eis.map(Into::into),
            supplier_id: x.supplier_id.map(Into::into),
            contract_subject: x.contract_subject.map(Into::into),
            contract_type_id: x.contract_type_id.map(Into::into),
            accepted_volume_included_vat_rub: x
                .accepted_volume_included_vat_rub
                .map(Into::into),
            is_banking_support: x.is_banking_support.map(Into::into),
            is_with_amendments: x.is_with_amendments.map(Into::into),
            //---------------------------
            is_secret_state: x.is_secret_state.map(Into::into),
            is_secret_commercial: x.is_secret_commercial.map(Into::into),
            is_material_registry: x.is_material_registry.map(Into::into),
            is_to_publish: x.is_to_publish.map(Into::into),
            rationale: x.rationale.map(Into::into),
            funding_availability: x.funding_availability.map(Into::into),
            is_chairman_order: x.is_chairman_order.map(Into::into),
            is_chairman_order_secret: x.is_chairman_order_secret.map(Into::into),
            chairman_order_number: x.chairman_order_number.map(Into::into),
            chairman_order_date: x.chairman_order_date.map(Into::into),
            is_vice_chairman_order: x.is_vice_chairman_order.map(Into::into),
            is_with_approval: x.is_with_approval.map(Into::into),
            is_need_for_departments: x.is_need_for_departments.map(Into::into),
            is_sum_increase_was_specified: x
                .is_sum_increase_was_specified
                .map(Into::into),
            is_sum_changed_via_key_rate: x
                .is_sum_changed_via_key_rate
                .map(Into::into),
            //------------------------------
            vat_id: x.vat_id,
            sum_excluded_vat: x.sum_excluded_vat.map(Into::into),
            sum_vat: x.sum_vat.map(Into::into),
            sum_included_vat: x.sum_included_vat.map(Into::into),
            currency_id: x.currency_id.map(Into::into),
            currency_rate: x.currency_rate.map(Into::into),
            sum_excluded_vat_rub: x.sum_excluded_vat_rub.map(Into::into),
            sum_vat_rub: x.sum_vat_rub.map(Into::into),
            sum_included_vat_rub: x.sum_included_vat_rub.map(Into::into),
            initial_sum_excluded_vat: x.initial_sum_excluded_vat.map(Into::into),
            initial_sum_included_vat: x.initial_sum_included_vat.map(Into::into),
            initial_currency_id: x.initial_currency_id.map(Into::into),
            initial_currency_rate: x.initial_currency_rate.map(Into::into),
            initial_sum_excluded_vat_rub: x
                .initial_sum_excluded_vat_rub
                .map(Into::into),
            initial_sum_included_vat_rub: x
                .initial_sum_included_vat_rub
                .map(Into::into),
            initial_vat_id: x.initial_vat_id.map(Into::into),
            initial_sum_vat: x.initial_sum_vat.map(Into::into),
            //----------------------------
            initial_sum_vat_rub: x.initial_sum_vat_rub.map(Into::into),
            previous_sum_excluded_vat: x.previous_sum_excluded_vat.map(Into::into),
            previous_sum_vat: x.previous_sum_vat.map(Into::into),
            previous_sum_included_vat: x.previous_sum_included_vat.map(Into::into),
            previous_currency_id: x.previous_currency_id.map(Into::into),
            previous_currency_rate: x.previous_currency_rate,
            previous_sum_excluded_vat_rub: x
                .previous_sum_excluded_vat_rub
                .map(Into::into),
            previous_sum_vat_rub: x.previous_sum_vat_rub.map(Into::into),
            previous_sum_included_vat_rub: x
                .previous_sum_included_vat_rub
                .map(Into::into),
            delta_sum_excluded_vat: x.delta_sum_excluded_vat.map(Into::into),
            delta_sum_included_vat: x.delta_sum_included_vat.map(Into::into),
            delta_sum_excluded_vat_rub: x
                .delta_sum_excluded_vat_rub
                .map(Into::into),
            delta_sum_included_vat_rub: x
                .delta_sum_included_vat_rub
                .map(Into::into),
            delta_sum_vat: x.delta_sum_vat.map(Into::into),
            delta_sum_vat_rub: x.delta_sum_vat_rub.map(Into::into),
            sign_date: x.sign_date.map(Into::into),
            start_date: x.start_date.map(Into::into),
            end_date: x.end_date.map(Into::into),
            whole_start_date: x.whole_start_date.map(Into::into),
            whole_end_date: x.whole_end_date.map(Into::into),
            //--------------------------------
            initial_start_date: x.initial_start_date.map(Into::into),
            initial_end_date: x.initial_end_date.map(Into::into),
            initial_whole_start_date: x.initial_whole_start_date.map(Into::into),
            initial_whole_end_date: x.initial_whole_end_date.map(Into::into),
            previous_start_date: x.previous_start_date.map(Into::into),
            previous_end_date: x.previous_end_date.map(Into::into),
            previous_whole_start_date: x.previous_whole_start_date.map(Into::into),
            previous_whole_end_date: x.previous_whole_end_date.map(Into::into),
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
            //-------------------------------
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
            priority_income_contract_partner_text: x
                .priority_income_contract_partner_text
                .map(Into::into),
            is_priority_far_eastern: x.is_priority_far_eastern.map(Into::into),
            is_priority_other: x.is_priority_other.map(Into::into),
            is_priority_headquarters: x.is_headquarters.map(Into::into),
            pricing_expert_id: x.pricing_expert_id.map(|x| x.unwrap_or_default()),
            pricing_method_id: x.pricing_method_id,
            pricing_resume: x.pricing_resume.map(|x| x.unwrap_or_default()),
            commission_kind_id: x.commission_kind_id.map(Into::into),
            commission_date: x.commission_date.map(Option::unwrap_or_default),
            //-------------------------------
            status_scheme_id: x.status_scheme_id,
            status_id: x.status_id.map(Into::into),
            is_approved_by_d646: x.is_approved_by_d646.map(Into::into),
            is_pricing_by_d646: x.is_pricing_by_d646.map(Into::into),
            is_pricing_by_d647: x.is_pricing_by_d647.map(Into::into),
            is_pricing_by_complectation: x
                .is_pricing_by_complectation
                .map(Into::into),
            plan_id: x.plan_id.map(|x| x.to_string()),
            purchase_id: x.purchase_id,
            //----------------------------------
            quotation_id: x.quotation_id,
            contract_id: x.contract_id,
            claim_id: x.claim_id.map(Into::into),
            items_number: x.items_number.map(Into::into),
            posting_date: x.posting_date.map(Into::into),
            created_at: x.created_at.map(Into::into),
            created_by: x.created_by.map(Into::into),
            changed_at: x.changed_at.map(Into::into),
            changed_by: x.changed_by.map(Into::into),
            kinds: x.contract_amendment_types,
            //----------------------------------
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

            pricing_delta_currency_id: x.pricing_delta_currency_id,
            pricing_delta_currency_rate: x.pricing_delta_currency_rate,
            pricing_delta_sum_excluded_vat: x.pricing_delta_sum_excluded_vat,
            pricing_delta_sum_excluded_vat_rub: x
                .pricing_delta_sum_excluded_vat_rub,
            pricing_delta_sum_included_vat: x.pricing_delta_sum_included_vat,
            pricing_delta_sum_included_vat_rub: x
                .pricing_delta_sum_included_vat_rub,
            pricing_delta_sum_vat: x.pricing_delta_sum_vat,
            pricing_delta_sum_vat_rub: x.pricing_delta_sum_vat_rub,
            pricing_delta_total_sum: x.pricing_delta_total_sum,
            pricing_delta_total_sum_rub: x.pricing_delta_total_sum_rub,
            pricing_delta_transportation_price: x
                .pricing_delta_transportation_price,
            pricing_delta_transportation_sum_included_vat: x
                .pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub: x
                .pricing_delta_transportation_sum_included_vat_rub,
            pricing_delta_transportation_sum_vat: x
                .pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub: x
                .pricing_delta_transportation_sum_vat_rub,
            pricing_started_at: x.pricing_started_at.map(Into::into),
            //---------------------------
            product_type_id: x.product_type_id,
            //-----------------------
            expert_conclusion_id: x.expert_conclusion_id,
            is_check_documentation: x.is_check_documentation,
            check_documentation_date: x
                .check_documentation_date
                .map(|t| t.map(Into::into)),
            //------------------------
            kod_st_buda: x.kod_st_buda,
            okdp2: x.okdp2,
            category_id: x.category_id,
            code_type: x.code_type,
            //---saving fields.
            savings_accounting_id: x.savings_accounting_id,
            savings_sum_excluded_vat: x.savings_sum_excluded_vat,
            savings_sum_excluded_vat_rub: x.savings_sum_excluded_vat_rub,
            savings_sum_included_vat: x.savings_sum_included_vat,
            savings_sum_included_vat_rub: x.savings_sum_included_vat_rub,
            //------------------------
            number_customer: x.number_customer,
            purchase_number_eis: x.purchase_number_eis,
            //------------------------
            budget_item_id: x.budget_item_id,
            payment_balance_item_id: x.payment_balance_item_id,
            associated_plan_id: x.associated_plan_id,
            termination_date: x.termination_date,
            close_date: x.close_date,
            previous_vat_id: x.previous_vat_id,
            repair_stage_id: x.repair_stage_id,
            ..Default::default()
        }
    }
}

impl TryFrom<ContractAmendmentLegacyRep> for ContractAmendmentRep {
    type Error = String;
    fn try_from(x: ContractAmendmentLegacyRep) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: x.uuid.map(Into::into),
            id: parse(x.id)?,
            version_type: x.version_type,
            version_number: x.version_number,
            active_uuid: x.active_uuid.map(Into::into),
            is_actual: x.is_actual.map(Into::into),
            is_pur_asbu: x.is_pur_asbu.map(Into::into),
            system_number: x.system_number.map(Into::into),
            external_number: x.external_number.map(Into::into),
            customer_id: x.customer_id.map(Into::into),
            declarant_id: x.declarant_id.map(Into::into),
            branch: x.branch.map(Into::into),
            agent_id: x.agent_id.map(Into::into),
            assignee_id: x.assignee_id.map(Into::into),
            project_institute_id: x.project_institute_id.map(Into::into),
            organizer_id: x.organizer_id.map(Into::into),
            initiator_user_id: x.initiator_user_id.map(Into::into),
            tender_user_id: x.tender_user_id.map(Into::into),
            //-----------------------------
            year: x.year.map(Into::into),
            purchasing_type_id: x.purchasing_type_id,
            purchasing_method_id: x.purchasing_method_id,
            section_id: x.section_id.map(Into::into),
            funding_source_id: x.funding_source_id.map(Into::into),
            single_supplier_reason_id: x.single_supplier_reason_id.map(Into::into),
            number_cgg: x.number_cgg.map(Into::into),
            contract_system_number: x.contract_system_number.map(Into::into),
            contract_external_number: x.contract_external_number.map(Into::into),
            number_eis: x.number_eis.map(Into::into),
            supplier_id: x.supplier_id.map(Into::into),
            contract_subject: x.contract_subject.map(Into::into),
            contract_type_id: x.contract_type_id.map(Into::into),
            accepted_volume_included_vat_rub: x
                .accepted_volume_included_vat_rub
                .map(Into::into),
            is_banking_support: x.is_banking_support.map(Into::into),
            is_with_amendments: x.is_with_amendments.map(Into::into),
            //---------------------------
            is_secret_state: x.is_secret_state.map(Into::into),
            is_secret_commercial: x.is_secret_commercial.map(Into::into),
            is_material_registry: x.is_material_registry.map(Into::into),
            is_to_publish: x.is_to_publish.map(Into::into),
            rationale: x.rationale.map(Into::into),
            funding_availability: x.funding_availability.map(Into::into),
            is_chairman_order: x.is_chairman_order.map(Into::into),
            is_chairman_order_secret: x.is_chairman_order_secret.map(Into::into),
            chairman_order_number: x.chairman_order_number.map(Into::into),
            chairman_order_date: x.chairman_order_date.map(Into::into),
            is_vice_chairman_order: x.is_vice_chairman_order.map(Into::into),
            is_with_approval: x.is_with_approval.map(Into::into),
            is_need_for_departments: x.is_need_for_departments.map(Into::into),
            is_sum_increase_was_specified: x
                .is_sum_increase_was_specified
                .map(Into::into),
            is_sum_changed_via_key_rate: x
                .is_sum_changed_via_key_rate
                .map(Into::into),
            //------------------------------
            vat_id: x.vat_id,
            sum_excluded_vat: x.sum_excluded_vat.map(Into::into),
            sum_vat: x.sum_vat.map(Into::into),
            sum_included_vat: x.sum_included_vat.map(Into::into),
            currency_id: x.currency_id.map(|x| x as i16),
            currency_rate: x.currency_rate.map(Into::into),
            sum_excluded_vat_rub: x.sum_excluded_vat_rub.map(Into::into),
            sum_vat_rub: x.sum_vat_rub.map(Into::into),
            sum_included_vat_rub: x.sum_included_vat_rub.map(Into::into),
            initial_sum_excluded_vat: x.initial_sum_excluded_vat.map(Into::into),
            initial_sum_included_vat: x.initial_sum_included_vat.map(Into::into),
            initial_currency_id: x.initial_currency_id.map(|x| x as i16),
            initial_currency_rate: x.initial_currency_rate.map(Into::into),
            initial_sum_excluded_vat_rub: x
                .initial_sum_excluded_vat_rub
                .map(Into::into),
            initial_sum_included_vat_rub: x
                .initial_sum_included_vat_rub
                .map(Into::into),
            initial_vat_id: x.initial_vat_id,
            initial_sum_vat: x.initial_sum_vat.map(Into::into),
            //----------------------------
            initial_sum_vat_rub: x.initial_sum_vat_rub.map(Into::into),
            previous_sum_excluded_vat: x.previous_sum_excluded_vat.map(Into::into),
            previous_sum_vat: x.previous_sum_vat.map(Into::into),
            previous_sum_included_vat: x.previous_sum_included_vat.map(Into::into),
            previous_currency_id: x.previous_currency_id.map(|x| x as i16),
            previous_currency_rate: x.previous_currency_rate,
            previous_sum_excluded_vat_rub: x
                .previous_sum_excluded_vat_rub
                .map(Into::into),
            previous_sum_vat_rub: x.previous_sum_vat_rub.map(Into::into),
            previous_sum_included_vat_rub: x
                .previous_sum_included_vat_rub
                .map(Into::into),
            delta_sum_excluded_vat: x.delta_sum_excluded_vat.map(Into::into),
            delta_sum_included_vat: x.delta_sum_included_vat.map(Into::into),
            delta_sum_excluded_vat_rub: x
                .delta_sum_excluded_vat_rub
                .map(Into::into),
            delta_sum_included_vat_rub: x
                .delta_sum_included_vat_rub
                .map(Into::into),
            delta_sum_vat: x.delta_sum_vat.map(Into::into),
            delta_sum_vat_rub: x.delta_sum_vat_rub.map(Into::into),
            sign_date: x.sign_date.map(Into::into),
            start_date: x.start_date.map(Into::into),
            end_date: x.end_date.map(Into::into),
            whole_start_date: x.whole_start_date.map(Into::into),
            whole_end_date: x.whole_end_date.map(Into::into),
            //--------------------------------
            initial_start_date: x.initial_start_date.map(Into::into),
            initial_end_date: x.initial_end_date.map(Into::into),
            initial_whole_start_date: x.initial_whole_start_date.map(Into::into),
            initial_whole_end_date: x.initial_whole_end_date.map(Into::into),
            previous_start_date: x.previous_start_date.map(Into::into),
            previous_end_date: x.previous_end_date.map(Into::into),
            previous_whole_start_date: x.previous_whole_start_date.map(Into::into),
            previous_whole_end_date: x.previous_whole_end_date.map(Into::into),
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
            //-------------------------------
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
            priority_income_contract_partner_text: x
                .priority_income_contract_partner_text
                .map(Into::into),
            is_priority_far_eastern: x.is_priority_far_eastern.map(Into::into),
            is_priority_other: x.is_priority_other.map(Into::into),
            is_headquarters: x.is_priority_headquarters.map(Into::into),
            pricing_expert_id: x.pricing_expert_id.map(Into::into),
            pricing_method_id: x.pricing_method_id,
            pricing_resume: x.pricing_resume.map(Into::into),
            //-------------------------------
            status_scheme_id: x.status_scheme_id,
            status_id: x.status_id.map(Into::into),
            is_approved_by_d646: x.is_approved_by_d646.map(Into::into),
            is_pricing_by_d646: x.is_pricing_by_d646.map(Into::into),
            is_pricing_by_d647: x.is_pricing_by_d647.map(Into::into),
            is_pricing_by_complectation: x
                .is_pricing_by_complectation
                .map(Into::into),
            plan_id: parse(x.plan_id)?,
            purchase_id: x.purchase_id,
            commission_kind_id: x.commission_kind_id,
            commission_date: x.commission_date.map(|x| {
                if x == Default::default() {
                    None
                } else {
                    Some(x)
                }
            }),
            //----------------------------------
            quotation_id: x.quotation_id,
            contract_id: x.contract_id,
            claim_id: x.claim_id.map(Into::into),
            items_number: x.items_number.map(|x| x as i16),
            posting_date: x.posting_date.map(Into::into),
            created_at: x.created_at.map(Into::into),
            created_by: x.created_by.map(Into::into),
            changed_at: x.changed_at.map(Into::into),
            changed_by: x.changed_by.map(Into::into),
            contract_amendment_types: x.kinds,
            //----------------------------------
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

            pricing_delta_currency_id: x.pricing_delta_currency_id,
            pricing_delta_currency_rate: x.pricing_delta_currency_rate,
            pricing_delta_sum_excluded_vat: x.pricing_delta_sum_excluded_vat,
            pricing_delta_sum_excluded_vat_rub: x
                .pricing_delta_sum_excluded_vat_rub,
            pricing_delta_sum_included_vat: x.pricing_delta_sum_included_vat,
            pricing_delta_sum_included_vat_rub: x
                .pricing_delta_sum_included_vat_rub,
            pricing_delta_sum_vat: x.pricing_delta_sum_vat,
            pricing_delta_sum_vat_rub: x.pricing_delta_sum_vat_rub,
            pricing_delta_total_sum: x.pricing_delta_total_sum,
            pricing_delta_total_sum_rub: x.pricing_delta_total_sum_rub,
            pricing_delta_transportation_price: x
                .pricing_delta_transportation_price,
            pricing_delta_transportation_sum_included_vat: x
                .pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub: x
                .pricing_delta_transportation_sum_included_vat_rub,
            pricing_delta_transportation_sum_vat: x
                .pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub: x
                .pricing_delta_transportation_sum_vat_rub,
            pricing_started_at: x.pricing_started_at.map(Into::into),
            //-----------------------
            expert_conclusion_id: x.expert_conclusion_id,
            is_check_documentation: x.is_check_documentation,
            check_documentation_date: x
                .check_documentation_date
                .map(|t| t.map(Into::into)),
            //------------------------
            kod_st_buda: x.kod_st_buda,
            okdp2: x.okdp2,
            category_id: x.category_id,
            code_type: x.code_type,
            //---------------------------
            //---saving fields.
            savings_accounting_id: x.savings_accounting_id,
            savings_sum_excluded_vat: x.savings_sum_excluded_vat,
            savings_sum_excluded_vat_rub: x.savings_sum_excluded_vat_rub,
            savings_sum_included_vat: x.savings_sum_included_vat,
            savings_sum_included_vat_rub: x.savings_sum_included_vat_rub,
            //----------------------------
            product_type_id: x.product_type_id,
            //------------------------
            number_customer: x.number_customer,
            purchase_number_eis: x.purchase_number_eis,
            budget_item_id: x.budget_item_id,
            payment_balance_item_id: x.payment_balance_item_id,
            associated_plan_id: x.associated_plan_id,
            termination_date: x.termination_date,
            close_date: x.close_date,
            previous_vat_id: x.previous_vat_id,
            repair_stage_id: x.repair_stage_id,
            ..Default::default()
        })
    }
}

impl FieldTolerance for ContractAmendmentLegacy {
    const TOLERATED: &'static [(&'static str, &'static str)] = &[];
}
