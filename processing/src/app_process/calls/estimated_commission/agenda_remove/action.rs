/// Обработчик action/remove_remove (предзапрос по удалению Повестки) в Процессинге
/// Контракт: /rest/estimated_commission/v1/action/agenda_remove/
///
/// Если в Повестке есть связь с Протоколом или в Повестке указан
/// статус/status_id = 300/Сформирован Протокол или 400/Удалена, то формируем ошибку.
use crate::app_process::estimated_commission::agenda_remove::*;
use crate::common::{ProcessingCtx, Result};
use shared_essential::application::records::Recorder;
use shared_essential::domain::EcAgenda;
use shared_essential::presentation::dto::processing::{
    AgendaRemoveReq, AgendaRemoveResponse, AgendaRemoveResponseData,
};
use shared_essential::presentation::dto::response_request::Message;
use shared_essential::presentation::dto::response_request::{
    ApiResponse, Messages, Status,
};

use std::collections::HashMap;

use uuid::Uuid;

const ACTION_AGENDA_REMOVE: &str =
    "/rest/estimated_commission/v1/action/agenda_remove/";

/// # Описание
///
/// Процессинг удаления Повесток
///
/// # Аргументы
/// * `request` - [Тело запроса](`AgendaRemoveReq`) запрос
/// * `nest` - [`RabbitNest`] окружение
///
/// # Возвращает
/// * Ok([`AgendaRemoveResponse`]) - Массив данных по запрашиваемым Повесткам
pub(crate) async fn action_agenda_remove(
    request: AgendaRemoveReq,
    proc_ctx: ProcessingCtx,
) -> Result<AgendaRemoveResponse> {
    tracing::info!(
        kind = "get",
        "Процессинг: Удаление повесток ({get}): {req:?}\n",
        req = request,
        get = ACTION_AGENDA_REMOVE
    );

    let agenda_list: Vec<EcAgenda> =
        select_agenda_list(&request.item_list, &proc_ctx.db_pool).await?;

    let agenda_uuid_list: Vec<Uuid> = extract_uuids(&agenda_list);

    let related_protocols: Vec<RelatedProtocols> =
        select_related_protocols(agenda_uuid_list.as_slice(), &proc_ctx.db_pool)
            .await?;

    let related_protocols_map: HashMap<Uuid, String> =
        make_related_protocols_map(related_protocols);

    let errors: Vec<String> = collect_errors(&agenda_list, related_protocols_map);

    let result_list: Vec<EcAgenda> = if errors.is_empty() && !agenda_list.is_empty()
    {
        let mut recorder = proc_ctx
            .create_record_context()
            .with_user_id(request.user_id)
            .begin()
            .await?;
        let updated_agenda_list = agenda_remove(agenda_list, &mut recorder).await?;
        recorder.commit().await?;

        updated_agenda_list
    } else {
        agenda_list
    };

    Ok(make_response(result_list, errors))
}

fn collect_errors(
    agenda_list: &[EcAgenda],
    related_protocols_map: HashMap<Uuid, String>,
) -> Vec<String> {
    collect_status_errors(agenda_list)
        .chain(collect_protocol_errors(agenda_list, related_protocols_map))
        .collect()
}

// TODO: Check rows affected ?
async fn agenda_remove(
    mut agenda_list: Vec<EcAgenda>,
    recorder: &mut Recorder<'_>,
) -> Result<Vec<EcAgenda>> {
    agenda_list.iter_mut().for_each(|agenda| {
        agenda.status_id = EcAgendaStatus::Deleted;
        agenda.is_removed = true;
    });

    Ok(recorder
        .process_update(
            agenda_list,
            &["status_id", "is_removed"],
            &mut Messages::default(),
        )
        .await?)
}

fn make_response(
    agenda_list: Vec<EcAgenda>,
    errors: Vec<String>,
) -> AgendaRemoveResponse {
    let (status, _, messages) = if errors.is_empty() {
        make_ok_response(agenda_list)
    } else {
        make_error_response(errors)
    };

    ApiResponse {
        status,
        data: AgendaRemoveResponseData {
            status_id: EcAgendaStatus::Deleted,
        },
        objects: vec![],
        messages,
    }
}

fn make_ok_response(agenda_list: Vec<EcAgenda>) -> InternalResponseType<()> {
    let messages = Messages::from(vec![Message::success(
        "Повестка очной СК успешно удалена".to_string(),
    )
    .with_param_items(agenda_list)]);

    (Status::Ok, (), messages)
}

fn make_error_response(errors: Vec<String>) -> InternalResponseType<()> {
    let messages = Messages::from(
        errors.into_iter().map(Message::error).collect::<Vec<Message>>(),
    );

    (Status::Ok, (), messages)
}
