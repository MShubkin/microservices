//! Бизнес логика по ручке "/rest/estimated_commission/v1/pre_request/add_plans_protocol/".
use shared_essential::{
    domain::tables::{
        processing::agenda::JoinedEcAgendaEcAgendaItemPlanContractAmendment as JoinedAgenda,
        *,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::PreAddPlansProtocolResponse,
        response_request::{Messages, *},
    },
};
use sqlx::PgPool;

use crate::{
    app_process::{
        common::plan::fetch_plans_by_ids, estimated_commission::create_protocol,
    },
    common::Result,
    presentation::business_messages::protocol::ProtocolAddPlansMessage,
};

pub(super) async fn pre_add_plans_protocol_in_person(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<JoinedAgenda>, Messages)> {
    let joined_agendas =
        create_protocol::fetch_joined_agendas(item_list, db_pool).await?;

    let mut messages = Messages::default();

    examine_agendas(&joined_agendas, &mut messages);

    Ok((joined_agendas, messages))
}

pub(super) async fn pre_add_plans_protocol_correspondence(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Messages)> {
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;

    let mut messages = Messages::default();

    examine_plan_status(&plans, &mut messages);
    examine_protocols(&plans, &mut messages, db_pool).await?;

    Ok((plans, messages))
}

fn examine_agendas(joined_agendas: &[JoinedAgenda], messages: &mut Messages) {
    create_protocol::examine_agendas(
        joined_agendas,
        |kind, invalid_agenda| match kind {
            create_protocol::AgendaErrorKind::Empty => {
                ProtocolAddPlansMessage::empty_agenda(invalid_agenda)
            }
            create_protocol::AgendaErrorKind::InvalidStatus => {
                ProtocolAddPlansMessage::invalid_agenda_status(invalid_agenda)
            }
        },
        messages,
    )
}

fn examine_plan_status(plans: &[PlanOrAmendment], messages: &mut Messages) {
    create_protocol::examine_plan_status(
        plans,
        |invalid_plans| {
            ProtocolAddPlansMessage::InvalidPlanStatus.plural(&invalid_plans)
        },
        messages,
    )
}

async fn examine_protocols(
    plans: &[PlanOrAmendment],
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    create_protocol::examine_protocols(
        plans,
        |protocol_item, invalid_plan| {
            ProtocolAddPlansMessage::AlreadyInProtocol(&protocol_item.protocol)
                .singular(invalid_plan)
                .into()
        },
        messages,
        db_pool,
    )
    .await
}

pub(super) fn finalise_response(
    joined_agendas: Option<Vec<JoinedAgenda>>,
    plans: Option<Vec<PlanOrAmendment>>,
    messages: Messages,
) -> Result<PreAddPlansProtocolResponse> {
    create_protocol::finalise_response(joined_agendas, plans, messages)
}
