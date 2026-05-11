//! This is used in processing for Estimates Commission related work:

use asez2_shared_db::db_item::{
    AsezDate, AsezTimestamp, DbAdaptor, DbItem, DbUpsert, FieldTolerance,
};
use asez2_shared_db::{impl_join_on, joined};
use fieldname_access::FieldnameAccess;
use serde::{Deserialize, Serialize};
use shared_db_derive::{DbEnum, DbItemExt};
use sqlx::Type;
use uuid::Uuid;

use super::*;
use crate::PricingUnitId;

impl_join_on!(EcAgenda:uuid => EcAgendaItem:agenda_uuid, aggr);
impl_join_on!(EcAgenda:uuid => RelAgendaProtocolItem:agenda_uuid, aggr);
impl_join_on!(EcAgenda:uuid => EcPartner:protocol_agenda_uuid, aggr);
impl_join_on!(EcAgenda:uuid => Attachment:object_uuid, aggr);
impl_join_on!(EcAgenda:uuid => StatusHistory:object_uuid, aggr);
joined!(
    agenda: EcAgenda,
    agenda_items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr],
    plans: Plan[EcAgendaItem => Plan, aggr],
);
// Структура сложная но позволяет нам всё за одн раз вытянуть без пара.
joined!(
    agenda: EcAgenda,
    items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr],
    plans: Plan[EcAgendaItem => Plan, aggr],
    amendments: ContractAmendment[EcAgendaItem => ContractAmendment, aggr]
);

joined!(
    agenda: EcAgenda,
    agenda_items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr],

    plans: Plan[EcAgendaItem => Plan, aggr],
    amendments: ContractAmendment[EcAgendaItem => ContractAmendment, aggr],

    item_relation_agenda_protocol: RelAgendaProtocolItem[EcAgendaItem => RelAgendaProtocolItem, aggr]
);

joined!(
    agenda: EcAgenda,
    agenda_items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr],

    plans: Plan[EcAgendaItem => Plan, aggr],
    amendments: ContractAmendment[EcAgendaItem => ContractAmendment, aggr],

    protocol_rel: RelAgendaProtocol[EcAgenda => RelAgendaProtocol, aggr],
    protocols: EcProtocol[RelAgendaProtocol => EcProtocol, aggr]
);

joined!(
    agenda: EcAgenda,
    protocol_rel: RelAgendaProtocol[EcAgenda => RelAgendaProtocol, aggr],
    protocols: EcProtocol[RelAgendaProtocol => EcProtocol, aggr]
);

joined!(
    agenda: EcAgenda,
    agenda_items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr]
);

joined!(
    agenda: EcAgenda,
    items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr],
    item_rels: RelAgendaProtocolItem[EcAgenda => RelAgendaProtocolItem, aggr]
);

joined!(
    agenda: EcAgenda,
    status_histories: StatusHistory[EcAgenda => StatusHistory, aggr],
);

pub mod relation_by_agenda_item {
    use super::*;
    joined!(
        !AgendaWithItemsAndRelsByAgendaItem,
        agenda: EcAgenda,
        items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr],
        item_rels: RelAgendaProtocolItem[EcAgendaItem => RelAgendaProtocolItem, aggr]
    );
}

joined!(
    !AgendaDetails,
    agenda: EcAgenda,
    agenda_items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr],

    plans: Plan[EcAgendaItem => Plan, aggr],
    amendments: ContractAmendment[EcAgendaItem => ContractAmendment, aggr],

    partner_list: EcPartner[EcAgenda => EcPartner, aggr],
    attachment_list: Attachment[EcAgenda => Attachment, aggr],
    status_histories: StatusHistory[EcAgenda => StatusHistory, aggr],
);

#[derive(
    Debug, Default, Clone, DbItem, DbUpsert, DbAdaptor, PartialEq, DbItemExt,
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
#[item_table = "agenda"]
#[item_skip_field_tolerance]
#[item_aggr_insert]
pub struct EcAgenda {
    #[item_field_pkey]
    pub uuid: Uuid,
    #[adaptor_field_duplicate = "agenda_id"]
    pub id: i64,
    #[adaptor_field_duplicate = "agenda_status_id"]
    pub status_id: EcAgendaStatus,
    pub pricing_organization_unit_id: PricingUnitId,
    pub is_removed: bool,
    pub meeting_date: AsezDate,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

impl FieldTolerance for EcAgenda {
    const TOLERATED: &'static [(&'static str, &'static str)] =
        &[("agenda_status_id", "status_id"), ("agenda_id", "id")];
}

/// EXPERIMENTAL
/// This exists as it obliges the upstream service to send the correct set of fields
/// when creating an `EcAgenda`.
#[derive(Debug, Clone, Deserialize)]
pub struct NewEcAgendaRep {
    pub id: i64,
    pub created_by: i32,
    pub meeting_date: AsezDate,
}

impl From<NewEcAgendaRep> for EcAgendaRep {
    fn from(n: NewEcAgendaRep) -> Self {
        Self {
            id: Some(n.id),
            created_by: Some(n.created_by),
            meeting_date: Some(n.meeting_date),
            ..Default::default()
        }
    }
}

impl AsRef<EcAgenda> for EcAgenda {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    DbEnum,
    Type,
    Ord,
    PartialOrd,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum EcAgendaStatus {
    #[db_default]
    Undefined = 0,
    Formed = 100,
    Sent = 200,
    ProtocolFormed = 300,
    Deleted = 400,
}

impl std::fmt::Display for EcAgendaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EcAgendaStatus::*;
        let msg = match self {
            Undefined => "Статус отсутствует",
            Formed => "Сформирована",
            Sent => "Отправлена",
            ProtocolFormed => "Сформирован Протокол",
            Deleted => "Удалена",
        };
        write!(f, "{}", msg)
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    DbEnum,
    Type,
    Ord,
    PartialOrd,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum CommissionKind {
    #[db_default]
    Undefined = 0,
    InPerson = 1,
    Correspondence = 2,
    NotRequired = 3,
}

impl CommissionKind {
    pub fn from_i64(value: i64) -> Option<Self> {
        match value {
            0 => Some(CommissionKind::Undefined),
            1 => Some(CommissionKind::InPerson),
            2 => Some(CommissionKind::Correspondence),
            3 => Some(CommissionKind::NotRequired),
            _ => None,
        }
    }
}

impl std::fmt::Display for CommissionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CommissionKind::*;
        let msg = match self {
            Undefined => "Форма СК отсутствует",
            InPerson => "Очная СК",
            Correspondence => "Заочная СК",
            NotRequired => "СК не требуется",
        };
        write!(f, "{}", msg)
    }
}
