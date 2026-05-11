use std::sync::Arc;

use ahash::AHashMap;
use asez2_shared_db::{db_item::Select, DbItem};
use shared_essential::{
    domain::{
        CommissionKind, ContractAmendment, EcAgenda, Plan, PlanOrAmendment,
        PlanOrAmendmentRep, SectionKind,
    },
    presentation::dto::{
        processing::{
            GetAgendaItemsByIdRangeReq, GetAgendaItemsByIdRangeResponseData,
        },
        response_request::{
            ApiResponse, BusinessMessage, Message, Messages, Status,
        },
    },
};
use sqlx::PgPool;
use uuid::Uuid;

use super::{add_plans_agenda, create_agenda};
use crate::app_process::{
    common::plan::fetch_plans_by_range_ids, sections::mapping::SectionMapExt,
};
use crate::{
    common::{ProcessingError, Result},
    presentation::business_messages::agenda::AgendaGetItemsMessage,
};

const RESPONSE_FIELD_LIST: &[&str] = &[
    Plan::uuid,
    "plan_id",
    Plan::customer_id,
    Plan::contract_subject,
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::supplier_id,
    Plan::sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat,
    Plan::pricing_sum_excluded_vat,
    ContractAmendment::pricing_delta_sum_excluded_vat,
    Plan::currency_id,
    Plan::section_id,
    Plan::status_id,
];

pub(crate) async fn get_agenda_items_by_id_range(
    dto: GetAgendaItemsByIdRangeReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetAgendaItemsByIdRangeResponseData, ()>> {
    let GetAgendaItemsByIdRangeReq {
        agenda_id,
        item_list,
        ..
    } = dto;

    let ids = item_list.into_iter().filter_map(|range| {
        match (range.first(), range.get(1)) {
            (Some(&left), Some(&right)) => Some(left..=right),
            (Some(&left), None) => Some(left..=left),
            _ => None,
        }
    });

    let agenda = fetch_agenda(agenda_id, &db_pool).await?;
    let plans = fetch_plans_by_range_ids(ids, &db_pool).await?;
    let uuid_sequence = plans.iter().map(|p| *p.uuid()).collect::<Vec<_>>();

    let mut plan_map =
        plans.into_iter().map(|p| (*p.uuid(), p)).collect::<AHashMap<_, _>>();
    let mut message_buf = Messages::default();

    examine_commission_kind(&mut plan_map, &mut message_buf, |invalid_plans| {
        AgendaGetItemsMessage::InvalidCommissionKind
            .resolve(invalid_plans)
            .unwrap()
    });
    examine_protocols(&mut plan_map, &mut message_buf, &db_pool).await?;
    examine_agendas(&mut plan_map, agenda_id.into(), &mut message_buf, &db_pool)
        .await?;
    add_plans_agenda::examine_agenda_pricing_unit(
        &plan_map,
        &agenda,
        &mut message_buf,
        |plans_with_warn, agenda| {
            AgendaGetItemsMessage::DifferentDepartment(agenda)
                    .resolve(plans_with_warn)
                    .expect("examine_agenda_pricing_unit гарантирует непустой plans_with_warn")
        },
    );

    let plans = uuid_sequence
        .into_iter()
        .filter_map(|oid| plan_map.remove(&oid))
        .collect::<Vec<_>>();

    finalise_response(plans, agenda, message_buf)
}

/// message_fn принимает !invalid_plans.is_empty() массив невалидных ППЗ/ДС
pub(crate) fn examine_commission_kind<F>(
    plan_map: &mut AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
    message_fn: F,
) where
    F: Fn(&[PlanOrAmendment]) -> Message,
{
    let invalid_plan_uuids = plan_map
        .values()
        .filter(|p| {
            !matches!(
                *p.commission_kind_id(),
                CommissionKind::Undefined | CommissionKind::InPerson,
            )
        })
        .map(|p| *p.uuid())
        .collect::<Vec<_>>();

    let mut invalid_plans = Vec::new();
    for invalid_uuid in invalid_plan_uuids {
        if let Some(plan) = plan_map.remove(&invalid_uuid) {
            invalid_plans.push(plan);
        }
    }

    if !invalid_plans.is_empty() {
        messages.add_prepared_message(message_fn(&invalid_plans));
    }
}

async fn examine_protocols(
    plan_map: &mut AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    create_agenda::examine_protocols(
        plan_map,
        messages,
        db_pool,
        |protocol, protocol_item, plan| {
            AgendaGetItemsMessage::AlreadyInProtocol(protocol, protocol_item)
                .singular(plan)
        },
    )
    .await
}

/// По ППЗ/ДС проверить, что она уже не включена в данную Повестку в конкретный
/// раздел.
///
/// ...
///
/// По ППЗ/ДС проверить наличие Повесток. Если Повестка отсутствует, то перейти
/// к следующей проверке. Если присутствует, то проверить по ППЗ/ДС значение в
/// поле «Снято с рассмотрения» и что позиция Повестки по ППЗ/ДС не включена в
/// позицию Протокола.
async fn examine_agendas(
    plan_map: &mut AHashMap<Uuid, PlanOrAmendment>,
    agenda_id: Option<i64>,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    create_agenda::examine_agendas(
        plan_map,
        messages,
        db_pool,
        |agenda, _, plan| {
            if Some(agenda.id) == agenda_id {
                AgendaGetItemsMessage::AlreadyInCurrentAgenda(agenda)
                    .singular(plan)
                    .into()
            } else {
                // ППЗ/ДС включен в другую повестку
                AgendaGetItemsMessage::AlreadyInAgenda(agenda).singular(plan).into()
            }
        },
    )
    .await
}

async fn fetch_agenda(agenda_id: i64, pool: &PgPool) -> Result<EcAgenda> {
    let agenda_select = Select::full::<EcAgenda>().eq(EcAgenda::id, agenda_id);
    let mut agendas = EcAgenda::select(&agenda_select, pool).await?;

    if agendas.is_empty() {
        return Err(ProcessingError::GetItemList(format!(
            "Повестка СК с идентификатором {} не найдена",
            agenda_id
        )));
    }

    Ok(agendas.remove(0))
}

fn finalise_response(
    plans: Vec<PlanOrAmendment>,
    agenda: EcAgenda,
    mut messages: Messages,
) -> Result<ApiResponse<GetAgendaItemsByIdRangeResponseData, ()>> {
    let plans = if messages.is_error() {
        Vec::new()
    } else {
        AgendaGetItemsMessage::Success(&agenda)
            .checked_append(&mut messages, &plans);

        plans
            .into_iter()
            .map(|p| {
                PlanOrAmendmentRep::from_item_with_section_mapping(
                    p,
                    SectionKind::EstimatedCommission,
                    Some(RESPONSE_FIELD_LIST),
                )
            })
            .collect::<Vec<_>>()
    };

    Ok(ApiResponse {
        objects: vec![],
        status: Status::Ok,
        messages,
        data: GetAgendaItemsByIdRangeResponseData { item_list: plans },
    })
}
