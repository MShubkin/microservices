//! Заголовок ППЗ, но уже по логике АСЕЗ-2.0
use crate::legacy::plans::PlanStatus;
use crate::maths::*;
use crate::{
    Attachment, CommissionKind, ContractAmendment, EcAgenda, EcAgendaItem,
    EcProtocol, EcProtocolItem, PlanItem, PlanItemFull, PlanItemFullRep,
    PlanItemFullVersion, RelAgendaProtocolItem, StatusHistory,
};

use asez2_shared_db::db_item::*;
use asez2_shared_db::{impl_join_on, joined, DbAdaptor, DbItem};
use fieldname_access::FieldnameAccess;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use shared_db_derive::DbVersioned;
use sqlx::Type;
use std::fmt::Display;
use std::ops::RangeInclusive;
use uuid::Uuid;

pub const PLAN_ID_RANGE: RangeInclusive<i64> = 1000000000..=1999999999;

impl_join_on!(Plan:uuid => EcAgendaItem:source_uuid);
impl_join_on!(Plan:uuid => EcAgendaItem:source_uuid, left);
impl_join_on!(Plan:uuid => EcProtocolItem:source_uuid);
impl_join_on!(Plan:uuid => EcProtocolItem:source_uuid, left);
impl_join_on!(Plan:uuid => PlanItem:plan_uuid, aggr);
impl_join_on!(Plan:uuid => PlanItemFull:plan_uuid, aggr);
impl_join_on!(Plan:uuid => Attachment:object_uuid, aggr);
impl_join_on!(Plan:uuid => PlanVersion:uuid, aggr);
impl_join_on!(Plan:uuid => StatusHistory:object_uuid, left);

impl_join_on!(PlanVersion:uuid => PlanItemFullVersion:plan_uuid, aggr);
impl_join_on!(PlanVersion:uuid => Attachment:object_uuid, aggr);
// A Joined plan structure (it is not necessary yet)
// ```
// SELECT plan,aggr(plan_item) FROM plan
//    LEFT JOIN plan_item ON plan.uuid=plan_item.plan_uuid GROUP BY plan.*
// ```
// (Simplified and without filters.)
joined!(
    plan: Plan,
    items: PlanItem[Plan => PlanItem, aggr],
);
joined!(
    plan: Plan,
    items: PlanItemFull[Plan => PlanItemFull, aggr],
);
joined!(
    !GetPlanData,
    plan: Plan,
    items: PlanItemFull[Plan => PlanItemFull, aggr],
    attachments: Attachment[Plan => Attachment, aggr],
    versions: PlanVersion[Plan => PlanVersion, aggr],
);

joined!(
    !GetPlanVersionData,
    plan: PlanVersion,
    items: PlanItemFullVersion[PlanVersion => PlanItemFullVersion, aggr],
    attachments: Attachment[PlanVersion => Attachment, aggr],
);

joined!(
    !PlanWithAttachments,
    plan: Plan,
    attachments: Attachment[Plan => Attachment, aggr],
);

joined!(
    !PlanWithLastStatus,
    plan: Plan,
    status: StatusHistory[Plan => StatusHistory, left],
);

pub type FullPlan = JoinedPlanPlanItemFull;
pub type FullPlanSelect = JoinedPlanPlanItemFullSelector;

// This structure exists to be able to filter plans by agenda item and protocol items.
// Basically:
// ```
// SELECT plan,agenda_item,protocol_item
//      FROM plan
//      INNER JOIN agenda_item ON plan.uuid=agenda_item.plan_uuid
//      INNER JOIN protocol_item ON plan.uuid=protocol_item.plan_uuid
// ```
// (Simplification without filters.)
joined!(
    plan: Plan,
    agenda_item: EcAgendaItem[Plan => EcAgendaItem],
    protocol_item: EcProtocolItem[Plan => EcProtocolItem],
);

joined!(
    !PlanWithProtocolItems,
    plan: Plan,
    protocol_items: EcProtocolItem[Plan => EcProtocolItem, aggr],
);

joined!(
    plan: Plan,
    agenda_item: EcAgendaItem[Plan => EcAgendaItem, left],
);

joined!(
    plan: Plan,

    protocol_item: EcProtocolItem[Plan => EcProtocolItem, left],
    protocol: EcProtocol[EcProtocolItem => EcProtocol, left],

    agenda_item: EcAgendaItem[Plan => EcAgendaItem, left],
    agenda: EcAgenda[EcAgendaItem => EcAgenda, left],

    agenda_protocol_item_rel: RelAgendaProtocolItem[EcAgendaItem => RelAgendaProtocolItem, aggr]
);

pub mod section_selection {
    use super::*;

    joined!(
        plan: Plan,
        agenda_item: EcAgendaItem[Plan => EcAgendaItem, left],
        agenda: EcAgenda[EcAgendaItem => EcAgenda, left],
        protocol_item: EcProtocolItem[Plan => EcProtocolItem, left],
        protocol: EcProtocol[EcProtocolItem => EcProtocol, left],
        agenda_protocol_item_rel: RelAgendaProtocolItem[EcAgendaItem => RelAgendaProtocolItem, aggr]
    );

    joined!(
        amendment: ContractAmendment,
        agenda_item: EcAgendaItem[ContractAmendment => EcAgendaItem, left],
        agenda: EcAgenda[EcAgendaItem => EcAgenda, left],
        protocol_item: EcProtocolItem[ContractAmendment => EcProtocolItem, left],
        protocol: EcProtocol[EcProtocolItem => EcProtocol, left],
        agenda_protocol_item_rel: RelAgendaProtocolItem[EcAgendaItem => RelAgendaProtocolItem, aggr]
    );
}

/// A Plan with all of its items.
/// TODO: Make this a queryable db thing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompletePlanRep {
    pub plan: PlanRep,
    pub items: Vec<PlanItemFullRep>,
}

/// Заголовок ППЗ, но уже по логике АСЕЗ-2.0.
#[derive(
    Debug,
    Default,
    Clone,
    DbItem,
    DbItemExt,
    DbAdaptor,
    DbUpsert,
    DbVersioned,
    PartialEq,
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
#[fieldname_enum(derive = [Eq, PartialEq, Ord, PartialOrd])]
#[adaptor_attributes(
    #[fieldname_enum(derive = [Eq, PartialEq, Ord, PartialOrd])]
)]
#[adaptor_fields_with_values]
#[item_table = "plan"]
#[db_version_table = "plan_version"]
#[item_skip_field_tolerance]
#[item_aggr_insert]
pub struct Plan {
    #[item_field_pkey]
    pub uuid: Uuid,
    #[adaptor_field_duplicate = "plan_id"]
    pub id: i64,
    pub version_type: i16,
    pub version_number: i16,
    pub posting_date: AsezDate,
    pub year: i16,
    pub commission_kind_id: CommissionKind,
    pub commission_date: Option<AsezDate>,
    pub customer_id: i32,
    pub supplier_id: i32,
    pub executor_method_id: ExecutorMethodId,
    pub supplier_text: Option<String>,
    #[adaptor_field_duplicate = "contract_subject_short"]
    pub contract_subject: String,
    pub sum_excluded_vat: CurrencyValue,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_vat: CurrencyValue,
    pub currency_id: i16,
    pub currency_rate: CurrencyRate,
    pub for_price_analysis: bool,
    pub purchasing_type_id: i16,
    pub status_id: PlanStatus,
    pub delivery_start_date: AsezDate,
    pub delivery_end_date: AsezDate,
    pub section_id: i16,
    pub kod_st_buda: Option<String>,
    pub okdp2: Option<String>,
    pub category_id: Option<String>,
    pub code_type: Option<TypeOfPurchaseId>,
    // Fields below here are generally new.
    pub declarant_id: i32,
    pub agent_id: i32,
    pub initiator_user_id: i32,
    pub tender_user_id: i32,
    pub organizer_id: Option<i32>,
    pub minimal_requirements: String,
    pub customer_note: Option<String>,
    pub number_customer: String,
    pub number_cgg: Option<String>,
    pub vat_id: VatId,
    pub sum_included_vat: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
    pub purchasing_method_id: i16,
    pub purchasing_kind_id: i16,
    pub regulation_document_id: i32,
    pub publication_type_id: i16,
    pub master_system_id: Option<i16>,
    pub funding_source_id: i16,
    pub purchasing_trend_id: i16,
    pub is_smb: bool,
    pub is_smb_sub: bool,
    pub smb_exception_id: Option<i16>,
    pub smb_sub_percent: Option<i64>,
    pub smb_sub_sum: Option<CurrencyValue>,
    pub documentation_date: Option<AsezDate>,
    pub publication_date: Option<AsezDate>,
    pub summing_up_date: Option<AsezDate>,
    pub contract_sing_date: Option<AsezDate>,
    pub single_supplier_reason_id: i16,
    pub single_supplier_expert_id: Option<i32>,
    pub single_supplier_decision_id: Option<i32>,
    pub single_supplier_decision_resume: Option<String>,
    pub is_affiliated: bool,
    pub is_supplier_smb: bool,
    pub is_competitive_now: bool,
    pub management_order_number: Option<i32>,
    pub management_order_date: Option<AsezDate>,
    pub reason_document: Option<String>,
    pub is_approved_by_d646: bool,
    pub is_price_analysis_by_d646: bool,
    pub is_approver_by_d647: bool,
    pub is_onm: bool,
    pub is_agent_fee: bool,
    pub agent_contract_number: Option<String>,
    pub is_design_stage: bool,
    pub repair_stage_id: Option<i16>,
    pub is_gas_supply: bool,
    pub is_little_cost: bool,
    pub is_banking_support: bool,
    pub is_innovative: bool,
    pub is_to_publish: bool,
    pub rationale_for_not_publication: Option<String>,
    pub rationale_for_publication: Option<String>,
    pub is_cooperative: bool,
    pub is_not_purchase: bool,
    pub is_under_control: bool,
    pub rationale_is_under_control: Option<String>,
    pub technical_developer: Option<String>,
    pub is_priority_project: bool,
    pub priority_project_document: Option<String>,
    pub is_priority_introductory: bool,
    pub priority_introductory_date: Option<AsezDate>,
    pub priority_introductory_document: Option<String>,
    pub is_priority_repair: bool,
    pub priority_repair_document: Option<String>,
    pub is_priority_ozp: bool,
    pub priority_ozp_document: Option<String>,
    pub is_priority_income_contract: bool,
    pub priority_income_contract_document: Option<String>,
    pub priority_income_contract_partner_id: Option<i32>,
    pub is_priority_other: bool,
    pub is_headquarters: bool,
    pub is_first_time: bool,
    pub is_list_price: bool,
    pub is_removed: bool,
    pub general_contract_date: Option<AsezDate>,
    #[adaptor_field_duplicate = "contract_system_number"]
    pub general_contract_number: Option<String>,
    pub general_contract_stages: Option<String>,
    pub items_number: i16,
    pub publication_start_date: AsezDate,
    pub is_lease_supplier_selection: bool,
    pub is_performance_indicator: bool,
    pub competitive_note_for_expert: Option<String>,
    pub expert_conclusion_id: Option<ExpertConclusionId>,
    pub is_check_documentation: bool,
    pub check_documentation_date: Option<AsezTimestamp>,

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

    // TODO не знаю нужны ли тут, но в ДС есть еще поля:
    // pub is_pricing_by_d646: bool,
    // pub is_pricing_by_d647: bool,
    pub is_pricing_by_complectation: bool,

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
    pub pricing_started_at: AsezTimestamp,

    // Доп. Поля с 2024-11-10
    pub sum_vat_rub: CurrencyValue,
    pub budget_item_id: i16,
    pub payment_balance_item_id: i16,
    pub limit_on_construction: i64,
    pub limit_on_works: i64,
    pub priority: i16,
    pub priority_income_contract_partner_text: String,
    pub extract_number_d646: String,
    pub extract_date_d646: AsezDate,
    pub extract_sum_included_vat_rub_d646: CurrencyValue,
    pub extract_number_d647: String,
    pub extract_date_d647: AsezDate,
    pub extract_sum_included_vat_rub_d647: CurrencyValue,
    pub product_type_id: i16,
    pub organizer_note: String,
    pub description: String,
    pub status_scheme_id: i16,
    pub bid_opening_date: AsezDate,
    pub single_supplier_note_for_expert: String,
    pub control_pp_2013: i16,
    pub is_no_qualification: bool,
    pub is_commission: bool,
    pub is_priority_far_eastern: bool,
    pub is_nko: bool,
    pub is_priority_nonprofit: bool,
    //----
    pub contract_sign_date: AsezDate,
    pub active_uuid: Option<Uuid>,

    // Доп. поля plan_reason_cancel
    #[adaptor_attributes(#[serde(rename = "plan_reason_cancel_id")])]
    pub reason_cancel_id: Option<i32>,
    #[adaptor_attributes(#[serde(rename = "plan_replaced_id")])]
    pub replaced_id: Option<i64>,
    //----

    //  is actual
    pub is_actual: bool,

    // created & changed
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
    /// When the current version was created at.
    pub pricing_created_at: AsezTimestamp,
    /// When the version was last updated from an external source.
    pub pricing_changed_at: AsezTimestamp,
}

impl AsRef<Plan> for Plan {
    fn as_ref(&self) -> &Plan {
        self
    }
}

impl FieldTolerance for Plan {
    const TOLERATED: &'static [(&'static str, &'static str)] = &[
        ("plan_id", Plan::id),
        ("contract_subject_short", Plan::contract_subject),
        ("pricing_resume_short", Plan::pricing_resume),
        ("contract_system_number", Plan::general_contract_number),
    ];
}

impl FieldTolerance for PlanVersion {
    const TOLERATED: &'static [(&'static str, &'static str)] = Plan::TOLERATED;
}

/// Идентификатор решения
#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum ExpertConclusionId {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    /// Согласовано с заявленной стоимостью
    AgreedWithDeclaredPrice = 1,
    /// Согласовано со снижением стоимости
    AgreedWithDecreasingPrice = 2,
    /// Согласовано с повышением стоимости
    AgreedWithIncreasingPrice = 3,
    /// Возврат Заказчику
    RefundToCustomer = 4,
    /// Запрос документации
    DocumentationRequest = 5,
}

impl Display for ExpertConclusionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Undefined => "Не установлено",
            Self::AgreedWithDeclaredPrice => "Согласовано с заявленной стоимостью",
            Self::AgreedWithDecreasingPrice => "Согласовано со снижением стоимости",
            Self::AgreedWithIncreasingPrice => "Согласовано с повышением стоимости",
            Self::RefundToCustomer => "Возврат Заказчику",
            Self::DocumentationRequest => " Запрос документации",
        };
        write!(f, "{}", str)
    }
}

/// Тип назначения Эксперта АЦ
#[derive(
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Debug,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum ExecutorMethodId {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    /// Автоматическое назначение
    Automatic = 1,
    /// Назначение вручную
    Manual = 2,
}

impl Display for ExecutorMethodId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ExecutorMethodId::Undefined => "Не установлено",
            ExecutorMethodId::Automatic => {
                "Выполнено автоматическое назначение исполнителя"
            }
            ExecutorMethodId::Manual => "Выполнено ручное назначение исполнителя",
        };
        write!(f, "{}", str)
    }
}

/// Тип департамента
#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    Hash,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum PricingUnitId {
    /// Unknown (this is a hack to allow into)
    #[db_default]
    Undefined = 0,
    /// Д646
    D646 = 1,
    /// Д647
    D647 = 2,
    /// ГПК
    Gpk = 3,
    /// Д647
    D645 = 4,
}

impl Display for PricingUnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            Self::D645 => "Д645",
            Self::D646 => "Д646",
            Self::D647 => "Д647",
            Self::Gpk => "ГПК",
            Self::Undefined => "не указан",
        };
        write!(f, "{}", x)
    }
}

/// Тип закупки
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
pub enum TypeOfPurchaseId {
    #[db_default]
    Undefined = 0,
    /// Конкурентная
    Competitive = 1,
    /// Неконкурентная
    NotCompetitive = 2,
}

impl Display for TypeOfPurchaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TypeOfPurchaseId::Undefined => "Не установлено",
            TypeOfPurchaseId::Competitive => "Конкурентная закупка",
            TypeOfPurchaseId::NotCompetitive => "Неконкурентная закупка",
        };
        write!(f, "{}", str)
    }
}

/// Учитывать экономию
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
pub enum SavingsAccountingId {
    #[db_default]
    Undefined = 0,
    No = 1,
    Full = 2,
    Partial = 3,
}

impl Display for SavingsAccountingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SavingsAccountingId::Undefined => "Не установлено",
            SavingsAccountingId::No => "Нет",
            SavingsAccountingId::Full => "Да (Полностью)",
            SavingsAccountingId::Partial => "Да (Частично)",
        };
        write!(f, "{}", str)
    }
}

impl Display for PlanRepField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanRepField::OptionI16(Some(int))
            | PlanRepField::OptionOptionI16(Some(Some(int))) => {
                write!(f, "{}", int)
            }
            PlanRepField::OptionI32(Some(int))
            | PlanRepField::OptionOptionI32(Some(Some(int))) => {
                write!(f, "{}", int)
            }
            PlanRepField::OptionI64(Some(int))
            | PlanRepField::OptionOptionI64(Some(Some(int))) => {
                write!(f, "{}", int)
            }
            PlanRepField::OptionExecutorMethodId(Some(method)) => {
                write!(f, "{}", method)
            }
            PlanRepField::OptionPricingUnitId(Some(unit)) => {
                write!(f, "{}", unit)
            }
            PlanRepField::OptionUuid(Some(uuid)) => {
                write!(f, "{}", uuid)
            }
            PlanRepField::OptionBool(Some(boolean)) => {
                write!(f, "{}", boolean)
            }
            PlanRepField::OptionString(Some(string))
            | PlanRepField::OptionOptionString(Some(Some(string))) => {
                write!(f, "{}", string)
            }
            PlanRepField::OptionAsezDate(Some(date))
            | PlanRepField::OptionOptionAsezDate(Some(Some(date))) => {
                write!(f, "{}", date)
            }
            PlanRepField::OptionAsezTimestamp(Some(timestamp))
            | PlanRepField::OptionOptionAsezTimestamp(Some(Some(timestamp))) => {
                write!(f, "{}", timestamp)
            }
            PlanRepField::OptionPlanStatus(Some(status)) => {
                write!(f, "{}", status)
            }
            PlanRepField::OptionOptionExpertConclusionId(Some(Some(
                conclusion,
            ))) => {
                write!(f, "{}", conclusion)
            }
            PlanRepField::OptionOptionTypeOfPurchaseId(Some(Some(status))) => {
                write!(f, "{}", status)
            }
            PlanRepField::OptionCommissionKind(Some(kind)) => {
                write!(f, "{}", kind)
            }
            PlanRepField::OptionSavingsAccountingId(Some(savings)) => {
                write!(f, "{}", savings)
            }
            PlanRepField::OptionOptionUuid(Some(Some(u))) => {
                write!(f, "{}", u)
            }
            PlanRepField::OptionVatId(Some(u)) => {
                write!(f, "{}", u)
            }
            PlanRepField::OptionCurrencyValue(Some(u)) => {
                write!(f, "{}", u)
            }
            PlanRepField::OptionOptionCurrencyValue(Some(Some(u))) => {
                write!(f, "{}", u)
            }
            PlanRepField::OptionCurrencyRate(Some(u)) => {
                write!(f, "{}", u)
            }
            PlanRepField::OptionOptionCurrencyRate(Some(Some(u))) => {
                write!(f, "{}", u)
            }
            // Не надо убирать этот большой матч, потому что он является
            // своего рода проверкой того, что все правильно матчится
            PlanRepField::OptionI16(None)
            | PlanRepField::OptionOptionI16(Some(None))
            | PlanRepField::OptionOptionI16(None)
            | PlanRepField::OptionI32(None)
            | PlanRepField::OptionOptionI32(Some(None))
            | PlanRepField::OptionOptionI32(None)
            | PlanRepField::OptionI64(None)
            | PlanRepField::OptionOptionI64(Some(None))
            | PlanRepField::OptionOptionI64(None)
            | PlanRepField::OptionExecutorMethodId(None)
            | PlanRepField::OptionPricingUnitId(None)
            | PlanRepField::OptionUuid(None)
            | PlanRepField::OptionBool(None)
            | PlanRepField::OptionString(None)
            | PlanRepField::OptionOptionString(Some(None))
            | PlanRepField::OptionOptionString(None)
            | PlanRepField::OptionAsezDate(None)
            | PlanRepField::OptionAsezTimestamp(None)
            | PlanRepField::OptionOptionAsezTimestamp(Some(None))
            | PlanRepField::OptionOptionAsezTimestamp(None)
            | PlanRepField::OptionOptionAsezDate(Some(None))
            | PlanRepField::OptionOptionAsezDate(None)
            | PlanRepField::OptionPlanStatus(None)
            | PlanRepField::OptionOptionTypeOfPurchaseId(Some(None))
            | PlanRepField::OptionOptionExpertConclusionId(None)
            | PlanRepField::OptionOptionExpertConclusionId(Some(None))
            | PlanRepField::OptionOptionTypeOfPurchaseId(None)
            | PlanRepField::None
            | PlanRepField::OptionSavingsAccountingId(None)
            | PlanRepField::OptionCommissionKind(None)
            | PlanRepField::OptionOptionUuid(Some(None))
            | PlanRepField::OptionOptionUuid(None)
            | PlanRepField::OptionVatId(None)
            | PlanRepField::OptionCurrencyValue(None)
            | PlanRepField::OptionOptionCurrencyValue(None)
            | PlanRepField::OptionOptionCurrencyValue(Some(None))
            | PlanRepField::OptionCurrencyRate(None)
            | PlanRepField::OptionOptionCurrencyRate(None)
            | PlanRepField::OptionOptionCurrencyRate(Some(None)) => {
                write!(f, "null")
            }
        }
    }
}

impl PlanRepField<'_> {
    pub fn is_none(&self) -> bool {
        matches!(
            self,
            PlanRepField::OptionI16(None)
                | PlanRepField::OptionOptionI16(Some(None))
                | PlanRepField::OptionOptionI16(None)
                | PlanRepField::OptionI32(None)
                | PlanRepField::OptionOptionI32(Some(None))
                | PlanRepField::OptionOptionI32(None)
                | PlanRepField::OptionI64(None)
                | PlanRepField::OptionOptionI64(Some(None))
                | PlanRepField::OptionOptionI64(None)
                | PlanRepField::OptionExecutorMethodId(None)
                | PlanRepField::OptionPricingUnitId(None)
                | PlanRepField::OptionUuid(None)
                | PlanRepField::OptionBool(None)
                | PlanRepField::OptionString(None)
                | PlanRepField::OptionOptionString(Some(None))
                | PlanRepField::OptionOptionString(None)
                | PlanRepField::OptionAsezDate(None)
                | PlanRepField::OptionAsezTimestamp(None)
                | PlanRepField::OptionOptionAsezDate(Some(None))
                | PlanRepField::OptionOptionAsezDate(None)
                | PlanRepField::OptionPlanStatus(None)
                | PlanRepField::OptionOptionTypeOfPurchaseId(Some(None))
                | PlanRepField::OptionOptionExpertConclusionId(None)
                | PlanRepField::OptionOptionExpertConclusionId(Some(None))
                | PlanRepField::OptionOptionTypeOfPurchaseId(None)
                | PlanRepField::OptionCommissionKind(None)
                | PlanRepField::OptionOptionUuid(Some(None))
                | PlanRepField::OptionOptionUuid(None)
                | PlanRepField::OptionVatId(None)
                | PlanRepField::None
        )
    }
}
