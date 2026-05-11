use super::*;
use crate::master_data::estimated_commission::protocol_type::EstimatedCommissionProtocolTypeId;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};

/// Id статуса протокола
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
#[repr(i16)]
pub enum EstimatedCommissionProtocolStatusId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Сформирован
    Formed = 100,
    /// На согласовании
    OnApproval = 200,
    /// На подписании
    AtTheSigning = 300,
    /// Утвержден
    Approved = 400,
    /// Удален
    Removed = 500,
}

impl Display for EstimatedCommissionProtocolStatusId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EstimatedCommissionProtocolStatusId::Undefined => "Не установлено",
            EstimatedCommissionProtocolStatusId::OnApproval => "На согласовании",
            EstimatedCommissionProtocolStatusId::AtTheSigning => "На подписании",
            EstimatedCommissionProtocolStatusId::Approved => "Утвержден",
            EstimatedCommissionProtocolStatusId::Removed => "Удален",
            EstimatedCommissionProtocolStatusId::Formed => "Сформирован",
        };
        write!(f, "{}", str)
    }
}

/// Справочник "Статусы Протокола"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "protocol_status"]
pub struct EstimatedCommissionProtocolStatus {
    /// Id статуса протокола
    #[item_field_pkey]
    #[item_field_autogen]
    #[adaptor_field_duplicate = "status_id"]
    pub id: EstimatedCommissionProtocolStatusId,
    /// Id типа протокола
    #[item_field_pkey]
    pub protocol_type_id: EstimatedCommissionProtocolTypeId,
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
