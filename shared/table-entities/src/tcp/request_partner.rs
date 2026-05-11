use asez2_shared_db::db_item::{DbItemDel, DbUpsert};
use asez2_shared_db::DbItem;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbAdaptor;
use uuid::Uuid;

use super::TcpDbItem;

/// Атрибуты поставщиков запроса ценовой информации
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "request_partner"]
pub struct RequestPartner {
    /// UUID записи
    #[item_field_pkey]
    pub uuid: Uuid,
    /// UUID ЗЦИ
    pub request_uuid: Uuid,
    /// Поставщик
    pub supplier_id: i32,
    /// Порядковый номер записи в рамках ЗЦИ
    pub number: i16,
    /// Публикуется на ЭТП ГПБ
    pub is_public: bool,
    /// Отправка материалов по электронной почте
    pub is_phone_check: bool,
    /// Телефонные переговоры
    pub is_email_check: bool,
    /// Результат коммуникаций
    pub resume: Option<String>,
    /// Комментарий, текущая ситуация
    pub comment: Option<String>,
    /// Метка удаления
    pub is_removed: bool,
}

impl DbItemDel for RequestPartner {}
impl TcpDbItem for RequestPartner {}
impl DbUpsert for RequestPartner {}
