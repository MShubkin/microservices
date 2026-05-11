//! This is used in processing for Estimates Commission related work:
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem, DbItemDel};
use asez2_shared_db::{impl_join_on, joined};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{EcAgenda, EcProtocol, RelAgendaProtocolItem};

impl_join_on!(EcAgenda:uuid => RelAgendaProtocol:agenda_uuid, aggr);
impl_join_on!(EcProtocol:uuid => RelAgendaProtocol:protocol_uuid);

impl_join_on!(RelAgendaProtocol:agenda_uuid => EcAgenda:uuid);
impl_join_on!(RelAgendaProtocol:protocol_uuid => EcProtocol:uuid, aggr);

impl_join_on!(RelAgendaProtocol:agenda_uuid => RelAgendaProtocolItem:agenda_uuid, aggr);

joined!(
    !JoinedAgendaProtocolRelsItems,
    rel: RelAgendaProtocol,
    rel_item: RelAgendaProtocolItem[RelAgendaProtocol => RelAgendaProtocolItem, aggr]
);

#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "agenda_protocol_relation"]
#[item_aggr_insert]
pub struct RelAgendaProtocol {
    #[item_field_pkey]
    pub protocol_uuid: Uuid,
    #[item_field_pkey]
    pub agenda_uuid: Uuid,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
}

impl DbItemDel for RelAgendaProtocol {}
