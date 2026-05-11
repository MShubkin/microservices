//! Бизнес логика по ручкам "/rest/estimated_commission/v1/(pre_request/action)/protocol_remove/".
use std::sync::Arc;

use ahash::AHashMap;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{selection::*, AdaptorableIter, DbItemDel},
    DbItem,
};
use shared_essential::{
    application::records::Recorder,
    domain::tables::*,
    presentation::dto::{
        processing::{
            PreRemoveProtocolReq, RemoveProtocolReq, RemoveProtocolResponseData,
        },
        response_request::*,
    },
};

use crate::{
    common::{ProcessingCtx, Result},
    presentation::business_messages::protocol::ProtocolRemoveMessage,
};

const REMOVE_PROTOCOL_ACTION: &str = "v1/action/protocol_remove/";
const REMOVE_PROTOCOL_PRE_REQUEST: &str = "v1/pre_request/protocol_remove/";

const REQUEST_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    "protocol_id",
    EcProtocol::registration_number,
    "protocol_status_id",
    EcProtocol::protocol_date,
];
const RETURN_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    "protocol_id",
    EcProtocol::registration_number,
    "protocol_status_id",
    EcProtocol::protocol_date,
];

pub(crate) type ApiRemoveProtocolResponse =
    ApiResponse<RemoveProtocolResponseData, ()>;

pub(crate) async fn remove_protocol(
    request: RemoveProtocolReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiRemoveProtocolResponse> {
    tracing::info!(
        kind = "get",
        "Запрос на удаление Протокола СК ({what}): {req:?}\n",
        what = REMOVE_PROTOCOL_ACTION,
        req = request,
    );

    let RemoveProtocolReq {
        protocol_type_id,
        user_id: _,
        item_list,
    } = request;

    let (protocols, mut messages) = pre_remove_protocol_inner(
        item_list.iter().map(|i| i.uuid),
        &proc_ctx.db_pool,
    )
    .await?;

    if messages.is_error() {
        return Ok(ApiResponse::default().with_messages(messages));
    }
    messages.clear();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .with_status_notes(item_list.clone())
        .begin()
        .await?;

    let updated_protocols =
        update_protocols(protocols, &mut messages, &mut recorder).await?;

    if protocol_type_id == ProtocolType::InPersonMeeting {
        let agenda_protocol_rels =
            delete_protocol_relations(&updated_protocols, recorder.tx()).await?;
        update_agendas(agenda_protocol_rels, &mut messages, &mut recorder).await?;
    }

    recorder.commit().await?;

    ProtocolRemoveMessage::Success(protocol_type_id)
        .checked_append(&mut messages, &updated_protocols);

    finalise(updated_protocols, messages)
}

pub(crate) async fn pre_remove_protocol(
    request: PreRemoveProtocolReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiRemoveProtocolResponse> {
    tracing::info!(
        kind = "get",
        "Предзапрос на удаление Протокола СК ({what}): {req:?}\n",
        what = REMOVE_PROTOCOL_PRE_REQUEST,
        req = request,
    );

    let PreRemoveProtocolReq { item_list, .. } = request;

    let (protocols, messages) =
        pre_remove_protocol_inner(item_list.iter().map(|i| i.uuid), &db_pool)
            .await?;

    finalise(protocols, messages)
}

pub(crate) async fn pre_remove_protocol_inner<I>(
    protocol_uuids: I,
    db_pool: &PgPool,
) -> Result<(Vec<EcProtocol>, Messages)>
where
    I: IntoIterator<Item = Uuid>,
{
    let select = Select::with_fields(REQUEST_FIELDS)
        .in_any(EcProtocol::uuid, protocol_uuids);
    let protocols = EcProtocol::select(&select, db_pool).await?;

    let mut messages = Messages::default();

    examine_protocols(&protocols, &mut messages);

    Ok((protocols, messages))
}

async fn update_protocols(
    mut protocols: Vec<EcProtocol>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<Vec<EcProtocol>> {
    protocols.iter_mut().for_each(|x| {
        x.is_removed = true;
        x.status_id = EcProtocolStatus::Deleted;
    });

    Ok(recorder
        .process_update(
            protocols,
            &[EcProtocol::is_removed, EcProtocol::status_id],
            messages,
        )
        .await?)
}

async fn delete_protocol_relations(
    protocols: &[EcProtocol],
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<RelAgendaProtocol>> {
    let protocol_rel_filter = Filter::in_any(
        RelAgendaProtocolItem::protocol_uuid,
        protocols.iter().map(|p| p.uuid),
    )
    .into();

    RelAgendaProtocolItem::delete_returning(&protocol_rel_filter, &mut *tx).await?;
    let agenda_protocol_rels =
        RelAgendaProtocol::delete_returning(&protocol_rel_filter, &mut *tx).await?;

    Ok(agenda_protocol_rels)
}

async fn update_agendas(
    agenda_protocol_rels: Vec<RelAgendaProtocol>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let history_select = Select::full::<StatusHistory>()
        .in_any(
            StatusHistory::object_uuid,
            agenda_protocol_rels.iter().map(|i| i.agenda_uuid),
        )
        .ne(StatusHistory::status_id, EcAgendaStatus::ProtocolFormed)
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid]);

    let histories = StatusHistory::select(&history_select, recorder.tx())
        .await?
        .into_iter()
        .map(|history_x| (history_x.object_uuid, history_x))
        .collect::<AHashMap<_, _>>();

    let agendas_to_update = agenda_protocol_rels
        .into_iter()
        .map(|rel| EcAgenda {
            uuid: rel.agenda_uuid,
            ..Default::default()
        })
        .filter_map(|mut agenda| {
            histories.get(&agenda.uuid).map(|history| {
                agenda.status_id = history.status_id.into();
                agenda
            })
        })
        .collect::<Vec<_>>();

    recorder
        .process_update(agendas_to_update, &[EcAgenda::status_id], messages)
        .await?;

    Ok(())
}

fn examine_protocols(protocols: &[EcProtocol], messages: &mut Messages) {
    let initial_message_len = messages.messages.len();

    protocols
        .iter()
        .filter(|p| {
            matches!(
                p.status_id,
                EcProtocolStatus::Confirmed | EcProtocolStatus::Deleted
            )
        })
        .for_each(|p| {
            messages.add_prepared_message(
                ProtocolRemoveMessage::InvalidProtocolStatus.singular(p),
            )
        });

    // Значит что ошибок не было
    if initial_message_len == messages.messages.len() {
        protocols
            .iter()
            .filter(|p| {
                matches!(
                    p.status_id,
                    EcProtocolStatus::AgreementPending
                        | EcProtocolStatus::SignaturePending
                )
            })
            .for_each(|p| {
                messages.add_prepared_message(
                    ProtocolRemoveMessage::ProtocolStatusWarn.singular(p),
                )
            })
    }
}

fn finalise(
    protocols: Vec<EcProtocol>,
    messages: Messages,
) -> Result<ApiRemoveProtocolResponse> {
    let data = if messages.is_error() {
        Vec::new()
    } else {
        protocols
            .into_iter()
            .adaptors_with_fields(RETURN_FIELDS)
            .collect::<Vec<_>>()
    };

    Ok((data, messages).into())
}
