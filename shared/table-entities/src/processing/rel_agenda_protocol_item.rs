//! This is used in processing for Estimates Commission related work:
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem, DbItemDel};
use asez2_shared_db::{impl_join_on, joined};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ContractAmendment, EcAgenda, EcAgendaItem, EcProtocol, EcProtocolItem, Plan,
};

impl_join_on!(RelAgendaProtocolItem:agenda_item_uuid => EcAgendaItem:uuid, aggr);
impl_join_on!(RelAgendaProtocolItem:agenda_item_uuid => EcAgendaItem:uuid);
impl_join_on!(RelAgendaProtocolItem:protocol_item_uuid => EcProtocolItem:uuid, aggr);
impl_join_on!(RelAgendaProtocolItem:agenda_uuid => EcAgenda:uuid, aggr);
impl_join_on!(RelAgendaProtocolItem:protocol_uuid => EcProtocol:uuid, aggr);
impl_join_on!(RelAgendaProtocolItem:protocol_uuid => EcProtocol:uuid);

joined!(
    rel: RelAgendaProtocolItem,
    // Only for specific us, ie we must already know what we are looking for
    // since the relationship is many to many.
    agenda_item: EcAgendaItem[RelAgendaProtocolItem => EcAgendaItem]
);

joined!(
    rel: RelAgendaProtocolItem,
    agenda_item: EcAgendaItem[RelAgendaProtocolItem => EcAgendaItem],
    agenda: EcAgenda[EcAgendaItem => EcAgenda],
    plan: Plan[EcAgendaItem => Plan, left],
    amendment: ContractAmendment[EcAgendaItem => ContractAmendment, left],
);

joined!(
    rel: RelAgendaProtocolItem,
    protocol: EcProtocol[RelAgendaProtocolItem => EcProtocol]
);

#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "item_relation_agenda_protocol"]
#[item_aggr_insert]
pub struct RelAgendaProtocolItem {
    // NB: For now we must treat one field as a pkey for updates
    // This is probably not correct in this case.
    #[item_field_pkey]
    pub protocol_item_uuid: Uuid,
    pub protocol_uuid: Uuid,
    #[item_field_pkey]
    pub agenda_item_uuid: Uuid,
    pub agenda_uuid: Uuid,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
}

impl DbItemDel for RelAgendaProtocolItem {}
