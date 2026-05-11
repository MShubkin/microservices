//! This is used in processing for Estimates Commission related work:
use std::fmt::Display;

use asez2_shared_db::db_item::{
    AsezDate, AsezTimestamp, DbItemExt, DbUpsert, FieldTolerance,
};
use asez2_shared_db::{impl_join_on, joined};
use asez2_shared_db::{DbAdaptor, DbItem};
use fieldname_access::FieldnameAccess;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::prelude::Type;
use uuid::Uuid;

use super::*;
use crate::PricingUnitId;

impl_join_on!(EcProtocol:uuid => EcProtocolItem:protocol_uuid, aggr);
impl_join_on!(EcProtocol:uuid => EcPartner:protocol_agenda_uuid, aggr);
impl_join_on!(EcProtocol:uuid => Attachment:object_uuid, aggr);
impl_join_on!(EcProtocol:uuid => RelAgendaProtocolItem:protocol_uuid, aggr);

// Структура сложная но позволяет нам всё за одн раз вытянуть без пара.
joined!(
    protocol: EcProtocol,
    items: EcProtocolItem[EcProtocol => EcProtocolItem, aggr],
    plans: Plan[EcProtocolItem => Plan, aggr],
    amendments: ContractAmendment[EcProtocolItem => ContractAmendment, aggr]
);

joined!(
    protocol: EcProtocol,
    protocol_items: EcProtocolItem[EcProtocol => EcProtocolItem, aggr],

    plans: Plan[EcProtocolItem => Plan, aggr],
    amendments: ContractAmendment[EcProtocolItem => ContractAmendment, aggr],

    rel_agenda_protocol: RelAgendaProtocol[EcProtocol => RelAgendaProtocol],

    agenda: EcAgenda[RelAgendaProtocol => EcAgenda],
    agenda_items: EcAgendaItem[EcAgenda => EcAgendaItem, aggr]
);

joined!(
    protocol: EcProtocol,
    rels: RelAgendaProtocolItem[EcProtocol => RelAgendaProtocolItem, aggr],
);

joined!(
    protocol: EcProtocol,
    protocol_items: EcProtocolItem[EcProtocol => EcProtocolItem, aggr],
);

joined!(
    !ProtocolDetails,
    protocol: EcProtocol,

    items: EcProtocolItem[EcProtocol => EcProtocolItem, aggr],
    plans: Plan[EcProtocolItem => Plan, aggr],
    amendments: ContractAmendment[EcProtocolItem  => ContractAmendment, aggr],

    partner_list: EcPartner[EcProtocol => EcPartner, aggr],
    attachment_list: Attachment[EcProtocol => Attachment, aggr],
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
#[item_table = "protocol"]
#[item_skip_field_tolerance]
#[item_manually_activate_fields]
#[item_aggr_insert]
pub struct EcProtocol {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    #[adaptor_field_duplicate = "protocol_id"]
    pub id: i64,
    pub protocol_type_id: ProtocolType,
    pub registration_number: Option<String>,
    #[adaptor_field_duplicate = "protocol_status_id"]
    #[item_field_activate_with = "EcProtocolStatus::Formed"]
    pub status_id: EcProtocolStatus,
    pub pricing_organization_unit_id: PricingUnitId,
    pub is_secret: bool,
    pub is_removed: bool,
    pub protocol_date: AsezDate,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

impl FieldTolerance for EcProtocol {
    const TOLERATED: &'static [(&'static str, &'static str)] =
        &[("protocol_id", "id"), ("protocol_status_id", "status_id")];
}

impl EcProtocol {
    /// We set the date when we insert the item. The other fields
    /// MUST be set beforehand by the user or we MUST crash.
    fn activate_fields_manually(&mut self) {
        if self.changed_by == i32::default() {
            self.changed_by = self.created_by.to_owned();
        }
    }
}

impl AsRef<EcProtocol> for EcProtocol {
    fn as_ref(&self) -> &EcProtocol {
        self
    }
}

/// EXPERIMENTAL
/// This exists as it obliges the upstream service to send the correct set of fields
/// when creating an `EcProtocol`.
#[derive(Debug, Clone, Deserialize)]
pub struct NewEcProtocolRep {
    pub id: i64,
    /// TODO: meeting_date and protocol_type are interdependent, so we should think
    /// about removing protocol_type.
    pub protocol_type_id: ProtocolType,
    pub is_secret: bool,
    pub created_by: i32,
    pub registration_number: Option<String>,
    pub protocol_date: AsezDate,
}

impl From<NewEcProtocolRep> for EcProtocolRep {
    fn from(n: NewEcProtocolRep) -> Self {
        Self {
            id: Some(n.id),
            protocol_type_id: Some(n.protocol_type_id),
            is_secret: Some(n.is_secret),
            registration_number: Some(n.registration_number),
            created_by: Some(n.created_by),
            protocol_date: Some(n.protocol_date),
            ..Default::default()
        }
    }
}

/// Тип протокола
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    Ord,
    PartialOrd,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum EcProtocolStatus {
    /// Unknown (this is a hack to allow into)
    #[db_default]
    Undefined = 0,
    /// Сформирован
    Formed = 100,
    /// На согласовании
    AgreementPending = 200,
    /// На подписании
    SignaturePending = 300,
    /// Утвержден
    Confirmed = 400,
    /// Удален
    Deleted = 500,
}

/// Тип протокола
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    DbEnum,
)]
#[repr(i16)]
#[serde(into = "i16", from = "i16")]
pub enum ProtocolType {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    /// Протокол очного заседания СК
    InPersonMeeting = 1,
    /// Протокол заочного заседания СК
    CorrespondenceMeeting = 2,
}

impl Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::InPersonMeeting => "Протокол очного заседания СК",
            Self::CorrespondenceMeeting => "Протокол заочного заседания СК",
            Self::Undefined => "Не установлено",
        };
        write!(f, "{}", str)
    }
}

impl Display for EcProtocolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EcProtocolStatus::Formed => "Сформирован",
            EcProtocolStatus::AgreementPending => "На согласовании",
            EcProtocolStatus::SignaturePending => "На подписании",
            EcProtocolStatus::Confirmed => "Утвержден",
            EcProtocolStatus::Deleted => "Удален",
            EcProtocolStatus::Undefined => "не указан",
        };
        write!(f, "{}", str)
    }
}
