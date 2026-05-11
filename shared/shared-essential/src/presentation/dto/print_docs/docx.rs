//! Хранение структуры для документов docx
use serde::{Deserialize, Serialize};

/// Повестка СК
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EsSubpoenaReq {
    pub id: u64,
    pub date: String,
    pub pa_department: String,
    pub pa_expert: String,
    pub contracts: Vec<EsSubpoenaContractReq>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EsSubpoenaContractReq {
    pub customer: String,
    pub agent: String,
    pub subject: String,
    pub date: String,
}

/// Протокол СК
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EcProtocolReq {
    pub id: u64,
    pub date: String,
    pub doc_number: String,
    pub city: String,
    pub chairman: String,
    pub secretary: String,
    pub members_commission: String,
    pub questions: Vec<ProtocolReqSomeQuestions>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProtocolReqSomeQuestions {
    pub document_in_question: String,
    pub question: String,
    pub price: String,
}

/// Заключение АЦ
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EsConclusionReq {
    pub id: u64,
    pub date: String,
    pub analysis_expert: String,

    pub statement: String,
    pub customer_name: String,
    pub performer_name: String,
    pub contract_price: f64,
    pub contract_justification: String,
    pub contract_confirmation: String,
    pub contract_confirmation_info: String,
    pub contract_justification_without_competetive: String,
    pub contract_subjects: String,
    pub contract_subjects_confirmation: String,
    pub additional_info: String,
    pub validity_period: u64,
    pub contract_period: u64,
    pub contract_expenses_price: f64,
    pub links: String,
    pub price_proposal: String,
    pub decision_presentation_proposal: String,
    pub location: String,
}
