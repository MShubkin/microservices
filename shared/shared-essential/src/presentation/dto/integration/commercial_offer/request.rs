use asez2_shared_db::db_item::AsezTimestamp;
use monolith_service::dto::attachment::Attachment as MonolithAttachment;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::request_confirmation::ReqInfo as ReqInfoConf;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommercialOfferData {
    pub data: CommercialOfferRequest,
    pub monolith_attachments: Vec<MonolithAttachment>,
    pub monolith_token: String,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommercialOfferAddDocRequest {
    /// Идентификатор сообщения
    pub request_id: Uuid,
    ///ID ЗЦИ ЭТП ГПБ
    pub req_info: ReqInfoConf,
    /// Документы
    pub attachment: Attachment,
    /// Признак завершения загрузки
    pub is_upload_complete: Option<bool>,
}

/// ЗЦИ и пакет документов по закупке
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommercialOfferRequest {
    /// Идентификатор сообщения
    pub request_id: Uuid,
    /// ЗЦИ
    pub request_price_info: RequestPriceInfo,
    /// Документы
    pub attachment: Attachment,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestPriceInfo {
    /// Идентификатор объекта
    pub req_info: ReqInfo,
    /// Тип ЗЦИ
    pub private: bool,
    /// Дата и время окончания сбора ТКП
    pub submission_close_date_time: Option<AsezTimestamp>,
    /// Дата завершения процедуры
    pub procedure_completion_date: Option<AsezTimestamp>,
    /// Дата публикации
    pub publication_planned_date: AsezTimestamp,
    /// Предмет ЗЦИ
    pub request_subject: String,
    /// Вид предмета закупки
    pub subject_type: Vec<String>,
    /// ОКВЭД2
    pub okved2: Vec<Okved2>,
    /// ОКПД2
    pub okpd2: Vec<Okpd2>,
    /// Заказчик
    pub customer: Customer,
    /// Санкционный
    pub sanctions: Option<bool>,
    /// Организатор ЗЦИ
    pub placer: Placer,
    /// Контактные данные
    pub contract_info: ContactInfo,
    /// Валюта
    pub currency: String,
    /// Позиции ЗЦИ
    pub specifications: Vec<Specification>,
    /// Поставщики
    pub supplier: Option<Vec<Supplier>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqInfo {
    /// Номер ЗЦИ
    pub req_number: String,
    /// UUID ЗЦИ
    pub req_uuid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Okved2 {
    /// Код ОКВЭД2
    pub code: String,
    /// Наименование ОКВЭД2
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Okpd2 {
    /// Код ОКПД2
    pub code: String,
    /// /// Наименование ОКПД2
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Customer {
    /// ИНН
    pub inn: String,
    /// КПП
    pub kpp: String,
    /// Номер заказчика (код АСЭЗ)
    pub asez_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Placer {
    /// ИНН
    pub inn: String,
    /// КПП
    pub kpp: String,
    /// Номер заказчика (код АСЭЗ)
    pub asez_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ContactInfo {
    /// Контактный телефон
    pub phone: String,
    /// E-mail
    pub email: String,
    /// Имя
    pub first_name: String,
    /// Фамилия
    pub last_name: String,
    /// Отчество
    pub patronymic: String,
    /// Юр. адрес
    pub legal_address: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Specification {
    /// Номер позиции
    pub pos_nr: String,
    /// Наименование позиции
    pub pos_name: String,
    /// Количество
    pub quantity: String,
    /// Единица измерения
    pub unit_of_measure: i32,
    /// Базис поставки
    pub delivery_basis: Option<String>,
    /// Технические требования
    pub technical_requirements: Option<String>,
    /// Марка, ТУ
    pub product_mark: Option<String>,
    /// Срок поставки/выполнения работ/оказания услуг
    pub delivery_date: String,
    /// Тип позиции
    pub pos_vid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Supplier {
    pub inn: String,
    pub kpp: String,
    pub email: String,
    pub asez_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Attachment {
    /// Общее число прикрепляемых документов
    pub total_doc_count: Option<u32>,
    /// Число документов, загружаемых вне основного пакета
    pub additional_doc_count: Option<u32>,
    /// Документы
    pub documents: Option<Vec<Document>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Document {
    /// Глобальный идентификатор документа
    pub guid: String,
    /// Имя файла
    pub file_name: String,
    /// Описание прикрепляемого документа
    pub description: String,
    /// Содержимое файла
    pub content: String,
    /// Признак удаления
    pub removed: Option<bool>,
}

impl Customer {
    pub fn new(inn: impl Into<String>, kpp: impl Into<String>, id: i32) -> Self {
        Self {
            inn: inn.into(),
            kpp: kpp.into(),
            asez_id: id.to_string(),
        }
    }
}

impl Placer {
    pub fn new(inn: impl Into<String>, kpp: impl Into<String>, id: i32) -> Self {
        Self {
            inn: inn.into(),
            kpp: kpp.into(),
            asez_id: id.to_string(),
        }
    }
}
