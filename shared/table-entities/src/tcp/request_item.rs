use crate::maths::{CurrencyValue, Quantity, VatId};

use asez2_shared_db::db_item::{AsezDate, DbItemDel, DbUpsert};
use asez2_shared_db::DbItem;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbAdaptor;
use uuid::Uuid;

use super::TcpDbItem;

/// Атрибуты позиций ЗЦИ
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[adaptor_fields_with_values]
#[item_table = "request_item"]
pub struct RequestItem {
    /// UUID позиции ЗЦИ
    #[item_field_pkey]
    pub uuid: Uuid,
    /// UUID ЗЦИ
    pub request_uuid: Uuid,
    /// UUID позиции ППЗ/ДС
    pub plan_item_uuid: Uuid,
    /// Номер позиции ЗЦИ (для порядка на ФЕ)
    pub number: i16,
    /// Наименование позиции
    pub description_internal: String,
    /// Количество
    pub quantity: Quantity,
    /// Единица измерения
    pub unit_id: i16,
    /// Вид предмета закупки
    pub category_id: i16,
    /// Тип позиции
    pub product_type_id: i16,
    /// ОКВЭД2
    pub okved2_id: i32,
    /// ОКПД2
    pub okpd2_id: i32,
    /// Марка, ТУ
    pub mark: Option<String>,
    /// Технические требования
    pub technical_requirements: Option<String>,
    /// Базис поставки
    pub delivery_basis: String,
    /// Дата поставки/начало работ/оказания услуг
    pub delivery_start_date: AsezDate,
    /// Дата окончания выполнения работ/оказания услуг
    pub delivery_end_date: AsezDate,
    /// Цена (без НДС)
    pub price: CurrencyValue,
    /// Стоимость (без НДС)
    pub sum_excluded_vat: CurrencyValue,
    /// Ставка НДС
    pub vat_id: VatId,
}

impl DbItemDel for RequestItem {}
impl TcpDbItem for RequestItem {}
impl DbUpsert for RequestItem {}
