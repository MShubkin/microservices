use asez2_shared_db::db_item::{Select, SelectionKind};
use asez2_shared_db::result::SharedDbError;
use asez2_shared_db::{DbItem, Value};
use shared_essential::common::maps::map_2;
use shared_essential::domain::{EcAgenda, EcAgendaStatus};
use shared_essential::presentation::dto::general::ObjectIdentifier;
use shared_essential::presentation::dto::response_request::{Messages, Status};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub(crate) mod action;
pub(crate) mod pre_request;

const PRECHECK_FIELDS: &[&str] = &[
    EcAgenda::uuid,
    EcAgenda::id,
    EcAgenda::status_id,
    EcAgenda::meeting_date,
    EcAgenda::pricing_organization_unit_id,
    EcAgenda::created_by,
];

/// Common "Agenda Remove" (pre_request/action) functions
#[derive(Debug)]
pub(crate) struct RelatedProtocols {
    agenda_uuid: Option<Uuid>,
    registration_number: Option<String>,
}

pub(crate) async fn select_related_protocols(
    agenda_uuid_list: &[Uuid],
    db_pool: &PgPool,
) -> Result<Vec<RelatedProtocols>, sqlx::Error> {
    sqlx::query_as!(
        RelatedProtocols,
        "SELECT apr.agenda_uuid, ecp.registration_number \
         FROM agenda_protocol_relation apr \
         INNER JOIN protocol ecp ON ecp.uuid=apr.protocol_uuid \
         WHERE apr.agenda_uuid = ANY($1)",
        agenda_uuid_list
    )
    .fetch_all(db_pool)
    .await
}

pub(crate) fn extract_uuids(agenda_list: &[EcAgenda]) -> Vec<Uuid> {
    agenda_list.iter().map(|item| item.uuid).collect()
}

pub(crate) fn make_related_protocols_map(
    related_protocols: Vec<RelatedProtocols>,
) -> HashMap<Uuid, String> {
    related_protocols
        .into_iter()
        .flat_map(|item| {
            map_2(item.agenda_uuid, item.registration_number, |a, b| (a, b))
        })
        .collect()
}

pub(crate) async fn select_agenda_list(
    request: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<Vec<EcAgenda>, SharedDbError> {
    let uuid_list = request.iter().map(|item| Value::from(item.uuid));
    let select = Select::with_fields(PRECHECK_FIELDS).add_expand_filter(
        "uuid",
        SelectionKind::In,
        uuid_list,
    );
    EcAgenda::select(&select, db_pool).await
}

pub(crate) type InternalResponseType<T> = (Status, T, Messages);

pub(crate) fn collect_status_errors(
    agenda_list_repr: &[EcAgenda],
) -> impl Iterator<Item = String> + '_ {
    agenda_list_repr.iter().filter_map(|agenda| {
        match agenda.status_id {
            EcAgendaStatus::ProtocolFormed | EcAgendaStatus::Deleted =>
                Some(format!(r#"Выполнить удаление Повестки {} на {} невозможно. Повестка находится на статусе "{}""#,
                             agenda.id, agenda.meeting_date, agenda.status_id)),
            _ => None,
        }
    })
}

pub(crate) fn collect_protocol_errors(
    agenda_list_repr: &[EcAgenda],
    related_protocols_map: HashMap<Uuid, String>,
) -> impl Iterator<Item = String> + '_ {
    agenda_list_repr.iter().filter_map(move |agenda| {
        related_protocols_map.get(&agenda.uuid).map(|registration_number| {
            format!("Выполнить удаление Повестки {} на {} невозможно. Повестка включена в Протокол {}",
                    agenda.id, agenda.meeting_date, registration_number)
        })
    })
}
