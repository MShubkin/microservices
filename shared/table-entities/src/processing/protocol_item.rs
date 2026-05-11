//! This is used in processing for Estimates Commission related work:
use std::fmt::Display;

use fieldname_access::FieldnameAccess;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{AsezTimestamp, DbAdaptor, DbItem, DbItemExt, DbUpsert},
    impl_join_on, joined,
};

use crate::maths::CurrencyValue;
use crate::*;

impl_join_on!(Plan:uuid => EcProtocolItem:source_uuid, aggr);
impl_join_on!(ContractAmendment:uuid => EcProtocolItem:source_uuid, aggr);
impl_join_on!(EcProtocolItem:source_uuid => Plan:uuid);
impl_join_on!(EcProtocolItem:source_uuid => Plan:uuid, left);
impl_join_on!(EcProtocolItem:source_uuid => Plan:uuid, aggr);
impl_join_on!(EcProtocolItem:source_uuid => ContractAmendment:uuid, left);
impl_join_on!(EcProtocolItem:source_uuid => ContractAmendment:uuid, aggr);
impl_join_on!(EcProtocolItem:source_uuid => EcAgendaItem:source_uuid, aggr);
impl_join_on!(EcProtocolItem:protocol_uuid => EcProtocol:uuid);
impl_join_on!(EcProtocolItem:protocol_uuid => EcProtocol:uuid, left);
impl_join_on!(EcProtocolItem:uuid => RelAgendaProtocolItem:protocol_item_uuid, aggr);

joined!(
    // Используется именно джойн по `Plan`, так как требуются все планы, а протоколы
    // опциональны
  plan: Plan,
  protocol_items: EcProtocolItem[Plan => EcProtocolItem, aggr]
);
joined!(
    // Используется именно джойн по `Plan`, так как требуются все планы, а протоколы
    // опциональны
  item: EcProtocolItem,
  plan: Plan[EcProtocolItem => Plan],
  protocol: EcProtocol[EcProtocolItem => EcProtocol],
);
joined!(
    !ProtocolItemWithProtocol,
    item: EcProtocolItem,
    protocol: EcProtocol[EcProtocolItem => EcProtocol],
);
joined!(
    item: EcProtocolItem,
    protocol: EcProtocol[EcProtocolItem => EcProtocol],
    plan: Plan[EcProtocolItem => Plan, left],
    amendment: ContractAmendment[EcProtocolItem => ContractAmendment, left],
);
joined!(
    !ProtocolItemWithPlan,
    item: EcProtocolItem,
    plan: Plan[EcProtocolItem => Plan, left],
    amendment: ContractAmendment[EcProtocolItem => ContractAmendment, left],
);

#[derive(
    Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, DbUpsert, PartialEq,
)]
#[adaptor_derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Deserialize,
    Serialize,
    FieldnameAccess
)]
#[adaptor_attributes(
    #[fieldname_enum(derive = [Debug, Clone, Eq, PartialEq, Ord, PartialOrd])]
)]
#[adaptor_fields_with_values]
#[item_table = "protocol_item"]
#[item_aggr_insert]
pub struct EcProtocolItem {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub protocol_uuid: Uuid,
    pub source_uuid: Uuid,
    pub number: i64,
    pub is_registered_by_d647: bool,
    pub is_removed: bool,
    pub is_excluded: bool,
    pub result_id: ResultId,
    pub sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_sum_excluded_vat: Option<CurrencyValue>,
    pub commission_sum_excluded_vat: Option<CurrencyValue>,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

/// Решение комиссии
///
/// Можно найти описание в документе `Спека СК_новые таблицы`
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    Type,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum ResultId {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    /// Утверждено
    Approved = 1,
    /// Согласовано с корректировкой стоимости
    AgreedWithPriceCorrection = 2,
    /// Не согласовано.
    NotAgreed = 3,
    /// Аннулировать
    Cancel = 4,
}

impl Display for ResultId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ResultId::Approved => "Утверждено",
            ResultId::AgreedWithPriceCorrection => {
                "Согласовано с корректировкой стоимости"
            }
            ResultId::NotAgreed => "Не согласовано.",
            ResultId::Cancel => "Аннулировать",
            ResultId::Undefined => "Не установлено",
        };
        write!(f, "{}", str)
    }
}
