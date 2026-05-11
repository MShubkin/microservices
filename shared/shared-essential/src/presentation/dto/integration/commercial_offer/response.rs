use asez2_shared_db::db_item::AsezDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use asez2_tables::maths::VatId;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommercialOfferResponseData {
    pub tcp: TechnicalCommercialProposal,
    pub monolith_token: String,
    pub user_id: i32,
    pub hierarchy_uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommercialOfferResponse {
    /// Идентификатор сообщения
    pub request_id: String,
    /// ТКП
    pub technical_commercial_proposal: TechnicalCommercialProposal,
    /// Документы
    pub attachment: Attachment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommercialOfferAddDocResponse {
    /// Идентификатор сообщения
    pub request_id: String,
    ///ID ЗЦИ ЭТП ГПБ
    pub tcp_id: i32,
    /// Документы
    pub attachment: Attachment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TechnicalCommercialProposal {
    /// Идентификатор объекта
    pub req_info: ReqInfo,
    ///ID ЗЦИ ЭТП ГПБ
    pub tcp_id: i32,
    /// Начало срока действия
    pub date_start_proposal: AsezDate,
    /// Окончание срока действия
    pub date_end_proposal: AsezDate,
    /// Поставщики
    pub supplier: Supplier,
    /// Позиции ТКП
    pub price_info: Vec<PriceInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqInfo {
    /// Номер ЗЦИ
    pub req_number: i64,
    /// UUID ЗЦИ
    pub req_uuid: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub inn: String,
    pub kpp: String,
    pub email: String,
    /// Номер поставщика (код АСЭЗ)
    pub asez_id: Option<i32>,
    /// Номер ЭТП ГПБ
    pub etp_id: Option<i32>,
    /// Контактный телефон
    pub contact_phone: Option<String>,
    /// Юридический адрес
    pub legal_address: String,
    /// Контактные данные поставщика
    pub contact_info: Option<ContactInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactInfo {
    pub phone: String,
    pub email: String,
    pub additional_email: Option<String>,
    pub full_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceInfo {
    /// Номер позиции
    pub price_info_pos_nr: i64,
    /// Цена за ед. без НДС
    pub price: f64,
    /// Ставка НДС
    pub nds_rate: String,
    /// Стоимость за весь объем без НДС
    pub cost: f64,
    /// Стоимость за весь объем с НДС
    pub cost_nds: f64,
    /// Условия оплаты
    pub terms_of_payment: String,
    /// Размер аванса, %
    pub prepayment_percent: Option<f64>,
    /// Условия поставки
    pub terms_of_delivery: Option<String>,
    /// % выполнения собственными силами
    pub execution_percent: Option<f64>,
    /// Возможность поставки
    pub impossible_to_do: Option<bool>,
    /// Причина невозможности поставки
    pub cause_impossible: Option<String>,
    /// Срок поставки
    pub delivery_period: Option<String>,
    /// Соответствие требованиям заказчика
    pub compliance_customer_requirements: String,
    /// Производитель
    pub manufacturer: String,
    /// Тип, марка продукции
    pub product_mark: Option<String>,
    /// Техническое описание предлагаемого эквивалента
    pub analog_description: Option<String>,
    pub vat_id: Option<VatId>,
    pub pay_condition_id: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    /// Общее число прикрепляемых документов
    pub total_doc_count: Option<i32>,
    /// Документы
    pub documents: Option<Vec<Document>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    /// Глобальный идентификатор документа
    pub id: i32,
    /// Имя файла
    pub file_name: String,
    /// Содержимое файла
    pub content: String,
}
