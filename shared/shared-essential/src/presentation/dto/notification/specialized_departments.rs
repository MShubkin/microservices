use asez2_shared_db::db_item::AsezDate;
use asez2_tables::maths::CurrencyValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SdAgreementDirector {
    pub plan_id: i64,
    pub department_name: String,
    pub planned_date: AsezDate,
    pub customer_name: String,
    pub contract_subject: String,
    pub number_customer: String,
    pub sum_included_vat_rub: CurrencyValue,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SdAgreementExpert {
    pub plan_id: i64,
    pub department_name: String,
    pub division_name: String,
    pub expert_name: String,
    pub planned_date: AsezDate,
    pub customer_name: String,
    pub contract_subject: String,
    pub number_customer: String,
    pub sum_included_vat_rub: CurrencyValue,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SdAgreementExecutor {
    pub plan_id: i64,
    pub department_name: String,
    pub contract_subject: String,
    pub number_customer: String,
    pub sum_included_vat_rub: CurrencyValue,
    pub response: String,
    pub response_note: String,
}
