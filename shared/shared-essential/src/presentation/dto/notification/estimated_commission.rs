//! Модуль описывает контракты для отправления уведомления по модуль `Сметная комиссия`
use asez2_tables::maths::CurrencyValue;
use asez2_tables::PricingUnitId;
use serde::{Deserialize, Serialize};

/// Проведение АЦ повторно по ППЗ/ДС
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EcExpertAppointmentRepeat {
    /// Айди ППЗ/ДС,
    pub plan_id: i64,
    /// Заказчик
    pub customer_name: String,
    /// Номер закупки Заказчика
    pub number_customer: String,
    /// Предмет договора
    pub contract_subject: String,
    /// Ответственное подразделение
    pub unit_id: PricingUnitId,
    /// Общая заявленная стоимость закупки (с НДС), в руб.
    pub sum_excluded_vat: CurrencyValue,
}
