use asez2_tables::maths::CurrencyValue;
use asez2_tables::PricingUnitId;
use serde::{Deserialize, Serialize};

/// Завершение "лотирования" со статусом 351 (АЦ МТР. Назначение исполнителя)
/// TODO: Fields may change.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LottingCompleted351 {
    pub plan_id: i64,
    pub unit_id: PricingUnitId,
    pub customer_name: String,
    pub purchase_subject: String,
    pub purchase_id: String,
    pub sum_without_vat_rub: CurrencyValue,
    pub comment: String,
}
/// Завершение "лотирования" со статусом 352 (АЦ МТР. Исполнитель назначен)
/// TODO: Fields may change.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LottingCompleted352 {
    pub plan_id: i64,
    pub unit_id: PricingUnitId,
    pub customer_name: String,
    pub purchase_subject: String,
    pub customer_purchase_id: String,
    pub sum_without_vat_rub: CurrencyValue,
    pub comment: String,
}
/// Завершение "лотирования" со статусом 353 (АЦ МТР. Анализ проведен)
/// TODO: Fields may change.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LottingCompleted353 {
    pub plan_id: i64,
    pub unit_id: PricingUnitId,
    pub customer_name: String,
    pub purchase_subject: String,
    pub customer_purchase_id: String,
    pub sum_without_vat_rub: CurrencyValue,
    pub pricing_sum_without_vat_rub: CurrencyValue,
    pub comment: String,
}
