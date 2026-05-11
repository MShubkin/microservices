//! This module handles the implementation of `DbItem` and `DbAdaptor` for plans.
//! In this case, since we only ever need a small subset of the fields, the macros
//! are probably not optimal (we will construct a structure with a hundred fields)
//! but only ever send 10-30 of them. It may help to use a simplified structure
//! if performance is not satisfactory.
use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::{impl_join_on, joined, DbAdaptor, DbItem};
use monolith_service::dto::time::PlanningTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::maths::*;
use crate::{PlanItemFullRep, PlanLegacy};

impl_join_on!(PlanLegacy:uuid => SimplestPlanItem:plan_uuid, aggr);
impl_join_on!(PlanLegacy:uuid => PlanItemLegacy:plan_uuid, aggr);
joined!(
    simple_plan: PlanLegacy,
    items: SimplestPlanItem[PlanLegacy => SimplestPlanItem, aggr],
);
joined!(
    plan: PlanLegacy,
    items: PlanItemLegacy[PlanLegacy => PlanItemLegacy, aggr],
);

/// This is used purely for retrieving the pricing department Id.
/// TODO: Make a proper `PlanItemLegacy` with all the fields. (with all 119 fields)
#[derive(Debug, Default, Clone, PartialEq, DbItem)]
#[item_table = "plan_items_legacy"]
pub struct SimplestPlanItem {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub plan_uuid: Uuid,
    pub id: String,
    pub pricing_department_id: i32,
    pub pricing_unit_id: i16,
}

/// This is a full representation of a plan item.
/// This is used purely for retrieving the pricing department Id.
/// TODO: Make a proper `PlanItemLegacy` with all the fields. (with all 119 fields)
#[derive(Debug, Default, Clone, PartialEq, DbItem, DbAdaptor)]
#[adaptor_derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize
)]
#[item_table = "plan_items_legacy"]
pub struct PlanItemLegacy {
    /// Not inherited from legacy structure.
    pub plan_uuid: Uuid,
    #[item_field_pkey]
    pub uuid: Uuid,
    pub number: i16,
    pub number_customer: String,
    pub description_internal: String,
    pub description_external: String,
    pub product_type_id: i16,
    pub category_id: i16,
    pub budget_item_id: i16,
    pub payment_balance_item_id: i16,
    pub pricing_method_id: i16,
    pub okpd2_id: i32,
    pub okved2_id: i16,
    pub okato_id: i32,
    pub is_not_russian_delivery: bool,
    pub delivery_basis: String,
    pub unit_id: i32,
    pub quantity: Quantity,
    pub price: CurrencyValue,
    //----------------
    pub price_unit: i32,
    pub currency_id: i32,
    pub currency_rate: CurrencyRate,
    pub currency_rate_date: AsezDate,
    pub vat_id: VatId,
    pub sum_excluded_vat: CurrencyValue,
    pub sum_vat: CurrencyValue,
    pub sum_included_vat: CurrencyValue,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_vat_rub: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
    pub delivery_start_date: AsezDate,
    pub delivery_end_date: AsezDate,
    pub price_source_1_text: String,
    pub price_source_1_price: CurrencyValue,
    pub price_source_1_date: AsezDate,
    pub price_source_2_text: String,
    pub price_source_2_price: CurrencyValue,
    pub price_source_2_date: AsezDate,
    //----------------------
    pub price_source_3_text: String,
    pub price_source_3_price: CurrencyValue,
    pub price_source_3_date: AsezDate,
    pub is_analog_allowed: bool,
    pub analog_price: CurrencyValue,
    pub analog_text: String,
    pub analog_producer_id: i32,
    pub analog_country_id: i16,
    pub analog_requirements: String,
    pub mark: String,
    pub mark_main: String,
    pub technical_characteristics: String,
    pub technical_requirements: String,
    pub gosts: String,
    pub material_code_local: String,
    pub material_code_ius_mtr: String,
    pub is_serial: bool,
    //----------------------
    pub pzp_code: String,
    pub nomenclature_group_id: i16,
    pub source_country_id: i16,
    pub producer_country_id: i16,
    pub producer_id: i32,
    pub previous_price: CurrencyValue,
    pub previous_delivery_date: AsezDate,
    pub investment_project_id: i32,
    pub is_dealer: bool,
    pub is_material_registry: bool,
    pub certificate_holder_id: i32,
    pub certificate_text: String,
    pub certificate_number: String,
    pub repair_summary_code: String,
    pub repair_inventory_number: String,
    pub repair_text: String,
    pub repair_plan_code: String,
    pub is_corp_accept: String,
    //--------------------
    pub is_centralized_delivery: bool,
    pub centralized_sum: CurrencyValue,
    pub start_price: CurrencyValue,
    pub is_removed: bool,
    pub created_at: PlanningTimestamp,
    pub created_by: i32,
    pub changed_at: PlanningTimestamp,
    pub changed_by: i32,
    pub is_tariff_plan: bool,
    pub row_index: i64,
    pub mtr_group_code: String,
    pub consumer_id: i32,
    pub consumer_department_id: i32,
    pub delivery_plan_date: AsezDate,
    pub demand_income_date: AsezDate,
    pub expertise_fact_date: AsezDate,
    pub rus_pp_2013_mark: bool,
    pub okpd2_pp_2013_id: i64,
    pub rf_products_reason_id: i16,
    //-------------------------
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
    //------------------------
    // The below fields are not compatible and unused.
    // pub rpp_rf_pp_719_value: String,
    // pub rpp_eaes_pp_616_value: String,
    // pub errrp_pp_878_value: String,
    // The above fields are not compatible and used.
    //
    // Previously used fields are commented out below.
    // pub transportation_price: i64,
    // pub transportation_vat_id: i16,
    // pub transportation_sum_included_vat: i64,
    // pub analog_producer_text: String,
    // pub prepayment_percent: i64,
    // pub payment_delay: i16,
    // pub psd_price: i64,
    // pub psd_date: AsezDate,
    // pub psd_code: String,
    // pub onm_price: i64,
    // pub material_registry_price: i64,
    // pub expert_price: i64,
    // pub expert_sum_included_vat: i64,
    // pub pricing_quantity: i64,
    // pub pricing_price: i64,
    // pub pricing_vat_id: i16,
    // pub pricing_currency_id: i16,
    // pub pricing_currency_rate: i64,
    // pub pricing_transportation_price: i64,
    // pub pricing_transportation_vat_id: i16,
    // pub pricing_unit_id: i16,
    // pub pricing_department_id: i32,
    // pub pricing_expert_id: i32,
    // pub pricing_resume: String,
    // pub status_id: i16,
    // pub active_uuid: Uuid,
    // pub items_number: i16,
    // pub repair_approved_at: AsezTimestamp,
    // pub budget_uuid: Uuid,
    // pub is_lot: bool,
    // pub lot_uuid: Uuid,
    // pub rus_pp_2013_mark: Option<bool>,
    // pub rpp_rf_pp_719_id: Option<i64>,
    // pub rpp_eaes_pp_616_id: Option<i64>,
    // pub errrp_pp_878_id: Option<i64>,
    // pub rf_products_reason_id: Option<i16>,
    // pub not_exist_in_gisp_reestr: Option<bool>,
    // pub errrp_pp_878_number: String,
}

impl From<PlanItemFullRep> for PlanItemLegacyRep {
    fn from(x: PlanItemFullRep) -> Self {
        Self {
            uuid: x.uuid,
            // FIXME : number must differ from id
            number: x.id.map(|x| x as i16),
            plan_uuid: x.plan_uuid,
            description_internal: x
                .description_internal
                .map(|x| x.unwrap_or_default()),
            currency_id: x.currency_id.map(|x| x as i32),
            currency_rate: x.currency_rate,
            category_id: x.category_id,
            product_type_id: x.product_type_id,
            budget_item_id: x.budget_item_id,
            okved2_id: x.okved2_id,
            okato_id: x.okato_id.map(|x| x.unwrap_or(-1)),
            unit_id: x.unit_id.map(Into::into),
            payment_balance_item_id: x.payment_balance_item_id,
            is_not_russian_delivery: x.is_not_russian_delivery,
            quantity: x.quantity,
            created_at: x.created_at.map(Into::into),
            changed_at: x.changed_at.map(Into::into),
            created_by: x.created_by,
            changed_by: x.changed_by,
            //-------------------------
            pricing_quantity: x.pricing_quantity,
            pricing_unit_id: x.pricing_unit_id,
            price_unit: x
                .pricing_unit_id
                .map(|x| x.map(Into::into).unwrap_or_default()),
            pricing_price: x.pricing_price,
            pricing_price_rub: x.pricing_price_rub,

            pricing_vat_id: x.pricing_vat_id,
            pricing_currency_id: x.pricing_currency_id,
            pricing_currency_rate: x.pricing_currency_rate,
            pricing_currency_rate_date: x.pricing_currency_rate_date,
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
            ..Default::default()
        }
    }
}

impl From<crate::PlanItemRep> for PlanItemLegacyRep {
    fn from(x: crate::PlanItemRep) -> Self {
        Self {
            uuid: x.uuid,
            plan_uuid: x.plan_uuid,
            number: x.id.map(|x| x as i16),
            description_internal: x
                .description_internal
                .map(|x| x.unwrap_or_default()),
            currency_id: x.currency_id.map(Into::into),
            currency_rate: x.currency_rate,
            category_id: x.category_id,
            product_type_id: x.product_type_id,
            budget_item_id: x.budget_item_id,
            okved2_id: x.okved2_id,
            okato_id: x.okato_id.map(|x| x.unwrap_or_default()),
            unit_id: x.unit_id.map(Into::into),
            payment_balance_item_id: x.payment_balance_item_id,
            is_not_russian_delivery: x.is_not_russian_delivery,
            quantity: x.quantity,
            created_at: x.created_at.map(Into::into),
            changed_at: x.changed_at.map(Into::into),
            created_by: x.created_by,
            changed_by: x.changed_by,
            //-------------------------
            ..Default::default()
        }
    }
}

/// Не все поля преобразуются. Это надо потом учесть.
impl From<PlanItemLegacyRep> for PlanItemFullRep {
    fn from(x: PlanItemLegacyRep) -> Self {
        Self {
            uuid: x.uuid,
            // FIXME: id must go away
            id: x.number.map(Into::into),
            number: x.number,
            description_internal: x.description_internal.map(Into::into),
            product_type_id: x.product_type_id,
            category_id: x.category_id,
            budget_item_id: x.budget_item_id,
            payment_balance_item_id: x.payment_balance_item_id,
            okpd2_id: x.okpd2_id.map(Into::into),
            okved2_id: x.okved2_id,
            okato_id: x.okato_id.map(Into::into),
            is_not_russian_delivery: x.is_not_russian_delivery,
            delivery_basis: x.delivery_basis,
            unit_id: x.unit_id.map(|x| x as i16),
            quantity: x.quantity,
            price: x.price,
            //----------------
            pricing_unit_id: x.price_unit.map(|x| match x == 0 {
                true => None,
                false => Some(x as i16),
            }),
            currency_id: x.currency_id.map(|x| x as i16),
            currency_rate: x.currency_rate,
            currency_rate_date: x.currency_rate_date.map(Into::into),
            vat_id: x.vat_id,
            sum_excluded_vat: x.sum_excluded_vat,
            sum_vat: x.sum_vat,
            sum_included_vat: x.sum_included_vat,
            sum_excluded_vat_rub: x.sum_excluded_vat_rub,
            sum_vat_rub: x.sum_vat_rub,
            sum_included_vat_rub: x.sum_included_vat_rub.map(Into::into),
            delivery_start_date: x.delivery_start_date.map(Into::into),
            delivery_end_date: x.delivery_end_date.map(Into::into),
            price_source_1_text: x.price_source_1_text.map(Into::into),
            price_source_1_price: x.price_source_1_price.map(Into::into),
            price_source_1_date: x.price_source_1_date.map(Into::into),
            price_source_2_text: x.price_source_2_text.map(Into::into),
            price_source_2_price: x.price_source_2_price.map(Into::into),
            price_source_2_date: x.price_source_2_date.map(Into::into),
            //----------------------
            price_source_3_text: x.price_source_3_text.map(Into::into),
            price_source_3_price: x.price_source_3_price.map(Into::into),
            price_source_3_date: x.price_source_3_date.map(Into::into),
            is_analog_allowed: x.is_analog_allowed.map(Into::into),
            analog_price: x.analog_price.map(Into::into),
            analog_text: x.analog_text.map(Into::into),
            analog_producer_id: x.analog_producer_id.map(Into::into),
            analog_country_id: x.analog_country_id.map(Into::into),
            analog_requirements: x.analog_requirements.map(Into::into),
            mark: x.mark.map(Into::into),
            mark_main: x.mark_main.map(Into::into),
            technical_characteristics: x.technical_characteristics.map(Into::into),
            technical_requirements: x.technical_requirements.map(Into::into),
            gosts: x.gosts.map(Into::into),
            material_code_ius_mtr: x.material_code_ius_mtr.map(Into::into),
            is_serial: x.is_serial.map(Into::into),
            //----------------------
            pzp_code: x.pzp_code.map(Into::into),
            nomenclature_group_id: x.nomenclature_group_id.map(Into::into),
            source_country_id: x.source_country_id.map(Into::into),
            producer_country_id: x.producer_country_id.map(Into::into),
            producer_id: x.producer_id.map(Into::into),
            previous_price: x.previous_price.map(Into::into),
            previous_delivery_date: x
                .previous_delivery_date
                .map(super::make_none_if_default),
            investment_project_id: x.investment_project_id.map(Into::into),
            is_dealer: x.is_dealer.map(Into::into),
            is_material_registry: x.is_material_registry.map(Into::into),
            certificate_holder_id: x.certificate_holder_id.map(Into::into),
            certificate_text: x.certificate_text.map(Into::into),
            certificate_number: x.certificate_number.map(Into::into),
            repair_summary_code: x.repair_summary_code.map(Into::into),
            repair_inventory_number: x.repair_inventory_number.map(Into::into),
            repair_text: x.repair_text.map(Into::into),
            repair_plan_code: x.repair_plan_code.map(Into::into),
            //--------------------
            is_centralized_delivery: x.is_centralized_delivery.map(Into::into),
            centralized_sum: x.centralized_sum.map(Into::into),
            is_removed: x.is_removed,
            created_at: x.created_at.map(Into::into),
            created_by: x.created_by,
            changed_at: x.changed_at.map(Into::into),
            changed_by: x.changed_by,
            //-------------------------
            pricing_quantity: x.pricing_quantity,
            pricing_price: x.pricing_price,
            pricing_price_rub: x.pricing_price_rub,

            pricing_vat_id: x.pricing_vat_id,
            pricing_currency_id: x.pricing_currency_id,
            pricing_currency_rate: x.pricing_currency_rate,
            pricing_currency_rate_date: x.pricing_currency_rate_date,
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
            plan_uuid: x.plan_uuid,

            note: None,
            transportation_price: None,
            transportation_vat_id: None,
            transportation_sum_included_vat: None,
            price_source_1_sum_included_vat: None,
            price_source_2_sum_included_vat: None,
            price_source_3_sum_included_vat: None,
            material_code_ius_local: None,
            investment_project_code: None,
            prepayment_percent: None,
            payment_delay: None,
            psd_price: None,
            psd_date: None,
            psd_code: None,
            onm_price: None,
            material_registry_price: None,
            expert_price: None,
            expert_sum_included_vat: None,
            created_at_date: None,
            changed_at_date: None,
            plan_id_lotting: None,
            uuid_item_proposal: None,
            pricing_created_at: None,
            pricing_changed_at: None,
        }
    }
}
