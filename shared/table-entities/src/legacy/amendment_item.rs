//! Работает с ДС (contract_amendment) из монолита.
use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::db_item::{DbItemExt, FieldTolerance};
use asez2_shared_db::{DbAdaptor, DbItem};
use monolith_service::dto::time::PlanningTimestamp;
use shared_db_derive::DbEnum;

use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

use crate::maths::*;

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
#[item_table = "contract_amendment_item_legacy"]
#[item_skip_field_tolerance]
pub struct ContractAmendmentItemLegacy {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub number: i32,
    pub active_uuid: Uuid,
    pub description_internal: String,
    pub description_external: String,
    pub product_type_id: i16,
    pub kind_id: ContractAmendmentItemKindId,
    pub category_id: i32,
    pub budget_item_id: i32,
    pub payment_balance_item_id: i32,
    pub investment_project_id: i32,
    pub okpd2_id: i64,
    pub okved2_id: i32,
    pub okato_id: i32,
    pub is_not_russian_delivery: bool,
    pub delivery_basis: String,
    //------------------------
    pub quantity: Quantity,
    pub unit_id: i32,
    pub price: CurrencyValue,
    pub price_unit: i32,
    pub vat_id: VatId,
    pub sum_excluded_vat: CurrencyValue,
    pub sum_vat: CurrencyValue,
    pub sum_included_vat: CurrencyValue,
    pub currency_id: i32,
    pub currency_rate: CurrencyRate,
    pub currency_rate_date: AsezDate,
    pub sum_excluded_vat_rub: CurrencyValue,
    pub sum_vat_rub: CurrencyValue,
    pub sum_included_vat_rub: CurrencyValue,
    pub delivery_start_date: AsezDate,
    pub delivery_end_date: AsezDate,
    //----------------------------
    pub initial_quantity: Quantity,
    pub initial_unit_id: i32,
    pub initial_price: CurrencyValue,
    pub initial_vat_id: VatId,
    pub initial_sum_excluded_vat: CurrencyValue,
    pub initial_sum_included_vat: CurrencyValue,
    pub initial_currency_id: i32,
    pub initial_currency_rate: CurrencyRate,
    pub initial_currency_rate_date: AsezDate,
    pub initial_sum_excluded_vat_rub: CurrencyValue,
    pub initial_sum_included_vat_rub: CurrencyValue,
    //-----------------------------
    pub previous_quantity: Quantity,
    pub previous_unit_id: i32,
    pub previous_price: CurrencyValue,
    pub previous_vat_id: VatId,
    pub previous_sum_vat: CurrencyValue,
    pub previous_sum_excluded_vat: CurrencyValue,
    pub previous_sum_included_vat: CurrencyValue,
    //------------------------------
    pub previous_currency_id: i32,
    pub previous_currency_rate: CurrencyRate,
    pub previous_currency_rate_date: AsezDate,
    pub previous_sum_excluded_vat_rub: CurrencyValue,
    pub previous_sum_included_vat_rub: CurrencyValue,
    //--------------------------
    pub material_code_local: String,
    pub material_code_ius_mtr: String,
    pub is_serial: bool,
    pub pzp_code: String,
    pub nomenclature_group_id: i16,
    pub source_country_id: i32,
    pub producer_country_id: i32,
    pub producer_id: i32,
    pub is_dealer: bool,
    pub repair_summary_code: String,
    pub repair_inventory_number: String,
    //----------------------------
    pub pricing_vat_id: VatId,
    pub pricing_currency_id: i16,
    pub pricing_currency_rate_date: Option<AsezDate>,
    pub pricing_sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_sum_excluded_vat_rub: Option<CurrencyValue>,
    pub pricing_sum_included_vat: Option<CurrencyValue>,
    pub pricing_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_sum_vat: Option<CurrencyValue>,
    pub pricing_sum_vat_rub: Option<CurrencyValue>,
    pub pricing_transportation_vat_id: VatId,
    pub pricing_transportation_price: CurrencyValue,
    pub pricing_transportation_price_rub: Option<CurrencyValue>,
    pub pricing_transportation_sum_vat: Option<CurrencyValue>,
    pub pricing_transportation_sum_vat_rub: Option<CurrencyValue>,
    pub pricing_transportation_sum_included_vat: Option<CurrencyValue>,
    pub pricing_transportation_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_total_sum: Option<CurrencyValue>,
    pub pricing_total_sum_rub: Option<CurrencyValue>,

    pub pricing_delta_unit_id: Option<i64>,
    pub pricing_delta_quantity: Option<Quantity>,
    pub pricing_delta_currency_id: Option<i64>,
    pub pricing_delta_currency_rate_date: Option<AsezDate>,
    pub pricing_currency_rate: Option<CurrencyRate>,
    pub pricing_delta_price: Option<CurrencyValue>,
    pub pricing_delta_price_rub: Option<CurrencyValue>,
    pub pricing_delta_sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_delta_sum_excluded_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_sum_vat: Option<CurrencyValue>,
    pub pricing_delta_sum_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_sum_included_vat: Option<CurrencyValue>,
    pub pricing_delta_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_transportation_price: Option<CurrencyValue>,
    pub pricing_delta_transportation_price_rub: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_vat: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_included_vat: Option<CurrencyValue>,
    pub pricing_delta_transportation_sum_included_vat_rub: Option<CurrencyValue>,
    pub pricing_delta_total_sum: Option<CurrencyValue>,
    pub pricing_delta_total_sum_rub: Option<CurrencyValue>,
    pub pricing_quantity: Quantity,
    pub pricing_unit_id: i16,
    pub pricing_price: CurrencyValue,
    pub pricing_price_rub: Option<CurrencyValue>,
    //----------------------------
    pub repair_text: String,
    pub repair_plan_code: String,
    pub is_materia_registry: bool,
    pub certificate_holder_id: i32,
    pub certificate_text: String,
    pub certificate_number: String,
    pub is_removed: bool,
    //----------------------------
    pub created_at: PlanningTimestamp,
    pub created_by: i32,
    pub changed_at: PlanningTimestamp,
    pub changed_by: i32,
}

impl From<crate::ContractAmendmentItemRep> for ContractAmendmentItemLegacyRep {
    fn from(x: crate::ContractAmendmentItemRep) -> Self {
        Self {
            uuid: x.uuid.map(Into::into),
            number: x.id.map(|x| x as i32),
            active_uuid: x.active_uuid.map(Into::into),
            description_internal: x.description_internal.map(Into::into),
            description_external: x.description_external.map(Into::into),
            product_type_id: x.product_type_id,
            kind_id: x.kind_id,
            category_id: x.category_id.map(Into::into),
            budget_item_id: x.budget_item_id.map(Into::into),
            payment_balance_item_id: x.payment_balance_item_id.map(Into::into),
            investment_project_id: x.investment_project_id.map(Into::into),
            okpd2_id: x.okpd2_id.map(Into::into),
            okved2_id: x.okved2_id.map(Into::into),
            okato_id: x.okato_id.map(Into::into),
            is_not_russian_delivery: x.is_not_russian_delivery.map(Into::into),
            delivery_basis: x.delivery_basis.map(Into::into),
            //------------------------
            quantity: x.quantity.map(Into::into),
            unit_id: x.unit_id.map(Into::into),
            price: x.price.map(Into::into),
            price_unit: x.price_unit.map(Into::into),
            vat_id: x.vat_id,
            sum_excluded_vat: x.sum_excluded_vat.map(Into::into),
            sum_vat: x.sum_vat.map(Into::into),
            sum_included_vat: x.sum_included_vat.map(Into::into),
            currency_id: x.currency_id.map(Into::into),
            currency_rate: x.currency_rate.map(Into::into),
            currency_rate_date: x.currency_rate_date.map(Into::into),
            sum_excluded_vat_rub: x.sum_excluded_vat_rub.map(Into::into),
            sum_vat_rub: x.sum_vat_rub.map(Into::into),
            sum_included_vat_rub: x.sum_included_vat_rub.map(Into::into),
            delivery_start_date: x.delivery_start_date.map(Into::into),
            delivery_end_date: x.delivery_end_date.map(Into::into),
            //----------------------------
            initial_quantity: x.initial_quantity.map(Into::into),
            initial_unit_id: x.initial_unit_id.map(Into::into),
            initial_price: x.initial_price.map(Into::into),
            initial_vat_id: x.initial_vat_id,
            initial_sum_excluded_vat: x.initial_sum_excluded_vat.map(Into::into),
            initial_sum_included_vat: x.initial_sum_included_vat.map(Into::into),
            initial_currency_id: x.initial_currency_id.map(Into::into),
            initial_currency_rate: x.initial_currency_rate.map(Into::into),
            initial_currency_rate_date: x
                .initial_currency_rate_date
                .map(Into::into),
            initial_sum_excluded_vat_rub: x
                .initial_sum_excluded_vat_rub
                .map(Into::into),
            initial_sum_included_vat_rub: x
                .initial_sum_included_vat_rub
                .map(Into::into),
            //-----------------------------
            previous_quantity: x.previous_quantity.map(Into::into),
            previous_unit_id: x.previous_unit_id.map(Into::into),
            previous_price: x.previous_price.map(Into::into),
            previous_vat_id: x.previous_vat_id.map(Into::into),
            previous_sum_vat: x.previous_sum_vat.map(Into::into),
            previous_sum_excluded_vat: x.previous_sum_excluded_vat.map(Into::into),
            previous_sum_included_vat: x.previous_sum_included_vat.map(Into::into),
            //------------------------------
            previous_currency_id: x.previous_currency_id.map(Into::into),
            previous_currency_rate: x.previous_currency_rate.map(Into::into),
            previous_currency_rate_date: x
                .previous_currency_rate_date
                .map(Into::into),
            previous_sum_excluded_vat_rub: x
                .previous_sum_excluded_vat_rub
                .map(Into::into),
            previous_sum_included_vat_rub: x
                .previous_sum_included_vat_rub
                .map(Into::into),
            //--------------------------
            material_code_local: x.material_code_local.map(Into::into),
            material_code_ius_mtr: x.material_code_ius_mtr.map(Into::into),
            is_serial: x.is_serial.map(Into::into),
            pzp_code: x.pzp_code.map(Into::into),
            nomenclature_group_id: x.nomenclature_group_id,
            source_country_id: x.source_country_id.map(Into::into),
            producer_country_id: x.producer_country_id.map(Into::into),
            producer_id: x.producer_id.map(Into::into),
            is_dealer: x.is_dealer.map(Into::into),
            repair_summary_code: x.repair_summary_code.map(Into::into),
            repair_inventory_number: x.repair_inventory_number.map(Into::into),
            //----------------------------
            repair_text: x.repair_text.map(Into::into),
            repair_plan_code: x.repair_plan_code.map(Into::into),
            is_materia_registry: x.is_material_registry.map(Into::into),
            certificate_holder_id: x.certificate_holder_id.map(Into::into),
            certificate_text: x.certificate_text.map(Into::into),
            certificate_number: x.certificate_number.map(Into::into),
            is_removed: x.is_removed.map(Into::into),
            //--------------------------
            pricing_vat_id: x.pricing_vat_id,
            pricing_currency_id: x.pricing_currency_id,
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

            pricing_delta_unit_id: x.pricing_delta_unit_id,
            pricing_delta_quantity: x.pricing_delta_quantity,
            pricing_delta_currency_id: x.pricing_delta_currency_id,
            pricing_delta_currency_rate_date: x.pricing_delta_currency_rate_date,
            pricing_currency_rate: x.pricing_currency_rate,
            pricing_delta_price: x.pricing_delta_price,
            pricing_delta_price_rub: x.pricing_delta_price_rub,
            pricing_delta_sum_excluded_vat: x.pricing_delta_sum_excluded_vat,
            pricing_delta_sum_excluded_vat_rub: x
                .pricing_delta_sum_excluded_vat_rub,
            pricing_delta_sum_vat: x.pricing_delta_sum_vat,
            pricing_delta_sum_vat_rub: x.pricing_delta_sum_vat_rub,
            pricing_delta_sum_included_vat: x.pricing_delta_sum_included_vat,
            pricing_delta_sum_included_vat_rub: x
                .pricing_delta_sum_included_vat_rub,
            pricing_delta_transportation_price: x
                .pricing_delta_transportation_price,
            pricing_delta_transportation_price_rub: x
                .pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat: x
                .pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub: x
                .pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat: x
                .pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub: x
                .pricing_delta_transportation_sum_included_vat_rub,
            pricing_delta_total_sum: x.pricing_delta_total_sum,
            pricing_delta_total_sum_rub: x.pricing_delta_total_sum_rub,
            pricing_quantity: x.pricing_quantity,
            pricing_unit_id: x.pricing_unit_id,
            pricing_price: x.pricing_price,
            pricing_price_rub: x.pricing_price_rub,
            //---------------------
            created_at: x.created_at.map(Into::into),
            created_by: x.created_by.map(Into::into),
            changed_at: x.changed_at.map(Into::into),
            changed_by: x.changed_by.map(Into::into),
        }
    }
}

impl From<ContractAmendmentItemLegacyRep> for crate::ContractAmendmentItemRep {
    fn from(x: ContractAmendmentItemLegacyRep) -> Self {
        Self {
            uuid: x.uuid.map(Into::into),
            id: x.number.map(|x| x as i64),
            number: x.number.map(|x| x as i16),
            active_uuid: x.active_uuid.map(Into::into),
            description_internal: x.description_internal.map(Into::into),
            description_external: x.description_external.map(Into::into),
            product_type_id: x.product_type_id,
            kind_id: x.kind_id,
            category_id: x.category_id.map(|x| x as i16),
            budget_item_id: x.budget_item_id.map(|x| x as i16),
            payment_balance_item_id: x.payment_balance_item_id.map(|x| x as i16),
            investment_project_id: x.investment_project_id.map(Into::into),
            okpd2_id: x.okpd2_id.map(|x| x as i32),
            okved2_id: x.okved2_id.map(|x| x as i16),
            okato_id: x.okato_id.map(Into::into),
            is_not_russian_delivery: x.is_not_russian_delivery.map(Into::into),
            delivery_basis: x.delivery_basis.map(Into::into),
            //------------------------
            quantity: x.quantity.map(Into::into),
            unit_id: x.unit_id.map(|x| x as i16),
            price: x.price.map(Into::into),
            price_unit: x.price_unit.map(|x| x as i16),
            vat_id: x.vat_id,
            sum_excluded_vat: x.sum_excluded_vat.map(Into::into),
            sum_vat: x.sum_vat.map(Into::into),
            sum_included_vat: x.sum_included_vat.map(Into::into),
            currency_id: x.currency_id.map(|x| x as i16),
            currency_rate: x.currency_rate.map(Into::into),
            currency_rate_date: x.currency_rate_date.map(Into::into),
            sum_excluded_vat_rub: x.sum_excluded_vat_rub.map(Into::into),
            sum_vat_rub: x.sum_vat_rub.map(Into::into),
            sum_included_vat_rub: x.sum_included_vat_rub.map(Into::into),
            delivery_start_date: x.delivery_start_date.map(Into::into),
            delivery_end_date: x.delivery_end_date.map(Into::into),
            //----------------------------
            initial_quantity: x.initial_quantity.map(Into::into),
            initial_unit_id: x.initial_unit_id.map(|x| x as i16),
            initial_price: x.initial_price.map(Into::into),
            initial_vat_id: x.initial_vat_id,
            initial_sum_excluded_vat: x.initial_sum_excluded_vat.map(Into::into),
            initial_sum_included_vat: x.initial_sum_included_vat.map(Into::into),
            initial_currency_id: x.initial_currency_id.map(|x| x as i16),
            initial_currency_rate: x.initial_currency_rate.map(Into::into),
            initial_currency_rate_date: x
                .initial_currency_rate_date
                .map(Into::into),
            initial_sum_excluded_vat_rub: x
                .initial_sum_excluded_vat_rub
                .map(Into::into),
            initial_sum_included_vat_rub: x
                .initial_sum_included_vat_rub
                .map(Into::into),
            //-----------------------------
            previous_quantity: x.previous_quantity.map(Into::into),
            previous_unit_id: x.previous_unit_id.map(|x| x as i16),
            previous_price: x.previous_price.map(Into::into),
            previous_vat_id: x.previous_vat_id,
            previous_sum_vat: x.previous_sum_vat.map(Into::into),
            previous_sum_excluded_vat: x.previous_sum_excluded_vat.map(Into::into),
            previous_sum_included_vat: x.previous_sum_included_vat.map(Into::into),
            //------------------------------
            previous_currency_id: x.previous_currency_id.map(|x| x as i16),
            previous_currency_rate: x.previous_currency_rate.map(Into::into),
            previous_currency_rate_date: x
                .previous_currency_rate_date
                .map(Into::into),
            previous_sum_excluded_vat_rub: x
                .previous_sum_excluded_vat_rub
                .map(Into::into),
            previous_sum_included_vat_rub: x
                .previous_sum_included_vat_rub
                .map(Into::into),
            //--------------------------
            material_code_local: x.material_code_local.map(Into::into),
            material_code_ius_mtr: x.material_code_ius_mtr.map(Into::into),
            is_serial: x.is_serial.map(Into::into),
            pzp_code: x.pzp_code.map(Into::into),
            nomenclature_group_id: x.nomenclature_group_id,
            source_country_id: x.source_country_id.map(|x| x as i16),
            producer_country_id: x.producer_country_id.map(|x| x as i16),
            producer_id: x.producer_id.map(Into::into),
            is_dealer: x.is_dealer.map(Into::into),
            repair_summary_code: x.repair_summary_code.map(Into::into),
            repair_inventory_number: x.repair_inventory_number.map(Into::into),
            //----------------------------
            repair_text: x.repair_text.map(Into::into),
            repair_plan_code: x.repair_plan_code.map(Into::into),
            is_material_registry: x.is_materia_registry.map(Into::into),
            certificate_holder_id: x.certificate_holder_id.map(Into::into),
            certificate_text: x.certificate_text.map(Into::into),
            certificate_number: x.certificate_number.map(Into::into),
            is_removed: x.is_removed.map(Into::into),
            //--------------------------
            pricing_vat_id: x.pricing_vat_id,
            pricing_currency_id: x.pricing_currency_id,
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

            pricing_delta_unit_id: x.pricing_delta_unit_id,
            pricing_delta_quantity: x.pricing_delta_quantity,
            pricing_delta_currency_id: x.pricing_delta_currency_id,
            pricing_delta_currency_rate_date: x.pricing_delta_currency_rate_date,
            pricing_currency_rate: x.pricing_currency_rate,
            pricing_delta_price: x.pricing_delta_price,
            pricing_delta_price_rub: x.pricing_delta_price_rub,
            pricing_delta_sum_excluded_vat: x.pricing_delta_sum_excluded_vat,
            pricing_delta_sum_excluded_vat_rub: x
                .pricing_delta_sum_excluded_vat_rub,
            pricing_delta_sum_vat: x.pricing_delta_sum_vat,
            pricing_delta_sum_vat_rub: x.pricing_delta_sum_vat_rub,
            pricing_delta_sum_included_vat: x.pricing_delta_sum_included_vat,
            pricing_delta_sum_included_vat_rub: x
                .pricing_delta_sum_included_vat_rub,
            pricing_delta_transportation_price: x
                .pricing_delta_transportation_price,
            pricing_delta_transportation_price_rub: x
                .pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat: x
                .pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub: x
                .pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat: x
                .pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub: x
                .pricing_delta_transportation_sum_included_vat_rub,
            pricing_delta_total_sum: x.pricing_delta_total_sum,
            pricing_delta_total_sum_rub: x.pricing_delta_total_sum_rub,
            pricing_quantity: x.pricing_quantity,
            pricing_unit_id: x.pricing_unit_id,
            pricing_price: x.pricing_price,
            pricing_price_rub: x.pricing_price_rub,
            //---------------------
            created_at: x.created_at.map(Into::into),
            created_by: x.created_by.map(Into::into),
            changed_at: x.changed_at.map(Into::into),
            changed_by: x.changed_by.map(Into::into),
            ..Default::default()
        }
    }
}

impl FieldTolerance for ContractAmendmentItemLegacy {
    const TOLERATED: &'static [(&'static str, &'static str)] = &[];
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    DbEnum,
    Type,
    Ord,
    PartialOrd,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum ContractAmendmentItemKindId {
    #[db_default]
    Undefined = 0,
    /// Без изменений.
    Unchanged = 1,
    /// Увеличение стоимости с изменением сроков.
    SumIncreasedChangedTerms = 2,
    /// Увеличение стоимости без изменения сроков.
    SumIncreasedSameTerms = 3,
    /// Уменьшение стоимости с изменением сроков.
    SumDecreasedChangedTerms = 4,
    /// Уменьшение стоимости без изменения сроков.
    SumDecreasedSameTerms = 5,
    /// Изменение сроков без изменения стоимости.
    SameSumChangedTerms = 6,
    /// Новая.
    New = 7,
    /// Аннулирована.
    Cancelled = 8,
}
