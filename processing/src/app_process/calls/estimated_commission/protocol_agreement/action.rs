use crate::app_process::estimated_commission::protocol_agreement::*;
/// Обработчик action/protocol_agreement (Отправка протоколов на согласование) в Процессинге
/// Контракт: /rest/estimated_commission/v1/action/protocol_agreement/
///
/// Если в Протоколе указан статус/status_id = 200/На согласовании или 400/Утвержден или 500/Удален, то формируем ошибку.
/// Если в Протоколе указан статус/status_id = 300/На подписании, то формируем предупреждение.
use crate::common::{ProcessingCtx, Result};
use shared_essential::application::records::Recorder;
use shared_essential::domain::{EcProtocol, EcProtocolStatus};
use shared_essential::presentation::dto::processing::ProtocolAgreementReq;
use shared_essential::presentation::dto::response_request::Message;
use shared_essential::presentation::dto::response_request::{
    ApiResponse, Messages, Status,
};

use uuid::Uuid;

const ACTION_PROTOCOL_AGREEMENT: &str =
    "/rest/estimated_commission/v1/action/protocol_agreement/";

// Обрабатываем только записи со статусами EcProtocolStatus [Formed, SignaturePending]
const SOURCE_STATUSES: &[i16] = &[
    EcProtocolStatus::Formed as i16,
    EcProtocolStatus::SignaturePending as i16,
];

// Переводим на статус EcProtocolStatus::AgreementPending
const PROTOCOL_AGREEMENT_PENDING_STATUS: EcProtocolStatus =
    EcProtocolStatus::AgreementPending;

/// # Описание
///
/// Отправка протоколов на согласование
///
/// # Аргументы
/// * `request` - [Тело запроса](`ProtocolAgreementDto`) запрос
/// * `nest` - [`RabbitNest`] окружение
///
/// # Возвращает
/// * Ok() - Массив Протоколов в messages
pub(crate) async fn action_protocol_agreement(
    request: ProtocolAgreementReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<(), ()>> {
    tracing::info!(
        kind = "get",
        "Процессинг: Отправка протоколов на согласование ({get}): {req:?}\n",
        req = request,
        get = ACTION_PROTOCOL_AGREEMENT
    );

    let protocol_list: Vec<EcProtocol> = select_protocol_list(
        request
            .item_list
            .iter()
            .map(|item| item.uuid)
            .collect::<Vec<Uuid>>()
            .as_slice(),
        request.protocol_type_id,
        &proc_ctx.db_pool,
    )
    .await?;

    let errors: Vec<Message> =
        collect_status_errors(protocol_list.as_slice()).collect();

    let (status, messages): (Status, Messages) = if errors.is_empty() {
        let updated_protocol_list: Vec<EcProtocol> =
            prepare_protocols(protocol_list);

        let mut messages = Messages::default();

        let mut recorder = proc_ctx
            .create_record_context()
            .with_user_id(request.user_id)
            .with_status_notes(request.item_list.clone())
            .begin()
            .await?;

        let processed_protocol_list: Vec<EcProtocol> = process_update_status(
            updated_protocol_list,
            &mut messages,
            &mut recorder,
        )
        .await?;

        recorder.commit().await?;

        let mut warnings: Vec<Message> =
            collect_status_warnings(processed_protocol_list.as_slice()).collect();

        let success_message: Message =
            make_success_message(processed_protocol_list.as_slice());

        messages.messages.append(&mut warnings);
        messages.messages.push(success_message);

        (Status::Ok, messages)
    } else {
        (Status::Error, Messages::from(errors))
    };

    Ok(ApiResponse {
        status,
        data: (),
        objects: vec![],
        messages,
    })
}

fn prepare_protocols(protocol_list: Vec<EcProtocol>) -> Vec<EcProtocol> {
    protocol_list
        .into_iter()
        .filter_map(|mut protocol| {
            if SOURCE_STATUSES.contains(&(protocol.status_id as i16)) {
                protocol.status_id = PROTOCOL_AGREEMENT_PENDING_STATUS;
                Some(protocol)
            } else {
                None
            }
        })
        .collect()
}

async fn process_update_status(
    updated_protocol_list: Vec<EcProtocol>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<Vec<EcProtocol>> {
    Ok(recorder
        .process_update(updated_protocol_list, &["status_id"], messages)
        .await?)
}

fn make_success_message(protocol_list: &[EcProtocol]) -> Message {
    let text = if protocol_list.len() == 1 {
        format!(
            "Вы отправили на согласование Протокол {} очной СК.",
            protocol_list[0].id
        )
    } else {
        format!(
            "Вы отправили на согласование {} Протоколов очной СК.",
            protocol_list.len()
        )
    };

    Message::success(text).with_param_items(protocol_list)
}
