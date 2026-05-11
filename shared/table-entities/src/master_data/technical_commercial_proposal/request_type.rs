use super::*;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Id Типа запроса ЗЦИ
#[repr(i16)]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
pub enum PriceInformationRequestTypeId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Открытый
    Opened = 1,
    /// Закрытый
    Closed = 2,
    /// Открытый санкционный
    OpenedSanctions = 3,
    /// Закрытый санкционный
    ClosedSanctions = 4,
}
impl Display for PriceInformationRequestTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PriceInformationRequestTypeId::Undefined => write!(f, "Не задано"),
            PriceInformationRequestTypeId::Opened => write!(f, "Открытый"),
            PriceInformationRequestTypeId::Closed => write!(f, "Закрытый"),
            PriceInformationRequestTypeId::OpenedSanctions => {
                write!(f, "Открытый санкционный")
            }
            PriceInformationRequestTypeId::ClosedSanctions => {
                write!(f, "Закрытый санкционный")
            }
        }
    }
}

/// Справочник "Тип запроса ЗЦИ"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "request_type"]
pub struct PriceInformationRequestType {
    /// Id типа запроса
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: PriceInformationRequestTypeId,
    /// Наименование типа запроса
    #[serde(rename = "text")]
    pub name: String,
    /// Запись удалена
    pub is_removed: bool,
    /// Создано
    pub created_at: AsezTimestamp,
    /// Изменено
    pub changed_at: AsezTimestamp,
    /// Создатель
    pub created_by: i32,
    /// Кем изменено
    pub changed_by: i32,
}
