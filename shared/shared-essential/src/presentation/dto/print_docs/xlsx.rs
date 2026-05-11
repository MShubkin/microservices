//! Хранение структуры для документов xlsx
use serde::{Deserialize, Serialize};

/// Бюллетень СК
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EsBulletinReq {
    pub id: u64,
    pub date: String,
    pub location: String,
    pub commission_member_position: String,
    pub commission_member_sign: String,
    pub commission_member_organization: String,
    pub commission_member_name: String,
    pub contracts: Vec<EsBulletinContractReq>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EsBulletinContractReq {
    pub ppz_ds: String,
    pub customer_name: String,
    pub provider_name: String,
    pub contract_subject_name: String,
    pub currency: Currency,
    pub customer_price: f64,
    pub es_price: f64,
    pub es_decision: String,
    pub expert_comment: String,
    pub expert_name: String,
    pub section: String,
    pub purchase_basis: String,
    pub purchase_method: String,
}

/// Структура с индексировавнными значениями
#[derive(Serialize, Deserialize, Debug)]
pub struct IndexedBulletinReq {
    pub id: u64,
    pub date_index: usize,
    pub location_index: usize,
    pub commission_member_position_index: usize,
    pub commission_member_sign_index: usize,
    pub commission_member_organization_index: usize,
    pub commission_member_name_index: usize,
    pub contract_index: Vec<IndexedBulletinContractReq>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexedBulletinContractReq {
    pub ppz_ds_index: usize,
    pub customer_name_index: usize,
    pub provider_name_index: usize,
    pub contract_subject_name_index: usize,
    pub currency: usize,
    pub customer_price: f64,
    pub es_price: f64,
    pub es_decision_index: usize,
    pub expert_comment_index: usize,
    pub expert_name_index: usize,
    pub section_index: usize,
    pub purchase_basis_index: usize,
    pub purchase_method_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Currency {
    #[serde(rename = "USD")]
    Usd,
    #[serde(rename = "RUB")]
    Rub,
}

//
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaPlansReq {
    pub data: Vec<PaPlan>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaPlan {
    // pub ??    // Поступило на назначение Эксперта АЦ
    pub id: String,                   // Номер ППЗ/ДС
    pub customer: String,             // Закзачик
    pub section_id: String,           // Раздел Плана
    pub purchasing_method_id: String, // Способ закупки
    // pub agent_id??    // Контрагент
    // pub ??    // Основание закупки
    pub contract_subject_short: String, // Предмет договора (сокарщенный)
    // pub ??    // Стоимость Заказчика (без НДС)
    pub commission_kind_id: String, // Форма СК
    pub commission_date: String,    // Дата СК
    pub pricing_expert_id: String,  // Эксперт АЦ
    // pub expert_node??    // Внутрениий комментарий АЦ
    // pub ??    // История изменения статуса
    // pub ??    // Ход рассмотрения
    // pub ??    // Профильные Департаменты
    pub items_number: String, // Количество позиций
}
