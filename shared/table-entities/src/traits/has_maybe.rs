use crate::{
    ContractAmendmentItemRep, ContractAmendmentRep, PlanItemFullRep, PlanRep,
};

use asez2_shared_db::db_item::AsezTimestamp;
use uuid::Uuid;

pub trait HasMaybeIdentifiers {
    fn maybe_uuid(&self) -> Option<Uuid>;
    fn maybe_id(&self) -> Option<i64>;
}

pub trait HasMaybeSourceUuid {
    fn maybe_source_uuid(&self) -> Option<Uuid>;
}

pub trait HasMaybePricingStartedAt {
    fn maybe_pricing_started_at(&self) -> Option<AsezTimestamp>;
}

macro_rules! impl_has_maybe_identifiers {
    ($entity:ty) => {
        impl HasMaybeIdentifiers for $entity {
            fn maybe_uuid(&self) -> Option<Uuid> {
                self.uuid
            }
            fn maybe_id(&self) -> Option<i64> {
                self.id
            }
        }
    };
}
macro_rules! impl_has_maybe_source_uuid {
    ($entity:ty, $field:ident) => {
        impl HasMaybeSourceUuid for $entity {
            fn maybe_source_uuid(&self) -> Option<Uuid> {
                self.$field
            }
        }
    };
}
macro_rules! impl_has_maybe_pricing_started_at {
    ($entity:ty, $field:ident) => {
        impl HasMaybePricingStartedAt for $entity {
            fn maybe_pricing_started_at(&self) -> Option<AsezTimestamp> {
                self.$field
            }
        }
    };
}

impl_has_maybe_identifiers!(PlanRep);
impl_has_maybe_identifiers!(PlanItemFullRep);
impl_has_maybe_identifiers!(ContractAmendmentRep);
impl_has_maybe_identifiers!(ContractAmendmentItemRep);

impl_has_maybe_source_uuid!(PlanItemFullRep, plan_uuid);
impl_has_maybe_source_uuid!(ContractAmendmentItemRep, header_uuid);

impl_has_maybe_pricing_started_at!(PlanRep, pricing_started_at);
impl_has_maybe_pricing_started_at!(ContractAmendmentRep, pricing_started_at);
