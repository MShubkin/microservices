//! Here we store both convenience traits to simplify the trait bounds of some functions
//! And bona-fide traits.
use crate::common::Result;

use asez2_shared_db::db_item::{AsezTimestamp, DbFieldMask, DbVersioned};
use asez2_shared_db::{DbAdaptor, DbItem};
use shared_essential::application::records::{ProcessUpsert, UpdateCtx};
use shared_essential::domain::maths::*;
use shared_essential::domain::traits::*;
use shared_essential::domain::{
    CommissionKind, ContractAmendment, ContractAmendmentItem, Plan, PlanItemFull,
    PlanStatus, PricingUnitId,
};
use shared_essential::domain::{
    ContractAmendmentItemRep, ContractAmendmentRep, PlanItemFullRep, PlanRep,
};
use shared_essential::domain::{
    ContractAmendmentItemVersion, ContractAmendmentVersion, PlanItemFullVersion,
    PlanVersion,
};

// A collection trait for plan/amendments,
pub(super) trait HasPlanUpsertFields:
    HasIsActual + HasId + HasPlanStatusId + HasUuid
{
}

pub(super) trait UpdateHeaderRep<DbHeader, ItemRep, Item>:
    DbAdaptor<DbItem = DbHeader> + HasMaybeIdentifiers + HasMaybePricingStartedAt
where
    DbHeader: UpdateHeader<ItemRep, Item>,
    ItemRep: UpdateItemRep<Item>,
    Item: UpdateItem<DbHeader, ItemRep>,
{
    fn maybe_status_id(&self) -> Option<PlanStatus>;
    fn pricing_organization_unit_id(&self) -> Option<PricingUnitId>;
    fn commission_kind_id(&self) -> CommissionKind;
}

pub(super) trait UpdateHeader<ItemRep, Item>:
    ProcessUpsert + DbVersioned + HasPlanUpsertFields + HasPricing + HasPricingStartedAt
{
    fn commission_kind_id(&self) -> CommissionKind;
    fn commission_kind_id_mut(&mut self) -> &mut CommissionKind;
    fn pricing_organization_unit_id(&self) -> PricingUnitId;
}
pub(super) trait UpdateVersion: HasUuid + HasPricingVersion {}

pub(super) trait UpdateItemRep<Item>:
    DbAdaptor<DbItem = Item> + HasMaybeIdentifiers + HasMaybeSourceUuid
{
}
pub(super) trait UpdateItem<H, R>:
    ProcessUpsert + DbVersioned + HasSourceUuid + HasPricing + HasIsRemoved + HasUuid
where
    H: UpdateHeader<R, Self>,
    R: UpdateItemRep<Self>,
{
    fn calculate_deltas(self, _rate: CurrencyRate) -> Self {
        self
    }
}

impl UpdateHeaderRep<Plan, PlanItemFullRep, PlanItemFull> for PlanRep {
    fn maybe_status_id(&self) -> Option<PlanStatus> {
        self.status_id
    }
    fn pricing_organization_unit_id(&self) -> Option<PricingUnitId> {
        self.pricing_organization_unit_id
    }
    fn commission_kind_id(&self) -> CommissionKind {
        self.commission_kind_id.unwrap_or_default()
    }
}
impl
    UpdateHeaderRep<
        ContractAmendment,
        ContractAmendmentItemRep,
        ContractAmendmentItem,
    > for ContractAmendmentRep
{
    fn maybe_status_id(&self) -> Option<PlanStatus> {
        self.status_id
    }
    fn pricing_organization_unit_id(&self) -> Option<PricingUnitId> {
        self.pricing_organization_unit_id
    }
    fn commission_kind_id(&self) -> CommissionKind {
        self.commission_kind_id.unwrap_or_default()
    }
}

impl UpdateHeader<PlanItemFullRep, PlanItemFull> for Plan {
    fn commission_kind_id(&self) -> CommissionKind {
        self.commission_kind_id
    }
    fn commission_kind_id_mut(&mut self) -> &mut CommissionKind {
        &mut self.commission_kind_id
    }
    fn pricing_organization_unit_id(&self) -> PricingUnitId {
        self.pricing_organization_unit_id
    }
}
impl UpdateHeader<ContractAmendmentItemRep, ContractAmendmentItem>
    for ContractAmendment
{
    fn commission_kind_id(&self) -> CommissionKind {
        self.commission_kind_id
    }
    fn commission_kind_id_mut(&mut self) -> &mut CommissionKind {
        &mut self.commission_kind_id
    }
    fn pricing_organization_unit_id(&self) -> PricingUnitId {
        self.pricing_organization_unit_id
    }
}

impl UpdateItemRep<PlanItemFull> for PlanItemFullRep {}
impl UpdateItemRep<ContractAmendmentItem> for ContractAmendmentItemRep {}

impl UpdateVersion for PlanVersion {}
impl UpdateVersion for ContractAmendmentVersion {}

/// Для того чтобы можно было обобщать объекты по
#[allow(dead_code)]
pub(super) trait HasPricing {
    fn pricing_changed_at(&self) -> AsezTimestamp;
    fn pricing_created_at(&self) -> AsezTimestamp;
    fn set_pricing_changed_at(&mut self, t: AsezTimestamp);
    fn set_pricing_created_at(&mut self, t: AsezTimestamp);
}
/// Для того чтобы можно было обобщать объекты по
#[allow(dead_code)]
pub trait HasPricingStartedAt {
    fn pricing_started_at(&self) -> AsezTimestamp;
    fn set_pricing_started_at(&mut self, t: AsezTimestamp);
}
pub trait HasPricingVersion {
    fn pricing_version(&self) -> i16;
}
impl HasPlanUpsertFields for Plan {}
impl HasPlanUpsertFields for ContractAmendment {}

macro_rules! impl_has_pricing_started_at {
    ($entity:ty) => {
        impl HasPricingStartedAt for $entity {
            fn pricing_started_at(&self) -> AsezTimestamp {
                self.pricing_started_at
            }
            fn set_pricing_started_at(&mut self, t: AsezTimestamp) {
                self.pricing_started_at = t;
            }
        }
    };
}
macro_rules! impl_has_pricing {
    ($entity:ty) => {
        impl HasPricing for $entity {
            fn pricing_changed_at(&self) -> AsezTimestamp {
                self.pricing_changed_at
            }
            fn set_pricing_changed_at(&mut self, t: AsezTimestamp) {
                self.pricing_changed_at = t;
            }
            fn pricing_created_at(&self) -> AsezTimestamp {
                self.pricing_created_at
            }
            fn set_pricing_created_at(&mut self, t: AsezTimestamp) {
                self.pricing_created_at = t;
            }
        }
    };
}
macro_rules! impl_has_pricing_version {
    ($entity:ty, $field:ident) => {
        impl HasPricingVersion for $entity {
            fn pricing_version(&self) -> i16 {
                self.$field
            }
        }
    };
}

impl_has_pricing_version!(PlanVersion, pricing_version);
impl_has_pricing_version!(PlanItemFullVersion, pricing_version);
impl_has_pricing_version!(ContractAmendmentVersion, pricing_version);
impl_has_pricing_version!(ContractAmendmentItemVersion, pricing_version);

impl_has_pricing!(Plan);
impl_has_pricing!(PlanItemFull);
impl_has_pricing!(ContractAmendment);
impl_has_pricing!(ContractAmendmentItem);

impl_has_pricing_started_at!(Plan);
impl_has_pricing_started_at!(ContractAmendment);

impl UpdateItem<Plan, PlanItemFullRep> for PlanItemFull {}

impl UpdateItem<ContractAmendment, ContractAmendmentItemRep>
    for ContractAmendmentItem
{
    fn calculate_deltas(mut self, currency_rate: CurrencyRate) -> Self {
        let convert_currency = |x| currency_rate.convert_value(x);

        self.delta_quantity = Some(self.quantity - self.previous_quantity);
        self.delta_price = Some(self.price - self.previous_price);
        self.delta_sum_vat = Some(self.sum_vat - self.previous_sum_vat);
        self.delta_sum_excluded_vat =
            Some(self.sum_excluded_vat - self.previous_sum_excluded_vat);
        self.delta_sum_included_vat =
            Some(self.sum_included_vat - self.previous_sum_included_vat);

        self.delta_price_rub = self.delta_price.map(convert_currency);
        self.delta_sum_vat_rub = self.delta_sum_vat.map(convert_currency);
        self.delta_sum_excluded_vat_rub =
            self.delta_sum_excluded_vat.map(convert_currency);
        self.delta_sum_included_vat_rub =
            self.delta_sum_included_vat.map(convert_currency);

        self
    }
}

pub(super) trait Compare<Item, ItemRep, Rep>:
    UpdateHeader<ItemRep, Item>
where
    ItemRep: UpdateItemRep<Item>,
    Item: UpdateItem<Self, ItemRep>,
    Rep: UpdateHeaderRep<Self, ItemRep, Item>,
{
    fn new_insert(self, ctx: &UpdateCtx) -> Comparator<Item, ItemRep, Self, Rep>;
    fn new_update(
        h: Rep,
        _other: &Self,
        masks: &Masks<Self>,
        ctx: &UpdateCtx,
    ) -> Result<Comparator<Item, ItemRep, Self, Rep>>;
    fn complete(
        c: &Comparator<Item, ItemRep, Self, Rep>,
        new: ItemRep,
        old: Option<Item>,
        masks: &Masks<Item>,
    ) -> Result<Option<Item>>;
}

/// Apologies for the namings.
#[derive(Debug, Clone)]
pub(super) struct Masks<T: DbItem> {
    /// If new version is not created, do not update these fields.
    pub(super) drop_if_old_v: DbFieldMask<T>,
    /// If new version is not created, and PricingUnit is the same,
    /// add these fields to those "forgotten" by `drop_if_old_v`.
    pub(super) drop2_if_old_v_unit_same: DbFieldMask<T>,
    /// If new version is not created, and PricingUnit is changed,
    /// zero these fields (in addition to those "forgotten" by `drop_if_old_v`).
    pub(super) zero2_if_old_v_unit_changed: DbFieldMask<T>,
    /// Zero these fields if a new version is created.
    pub(super) zero_if_new_v: DbFieldMask<T>,
    /// If a new version is created and the pricing unit is changed,
    /// zero these fields (in addition to those zeroed by `zero_if_new_v`).
    pub(super) zero2_if_new_v_unit_changed: DbFieldMask<T>,
}

#[derive(Debug, Default)]
/// Comparator is created by creating original header in the DB and received header.\
/// The result of this comparison is recorded in the `h` field.
///
/// Each item (`plan_item`, `contract_amendment_item`) is compared with the header and
/// recorded parameters (`currency_rate`, `reset_pricing_fields`) to determine whether
/// or not the item should be recorded/updated and how.
pub(super) struct Comparator<I, R, T, Hr>
where
    T: UpdateHeader<R, I>,
    R: UpdateItemRep<I>,
    I: UpdateItem<T, R>,
    Hr: UpdateHeaderRep<T, R, I>,
{
    /// The header that will be inserted or updated.
    pub(super) h: T,
    /// is `pricing_unit_organization_id` equal between old and new header?
    /// - Is None we are inserting a new record, so we use all fields.
    /// - If true, we clear special fields.
    /// - If false, keep the values from our for these fields.
    pub(super) reset_pricing_fields: Option<bool>,
    /// This is purely a marker of whether we need to create a new version or not.
    /// It is used independently for to determine whether certain fields need to
    /// be updated or not.
    pub(super) new_version: bool,
    /// Trait markers to allow compilation
    pub(super) markers: core::marker::PhantomData<(I, R, Hr)>,
}

impl<Hr, R, I, T: UpdateHeader<R, I> + Compare<I, R, Hr>> Comparator<I, R, T, Hr>
where
    R: UpdateItemRep<I>,
    I: UpdateItem<T, R>,
    Hr: UpdateHeaderRep<T, R, I>,
{
    pub(super) fn new_insert(h: T, r: &UpdateCtx) -> Self {
        h.new_insert(r)
    }
    pub(super) fn new_update(
        h: Hr,
        o: &T,
        masks: &Masks<T>,
        r: &UpdateCtx,
    ) -> Result<Self> {
        T::new_update(h, o, masks, r)
    }

    // Adjust item in accordance with parameters in the comparator.
    pub(super) fn complete(
        &self,
        item: R,
        other: Option<I>,
        masks: &Masks<I>,
    ) -> Result<Option<I>> {
        T::complete(self, item, other, masks)
    }
}
