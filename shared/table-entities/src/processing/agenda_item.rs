//! This is used in processing for Estimates Commission related work:
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem, DbUpsert};
use asez2_shared_db::{impl_join_on, joined};
use fieldname_access::FieldnameAccess;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbItemExt;
use uuid::Uuid;

use super::{ContractAmendment, EcAgenda, Plan, RelAgendaProtocolItem};
use crate::maths::CurrencyValue;

impl_join_on!(EcAgendaItem:agenda_uuid => EcAgenda:uuid);
impl_join_on!(EcAgendaItem:agenda_uuid => EcAgenda:uuid, left);
impl_join_on!(EcAgendaItem:source_uuid => Plan:uuid);
impl_join_on!(EcAgendaItem:source_uuid => Plan:uuid, left);
impl_join_on!(EcAgendaItem:source_uuid => Plan:uuid, aggr);
impl_join_on!(EcAgendaItem:source_uuid => ContractAmendment:uuid, left);
impl_join_on!(EcAgendaItem:source_uuid => ContractAmendment:uuid, aggr);
impl_join_on!(EcAgendaItem:uuid => RelAgendaProtocolItem:agenda_item_uuid, left);
impl_join_on!(EcAgendaItem:uuid => RelAgendaProtocolItem:agenda_item_uuid, aggr);

// Требуется для получения повестки по каждому элементу
joined!(
    agenda_item: EcAgendaItem,
    plan: Plan[EcAgendaItem => Plan],
    agenda: EcAgenda[EcAgendaItem => EcAgenda],
);
joined!(
    item: EcAgendaItem,
    plan: Plan[EcAgendaItem => Plan, left],
    amendment: ContractAmendment[EcAgendaItem => ContractAmendment, left],
);

joined!(
    agenda_item: EcAgendaItem,
    agenda: EcAgenda[EcAgendaItem => EcAgenda],
);

joined!(
    agenda_item: EcAgendaItem,
    agenda: EcAgenda[EcAgendaItem => EcAgenda],
    item_agenda_protocol_rel: RelAgendaProtocolItem[EcAgendaItem => RelAgendaProtocolItem, left],
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
#[item_table = "agenda_item"]
#[item_aggr_insert]
pub struct EcAgendaItem {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub agenda_uuid: Uuid,
    pub source_uuid: Uuid,
    pub number: i64,
    pub is_registered_by_d647: bool,
    pub is_excluded: bool,
    pub is_removed: bool,
    pub reviewed_at: Option<AsezTimestamp>,
    pub sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_sum_excluded_vat: Option<CurrencyValue>,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

// impl DbUpsert for EcAgendaItem {}
