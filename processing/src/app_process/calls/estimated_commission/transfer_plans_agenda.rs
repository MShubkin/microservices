use std::sync::Arc;

use ahash::{AHashMap, AHashSet};
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{joined::JoinTo, Select},
    DbItem,
};
use shared_essential::{
    application::records::Recorder,
    domain::{
        ContractAmendment, EcAgenda, EcAgendaItem, EcAgendaStatus,
        JoinedEcAgendaItemEcAgendaSelector, Plan, PlanOrAmendment,
        PlanOrAmendmentRep, SectionKind,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{
            PreTransferPlansAgendaReq, PreTransferPlansAgendaResponse,
            TransferPlansAgendaReq, TransferPlansAgendaResponse,
        },
        response_request::{
            ApiResponse, BusinessMessage, MessageKind, Messages, Status,
        },
    },
};

use crate::{
    app_process::{
        common::{self, agenda::AgendaPricingUnitCheck},
        records::ProcessingRulesChecker,
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::agenda::AgendaTransferPlansMessage,
};

use super::{add_plans_agenda, create_agenda};

const PRE_TRANSFER_PLANS_AGENDA: &str = "v1/pre_request/transfer_plans_agenda";
const TRANSFER_PLANS_AGENDA: &str = "v1/action/transfer_plans_agenda";

const RESPONSE_FIELD_LIST: &[&str] = &[
    "plan_id",
    Plan::customer_id,
    Plan::contract_subject,
    Plan::pricing_expert_id,
    Plan::supplier_id,
    Plan::sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat,
    Plan::currency_id,
    Plan::pricing_organization_unit_id,
    Plan::commission_date,
    Plan::status_id,
    Plan::section_id,
];

/// Проверка возможности перемещения ППЗ/ДС между Повестками СК
pub(crate) async fn pre_transfer_plans_agenda(
    request: PreTransferPlansAgendaReq,
    db_pool: Arc<PgPool>,
) -> Result<PreTransferPlansAgendaResponse> {
    let (plans, messages) = pre_transfer_plans_agenda_inner(
        request,
        PRE_TRANSFER_PLANS_AGENDA,
        &db_pool,
    )
    .await?;

    if messages.is_error() {
        return Ok(ApiResponse {
            status: Status::Ok,
            messages,
            ..Default::default()
        });
    }

    let data = plans
        .into_iter()
        .map(|p| {
            PlanOrAmendmentRep::from_item_with_section_mapping(
                p,
                SectionKind::EstimatedCommission,
                Some(RESPONSE_FIELD_LIST),
            )
        })
        .collect::<Vec<_>>();

    Ok((data, messages).into())
}

/// Экшен на перемещение ППЗ/ДС между Повестками СК
pub(crate) async fn transfer_plans_agenda(
    request: TransferPlansAgendaReq,
    proc_ctx: ProcessingCtx,
) -> Result<TransferPlansAgendaResponse> {
    let TransferPlansAgendaReq {
        agenda_id,
        is_force,
        item_list,
        user_id,
    } = request;

    let agenda = EcAgenda::select(
        &Select::full::<EcAgenda>().eq(EcAgenda::id, agenda_id),
        proc_ctx.db_pool.as_ref(),
    )
    .await?
    .pop()
    .ok_or_else(|| {
        ProcessingError::TransferPlansAgenda(format!(
            "Повестка СК № {} не найдена",
            agenda_id
        ))
    })?;

    let (plans, mut messages) = pre_transfer_plans_agenda_inner(
        item_list,
        TRANSFER_PLANS_AGENDA,
        &proc_ctx.db_pool,
    )
    .await?;
    examine_agenda_status(&agenda, &mut messages);

    if messages.is_error() || (messages.kind == MessageKind::Warning && !is_force) {
        return Ok(ApiResponse::default().with_messages(messages));
    }
    messages.clear();

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    remove_old_agenda_items(&plans, &mut messages, &mut recorder).await?;
    let (updated_plans, agenda) = insert_new_agenda_items(
        agenda,
        plans,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    recorder.commit().await?;

    AgendaTransferPlansMessage::Success(&agenda)
        .checked_append(&mut messages, &updated_plans);
    let ids = updated_plans.into_iter().map(|p| *p.id()).collect::<Vec<_>>();

    Ok((ids, messages).into())
}

pub(crate) async fn pre_transfer_plans_agenda_inner(
    request: Vec<ObjectIdentifier>,
    tag: &str,
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Messages)> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = tag,
        req = request,
    );

    let mut message_buf = Messages::default();

    let mut valid_plan_map =
        fetch_and_examine_plans(&request, &mut message_buf, db_pool).await?;
    examine_protocol_items(&mut valid_plan_map, &mut message_buf, db_pool).await?;
    examine_agenda_items(&mut valid_plan_map, &mut message_buf, db_pool).await?;
    examine_pricing_unit(&valid_plan_map, &mut message_buf);

    let plans = request
        .iter()
        .filter_map(|oid| valid_plan_map.remove(&oid.uuid))
        .collect();

    Ok((plans, message_buf))
}

/// Добалвние перемещенных ППЗ/ДС в Повестку СК
///
/// Возвращает обновленные ППЗ/ДС в соответствии с Повесткой и саму Повестку
async fn insert_new_agenda_items(
    agenda: EcAgenda,
    plans: Vec<PlanOrAmendment>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<(Vec<PlanOrAmendment>, EcAgenda)> {
    let old_agenda_items = EcAgendaItem::select(
        &Select::full::<EcAgendaItem>().eq(EcAgendaItem::agenda_uuid, agenda.uuid),
        recorder.tx(),
    )
    .await?;

    let (updated_plans, agenda, _, _) = add_plans_agenda::insert_agenda_items(
        agenda,
        old_agenda_items,
        plans,
        messages,
        recorder,
        handler,
    )
    .await?;

    Ok((updated_plans, agenda))
}

/// Перенос на новый статус позиций Повестки, которые перемещаются между
/// Повестками
async fn remove_old_agenda_items(
    plans: &[PlanOrAmendment],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let agenda_select = Select::full::<EcAgenda>().eq(EcAgenda::is_removed, false);
    let agenda_item_select = Select::full::<EcAgendaItem>()
        .in_any(EcAgendaItem::source_uuid, plans.iter().map(|p| p.uuid()))
        .eq(EcAgendaItem::is_removed, false);

    let old_agenda_items =
        JoinedEcAgendaItemEcAgendaSelector::new(agenda_item_select)
            .set_agenda(EcAgenda::join_default().selecting(agenda_select))
            .get(recorder.tx())
            .await?;

    let updated_agenda_items = old_agenda_items
        .into_iter()
        .map(|mut i| {
            match i.agenda.status_id {
                EcAgendaStatus::Formed => i.agenda_item.is_removed = true,
                EcAgendaStatus::Sent => {
                    i.agenda_item.is_excluded = true;
                    i.agenda_item.reviewed_at = None;
                }
                _ => {}
            };

            i.agenda_item
        })
        .collect::<Vec<_>>();

    recorder
        .process_update(
            updated_agenda_items,
            &[
                EcAgendaItem::is_excluded,
                EcAgendaItem::is_removed,
                EcAgendaItem::reviewed_at,
            ],
            messages,
        )
        .await?;

    Ok(())
}

/// Получение ППЗ/ДС и проверка корректности статуса
///
/// Возвращается маппер (plan.uuid, plan)
async fn fetch_and_examine_plans(
    item_list: &[ObjectIdentifier],
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<AHashMap<Uuid, PlanOrAmendment>> {
    create_agenda::fetch_and_examine_plans(
        item_list,
        messages,
        db_pool,
        |invalid_plans| {
            AgendaTransferPlansMessage::InvalidPlanStatus
                .resolve(&invalid_plans)
                .expect("invalid_plans гарантированно непустой массив")
        },
    )
    .await
}

/// По ППЗ/ДС проверить наличие Протоколов (protocol_type_id = 1/Протокол очного заседания СК).
/// Если Протокол отсутствует, то перейти к следующей проверке. Если присутствует,
/// то проверить по ППЗ/ДС значение в поле «Решение СК».
async fn examine_protocol_items(
    plan_map: &mut AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    create_agenda::examine_protocols(
        plan_map,
        messages,
        db_pool,
        |protocol, protocol_item, plan| {
            AgendaTransferPlansMessage::AlreadyInProtocol(protocol, protocol_item)
                .singular(plan)
        },
    )
    .await
}

fn examine_pricing_unit(
    items: &AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
) {
    if let Err(msg) = common::agenda::examine_pricing_unit(items.values()) {
        let msg = match msg {
            AgendaPricingUnitCheck::DifferentDepartment => {
                AgendaTransferPlansMessage::different_department()
            }
            AgendaPricingUnitCheck::DifferentSections => {
                AgendaTransferPlansMessage::different_plan_sections()
            }
        };
        messages.add_prepared_message(msg);
    }
}

/// Проверить наличие по ППЗ/ДС актуальной (не удаленной) Повестки,
/// в которой ППЗ/ДС не удалена (is_removed = false) и не снята с рассмотрения (is_excluded=false).
async fn examine_agenda_items(
    plan_map: &mut AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    let plans_with_agenda = JoinedEcAgendaItemEcAgendaSelector::new(
        Select::default()
            .in_any(EcAgendaItem::source_uuid, plan_map.keys())
            .eq(EcAgendaItem::is_excluded, false)
            .eq(EcAgendaItem::is_removed, false)
            .add_replace_order_desc(EcAgendaItem::created_at),
    )
    .set_agenda(
        EcAgenda::join_default()
            .selecting(Select::default().eq(EcAgenda::is_removed, false)),
    )
    .get(db_pool)
    .await?
    .into_iter()
    .map(|i| i.agenda_item.source_uuid)
    .collect::<AHashSet<_>>();

    let mut invalid_plans = Vec::new();
    plan_map.retain(|uuid, plan| {
        if !plans_with_agenda.contains(uuid) {
            invalid_plans.push(plan.clone());
            false
        } else {
            true
        }
    });

    AgendaTransferPlansMessage::NotIncludedInAgenda
        .checked_append(messages, &invalid_plans);

    Ok(())
}

/// Выполнить проверку на статус новой Повестки, в которую добавляется ППЗ/ДС.
/// Если у Повестки установлен status_id = 100, 200, то переходим к следующей проверке,
/// иначе формируем сообщение об ошибке: "Повестка <Номер Повестки> находится
/// на статусе "Сформирован Протокол". Выполнить изменение невозможно."
fn examine_agenda_status(agenda: &EcAgenda, messages: &mut Messages) {
    if !matches!(agenda.status_id, EcAgendaStatus::Formed | EcAgendaStatus::Sent) {
        messages.add_prepared_message(
            AgendaTransferPlansMessage::invalid_agenda_status(agenda),
        );
    }
}
