use crate::app_process::estimated_commission::agenda_remove::*;
/// Обработчик pre_request/remove_remove (предзапрос по удалению Повестки) в Процессинге
/// Контракт: /rest/estimated_commission/v1/action/agenda_remove/
///
/// Если в Повестке есть связь с Протоколом или в Повестке указан
/// статус/status_id = 300/Сформирован Протокол или 400/Удалена, то формируем ошибку.
use crate::common::Result;
use asez2_shared_db::db_item::AdaptorableIter;

use shared_essential::domain::{EcAgenda, EcAgendaRep};
use shared_essential::presentation::dto::processing::{
    PreRequestAgendaRemoveReq, PreRequestAgendaRemoveResponse,
    PreRequestAgendaRemoveResponseData,
};
use shared_essential::presentation::dto::response_request::Message;
use shared_essential::presentation::dto::response_request::{
    ApiResponse, Messages, Status,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

const PRE_REQUEST_AGENDA_REMOVE: &str =
    "/rest/estimated_commission/v1/pre_request/agenda_remove/";

const PRECHECK_RETURN_FIELDS: &[&str] = &[
    EcAgenda::uuid,
    "agenda_id",
    "agenda_status_id",
    EcAgenda::meeting_date,
    EcAgenda::pricing_organization_unit_id,
    EcAgenda::created_by,
];

/// # Описание
///
/// Процессинг предзапроса по удалению Повестки
///
/// # Аргументы
/// * `request` - [Тело запроса](`PreRequestAgendaRemoveReq`) запрос
/// * `nest` - [`RabbitNest`] окружение
///
/// # Возвращает
/// * Ok([`PreRequestAgendaRemoveResponse`]) - Массив данных по запрашиваемым Повесткам
pub(crate) async fn pre_request_agenda_remove(
    request: PreRequestAgendaRemoveReq,
    db_pool: Arc<PgPool>,
) -> Result<PreRequestAgendaRemoveResponse> {
    tracing::info!(
        kind = "get",
        "Процессинг: Список повесток, планируемых для удаления ({get}): {req:?}\n",
        req = request,
        get = PRE_REQUEST_AGENDA_REMOVE
    );

    let agenda_list: Vec<EcAgenda> =
        select_agenda_list(&request.item_list, &db_pool).await?;

    let agenda_uuid_list: Vec<Uuid> = extract_uuids(&agenda_list);

    let related_protocols: Vec<RelatedProtocols> =
        select_related_protocols(agenda_uuid_list.as_slice(), &db_pool).await?;

    let related_protocols_map: HashMap<Uuid, String> =
        make_related_protocols_map(related_protocols);

    let errors: Vec<String> = collect_status_errors(agenda_list.as_slice())
        .chain(collect_protocol_errors(
            agenda_list.as_slice(),
            related_protocols_map,
        ))
        .collect();

    Ok(make_response(agenda_list, errors))
}

fn make_response(
    agenda_list: Vec<EcAgenda>,
    errors: Vec<String>,
) -> PreRequestAgendaRemoveResponse {
    let (status, data, messages) = if errors.is_empty() {
        make_ok_response(agenda_list)
    } else {
        make_error_response(errors)
    };

    ApiResponse {
        status,
        data,
        objects: vec![],
        messages,
    }
}

fn make_ok_response(
    agenda_list: Vec<EcAgenda>,
) -> InternalResponseType<PreRequestAgendaRemoveResponseData> {
    let agenda_list_repr: Vec<EcAgendaRep> = agenda_list
        .into_iter()
        .adaptors_with_fields(PRECHECK_RETURN_FIELDS)
        .collect();

    let data = PreRequestAgendaRemoveResponseData {
        total: Some(agenda_list_repr.len()),
        item_list: agenda_list_repr,
    };

    (Status::Ok, data, Messages::default())
}

fn make_error_response(
    errors: Vec<String>,
) -> InternalResponseType<PreRequestAgendaRemoveResponseData> {
    let data = PreRequestAgendaRemoveResponseData {
        total: Some(0),
        item_list: vec![],
    };

    let messages = Messages::from(
        errors.into_iter().map(Message::error).collect::<Vec<Message>>(),
    );

    (Status::Ok, data, messages)
}
