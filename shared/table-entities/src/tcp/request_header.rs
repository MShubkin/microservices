use serde::{Deserialize, Serialize};
use uuid::Uuid;

use asez2_shared_db::db_item::{AsezTimestamp, DbItemDel};
use asez2_shared_db::DbItem;
use shared_db_derive::DbAdaptor;

use super::TcpDbItem;
use super::{PriceInformationRequestStatus, PriceInformationRequestType};

/// Атрибуты заголовка ЗЦИ
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[adaptor_fields_with_values]
#[item_table = "request_head"]
pub struct RequestHeader {
    /// UUID ЗЦИ
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Номер ЗЦИ
    #[item_field_autogen_always]
    pub id: i64,
    /// UUID ППЗ/ДС
    pub plan_uuid: Option<Uuid>,
    /// Номер ППЗ/ДС
    pub plan_id: Option<i64>,
    /// UUID иерархии документов
    pub hierarchy_uuid: Option<Uuid>,
    /// Тип ЗЦИ
    pub type_request_id: Option<PriceInformationRequestType>,
    /// Предмет ЗЦИ
    pub request_subject: Option<String>,
    /// Дата и время начала сбора ТКП
    pub start_date: Option<AsezTimestamp>,
    /// Дата и время окончания сбора ТКП
    pub end_date: Option<AsezTimestamp>,
    /// Статус ЗЦИ
    pub status_id: PriceInformationRequestStatus,
    /// Заказчик
    pub customer_id: Option<i32>,
    /// Валюта
    pub currency_id: Option<i16>,
    /// Обоснование закрытого ЗЦИ
    pub request_type_text: Option<String>,
    /// Организатор ЗЦИ
    pub organizer_id: Option<i32>,
    /// Контактное лицо
    pub organizer_name: Option<String>,
    /// Электронный адрес
    pub organizer_mail: Option<String>,
    /// Телефон
    pub organizer_phone: Option<String>,
    /// Местонахождение
    pub organizer_location: Option<String>,
    /// Причина досрочного закрытия
    pub reason_closing: Option<String>,
    /// Направление закупки
    pub purchasing_trend_id: Option<i16>,
    /// Создал
    pub created_by: i32,
    /// Дата создания
    pub created_at: AsezTimestamp,
    /// Изменил
    pub changed_by: i32,
    /// Дата изменения
    pub changed_at: AsezTimestamp,
}

impl DbItemDel for RequestHeader {}
impl TcpDbItem for RequestHeader {}

impl AsRef<Self> for RequestHeader {
    fn as_ref(&self) -> &Self {
        self
    }
}
