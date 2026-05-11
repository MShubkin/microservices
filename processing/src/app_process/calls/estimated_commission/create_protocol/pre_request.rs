use ahash::AHashSet;
use asez2_shared_db::db_item::{
    from_item_with_fields, joined::JoinTo, selection::*,
};

use shared_essential::{
    domain::tables::{
        legacy::plans::PlanStatus,
        processing::{
            agenda::{
                JoinedEcAgendaEcAgendaItemPlanContractAmendment as JoinedAgenda,
                JoinedEcAgendaEcAgendaItemPlanContractAmendmentSelector as JoinedAgendaSel,
            },
            protocol_item::JoinedEcProtocolItemEcProtocol as JoinedProtocolItem,
        },
        *,
    },
    presentation::dto::{
        general::ObjectIdentifier, processing::*, response_request::*,
    },
};
use sqlx::PgPool;

use crate::{
    app_process::{
        common::{self, plan::fetch_plans_by_ids},
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingError, Result},
    presentation::business_messages::protocol::ProtocolCreateMessage,
};

const AGENDA_RET_FIELDS: &[&str] = &[
    EcAgenda::uuid,
    "agenda_id",
    "agenda_status_id",
    EcAgenda::meeting_date,
    EcAgenda::pricing_organization_unit_id,
    EcAgenda::created_by,
];

const PLAN_RET_FIELDS: &[&str] = &[
    "plan_id",
    Plan::customer_id,
    Plan::contract_subject,
    Plan::pricing_expert_id,
    Plan::supplier_id,
    Plan::sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat,
    Plan::currency_id,
    Plan::pricing_organization_unit_id,
    Plan::status_id,
    ContractAmendment::delta_sum_excluded_vat,
];

pub(super) async fn pre_create_protocol_in_person(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<JoinedAgenda>, Messages)> {
    let joined_agendas = fetch_joined_agendas(item_list, db_pool).await?;
    let mut messages = Messages::default();

    examine_agendas(
        &joined_agendas,
        |kind, invalid_agenda| match kind {
            AgendaErrorKind::Empty => {
                ProtocolCreateMessage::empty_agenda(invalid_agenda)
            }
            AgendaErrorKind::InvalidStatus => {
                ProtocolCreateMessage::invalid_agenda_status(invalid_agenda)
            }
        },
        &mut messages,
    );

    Ok((joined_agendas, messages))
}

pub(super) async fn pre_create_protocol_correspondence(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Messages)> {
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;
    let mut messages = Messages::default();

    examine_plan_status(
        &plans,
        |invalid_plans| {
            ProtocolCreateMessage::InvalidPlanStatus
                .resolve(&invalid_plans)
                .expect("examine_plan_status гарантирует !invalid_plans.is_empty()")
        },
        &mut messages,
    );
    examine_protocols(
        &plans,
        |protocol_item, invalid_plan| {
            ProtocolCreateMessage::AlreadyInProtocol(&protocol_item.protocol)
                .singular(invalid_plan)
                .into()
        },
        &mut messages,
        db_pool,
    )
    .await?;

    Ok((plans, messages))
}

pub(crate) enum AgendaErrorKind {
    Empty,
    InvalidStatus,
}

pub(crate) fn examine_agendas<F>(
    joined_agendas: &[JoinedAgenda],
    message_fn: F,
    messages: &mut Messages,
) where
    F: Fn(AgendaErrorKind, &EcAgenda) -> Message,
{
    for joined_agenda in joined_agendas {
        let JoinedAgenda { agenda, items, .. } = joined_agenda;

        if items.is_empty() {
            messages
                .add_prepared_message(message_fn(AgendaErrorKind::Empty, agenda));
            continue;
        }

        if matches!(
            agenda.status_id,
            EcAgendaStatus::Deleted | EcAgendaStatus::ProtocolFormed
        ) {
            messages.add_prepared_message(message_fn(
                AgendaErrorKind::InvalidStatus,
                agenda,
            ));
        }
    }
}

pub(crate) fn examine_plan_status<F>(
    plans: &[PlanOrAmendment],
    message_fn: F,
    messages: &mut Messages,
) where
    F: Fn(Vec<PlanOrAmendment>) -> Message,
{
    let invalid_plans = plans
        .iter()
        .filter(|x| {
            !matches!(
                *x.status_id(),
                PlanStatus::EstimatedCommissionCorrespondence
                    | PlanStatus::PriceDetermined
                    | PlanStatus::PriceConfirmed
            ) || *x.commission_kind_id() != CommissionKind::Correspondence
        })
        .cloned()
        .collect::<Vec<_>>();

    if !invalid_plans.is_empty() {
        messages.add_prepared_message(message_fn(invalid_plans))
    }
}

pub(crate) async fn examine_protocols<F>(
    plans: &[PlanOrAmendment],
    message_fn: F,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()>
where
    F: Fn(&JoinedProtocolItem, &PlanOrAmendment) -> Option<Message>,
{
    common::protocol::examine_protocol_items(
        plans,
        Some(ProtocolType::CorrespondenceMeeting),
        message_fn,
        messages,
        db_pool,
    )
    .await?;

    Ok(())
}

pub(crate) async fn fetch_joined_agendas(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<Vec<JoinedAgenda>> {
    let agenda_sel = Select::full::<EcAgenda>()
        .in_any(EcAgenda::uuid, item_list.iter().map(|i| i.uuid))
        .eq(EcAgenda::is_removed, false);
    let item_sel = Select::full::<EcAgendaItem>()
        .eq(EcAgendaItem::is_excluded, false)
        .eq(EcAgendaItem::is_removed, false);

    let joined_agendas = JoinedAgendaSel::new(agenda_sel)
        .set_items(EcAgendaItem::join_default().selecting(item_sel))
        .distinct()
        .get(db_pool)
        .await?;

    if joined_agendas.len() != item_list.len() {
        let found_uuids =
            joined_agendas.iter().map(|x| x.agenda.uuid).collect::<AHashSet<_>>();
        let missing = item_list
            .iter()
            .filter(|x| !found_uuids.contains(&x.uuid))
            .map(|x| x.id.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let msg = format!("Повестки СК с идентификаторами {} не найдены", missing);
        return Err(ProcessingError::CreateProtocol(msg));
    }

    Ok(joined_agendas)
}

fn calculate_threshold(
    plans: &[PlanOrAmendment],
    agenda_items: &[EcAgendaItem],
) -> Result<ColorThreshold> {
    let (item_threshold, _) =
        common::agenda::calculate_quantity_thresholds(plans, agenda_items, true)?;

    Ok(item_threshold.into())
}

pub(crate) fn finalise_response(
    joined_agendas: Option<Vec<JoinedAgenda>>,
    plans: Option<Vec<PlanOrAmendment>>,
    messages: Messages,
) -> Result<PreCreateProtocolResponse> {
    if messages.is_error() {
        return Ok(ApiResponse::default().with_messages(messages));
    }
    let agenda_from_item = from_item_with_fields(AGENDA_RET_FIELDS);

    let agenda_list = joined_agendas
        .map(|joined_agendas| {
            joined_agendas
                .iter()
                .cloned()
                .map(|joined_agenda| {
                    let JoinedAgenda {
                        agenda,
                        items,
                        plans,
                        amendments,
                    } = joined_agenda;
                    let plans =
                        PlanOrAmendment::collect::<Vec<_>>(plans, amendments);

                    let item_threshold = calculate_threshold(&plans, &items)?;

                    Ok(AgendaWithItemThreshold {
                        agenda: agenda_from_item(agenda),
                        agenda_item_quantity_threshold: Some(item_threshold),
                        // Remaining fields are not serialized.
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?;

    let plans = plans.map(|plans| {
        plans
            .into_iter()
            .map(|p| {
                PlanOrAmendmentRep::from_item_with_section_mapping(
                    p,
                    SectionKind::EstimatedCommission,
                    Some(PLAN_RET_FIELDS),
                )
            })
            .collect::<Vec<_>>()
    });

    let data = PreCreateProtocolResponseData { agenda_list, plans };

    Ok((data, messages).into())
}
