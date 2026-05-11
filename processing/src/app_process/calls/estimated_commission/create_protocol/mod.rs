//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
pub(crate) mod action;
pub(crate) mod pre_request;
use std::sync::Arc;

pub(crate) use pre_request::{
    examine_agendas, examine_plan_status, examine_protocols, fetch_joined_agendas,
    finalise_response, AgendaErrorKind,
};

use shared_essential::{
    domain::{
        tables::*, JoinedEcAgendaEcAgendaItemPlanContractAmendment as JoinedAgenda,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::*,
        response_request::{ApiResponse, Messages},
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingCtx, ProcessingError, Result};

const CREATE_PROTOCOL: &str =
    "/rest/estimated_commission/v1/action/protocol_create/";
const PRE_CREATE_PROTOCOL: &str =
    "/rest/estimated_commission/v1/pre_request/protocol_create/";

pub(crate) async fn create_protocol(
    req: CreateProtocolReq,
    proc_ctx: ProcessingCtx,
) -> Result<CreateProtocolResponse> {
    tracing::info!(
        kind = "insert",
        "Получен запрос на создание Протокола СК ({get}): {req:?}\n",
        get = CREATE_PROTOCOL,
        req = req,
    );

    let CreateProtocolReq {
        user_id,
        protocol_type_id,
        protocol_date,
        item_list,
    } = req;

    let ids = item_list.iter().map(|i| i.id.clone()).collect::<Vec<_>>();
    let (joined_agendas, plans, mut messages) =
        pre_create_protocol_inner(protocol_type_id, &ids, &proc_ctx.db_pool)
            .await?;

    if messages.is_error() {
        return Ok(ApiResponse::default().with_messages(messages));
    }
    messages.clear();

    let protocol = match protocol_type_id {
        ProtocolType::InPersonMeeting => {
            let joined_agendas = joined_agendas
                .expect("pre_create_protocol_inner гарантирует, что при ProtocolType::InPersonMeeting значение будет возвращено");
            let item_list = item_list.into_iter().map(|i| {
                let all_items = i.all_items
                    .ok_or(ProcessingError::CreateProtocol(String::from("При создании очного Протокола СК is_all_items_included является обязательным аргументом")))?;
                let item_list = i.item_list
                    .ok_or(ProcessingError::CreateProtocol(String::from("При создании очного Протокола СК item_list является обязательным аргументом")))?;

                Ok(action::CreateProtocolItemInPerson { agenda_id: i.id, all_items, agenda_items: item_list })
            }).collect::<Result<Vec<_>>>()?;

            action::create_protocol_in_person(
                protocol_type_id,
                protocol_date,
                item_list,
                joined_agendas,
                user_id,
                &mut messages,
                proc_ctx,
            )
            .await?
        }
        ProtocolType::CorrespondenceMeeting => {
            let plans = plans
                .expect("pre_create_protocol_inner гарантирует, что при ProtocolType::CorrespondenceMeeting значение будет возвращено");

            action::create_protocol_correspondence(
                protocol_type_id,
                protocol_date,
                plans,
                user_id,
                &mut messages,
                proc_ctx,
            )
            .await?
        }
        ProtocolType::Undefined => {
            unreachable!("Проверено в pre_create_protocol_inner")
        }
    };

    action::finalise(protocol, messages)
}

pub(crate) async fn pre_create_protocol(
    req: PreCreateProtocolReq,
    db_pool: Arc<PgPool>,
) -> Result<PreCreateProtocolResponse> {
    tracing::info!(
        kind = "get",
        "Получен предзапрос на создание Протокола СК ({get}): {req:?}\n",
        get = PRE_CREATE_PROTOCOL,
        req = req,
    );

    let PreCreateProtocolReq {
        user_id: _,
        protocol_type_id,
        item_list,
    } = req;

    let (joined_agendas, plans, messages) =
        pre_create_protocol_inner(protocol_type_id, &item_list, &db_pool).await?;

    pre_request::finalise_response(joined_agendas, plans, messages)
}

/// Всегда возвращает [`Option::Some`] с Vec<JoinedAgenda> если в protocol_type=1
/// в обратном случае будет [`Option::None`]
///
/// Всегда возвращает [`Option::Some`] с Vec<PlanOrAmendment> если в protocol_type=2
/// в обратном случае будет [`Option::None`]
async fn pre_create_protocol_inner(
    protocol_type: ProtocolType,
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Option<Vec<JoinedAgenda>>, Option<Vec<PlanOrAmendment>>, Messages)> {
    match protocol_type {
        ProtocolType::InPersonMeeting => {
            let (joined_agendas, messages) =
                pre_request::pre_create_protocol_in_person(item_list, db_pool)
                    .await?;
            Ok((Some(joined_agendas), None, messages))
        }
        ProtocolType::CorrespondenceMeeting => {
            let (plans, messages) =
                pre_request::pre_create_protocol_correspondence(item_list, db_pool)
                    .await?;
            Ok((None, Some(plans), messages))
        }
        ProtocolType::Undefined => Err(ProcessingError::CreateProtocol(
            String::from("Невалидное значение для типа создаваемого протокола"),
        )),
    }
}
