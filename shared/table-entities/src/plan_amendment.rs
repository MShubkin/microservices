use std::collections::HashMap;

use paste::paste;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{
        AsezDate, AsezTimestamp, DbAdaptorFieldMask, FieldTolerance, Select,
    },
    result::SharedDbError,
    DbAdaptor, DbItem,
};

use crate::maths::*;
use crate::{
    CommissionKind, ContractAmendmentField, ContractAmendmentFieldMut,
    ContractAmendmentItemRep, ExpertConclusionId, PlanField, PlanFieldMut,
    PlanItemFullRep, SavingsAccountingId, TypeOfPurchaseId,
};

use super::legacy::plans::PlanStatus;
use super::{
    ContractAmendment, ContractAmendmentRep, ContractAmendmentRepField,
    ContractAmendmentRepFieldMut, ExecutorMethodId, Plan, PlanRep, PlanRepField,
    PlanRepFieldMut, PricingUnitId,
};

#[macro_export]
macro_rules! get {
    ($($fieldname: tt => $ty: ty);* $(;)?) => {
        $(pub fn $fieldname (&self) -> &$ty {
            match self {
                Self::Plan(plan) => &plan.$fieldname,
                Self::Amendment(amendment) => &amendment.$fieldname,
            }
        })*
    };
    (opt; $($fieldname: tt => $ty: ty);* $(;)?) => {
        $(pub fn $fieldname (&self) -> &Option<$ty> {
            match self {
                Self::Plan(plan) => &plan.$fieldname,
                Self::Amendment(amendment) => &amendment.$fieldname,
            }
        })*
    };
    (only_plan_opt; $($fieldname: tt => $ty: ty);* $(;)?) => {
        $(pub fn $fieldname (&self) -> Option<&$ty> {
            match self {
                Self::Plan(plan) => plan.$fieldname.as_ref(),
                Self::Amendment(_) => None,
            }
        })*
    };
    (only_plan; $($fieldname: tt => $ty: ty);* $(;)?) => {
        $(pub fn $fieldname (&self) -> Option<&$ty> {
            match self {
                Self::Plan(plan) => Some(&plan.$fieldname),
                Self::Amendment(_) => None,
            }
        })*
    };
}

macro_rules! get_mut {
    ($($fieldname: tt => $ty: ty);*) => {
        paste! {
            $(pub fn [<$fieldname _mut>] (&mut self) -> &mut $ty {
                match self {
                    Self::Plan(plan) => &mut plan.$fieldname,
                    Self::Amendment(amendment) => &mut amendment.$fieldname,
                }
            })*
        }
    };
    (opt; $($fieldname: tt => $ty: ty);* ) => {
        paste! {
            $(pub fn [<$fieldname _mut>] (&mut self) -> &mut Option<$ty> {
                match self {
                    Self::Plan(plan) => &mut plan.$fieldname,
                    Self::Amendment(amendment) => &mut amendment.$fieldname,
                }
            })*
        }
    };
    (only_plan_opt; $($fieldname: tt => $ty: ty);*) => {
        paste! {
            $(pub fn [<only_plan_opt_ $fieldname _mut>] (&mut self) -> Option<&mut $ty> {
                match self {
                    Self::Plan(plan) => plan.$fieldname.as_mut(),
                    Self::Amendment(_) => None,
                }
            })*
        }
    };
    (only_plan; $($fieldname: tt => $ty: ty);*) => {
        paste! {
            $(pub fn [<$fieldname _mut>] (&mut self) -> Option<&mut $ty> {
                match self {
                    Self::Plan(plan) => Some(&mut plan.$fieldname),
                    Self::Amendment(_) => None,
                }
            })*
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum PlanOrAmendment {
    Plan(Plan),
    Amendment(ContractAmendment),
}

impl PlanOrAmendment {
    get!(
        id => i64;
        uuid => Uuid;
        status_id => PlanStatus;
        contract_subject => String;
        pricing_sum_included_vat_rub => Option<CurrencyValue>;
        pricing_sum_excluded_vat => CurrencyValue;
        pricing_sum_excluded_vat_rub => Option<CurrencyValue>;
        savings_sum_excluded_vat => Option<CurrencyValue>;
        savings_sum_excluded_vat_rub => Option<CurrencyValue>;
        savings_sum_included_vat => Option<CurrencyValue>;
        savings_sum_included_vat_rub => Option<CurrencyValue>;
        sum_included_vat_rub => CurrencyValue;
        pricing_expert_id => Option<i32>;
        expert_conclusion_id => Option<ExpertConclusionId>;
        pricing_organization_unit_id => PricingUnitId;
        commission_kind_id => CommissionKind;
        commission_date => Option<AsezDate>;
        customer_id => i32;
        number_customer => String;
        supplier_id => i32;
        initiator_user_id => i32;
        purchasing_type_id => i16;
        pricing_resume => Option<String>;
        pricing_method_id => i16;
        pricing_competitive_note_for_expert => Option<String>;
        savings_accounting_id => SavingsAccountingId;
        created_at => AsezTimestamp;
        pricing_started_at => AsezTimestamp;
        section_id => i16;
        is_check_documentation => bool
    );

    get!(
        only_plan_opt;
        reason_cancel_id => i32;
        replaced_id => i64
    );

    get!(
        only_plan;
        is_list_price => bool
    );

    get_mut!(
        status_id => PlanStatus;
        expert_conclusion_id => Option<ExpertConclusionId>;
        changed_by => i32;
        changed_at => AsezTimestamp;
        commission_kind_id => CommissionKind;
        commission_date => Option<AsezDate>;
        pricing_expert_id => Option<i32>;
        is_check_documentation => bool;
        check_documentation_date => Option<AsezTimestamp>;
        purchasing_type_id => i16;
        pricing_competitive_note_for_expert => Option<String>;
        sum_excluded_vat => CurrencyValue;
        pricing_sum_excluded_vat => CurrencyValue
    );

    get_mut!(
        only_plan_opt;
        reason_cancel_id => i32;
        replaced_id => i64
    );

    get_mut!(
        only_plan;
        is_list_price => bool
    );

    /// Выборка по [`Plan`] и [`ContractAmendment`] с проверкой полей
    ///
    /// TODO: сделать db_pool: Executor, заменить два обращения к БД на один (?)
    pub async fn select(
        select: &Select,
        db_pool: &PgPool,
    ) -> Result<Vec<PlanOrAmendment>, SharedDbError> {
        let plan_select = select.filtered_copy_for::<PlanRep>();
        let plans = Plan::select(&plan_select, db_pool).await?;

        let am_select = select.filtered_copy_for::<ContractAmendmentRep>();
        let amendments = ContractAmendment::select(&am_select, db_pool).await?;

        Ok(plans
            .into_iter()
            .map(PlanOrAmendment::Plan)
            .chain(amendments.into_iter().map(PlanOrAmendment::Amendment))
            .collect())
    }

    pub async fn select_option(
        select: &Select,
        db_pool: &PgPool,
    ) -> Result<Option<PlanOrAmendment>, SharedDbError> {
        let mut items = Self::select(select, db_pool).await?;
        let item = items.pop();
        if !items.is_empty() {
            Err(SharedDbError::Other("select_option: too many items".into()))
        } else {
            Ok(item)
        }
    }

    pub async fn select_single(
        select: &Select,
        db_pool: &PgPool,
    ) -> Result<PlanOrAmendment, SharedDbError> {
        let mut items = Self::select(select, db_pool).await?;
        if let Some(item) = items.pop() {
            if !items.is_empty() {
                Err(SharedDbError::Other("select_single: too many items".into()))
            } else {
                Ok(item)
            }
        } else {
            Err(SharedDbError::Other("select_single: nothing is selected".into()))
        }
    }

    /// Выборка по [`Plan`] и [`ContractAmendment`] по соответствующим
    /// переданным селектам без проверки на корректность полей
    pub async fn select_dual(
        plan_select: &Select,
        amendment_select: &Select,
        db_pool: &PgPool,
    ) -> Result<Vec<PlanOrAmendment>, SharedDbError> {
        let plans = Plan::select(plan_select, db_pool).await?;
        let amendments =
            ContractAmendment::select(amendment_select, db_pool).await?;

        Ok(plans
            .into_iter()
            .map(PlanOrAmendment::Plan)
            .chain(amendments.into_iter().map(PlanOrAmendment::Amendment))
            .collect())
    }

    pub fn into_iter(
        plans: Vec<Plan>,
        amendments: Vec<ContractAmendment>,
    ) -> impl Iterator<Item = PlanOrAmendment> {
        plans
            .into_iter()
            .map(PlanOrAmendment::from)
            .chain(amendments.into_iter().map(PlanOrAmendment::from))
    }

    pub fn collect<F>(plans: Vec<Plan>, amendments: Vec<ContractAmendment>) -> F
    where
        F: FromIterator<PlanOrAmendment>,
    {
        Self::into_iter(plans, amendments).collect()
    }

    pub fn collect_map_by_uuid(
        plans: Vec<Plan>,
        amendments: Vec<ContractAmendment>,
    ) -> HashMap<Uuid, PlanOrAmendment> {
        Self::into_iter(plans, amendments).map(|p| (*p.uuid(), p)).collect()
    }

    pub fn split_vec<I>(plans: I) -> (Vec<Plan>, Vec<ContractAmendment>)
    where
        I: IntoIterator<Item = PlanOrAmendment>,
    {
        let plans_iter = plans.into_iter();
        // unwrap_or не сработает, так как plans имеет конечное количество элементов
        let size_hint = plans_iter.size_hint().1.unwrap_or(0);
        let (mut plans, mut amendments) =
            (Vec::with_capacity(size_hint), Vec::with_capacity(size_hint));

        for plan in plans_iter {
            match plan {
                PlanOrAmendment::Plan(p) => plans.push(p),
                PlanOrAmendment::Amendment(a) => amendments.push(a),
            }
        }

        (plans, amendments)
    }

    pub fn from_either(
        p: Option<Plan>,
        a: Option<ContractAmendment>,
    ) -> Option<Self> {
        p.map(Self::Plan).or_else(|| a.map(Self::Amendment))
    }

    pub fn system_name(&self) -> &'static str {
        match &self {
            PlanOrAmendment::Plan(_) => "ППЗ",
            PlanOrAmendment::Amendment(_) => "ДС",
        }
    }

    pub fn is_plan(&self) -> bool {
        matches!(self, Self::Plan(_))
    }

    pub fn is_amendment(&self) -> bool {
        matches!(self, Self::Amendment(_))
    }
}

impl From<Plan> for PlanOrAmendment {
    fn from(plan: Plan) -> Self {
        Self::Plan(plan)
    }
}

impl From<ContractAmendment> for PlanOrAmendment {
    fn from(amendment: ContractAmendment) -> Self {
        Self::Amendment(amendment)
    }
}

/// Существует чисто для того чтобы ФЕ было удобно. Как им будет удобнее получить
/// Несортированный список двух разных сущностей остаётся загадкой.
/// По умолчанию план. (Нужно для ApiResponse)
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(tag = "object_type")]
#[allow(clippy::large_enum_variant)]
pub enum PlanOrAmendmentRep {
    #[serde(rename = "plan")]
    Plan(PlanRep),
    #[serde(rename = "contract_amendment")]
    Amendment(ContractAmendmentRep),
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum PlanOrAmendmentItemsRep {
    PlanItems(Vec<PlanItemFullRep>),
    ContractAmendmentItems(Vec<ContractAmendmentItemRep>),
}
impl Default for PlanOrAmendmentItemsRep {
    fn default() -> Self {
        PlanOrAmendmentItemsRep::PlanItems(vec![])
    }
}

impl From<Vec<PlanItemFullRep>> for PlanOrAmendmentItemsRep {
    fn from(value: Vec<PlanItemFullRep>) -> Self {
        PlanOrAmendmentItemsRep::PlanItems(value)
    }
}

impl From<Vec<ContractAmendmentItemRep>> for PlanOrAmendmentItemsRep {
    fn from(value: Vec<ContractAmendmentItemRep>) -> Self {
        PlanOrAmendmentItemsRep::ContractAmendmentItems(value)
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(tag = "object_type", content = "data")]
#[allow(clippy::large_enum_variant)]
pub enum WidePlanOrAmendmentRep {
    #[serde(rename = "plan")]
    Plan(PlanRep),
    #[serde(rename = "contract_amendment")]
    Amendment(ContractAmendmentRep),
}

impl From<WidePlanOrAmendmentRep> for PlanOrAmendmentRep {
    fn from(poa: WidePlanOrAmendmentRep) -> Self {
        match poa {
            WidePlanOrAmendmentRep::Plan(x) => PlanOrAmendmentRep::Plan(x),
            WidePlanOrAmendmentRep::Amendment(x) => {
                PlanOrAmendmentRep::Amendment(x)
            }
        }
    }
}

impl PlanOrAmendmentRep {
    get!(
        opt;
        id => i64;
        plan_id => i64;
        uuid => Uuid;
        status_id => PlanStatus;
        customer_id => i32;
        sum_excluded_vat => CurrencyValue;
        sum_excluded_vat_rub => CurrencyValue;
        sum_included_vat => CurrencyValue;
        sum_included_vat_rub => CurrencyValue;
        savings_accounting_id => SavingsAccountingId;
        savings_sum_excluded_vat => Option<CurrencyValue>;
        savings_sum_excluded_vat_rub => Option<CurrencyValue>;
        savings_sum_included_vat => Option<CurrencyValue>;
        savings_sum_included_vat_rub => Option<CurrencyValue>;
        pricing_sum_excluded_vat => CurrencyValue;
        pricing_sum_excluded_vat_rub => Option<CurrencyValue>;
        pricing_organization_unit_id => PricingUnitId;
        commission_kind_id => CommissionKind;
        commission_date => Option<AsezDate>;
        contract_subject => String;
        pricing_expert_id => Option<i32>;
        pricing_resume => Option<String>;
        executor_method_id => ExecutorMethodId;
        okdp2 => Option<String>;
        kod_st_buda => Option<String>;
        category_id => Option<String>;
        code_type => Option<TypeOfPurchaseId>;
        is_check_documentation => bool;
        check_documentation_date => Option<AsezTimestamp>;
        purchasing_method_id => i16;
        purchasing_type_id => i16;
        currency_id => i16;
        number_customer => String;
        supplier_id => i32;
        section_id => i16;
        items_number => i16;
        created_at => AsezTimestamp;
        budget_item_id => i16;
        year => i16;
        single_supplier_reason_id => i16
    );

    get_mut!(
        opt;
        uuid => Uuid;
        changed_by => i32;
        changed_at => AsezTimestamp;
        commission_kind_id => CommissionKind;
        commission_date => Option<AsezDate>;
        pricing_organization_unit_id => PricingUnitId;
        status_id => PlanStatus;
        pricing_resume => Option<String>;
        executor_method_id => ExecutorMethodId;
        pricing_expert_id => Option<i32>;
        purchasing_type_id => i16;
        sum_excluded_vat => CurrencyValue;
        pricing_sum_excluded_vat => CurrencyValue
    );

    pub fn from_item_masked(
        value: PlanOrAmendment,
        plan_mask: &DbAdaptorFieldMask<PlanRep>,
        ca_mask: &DbAdaptorFieldMask<ContractAmendmentRep>,
    ) -> Self {
        match value {
            PlanOrAmendment::Plan(plan) => {
                PlanOrAmendmentRep::Plan(PlanRep::from_item_masked(plan, plan_mask))
            }
            PlanOrAmendment::Amendment(amendment) => PlanOrAmendmentRep::Amendment(
                ContractAmendmentRep::from_item_masked(amendment, ca_mask),
            ),
        }
    }

    pub fn from_item_with_fields<F>(
        fields: &[F],
    ) -> impl Fn(PlanOrAmendment) -> Self
    where
        F: AsRef<str>,
    {
        let plan_mask = DbAdaptorFieldMask::with_fields_and_pkeys(fields);
        let ca_mask = DbAdaptorFieldMask::with_fields_and_pkeys(fields);
        move |plan_ca| Self::from_item_masked(plan_ca, &plan_mask, &ca_mask)
    }

    pub fn from_item_with_fields_no_pkeys(
        fields: &[&str],
    ) -> impl Fn(PlanOrAmendment) -> Self {
        let plan_mask = DbAdaptorFieldMask::with_fields(fields);
        let ca_mask = DbAdaptorFieldMask::with_fields(fields);
        move |plan_ca| Self::from_item_masked(plan_ca, &plan_mask, &ca_mask)
    }

    pub fn from_item_with_fields_maybe(
        fields: Option<&[&str]>,
    ) -> impl Fn(PlanOrAmendment) -> Self {
        let (plan_mask, ca_mask) = if let Some(fields) = fields {
            (
                DbAdaptorFieldMask::with_fields_and_pkeys(fields),
                DbAdaptorFieldMask::with_fields_and_pkeys(fields),
            )
        } else {
            (DbAdaptorFieldMask::all(), DbAdaptorFieldMask::all())
        };
        move |plan_ca| Self::from_item_masked(plan_ca, &plan_mask, &ca_mask)
    }

    pub fn from_item_with_fields_split<T>(
        plan_fields: &[T],
        ca_fields: &[T],
    ) -> impl Fn(PlanOrAmendment) -> Self
    where
        T: AsRef<str>,
    {
        let plan_mask = DbAdaptorFieldMask::with_fields_and_pkeys(plan_fields);
        let ca_mask = DbAdaptorFieldMask::with_fields_and_pkeys(ca_fields);
        move |plan_ca| Self::from_item_masked(plan_ca, &plan_mask, &ca_mask)
    }

    pub fn from_item<T>(value: PlanOrAmendment, fields: Option<&[T]>) -> Self
    where
        T: AsRef<str>,
    {
        match value {
            PlanOrAmendment::Plan(x) => {
                PlanOrAmendmentRep::Plan(PlanRep::from_item(x, fields))
            }
            PlanOrAmendment::Amendment(x) => PlanOrAmendmentRep::Amendment(
                ContractAmendmentRep::from_item(x, fields),
            ),
        }
    }

    /// Полезно если нужен разный набор полей для ППЗ и ДС.
    pub fn from_item_split<T>(
        value: PlanOrAmendment,
        plan_fields: &[T],
        amendment_fields: &[T],
    ) -> Self
    where
        T: AsRef<str>,
    {
        match value {
            PlanOrAmendment::Plan(x) => {
                PlanOrAmendmentRep::Plan(PlanRep::from_item(x, Some(plan_fields)))
            }
            PlanOrAmendment::Amendment(x) => PlanOrAmendmentRep::Amendment(
                ContractAmendmentRep::from_item(x, Some(amendment_fields)),
            ),
        }
    }

    pub fn into_iter(
        plans: Vec<PlanRep>,
        amendments: Vec<ContractAmendmentRep>,
    ) -> impl Iterator<Item = PlanOrAmendmentRep> {
        plans
            .into_iter()
            .map(PlanOrAmendmentRep::from)
            .chain(amendments.into_iter().map(PlanOrAmendmentRep::from))
    }

    pub fn collect<F>(
        plans: Vec<PlanRep>,
        amendments: Vec<ContractAmendmentRep>,
    ) -> F
    where
        F: FromIterator<PlanOrAmendmentRep>,
    {
        Self::into_iter(plans, amendments).collect()
    }

    pub fn split_vec<I>(plans: I) -> (Vec<PlanRep>, Vec<ContractAmendmentRep>)
    where
        I: IntoIterator<Item = PlanOrAmendmentRep>,
    {
        let plans_iter = plans.into_iter();
        // unwrap_or не сработает, так как plans имеет конечное количество элементов
        let size_hint = plans_iter.size_hint().1.unwrap_or(0);
        let (mut plans, mut amendments) =
            (Vec::with_capacity(size_hint), Vec::with_capacity(size_hint));

        for plan in plans_iter {
            match plan {
                PlanOrAmendmentRep::Plan(p) => plans.push(p),
                PlanOrAmendmentRep::Amendment(a) => amendments.push(a),
            }
        }

        (plans, amendments)
    }

    pub fn is_plan(&self) -> bool {
        matches!(self, Self::Plan(_))
    }

    pub fn is_amendment(&self) -> bool {
        matches!(self, Self::Amendment(_))
    }

    #[track_caller]
    #[inline(always)]
    pub fn unwrap_plan(self) -> PlanRep {
        match self {
            PlanOrAmendmentRep::Plan(plan) => plan,
            PlanOrAmendmentRep::Amendment(_) => {
                panic!("Ожидалось найти ППЗ в PlanOrAmendmentRep")
            }
        }
    }

    #[track_caller]
    #[inline(always)]
    pub fn unwrap_amendment(self) -> ContractAmendmentRep {
        match self {
            PlanOrAmendmentRep::Plan(_) => {
                panic!("Ожидалось найти ДС в PlanOrAmendmentRep")
            }
            PlanOrAmendmentRep::Amendment(amendment) => amendment,
        }
    }
}

impl PlanOrAmendment {
    pub fn field<'a>(&'a self, field: &str) -> Option<PlanField<'a>> {
        match self {
            PlanOrAmendment::Plan(p) => p.field(field),
            PlanOrAmendment::Amendment(a) => a.field(field).map(Into::into),
        }
    }

    pub fn field_mut<'a>(&'a mut self, field: &str) -> Option<PlanFieldMut<'a>> {
        match self {
            PlanOrAmendment::Plan(p) => p.field_mut(field),
            PlanOrAmendment::Amendment(a) => a.field_mut(field).map(Into::into),
        }
    }

    pub fn actual_fieldname<'a>(&self, fieldname: &'a str) -> &'a str {
        match self {
            PlanOrAmendment::Plan(_) => Plan::actual_fieldname(fieldname),
            PlanOrAmendment::Amendment(_) => {
                ContractAmendment::actual_fieldname(fieldname)
            }
        }
    }
}

impl AsRef<PlanOrAmendment> for PlanOrAmendment {
    fn as_ref(&self) -> &PlanOrAmendment {
        self
    }
}

impl PlanOrAmendmentRep {
    pub fn field<'a>(&'a self, field: &str) -> Option<PlanRepField<'a>> {
        match self {
            PlanOrAmendmentRep::Plan(p) => p.field(field),
            PlanOrAmendmentRep::Amendment(a) => a.field(field).map(Into::into),
        }
    }

    pub fn field_mut<'a>(&'a mut self, field: &str) -> Option<PlanRepFieldMut<'a>> {
        match self {
            PlanOrAmendmentRep::Plan(p) => p.field_mut(field),
            PlanOrAmendmentRep::Amendment(a) => a.field_mut(field).map(Into::into),
        }
    }

    pub fn actual_fieldname<'a>(&self, fieldname: &'a str) -> &'a str {
        match self {
            Self::Plan(_) => Plan::actual_fieldname(fieldname),
            Self::Amendment(_) => ContractAmendment::actual_fieldname(fieldname),
        }
    }

    pub fn into_item_merged(
        self,
        item: PlanOrAmendment,
    ) -> Result<PlanOrAmendment, SharedDbError> {
        match (self, item) {
            (PlanOrAmendmentRep::Plan(rep), PlanOrAmendment::Plan(item)) => {
                Ok(PlanOrAmendment::Plan(rep.into_item_merged(item)?))
            }
            (
                PlanOrAmendmentRep::Amendment(rep),
                PlanOrAmendment::Amendment(item),
            ) => Ok(PlanOrAmendment::Amendment(rep.into_item_merged(item)?)),
            _ => Err(SharedDbError::Other(
                "PlanOrAmendmentRep::into_item_merged: adaptor and item mismatch"
                    .to_string(),
            )),
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            PlanOrAmendmentRep::Plan(_) => "ППЗ",
            PlanOrAmendmentRep::Amendment(_) => "ДС",
        }
    }
}

impl Default for PlanOrAmendmentRep {
    fn default() -> Self {
        PlanOrAmendmentRep::Plan(PlanRep::default())
    }
}

impl Default for WidePlanOrAmendmentRep {
    fn default() -> Self {
        WidePlanOrAmendmentRep::Plan(PlanRep::default())
    }
}

impl From<PlanRep> for PlanOrAmendmentRep {
    fn from(x: PlanRep) -> Self {
        Self::Plan(x)
    }
}

impl From<ContractAmendmentRep> for PlanOrAmendmentRep {
    fn from(x: ContractAmendmentRep) -> Self {
        Self::Amendment(x)
    }
}

impl<'a> From<ContractAmendmentRepField<'a>> for PlanRepField<'a> {
    fn from(val: ContractAmendmentRepField<'a>) -> Self {
        match val {
            ContractAmendmentRepField::OptionI16(val) => {
                PlanRepField::OptionI16(val)
            }
            ContractAmendmentRepField::OptionOptionI16(val) => {
                PlanRepField::OptionOptionI16(val)
            }
            ContractAmendmentRepField::OptionI32(val) => {
                PlanRepField::OptionI32(val)
            }
            ContractAmendmentRepField::OptionOptionI32(val) => {
                PlanRepField::OptionOptionI32(val)
            }
            ContractAmendmentRepField::OptionI64(val) => {
                PlanRepField::OptionI64(val)
            }
            ContractAmendmentRepField::OptionOptionI64(val) => {
                PlanRepField::OptionOptionI64(val)
            }
            ContractAmendmentRepField::OptionUuid(val) => {
                PlanRepField::OptionUuid(val)
            }
            ContractAmendmentRepField::OptionBool(val) => {
                PlanRepField::OptionBool(val)
            }
            ContractAmendmentRepField::OptionString(val) => {
                PlanRepField::OptionString(val)
            }
            ContractAmendmentRepField::OptionOptionString(val) => {
                PlanRepField::OptionOptionString(val)
            }
            ContractAmendmentRepField::OptionAsezTimestamp(val) => {
                PlanRepField::OptionAsezTimestamp(val)
            }
            ContractAmendmentRepField::OptionOptionAsezDate(val) => {
                PlanRepField::OptionOptionAsezDate(val)
            }
            ContractAmendmentRepField::OptionAsezDate(val) => {
                PlanRepField::OptionAsezDate(val)
            }
            ContractAmendmentRepField::OptionOptionAsezTimestamp(val) => {
                PlanRepField::OptionOptionAsezTimestamp(val)
            }
            ContractAmendmentRepField::OptionPlanStatus(val) => {
                PlanRepField::OptionPlanStatus(val)
            }
            ContractAmendmentRepField::OptionExecutorMethodId(val) => {
                PlanRepField::OptionExecutorMethodId(val)
            }
            ContractAmendmentRepField::OptionPricingUnitId(val) => {
                PlanRepField::OptionPricingUnitId(val)
            }
            ContractAmendmentRepField::OptionOptionTypeOfPurchaseId(val) => {
                PlanRepField::OptionOptionTypeOfPurchaseId(val)
            }
            ContractAmendmentRepField::OptionOptionExpertConclusionId(val) => {
                PlanRepField::OptionOptionExpertConclusionId(val)
            }
            ContractAmendmentRepField::OptionCommissionKind(val) => {
                PlanRepField::OptionCommissionKind(val)
            }
            ContractAmendmentRepField::OptionSavingsAccountingId(val) => {
                PlanRepField::OptionSavingsAccountingId(val)
            }
            ContractAmendmentRepField::OptionVatId(val) => {
                PlanRepField::OptionVatId(val)
            }
            ContractAmendmentRepField::OptionCurrencyValue(val) => {
                PlanRepField::OptionCurrencyValue(val)
            }
            ContractAmendmentRepField::OptionOptionCurrencyValue(val) => {
                PlanRepField::OptionOptionCurrencyValue(val)
            }
            ContractAmendmentRepField::OptionCurrencyRate(val) => {
                PlanRepField::OptionCurrencyRate(val)
            }
            ContractAmendmentRepField::OptionOptionCurrencyRate(val) => {
                PlanRepField::OptionOptionCurrencyRate(val)
            }
            ContractAmendmentRepField::OptionAsezArrayI32(_) => PlanRepField::None,
            ContractAmendmentRepField::None => PlanRepField::None,
        }
    }
}

impl<'a> From<ContractAmendmentRepFieldMut<'a>> for PlanRepFieldMut<'a> {
    fn from(val: ContractAmendmentRepFieldMut<'a>) -> Self {
        match val {
            ContractAmendmentRepFieldMut::OptionI16(val) => {
                PlanRepFieldMut::OptionI16(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionI16(val) => {
                PlanRepFieldMut::OptionOptionI16(val)
            }
            ContractAmendmentRepFieldMut::OptionI32(val) => {
                PlanRepFieldMut::OptionI32(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionI32(val) => {
                PlanRepFieldMut::OptionOptionI32(val)
            }
            ContractAmendmentRepFieldMut::OptionI64(val) => {
                PlanRepFieldMut::OptionI64(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionI64(val) => {
                PlanRepFieldMut::OptionOptionI64(val)
            }
            ContractAmendmentRepFieldMut::OptionUuid(val) => {
                PlanRepFieldMut::OptionUuid(val)
            }
            ContractAmendmentRepFieldMut::OptionBool(val) => {
                PlanRepFieldMut::OptionBool(val)
            }
            ContractAmendmentRepFieldMut::OptionString(val) => {
                PlanRepFieldMut::OptionString(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionString(val) => {
                PlanRepFieldMut::OptionOptionString(val)
            }
            ContractAmendmentRepFieldMut::OptionAsezTimestamp(val) => {
                PlanRepFieldMut::OptionAsezTimestamp(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionAsezTimestamp(val) => {
                PlanRepFieldMut::OptionOptionAsezTimestamp(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionAsezDate(val) => {
                PlanRepFieldMut::OptionOptionAsezDate(val)
            }
            ContractAmendmentRepFieldMut::OptionAsezDate(val) => {
                PlanRepFieldMut::OptionAsezDate(val)
            }
            ContractAmendmentRepFieldMut::OptionPlanStatus(val) => {
                PlanRepFieldMut::OptionPlanStatus(val)
            }
            ContractAmendmentRepFieldMut::OptionPricingUnitId(val) => {
                PlanRepFieldMut::OptionPricingUnitId(val)
            }
            ContractAmendmentRepFieldMut::OptionExecutorMethodId(val) => {
                PlanRepFieldMut::OptionExecutorMethodId(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionTypeOfPurchaseId(val) => {
                PlanRepFieldMut::OptionOptionTypeOfPurchaseId(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionExpertConclusionId(val) => {
                PlanRepFieldMut::OptionOptionExpertConclusionId(val)
            }
            ContractAmendmentRepFieldMut::OptionCommissionKind(val) => {
                PlanRepFieldMut::OptionCommissionKind(val)
            }
            ContractAmendmentRepFieldMut::OptionSavingsAccountingId(val) => {
                PlanRepFieldMut::OptionSavingsAccountingId(val)
            }
            ContractAmendmentRepFieldMut::OptionVatId(val) => {
                PlanRepFieldMut::OptionVatId(val)
            }
            ContractAmendmentRepFieldMut::OptionCurrencyValue(val) => {
                PlanRepFieldMut::OptionCurrencyValue(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionCurrencyValue(val) => {
                PlanRepFieldMut::OptionOptionCurrencyValue(val)
            }
            ContractAmendmentRepFieldMut::OptionCurrencyRate(val) => {
                PlanRepFieldMut::OptionCurrencyRate(val)
            }
            ContractAmendmentRepFieldMut::OptionOptionCurrencyRate(val) => {
                PlanRepFieldMut::OptionOptionCurrencyRate(val)
            }
            ContractAmendmentRepFieldMut::OptionAsezArrayI32(_) => {
                PlanRepFieldMut::None
            }
            ContractAmendmentRepFieldMut::None => PlanRepFieldMut::None,
        }
    }
}

impl<'a> From<ContractAmendmentField<'a>> for PlanField<'a> {
    fn from(val: ContractAmendmentField<'a>) -> Self {
        match val {
            ContractAmendmentField::Uuid(val) => PlanField::Uuid(val),
            ContractAmendmentField::I16(val) => PlanField::I16(val),
            ContractAmendmentField::OptionI16(val) => PlanField::OptionI16(val),
            ContractAmendmentField::I32(val) => PlanField::I32(val),
            ContractAmendmentField::OptionI32(val) => PlanField::OptionI32(val),
            ContractAmendmentField::I64(val) => PlanField::I64(val),
            ContractAmendmentField::OptionI64(val) => PlanField::OptionI64(val),
            ContractAmendmentField::String(val) => PlanField::String(val),
            ContractAmendmentField::Bool(val) => PlanField::Bool(val),
            ContractAmendmentField::OptionString(val) => {
                PlanField::OptionString(val)
            }
            ContractAmendmentField::AsezDate(val) => PlanField::AsezDate(val),
            ContractAmendmentField::AsezTimestamp(val) => {
                PlanField::AsezTimestamp(val)
            }
            ContractAmendmentField::PlanStatus(val) => PlanField::PlanStatus(val),
            ContractAmendmentField::ExecutorMethodId(val) => {
                PlanField::ExecutorMethodId(val)
            }
            ContractAmendmentField::PricingUnitId(val) => {
                PlanField::PricingUnitId(val)
            }
            ContractAmendmentField::OptionAsezDate(val) => {
                PlanField::OptionAsezDate(val)
            }
            ContractAmendmentField::OptionExpertConclusionId(val) => {
                PlanField::OptionExpertConclusionId(val)
            }
            ContractAmendmentField::OptionAsezTimestamp(val) => {
                PlanField::OptionAsezTimestamp(val)
            }
            ContractAmendmentField::OptionTypeOfPurchaseId(val) => {
                PlanField::OptionTypeOfPurchaseId(val)
            }
            ContractAmendmentField::CommissionKind(val) => {
                PlanField::CommissionKind(val)
            }
            ContractAmendmentField::SavingsAccountingId(val) => {
                PlanField::SavingsAccountingId(val)
            }
            ContractAmendmentField::VatId(val) => PlanField::VatId(val),
            ContractAmendmentField::CurrencyValue(val) => {
                PlanField::CurrencyValue(val)
            }
            ContractAmendmentField::OptionCurrencyValue(val) => {
                PlanField::OptionCurrencyValue(val)
            }
            ContractAmendmentField::CurrencyRate(val) => {
                PlanField::CurrencyRate(val)
            }
            ContractAmendmentField::OptionCurrencyRate(val) => {
                PlanField::OptionCurrencyRate(val)
            }
            ContractAmendmentField::AsezArrayI32(_) => PlanField::None,
            ContractAmendmentField::None => PlanField::None,
        }
    }
}

impl<'a> From<ContractAmendmentFieldMut<'a>> for PlanFieldMut<'a> {
    fn from(val: ContractAmendmentFieldMut<'a>) -> Self {
        match val {
            ContractAmendmentFieldMut::Uuid(val) => PlanFieldMut::Uuid(val),
            ContractAmendmentFieldMut::I16(val) => PlanFieldMut::I16(val),
            ContractAmendmentFieldMut::OptionI16(val) => {
                PlanFieldMut::OptionI16(val)
            }
            ContractAmendmentFieldMut::I32(val) => PlanFieldMut::I32(val),
            ContractAmendmentFieldMut::OptionI32(val) => {
                PlanFieldMut::OptionI32(val)
            }
            ContractAmendmentFieldMut::I64(val) => PlanFieldMut::I64(val),
            ContractAmendmentFieldMut::String(val) => PlanFieldMut::String(val),
            ContractAmendmentFieldMut::OptionI64(val) => {
                PlanFieldMut::OptionI64(val)
            }
            ContractAmendmentFieldMut::Bool(val) => PlanFieldMut::Bool(val),
            ContractAmendmentFieldMut::OptionString(val) => {
                PlanFieldMut::OptionString(val)
            }
            ContractAmendmentFieldMut::AsezDate(val) => PlanFieldMut::AsezDate(val),
            ContractAmendmentFieldMut::AsezTimestamp(val) => {
                PlanFieldMut::AsezTimestamp(val)
            }
            ContractAmendmentFieldMut::PlanStatus(val) => {
                PlanFieldMut::PlanStatus(val)
            }
            ContractAmendmentFieldMut::ExecutorMethodId(val) => {
                PlanFieldMut::ExecutorMethodId(val)
            }
            ContractAmendmentFieldMut::PricingUnitId(val) => {
                PlanFieldMut::PricingUnitId(val)
            }
            ContractAmendmentFieldMut::OptionAsezDate(val) => {
                PlanFieldMut::OptionAsezDate(val)
            }
            ContractAmendmentFieldMut::OptionExpertConclusionId(val) => {
                PlanFieldMut::OptionExpertConclusionId(val)
            }
            ContractAmendmentFieldMut::OptionAsezTimestamp(val) => {
                PlanFieldMut::OptionAsezTimestamp(val)
            }
            ContractAmendmentFieldMut::OptionTypeOfPurchaseId(val) => {
                PlanFieldMut::OptionTypeOfPurchaseId(val)
            }
            ContractAmendmentFieldMut::CommissionKind(val) => {
                PlanFieldMut::CommissionKind(val)
            }
            ContractAmendmentFieldMut::SavingsAccountingId(val) => {
                PlanFieldMut::SavingsAccountingId(val)
            }
            ContractAmendmentFieldMut::VatId(val) => PlanFieldMut::VatId(val),
            ContractAmendmentFieldMut::CurrencyValue(val) => {
                PlanFieldMut::CurrencyValue(val)
            }
            ContractAmendmentFieldMut::OptionCurrencyValue(val) => {
                PlanFieldMut::OptionCurrencyValue(val)
            }
            ContractAmendmentFieldMut::CurrencyRate(val) => {
                PlanFieldMut::CurrencyRate(val)
            }
            ContractAmendmentFieldMut::OptionCurrencyRate(val) => {
                PlanFieldMut::OptionCurrencyRate(val)
            }
            ContractAmendmentFieldMut::AsezArrayI32(_) => PlanFieldMut::None,
            ContractAmendmentFieldMut::None => PlanFieldMut::None,
        }
    }
}
