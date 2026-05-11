use std::sync::Arc;

use ahash::AHashSet;
use itertools::Itertools;
use sqlx::PgPool;

use asez2_shared_db::db_item::{joined::JoinTo, selection::*, AdaptorableIter};
use shared_essential::{
    application::records::Recorder,
    domain::{
        processing::agenda::{
            JoinedEcAgendaEcAgendaItem as AgendaWithItems,
            JoinedEcAgendaEcAgendaItemSelector as AgendaWithItemsSelector,
        },
        EcAgenda, EcAgendaItem, EcAgendaStatus,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{
            AgendaSendReq, AgendaSendResponseData, PreAgendaSendReq,
            PreAgendaSendResponseData,
        },
        response_request::{ApiResponse, BusinessMessage, Messages},
    },
};

use crate::{
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::agenda::AgendaSendMessage,
};

const PRE_AGENDA_SEND: &str =
    "/rest/estimated_commission/v1/pre_request/agenda_send/";
const AGENDA_SEND: &str = "/rest/estimated_commission/v1/action/agenda_send/";

const PRECHECK_FIELDS: &[&str] = &[
    EcAgenda::created_by,
    EcAgenda::id,
    EcAgenda::meeting_date,
    EcAgenda::pricing_organization_unit_id,
    EcAgenda::status_id,
    "agenda_id",
    "agenda_status_id",
    EcAgenda::uuid,
];
const PRECHECK_RETURN_FIELDS: &[&str] = &[
    EcAgenda::created_by,
    EcAgenda::meeting_date,
    EcAgenda::pricing_organization_unit_id,
    "agenda_id",
    "agenda_status_id",
    EcAgenda::uuid,
];

/// # Описание
///
/// Процессинг предзапроса по отправке Повестки
///
/// # Аргументы
/// * `request` - [Тело запроса](`PreAgendaSendReq`) запрос
/// * `nest` - [`Arc<PgPool>`] окружение
///
/// # Возвращает
/// * Ok([`ApiResponse<PreAgendaSendResponseData, ()>`]) - Массив данных по запрашиваемым Повесткам
pub(crate) async fn pre_agenda_send(
    request: PreAgendaSendReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreAgendaSendResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Процессинг: Список повесток, планируемых для отправки ({get}): {req:?}\n",
        req = request,
        get = PRE_AGENDA_SEND
    );

    let (agendas, messages) =
        pre_agenda_send_inner(&request.item_list, &db_pool).await?;

    finalise(messages, agendas)
}

pub(crate) async fn agenda_send(
    request: AgendaSendReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<AgendaSendResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Процессинг: Список повесток, планируемых для отправки ({get}): {req:?}\n",
        req = request,
        get = AGENDA_SEND
    );

    let (agendas, mut messages) =
        pre_agenda_send_inner(&request.item_list, &proc_ctx.db_pool).await?;

    if messages.is_error() {
        return Ok((
            AgendaSendResponseData {
                status_id: EcAgendaStatus::Undefined,
            },
            messages,
        )
            .into());
    }

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .begin()
        .await?;

    let updated_agendas =
        update_agenda(agendas, &mut messages, &mut recorder).await?;

    recorder.commit().await?;

    AgendaSendMessage::Success.checked_append(&mut messages, &updated_agendas);

    Ok((
        AgendaSendResponseData {
            status_id: EcAgendaStatus::Sent,
        },
        messages,
    )
        .into())
}

async fn pre_agenda_send_inner(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<AgendaWithItems>, Messages)> {
    let agenda_select = Select::with_fields(PRECHECK_FIELDS)
        .in_any(EcAgenda::uuid, item_list.iter().map(|item| item.uuid));
    let agenda_item_select = Select::full::<EcAgendaItem>()
        .eq(EcAgendaItem::is_removed, false)
        .eq(EcAgendaItem::is_excluded, false);

    let agendas_with_items = AgendaWithItemsSelector::new(agenda_select)
        .set_agenda_items(
            EcAgendaItem::join_default().selecting(agenda_item_select),
        )
        .get(db_pool)
        .await?;

    if agendas_with_items.len() != item_list.len() {
        let checker = agendas_with_items
            .iter()
            .map(|i| i.agenda.uuid)
            .collect::<AHashSet<_>>();
        let missing = item_list
            .iter()
            .filter(|i| !checker.contains(&i.uuid))
            .map(|i| i.id.to_string())
            .join(", ");

        return Err(ProcessingError::GetItemList(format!(
            "Повестки СК с идентификаторами {} не найдены",
            missing
        )));
    }

    let mut messages = Messages::default();

    examine_agendas(&agendas_with_items, &mut messages);

    Ok((agendas_with_items, messages))
}

/// Предупреждение:
/// Если в Повестке указан статус/status_id = 200/Отправлена или 400/Удалена,
/// то формируем ошибку: «Выполнить отправку Повестки <Системный номер Повестки>
/// на <дата заседания> невозможно. Повестка находится на статусе "…"».
fn examine_agendas(
    agendas_with_items: &[AgendaWithItems],
    messages: &mut Messages,
) {
    for agenda_with_items in agendas_with_items {
        let AgendaWithItems {
            agenda,
            agenda_items,
        } = agenda_with_items;

        if agenda_items.is_empty() {
            messages.add_prepared_message(
                AgendaSendMessage::EmptyAgenda.singular(agenda),
            );
            continue;
        }

        if agenda.status_id != EcAgendaStatus::Formed {
            messages.add_prepared_message(
                AgendaSendMessage::InvalidAgendaStatus.singular(agenda),
            );
        }
    }
}

async fn update_agenda(
    agendas_with_items: Vec<AgendaWithItems>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<Vec<EcAgenda>> {
    let agendas = agendas_with_items
        .into_iter()
        .map(|mut agenda_with_items| {
            agenda_with_items.agenda.status_id = EcAgendaStatus::Sent;
            agenda_with_items.agenda
        })
        .collect::<Vec<_>>();

    Ok(recorder
        .process_update(agendas, &[EcAgenda::status_id], messages)
        .await?)
}

fn finalise(
    messages: Messages,
    data: Vec<AgendaWithItems>,
) -> Result<ApiResponse<PreAgendaSendResponseData, ()>> {
    let data = if messages.is_error() {
        Vec::new()
    } else {
        data.into_iter()
            .map(|x| x.agenda)
            .adaptors_with_fields(PRECHECK_RETURN_FIELDS)
            .collect::<Vec<_>>()
    };

    Ok((data, messages).into())
}
