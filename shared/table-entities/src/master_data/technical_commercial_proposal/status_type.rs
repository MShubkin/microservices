use super::*;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};

/// Справочник "Статусы ТКП"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "status_type"]
pub struct TcpStatus {
    #[item_field_pkey]
    /// Id статуса
    pub id: i16,
    /// Тип объекта
    pub object_type: TcpObjectType,
    /// Тип статуса
    pub status_type: TcpStatusType,
    /// Наименование статуса
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
pub enum TcpObjectType {
    /// ЗЦИ
    #[db_default]
    PriceInformationRequest = 1,
    /// ТКП
    TechnicalCommercialProposal = 2,
}

/// Тип статуса ТКП
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
pub enum TcpStatusType {
    /// Общий
    #[db_default]
    General = 1,
    /// Статус рассмотрения
    Review = 2,
    /// Результат рассмотрения
    Result = 3,
}
