//! Позиции ППЗ, но уже по логике АСЕЗ-2.0
use crate::maths::*;

use asez2_shared_db::db_item::{
    AsezDate, AsezTimestamp, DbItemExt, DbUpsert, DbVersioned,
};
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Позиции ППЗ, но уже по логике АСЕЗ-2.0
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    DbItem,
    DbItemExt,
    DbAdaptor,
    DbUpsert,
    DbVersioned,
)]
#[adaptor_derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[item_table = "plan_item"]
#[db_version_table = "plan_item_version"]
#[item_aggr_insert]
pub struct PlanItem {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub plan_uuid: Uuid,
    pub id: i64,
    pub description_internal: Option<String>,
    pub currency_id: i16,
    pub currency_rate: CurrencyRate,
    pub category_id: i16,
    pub product_type_id: i16,
    pub budget_item_id: i16,
    pub okved2_id: i16,
    pub okato_id: Option<i32>,
    pub unit_id: i16,
    pub payment_balance_item_id: i16,
    pub is_not_russian_delivery: bool,
    pub note: Option<String>,
    pub quantity: Quantity,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
    /// When the current version was created at.
    pub pricing_created_at: AsezTimestamp,
    /// When the version was last updated from an external source.
    pub pricing_changed_at: AsezTimestamp,
}

impl From<PlanItemFullRep> for PlanItemRep {
    fn from(x: PlanItemFullRep) -> Self {
        Self {
            uuid: x.uuid,
            plan_uuid: x.plan_uuid,
            id: x.id,
            description_internal: x.description_internal,
            currency_id: x.currency_id,
            currency_rate: x.currency_rate,
            category_id: x.category_id,
            product_type_id: x.product_type_id,
            budget_item_id: x.budget_item_id,
            okved2_id: x.okved2_id,
            okato_id: x.okato_id,
            unit_id: x.unit_id,
            payment_balance_item_id: x.payment_balance_item_id,
            is_not_russian_delivery: x.is_not_russian_delivery,
            note: x.note,
            quantity: x.quantity,
            created_at: x.created_at,
            changed_at: x.changed_at,
            created_by: x.created_by,
            changed_by: x.changed_by,
            pricing_created_at: x.pricing_created_at,
            pricing_changed_at: x.pricing_changed_at,
        }
    }
}

/// Позиции ППЗ, но уже по логике АСЕЗ-2.0
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    DbItem,
    DbItemExt,
    DbAdaptor,
    DbUpsert,
    DbVersioned,
)]
#[adaptor_derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[item_table = "plan_item"]
#[db_version_table = "plan_item_version"]
#[item_aggr_insert]
#[adaptor_fields_with_values]
pub struct PlanItemFull {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub plan_uuid: Uuid,
    pub id: i64,
    pub number: i16,
    pub description_internal: Option<String>,
    pub currency_id: i16,
    pub currency_rate: CurrencyRate,
    pub category_id: i16,
    pub product_type_id: i16,
    pub budget_item_id: i16,
    pub okved2_id: i16,
    pub okato_id: Option<i32>,
    pub unit_id: i16,
    pub payment_balance_item_id: i16,
    pub is_not_russian_delivery: bool,
    pub note: Option<String>,
    pub quantity: Quantity,
    //END EXISTING FIELDS.
    pub okpd2_id: i64,
    pub delivery_basis: String,
    pub price: CurrencyValue,
    pub currency_rate_date: Option<AsezDate>,
    pub vat_id: VatId,
    pub transportation_price: Option<CurrencyValue>,
    pub transportation_vat_id: VatId,
    pub transportation_sum_included_vat: Option<CurrencyValue>,
    pub sum_vat: CurrencyValue,
    pub sum_vat_rub: CurrencyValue,
    pub sum_excluded_vat: CurrencyValue,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_included_vat: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
    pub delivery_start_date: AsezDate,
    pub delivery_end_date: AsezDate,
    pub price_source_1_text: Option<String>,
    pub price_source_1_price: Option<CurrencyValue>,
    pub price_source_1_date: Option<AsezDate>,
    pub price_source_1_sum_included_vat: Option<CurrencyValue>,
    pub price_source_2_text: Option<String>,
    pub price_source_2_price: Option<CurrencyValue>,
    pub price_source_2_date: Option<AsezDate>,
    pub price_source_2_sum_included_vat: Option<CurrencyValue>,
    pub price_source_3_text: Option<String>,
    pub price_source_3_price: Option<CurrencyValue>,
    pub price_source_3_date: Option<AsezDate>,
    pub price_source_3_sum_included_vat: Option<CurrencyValue>,
    pub is_analog_allowed: Option<bool>,
    pub analog_price: Option<CurrencyValue>,
    pub analog_text: Option<String>,
    pub analog_producer_id: Option<i32>,
    pub analog_country_id: Option<i16>,
    pub analog_requirements: Option<String>,
    pub mark: Option<String>,
    pub mark_main: Option<String>,
    pub technical_characteristics: Option<String>,
    pub technical_requirements: Option<String>,
    pub gosts: Option<String>,
    pub material_code_ius_local: Option<String>,
    pub material_code_ius_mtr: Option<String>,
    pub is_serial: Option<bool>,
    pub pzp_code: Option<String>,
    pub nomenclature_group_id: Option<i16>,
    pub source_country_id: Option<i16>,
    pub producer_country_id: Option<i16>,
    pub producer_id: Option<i32>,
    pub previous_price: Option<CurrencyValue>,
    pub previous_delivery_date: Option<AsezDate>,
    pub investment_project_id: Option<i32>,
    pub investment_project_code: Option<String>,
    pub is_dealer: Option<bool>,
    pub is_material_registry: Option<bool>,
    pub certificate_holder_id: Option<i32>,
    pub certificate_text: Option<String>,
    pub certificate_number: Option<String>,
    pub is_centralized_delivery: Option<bool>,
    pub centralized_sum: Option<CurrencyValue>,
    pub prepayment_percent: Option<i64>,
    pub payment_delay: Option<i16>,
    pub psd_price: Option<CurrencyValue>,
    pub psd_date: Option<AsezDate>,
    pub psd_code: Option<String>,
    pub onm_price: Option<CurrencyValue>,
    pub material_registry_price: Option<CurrencyValue>,
    pub expert_price: Option<CurrencyValue>,
    pub expert_sum_included_vat: Option<CurrencyValue>,
    pub is_removed: bool,
    pub created_at_date: AsezDate,
    pub changed_at_date: Option<AsezDate>,
    pub repair_inventory_number: Option<String>,
    pub repair_summary_code: Option<String>,
    pub repair_text: Option<String>,
    pub repair_plan_code: Option<String>,
    //-- new columns --
    pub plan_id_lotting: Option<String>,
    pub uuid_item_proposal: Option<Uuid>,

    // pricing
    pub pricing_quantity: Option<Quantity>,
    pub pricing_unit_id: Option<i16>,
    pub pricing_price: Option<CurrencyValue>,
    pub pricing_price_rub: Option<CurrencyValue>,

    pub pricing_vat_id: VatId,
    pub pricing_currency_id: Option<i16>,
    pub pricing_currency_rate: Option<CurrencyRate>,
    pub pricing_currency_rate_date: Option<AsezDate>,
    pub pricing_sum_excluded_vat: Option<CurrencyValue>,
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
