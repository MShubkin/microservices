use asez2_shared_db::db_item::DbUpsert;
use asez2_shared_db::DbItem;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbAdaptor;
use uuid::Uuid;

use super::{DbItemDel, TcpDbItem};
use crate::maths::{CurrencyValue, Quantity, VatId};

/// Позиция ТКП
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "proposal_item"]
pub struct ProposalItem {
    /// UID Позиции ТКП
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Номер позиции ЗЦИ (для порядка на ФЕ)
    pub number: i16,
    /// UID ТКП
    pub proposal_uuid: Uuid,
    /// UID позиции ЗЦИ
    pub request_item_uuid: Uuid,
    /// Наименование позиции
    pub description_internal: String,
    /// Количество от Организации
    pub quantity: Quantity,
    /// Единица измерения Организации
    pub unit_id: i32,
    /// Цена Организации (без НДС)
    #[adaptor_rename = "supplier_price"]
    pub price: Option<CurrencyValue>,
    /// Ставка НДС Организации
    #[adaptor_rename = "supplier_vat_id"]
    pub vat_id: Option<VatId>,
    /// Стоимость Организации (c НДС)
    #[adaptor_rename = "supplier_sum_included_vat"]
    pub sum_included_vat: Option<CurrencyValue>,
    /// Стоимость Организации (без НДС)
    #[adaptor_rename = "supplier_sum_excluded_vat"]
    pub sum_excluded_vat: Option<CurrencyValue>,
    /// Наименование производителя
    pub manufacturer: Option<String>,
    /// Тип, марка продукции
    pub mark: Option<String>,
    /// Условия оплаты
    pub pay_condition_id: Option<i16>,
    /// Размер аванса, %
    pub prepayment_percent: Option<CurrencyValue>,
    /// Условия поставки
    pub delivery_condition: Option<String>,
    /// % выполнения собственными силами
    pub execution_percent: Option<CurrencyValue>,
    /// Возможность поставки
    pub is_possibility: bool,
    /// Причина невозможности поставки
    pub possibility_note: Option<String>,
    /// Техническое описание предлагаемого эквивалента
    pub analog_description: Option<String>,
    /// Срок поставки
    pub delivery_period: Option<String>,
}

impl DbItemDel for ProposalItem {}
impl DbUpsert for ProposalItem {}
impl TcpDbItem for ProposalItem {}
