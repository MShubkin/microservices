//! Бизнес логика по ручкам /rest/estimated_commission/v1/action/protocol_for_signature/
use std::sync::Arc;

use crate::common::{ProcessingCtx, Result};
use crate::presentation::business_messages::protocol::ProtocolSignMessage;

use asez2_shared_db::db_item::{selection::*, AdaptorableIter};
use asez2_shared_db::{DbItem, Value};
use shared_essential::application::records::Recorder;
use shared_essential::domain::tables::*;
use shared_essential::presentation::dto::processing::{
    PreSignProtocolReq, SignProtocolReq, SignProtocolResponseData,
};
use shared_essential::presentation::dto::response_request::*;

use sqlx::PgPool;
use uuid::Uuid;

const SEND_PROTOCOL_FOR_SIGNING_ACTION: &str = "v1/action/protocol_for_signature/";
const SEND_PROTOCOL_FOR_SIGNING_PRE_REQUEST: &str =
    "v1/pre_request/protocol_for_signature/";

const RETURN_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    "protocol_id",
    EcProtocol::registration_number,
    "protocol_status_id",
    EcProtocol::protocol_date,
];
const FETCH_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    EcProtocol::id,
    EcProtocol::registration_number,
    EcProtocol::pricing_organization_unit_id,
    EcProtocol::status_id,
    EcProtocol::protocol_date,
];

pub(crate) type SendProtocolForSigningResponse =
    ApiResponse<SignProtocolResponseData, ()>;

pub(crate) async fn send_protocol_for_signing(
    request: SignProtocolReq,
    proc_ctx: ProcessingCtx,
) -> Result<SendProtocolForSigningResponse> {
    tracing::info!(
        kind = "get",
        "Получен запрос на подписание Протокола СК ({action}): {req:?}\n",
        action = SEND_PROTOCOL_FOR_SIGNING_ACTION,
        req = request,
    );

    let uuids = request.ids.iter().map(|id| id.uuid).collect();
    let (protocols, mut messages) =
        pre_send_protocol_for_signing_inner(uuids, &proc_ctx.db_pool).await?;

    if !messages.is_empty() {
        return Ok(ApiResponse::default().with_messages(messages));
    }

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .with_status_notes(request.ids)
        .begin()
        .await?;

    let updated_protocols =
        update_protocols(protocols, &mut messages, &mut recorder).await?;

    recorder.commit().await?;

    ProtocolSignMessage::Success.checked_append(&mut messages, &updated_protocols);

    finalise_response(messages, updated_protocols)
}

pub(crate) async fn pre_send_protocol_for_signing(
    request: PreSignProtocolReq,
    db_pool: Arc<PgPool>,
) -> Result<SendProtocolForSigningResponse> {
    tracing::info!(
        kind = "get",
        "Получен предзапрос на подписание Протокола СК ({action}): {req:?}\n",
        action = SEND_PROTOCOL_FOR_SIGNING_PRE_REQUEST,
        req = request,
    );

    let uuids = request.into_iter().map(|id| id.uuid).collect();
    let (protocols, messages) =
        pre_send_protocol_for_signing_inner(uuids, &db_pool).await?;

    finalise_response(messages, protocols)
}

pub(crate) async fn pre_send_protocol_for_signing_inner(
    uuids: Vec<Uuid>,
    db_pool: &PgPool,
) -> Result<(Vec<EcProtocol>, Messages)> {
    let select = Select::with_fields(FETCH_FIELDS).add_expand_filter(
        "uuid",
        SelectionKind::In,
        uuids.into_iter().map(Value::from),
    );
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
        x.status_id = EcProtocolStatus::SignaturePending;
    });

    Ok(recorder.process_update(protocols, &["status_id"], messages).await?)
}

/// - Если в Протоколе указан статус/status_id ≠ 200/На согласовании, то формируем ошибку:
/// «Перевести Протокол <Системный номер Протокола> на статус "На подписании" невозможно.
/// Текущий статус Протокола "…"».
fn examine_protocols(protocols: &[EcProtocol], messages: &mut Messages) {
    use EcProtocolStatus::*;
    protocols
        .iter()
        .filter(|p| !matches!(p.status_id, AgreementPending))
        .for_each(|p| {
            messages.add_prepared_message(
                ProtocolSignMessage::InvalidProtocolStatus.singular(p),
            );
        });
}

fn finalise_response(
    messages: Messages,
    protocols: Vec<EcProtocol>,
) -> Result<SendProtocolForSigningResponse> {
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
