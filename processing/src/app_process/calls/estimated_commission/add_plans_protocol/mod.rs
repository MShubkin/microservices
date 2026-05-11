use ahash::AHashMap;
use shared_essential::{
    domain::{
        tables::processing::agenda::JoinedEcAgendaEcAgendaItemPlanContractAmendment as JoinedAgenda,
        PlanOrAmendment, ProtocolType,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{
            AddPlansProtocolReq, AddPlansProtocolResponse, PreAddPlansProtocolReq,
            PreAddPlansProtocolResponse,
        },
        response_request::*,
    },
};
use sqlx::PgPool;

use crate::{
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::protocol::ProtocolAddPlansMessage,
};

pub(crate) mod action;
pub(crate) mod pre_request;
use pre_request::pre_add_plans_protocol_in_person;

use self::pre_request::pre_add_plans_protocol_correspondence;

const PRE_ADD_PLANS_PROTOCOL: &str = "v1/pre_request/add_plans_protocol";

pub(crate) async fn add_plans_protocol(
    request: AddPlansProtocolReq,
    proc_ctx: ProcessingCtx,
) -> Result<AddPlansProtocolResponse> {
    tracing::info!(
        kind = "get",
        "Запрос на добавление элементов Протокола СК {get}: {req:?}\n",
        get = PRE_ADD_PLANS_PROTOCOL,
        req = request,
    );

    let ids = request.item_list.iter().map(|i| i.id.clone()).collect::<Vec<_>>();
    let (joined_agendas, plans, mut messages) = pre_add_plans_protocol_inner(
        &ids,
        request.protocol_type_id,
        &proc_ctx.db_pool,
    )
    .await?;

    if messages.is_error() {
        return Ok(ApiResponse::default().with_messages(messages));
    }
    messages.clear();

    let (protocol, added_plans) = match request.protocol_type_id {
        ProtocolType::InPersonMeeting => {
            let joined_agendas = joined_agendas
                .expect("pre_add_plans_protocol_inner гарантирует, что при ProtocolType::InPersonMeeting значение будет возвращено");

            let selected_items = request.item_list.into_iter().map(|i| {
                    let all_items = i.all_items
                        .ok_or(ProcessingError::AddPlansProtocol(String::from("При добавлении ППЗ/ДС в Протокол очной СК is_all_items_included является обязательным аргументом")))?;
                    let item_list = i.item_list
                        .ok_or(ProcessingError::AddPlansProtocol(String::from("При добавлении ППЗ/ДС в Протокол очной СК item_list является обязательным аргументом")))?;

                    Ok(action::AddPlansProtocolItemInPerson { agenda_id: i.id, all_items, agenda_items: item_list })
                })
                .collect::<Result<Vec<_>>>()?;

            action::add_plans_protocol_in_person(
                selected_items,
                ObjectIdentifier::new_with_type(
                    request.protocol_id,
                    request.uuid,
                    EntityKind::Protocol,
                ),
                request.user_id,
                joined_agendas,
                &mut messages,
                &proc_ctx,
            )
            .await?
        }
        ProtocolType::CorrespondenceMeeting => {
            let mut plans = plans
                .expect("pre_add_plans_protocol_inner гарантирует, что при ProtocolType::CorrespondenceMeeting значение будет возвращено")
                .into_iter()
                .map(|p| (*p.uuid(), p))
                .collect::<AHashMap<_, _>>();

            let ordered_plans = request
                .item_list
                .iter()
                .filter_map(|i| plans.remove(&i.id.uuid))
                .collect::<Vec<_>>();

            action::add_plans_protocol_correspondence(
                ObjectIdentifier::new_with_type(
                    request.protocol_id,
                    request.uuid,
                    EntityKind::Protocol,
                ),
                ordered_plans,
                request.user_id,
                &mut messages,
                &proc_ctx,
            )
            .await?
        }
        ProtocolType::Undefined => {
            unreachable!("Проверено в pre_create_protocol_inner")
        }
    };

    ProtocolAddPlansMessage::Success(&protocol)
        .checked_append(&mut messages, &added_plans);

    Ok(((), messages).into())
}

pub(crate) async fn pre_add_plans_protocol(
    request: PreAddPlansProtocolReq,
    proc_ctx: ProcessingCtx,
) -> Result<PreAddPlansProtocolResponse> {
    tracing::info!(
        kind = "get",
        "Предзапрос на добавление элементов Протокола СК {get}: {req:?}\n",
        get = PRE_ADD_PLANS_PROTOCOL,
        req = request,
    );

    let PreAddPlansProtocolReq {
        protocol_type_id,
        item_list,
        ..
    } = request;

    let (joined_agendas, plans, messages) = pre_add_plans_protocol_inner(
        &item_list,
        protocol_type_id,
        &proc_ctx.db_pool,
    )
    .await?;

    pre_request::finalise_response(joined_agendas, plans, messages)
}

/// При protocol_type равном [`ProtocolType::InPersonMeeting`] гарантированно будут возвращены
/// [`Vec<JoinedAgenda>`]
///
/// При protocol_type равном [`ProtocolType::CorrespondenceMeeting`] гарантированно будут возвращены
/// [`Vec<PlanOrAmendment>`]
pub(crate) async fn pre_add_plans_protocol_inner(
    item_list: &[ObjectIdentifier],
    protocol_type: ProtocolType,
    db_pool: &PgPool,
) -> Result<(Option<Vec<JoinedAgenda>>, Option<Vec<PlanOrAmendment>>, Messages)> {
    match protocol_type {
        ProtocolType::InPersonMeeting => {
            let (joined_agendas, messages) =
                pre_add_plans_protocol_in_person(item_list, db_pool).await?;

            Ok((Some(joined_agendas), None, messages))
        }
        ProtocolType::CorrespondenceMeeting => {
            let (plans, messages) =
                pre_add_plans_protocol_correspondence(item_list, db_pool).await?;
            Ok((None, Some(plans), messages))
        }
        ProtocolType::Undefined => Err(ProcessingError::AddPlansProtocol(
            String::from("Невалидное значение для типа протокола"),
        )),
    }
}
