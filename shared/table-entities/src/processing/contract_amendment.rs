use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use asez2_shared_db::db_item::int_array::AsezArray;
use asez2_shared_db::db_item::*;
use asez2_shared_db::{impl_join_on, joined};
use fieldname_access::FieldnameAccess;
use shared_db_derive::DbItemExt;

use crate::legacy::plans::PlanStatus;
use crate::maths::*;
use crate::{
    Attachment, CommissionKind, ContractAmendmentItem,
    ContractAmendmentItemVersion, EcAgenda, EcAgendaItem, EcProtocol,
    EcProtocolItem, ExecutorMethodId, ExpertConclusionId, Plan, PricingUnitId,
    RelAgendaProtocolItem, SavingsAccountingId, TypeOfPurchaseId,
};

pub const AMENDMENT_ID_RANGE: RangeInclusive<i64> = 4000000000..=4999999999;

impl_join_on!(ContractAmendment:uuid => ContractAmendmentItem:header_uuid, aggr);
impl_join_on!(ContractAmendment:uuid => EcAgendaItem:source_uuid, left);
impl_join_on!(ContractAmendment:uuid => EcProtocolItem:source_uuid, left);
impl_join_on!(ContractAmendment:uuid => Attachment:object_uuid, aggr);
impl_join_on!(ContractAmendment:uuid => ContractAmendmentVersion:uuid, aggr);

impl_join_on!(ContractAmendmentVersion:uuid => ContractAmendmentItemVersion:header_uuid, aggr);
impl_join_on!(ContractAmendmentVersion:uuid => Attachment:object_uuid, aggr);
// A Joined plan structure (it is not necessary yet)
// ```
// SELECT contract_amendment,aggr(contract_amendment_item) FROM contract_amendment
//    LEFT JOIN contract_amendment_item
//        ON contract_amendment.uuid=contract_amendment_item.amendment_uuid
//        GROUP BY contract_amendment.*
// ```
// (Simplified and without filters.)
joined!(
    amendment: ContractAmendment,
    items: ContractAmendmentItem[ContractAmendment => ContractAmendmentItem, aggr],
);
pub type FullAmendment = JoinedContractAmendmentContractAmendmentItem;
pub type FullAmendmentSelect = JoinedContractAmendmentContractAmendmentItemSelector;

joined!(
    amendment: ContractAmendment,

    protocol_item: EcProtocolItem[ContractAmendment => EcProtocolItem, left],
    protocol: EcProtocol[EcProtocolItem => EcProtocol, left],

    agenda_item: EcAgendaItem[ContractAmendment => EcAgendaItem, left],
    agenda: EcAgenda[EcAgendaItem => EcAgenda, left],

    agenda_protocol_item_rel: RelAgendaProtocolItem[EcAgendaItem => RelAgendaProtocolItem, aggr]
);

joined!(
    !AmendmentWithProtocolItems,
    amendment: ContractAmendment,
    protocol_items: EcProtocolItem[ContractAmendment => EcProtocolItem, aggr],
);

joined!(
    amendment: ContractAmendment,
    agenda_item: EcAgendaItem[ContractAmendment => EcAgendaItem, left],
);

joined!(
    !GetContractAmendmentData,
    plan: ContractAmendment,
    items: ContractAmendmentItem[ContractAmendment => ContractAmendmentItem, aggr],
    attachments: Attachment[ContractAmendment => Attachment, aggr],
    versions: ContractAmendmentVersion[ContractAmendment => ContractAmendmentVersion, aggr],
);

joined!(
    !GetContractAmendmentVersionData,
    plan: ContractAmendmentVersion,
    items: ContractAmendmentItemVersion[ContractAmendmentVersion => ContractAmendmentItemVersion, aggr],
    attachments: Attachment[ContractAmendmentVersion => Attachment, aggr],
);

joined!(
    !ContractAmendmentWithAttachments,
    amendment: ContractAmendment,
    attachments: Attachment[ContractAmendment => Attachment, aggr],
);

#[derive(
    Debug,
    Default,
    Clone,
    DbItem,
    DbAdaptor,
    PartialEq,
    DbItemExt,
    DbUpsert,
    DbVersioned,
    FieldnameAccess,
)]
#[adaptor_derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Deserialize,
    Serialize,
    FieldnameAccess
)]
#[adaptor_fields_with_values]
#[item_table = "contract_amendment"]
#[db_version_table = "contract_amendment_version"]
#[item_skip_field_tolerance]
pub struct ContractAmendment {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    #[adaptor_field_duplicate = "plan_id"]
    pub id: i64,
    pub version_type: i16,
    pub version_number: i16,
    pub active_uuid: Uuid,
    pub source_uuid: Uuid,
    pub system_number: String,
    pub external_number: String,
    pub branch: String,
    pub is_actual: bool,
    pub is_pur_asbu: bool,
    pub customer_id: i32,
    pub declarant_id: i32,
    pub agent_id: i32,
    pub assignee_id: i32,
    pub project_institute_id: i32,
    pub organizer_id: i32,
    pub initiator_user_id: i32,
    pub tender_user_id: i32,
    pub year: i16,
    pub purchasing_type_id: i16,
    pub purchasing_method_id: i16,
    pub section_id: i16,
    pub funding_source_id: i16,
    pub single_supplier_reason_id: i16,
    pub number_cgg: String,
    #[adaptor_field_duplicate = "general_contract_number"]
    pub contract_system_number: String,
    pub contract_external_number: String,
    pub number_eis: String,
    pub supplier_id: i32,
    #[adaptor_field_duplicate = "contract_subject_short"]
    pub contract_subject: String,
    pub contract_type_id: i16,
    pub accepted_volume_included_vat_rub: i64,
    pub is_banking_support: bool,
    pub is_with_amendments: bool,
    pub is_secret_state: bool,
    pub is_secret_commercial: bool,
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
    pub is_sum_changed_via_key_rate: bool,
    pub is_material_registry: bool,
    pub is_to_publish: bool,
    pub repair_stage_id: i16,
    pub vat_id: VatId,
    #[adaptor_field_duplicate = "contract_amendment_sum_excluded_vat"]
    pub sum_excluded_vat: CurrencyValue,
    pub sum_vat: CurrencyValue,
    pub sum_included_vat: CurrencyValue,
    pub currency_id: i16,
    pub currency_rate: CurrencyRate,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_vat_rub: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
    pub initial_vat_id: VatId,
    pub initial_sum_excluded_vat: CurrencyValue,
    pub initial_sum_excluded_vat_rub: CurrencyValue,
    pub initial_sum_vat: CurrencyValue,
    pub initial_sum_vat_rub: CurrencyValue,
    pub initial_sum_included_vat: CurrencyValue,
    pub initial_sum_included_vat_rub: CurrencyValue,
    pub initial_currency_id: i16,
    pub initial_currency_rate: CurrencyRate,
    pub previous_vat_id: VatId,
    pub previous_sum_excluded_vat: CurrencyValue,
    pub previous_sum_vat: CurrencyValue,
    pub previous_sum_included_vat: CurrencyValue,
    pub previous_currency_id: i16,
    pub previous_currency_rate: CurrencyRate,
    pub previous_sum_excluded_vat_rub: CurrencyValue,
    pub previous_sum_vat_rub: CurrencyValue,
    pub previous_sum_included_vat_rub: CurrencyValue,
    pub delta_sum_excluded_vat: CurrencyValue,
    pub delta_sum_vat: CurrencyValue,
    pub delta_sum_included_vat: CurrencyValue,
    pub delta_sum_excluded_vat_rub: CurrencyValue,
    pub delta_sum_vat_rub: CurrencyValue,
    pub delta_sum_included_vat_rub: CurrencyValue,
    pub sign_date: AsezDate,
    pub close_date: AsezDate,
    pub termination_date: AsezDate,
    pub start_date: AsezDate,
    pub end_date: AsezDate,
    pub whole_start_date: AsezDate,
    pub whole_end_date: AsezDate,
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
    pub priority_ozp_document: String,
    pub is_priority_income_contract: bool,
    pub priority_income_contract_document: String,
    pub priority_income_contract_partner_id: i32,
    pub priority_income_contract_partner_text: String,
    pub is_priority_other: bool,
    pub is_headquarters: bool,
    pub status_scheme_id: i16,
    pub status_id: PlanStatus,
    pub is_approved_by_d646: bool,
    pub commission_kind_id: CommissionKind,
    pub budget_item_id: i16,
    pub payment_balance_item_id: i16,
    pub product_type_id: i16,
    pub items_number: i16,
    /// plan_id в оригинале у ГПИ
    pub associated_plan_id: i64,
    pub purchase_id: String,
    pub purchase_number_eis: String,
    pub quotation_id: String,
    pub contract_id: String,
    pub claim_id: i64,
    pub is_removed: bool,
    pub posting_date: AsezDate,
    pub is_priority_far_eastern: bool,

    pub executor_method_id: ExecutorMethodId,
    pub number_customer: String,
    pub commission_date: Option<AsezDate>,
    pub expert_conclusion_id: Option<ExpertConclusionId>,
    pub is_check_documentation: bool,
    pub check_documentation_date: Option<AsezTimestamp>,
    pub kod_st_buda: Option<String>,
    pub okdp2: Option<String>,
    pub category_id: Option<String>,
    pub code_type: Option<TypeOfPurchaseId>,
    pub contract_amendment_types: AsezArray<i32>,

    pub savings_accounting_id: SavingsAccountingId,
    pub savings_sum_excluded_vat: Option<CurrencyValue>,
    pub savings_sum_excluded_vat_rub: Option<CurrencyValue>,
    pub savings_sum_included_vat: Option<CurrencyValue>,
    pub savings_sum_included_vat_rub: Option<CurrencyValue>,

    // pricing
    pub pricing_method_id: i16,
    pub pricing_expert_id: Option<i32>,
    pub pricing_organization_unit_id: PricingUnitId,
    #[adaptor_field_duplicate = "pricing_resume_short"]
    pub pricing_resume: Option<String>,
    pub pricing_competitive_note_for_expert: Option<String>,

    pub is_pricing_by_d646: bool,
    pub is_pricing_by_d647: bool,
    pub is_pricing_by_complectation: bool,

    pub pricing_vat_id: VatId,

    pub pricing_currency_id: Option<i16>,
    pub pricing_currency_rate: Option<CurrencyRate>,

    #[adaptor_field_duplicate = "contract_amendment_pricing_sum_excluded_vat"]
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
    pub pricing_started_at: AsezTimestamp,

    // created & changed
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
    /// When the current version was created at.
    pub pricing_created_at: AsezTimestamp,
    /// When the version was last updated from an external source.
    pub pricing_changed_at: AsezTimestamp,
}

impl FieldTolerance for ContractAmendment {
    const TOLERATED: &'static [(&'static str, &'static str)] = &[
        ("plan_id", ContractAmendment::id),
        ("contract_subject_short", ContractAmendment::contract_subject),
        ("pricing_resume_short", ContractAmendment::pricing_resume),
        (
            "contract_amendment_sum_excluded_vat",
            ContractAmendment::sum_excluded_vat,
        ),
        (
            "contract_amendment_pricing_sum_excluded_vat",
            ContractAmendment::pricing_sum_excluded_vat,
        ),
        ("general_contract_number", ContractAmendment::contract_system_number),
    ];
}

impl FieldTolerance for ContractAmendmentVersion {
    const TOLERATED: &'static [(&'static str, &'static str)] =
        ContractAmendment::TOLERATED;
}

// This is a temporary structure. It is here to preserve incorrect business logic,
// which will be changed.
impl_join_on!(Plan:uuid => ContractAmendment:uuid);
joined!(
    plan: Plan,
    amendment: ContractAmendment[Plan => ContractAmendment],
);
