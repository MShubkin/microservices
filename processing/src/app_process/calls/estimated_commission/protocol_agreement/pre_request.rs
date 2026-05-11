use std::sync::Arc;

use crate::app_process::estimated_commission::protocol_agreement::*;
/// Обработчик pre_request/protocol_agreement (Предзапрос для отправки протоколов на согласование) в Процессинге
/// Контракт: /rest/estimated_commission/v1/pre_request/protocol_agreement/
///
/// Если в Протоколе указан статус/status_id = 200/На согласовании или 400/Утвержден или 500/Удален, то формируем ошибку.
/// Если в Протоколе указан статус/status_id = 300/На подписании, то формируем предупреждение.
use crate::common::Result;
use asez2_shared_db::db_item::AdaptorableIter;

use shared_essential::domain::{EcProtocol, EcProtocolRep};
use shared_essential::presentation::dto::processing::{
    PreProtocolAgreementReq, PreProtocolAgreementResponse,
};
use shared_essential::presentation::dto::response_request::Message;
use shared_essential::presentation::dto::response_request::{
    ApiResponse, Messages, Status,
};

const PRE_REQUEST_PROTOCOL_AGREEMENT: &str =
    "/rest/estimated_commission/v1/pre_request/protocol_agreement/";

/// # Описание
///
/// Предзапрос для отправки протоколов на согласование
///
/// # Аргументы
/// * `request` - [Тело запроса](`PreProtocolAgreementDto`) запрос
/// * `nest` - [`RabbitNest`] окружение
///
/// # Возвращает
/// * Ok([`PreProtocolAgreementResponse`]) - Массив данных запрашиваемых Протоколов
pub(crate) async fn pre_request_protocol_agreement(
    request: PreProtocolAgreementReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreProtocolAgreementResponse, ()>> {
    tracing::info!(
        kind = "get",
        "Процессинг: Предзапрос для отправки протоколов на согласование ({get}): {req:?}\n",
        req = request,
        get = PRE_REQUEST_PROTOCOL_AGREEMENT
    );

    let uuids: Vec<Uuid> = request.item_list.iter().map(|x| x.uuid).collect();

    let protocol_list: Vec<EcProtocol> =
        select_protocol_list(uuids.as_slice(), request.protocol_type_id, &db_pool)
            .await?;

    let errors: Vec<Message> =
        collect_status_errors(protocol_list.as_slice()).collect();

    Ok(make_response(protocol_list, errors))
}

fn make_response(
    protocol_list: Vec<EcProtocol>,
    errors: Vec<Message>,
) -> ApiResponse<PreProtocolAgreementResponse, ()> {
    let (status, data, messages): InternalResponseType<
        PreProtocolAgreementResponse,
    > = if errors.is_empty() {
        let warnings: Vec<Message> =
            collect_status_warnings(protocol_list.as_slice()).collect();
        make_ok_response(protocol_list, warnings)
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
    protocol_list: Vec<EcProtocol>,
    warnings: Vec<Message>,
) -> InternalResponseType<PreProtocolAgreementResponse> {
    let protocol_list: Vec<EcProtocolRep> =
        protocol_list.into_iter().adaptors_with_fields(RETURN_FIELDS).collect();

    let data = PreProtocolAgreementResponse {
        total: protocol_list.len() as u32,
        item_list: protocol_list,
    };

    (Status::Ok, data, Messages::from(warnings))
}

fn make_error_response(
    errors: Vec<Message>,
) -> InternalResponseType<PreProtocolAgreementResponse> {
    let data = PreProtocolAgreementResponse {
        total: 0,
        item_list: vec![],
    };

    (Status::Ok, data, Messages::from(errors))
}
