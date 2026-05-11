//! Бизнес логика по ручкам "/rest/v1/estimated_commission/pre_request/cancel/"
//! и "/rest/v1/estimated_commission/action/cancel/".
use ahash::AHashMap;
use processing::protocol_item::JoinedEcProtocolItemEcProtocol as JoinedEcProtocolItem;
use rabbit_services::master_data::MasterDataService;
use shared_essential::{
    application::records::Recorder,
    domain::tables::{
        legacy::plans::PlanStatus,
        JoinedEcAgendaItemEcAgendaRelAgendaProtocolItem as JoinedEcAgendaItem, *,
    },
    presentation::dto::{
        general::{ObjectIdentifier, ObjectIdentifierWithStatusNote},
        processing::{CancelPlansReq, CancelPlansResponseData, PreCancelPlansReq},
        response_request::*,
    },
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    app_process::{
        common::{
            self,
            agenda::fetch_agenda_items,
            plan::{examine_plan_status, fetch_plans_by_ids},
            protocol::examine_protocol_items,
        },
        records::{send_to_monolith, PlanCollectedUpdate, ProcessingRulesChecker},
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::plan::PlanCancelMessage,
};

#[allow(unused_imports)]
use ahash::AHashSet;

#[allow(unused_imports)]
use asez2_shared_db::db_item::Select;

#[allow(unused_imports)]
use shared_essential::{
    domain::plan_reasons_cancel::CheckReason,
    presentation::dto::{
        master_data::plan_reasons_cancel::PlanReasonCancel,
        master_data::request::SearchPlanReasonsCancelRabbitReq,
    },
};

const PRE_CANCEL: &str = "v1/pre_request/cancel/";
const CANCEL: &str = "v1/action/cancel/";

const PRE_REQUEST_RETURN_FIELDS: &[&str] = &[
    Plan::uuid,
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
    Plan::reason_cancel_id,
    Plan::replaced_id,
];

pub(crate) async fn cancel_plan(
    request: CancelPlansReq,
    proc_ctx: ProcessingCtx,
    #[allow(unused_variables)] master_data_service: MasterDataService,
) -> Result<ApiResponse<CancelPlansResponseData, ()>> {
    tracing::info!(
        kind = "cancel",
        "Запрос на аннулирование ППЗ/ДС ({get}): {request:?}\n",
        request = request,
        get = CANCEL
    );

    let identifier_list = request
        .item_list
        .iter()
        .map(|i| ObjectIdentifier::new_with_type(i.id, i.uuid, i.object_type))
        .collect::<Vec<_>>();

    let (plans, joined_protocols, mut messages) = pre_cancel_plan_inner(
        &identifier_list,
        request.section_id,
        &proc_ctx.db_pool,
    )
    .await?;

    if request.section_id == Section::EstimatedCommissionInPerson
        || request.section_id == Section::EstimatedCommissionCorrespondence
    {
        #[cfg(feature = "advanced-cancellation-control")]
        examine_cancel_data(
            &request.item_list,
            &mut messages,
            &proc_ctx,
            &master_data_service,
        )
        .await?;
    }

    if messages.is_error() {
        return finalise(plans, None, messages);
    }
    messages.clear();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .with_status_notes(request.item_list.iter().cloned())
        .begin()
        .await?;

    let updated_plans = update_plans(
        request.section_id,
        plans,
        &request.item_list,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    if request.section_id == Section::EstimatedCommissionInPerson {
        update_agenda_items(&updated_plans, &mut messages, &mut recorder).await?;
    }

    if request.section_id == Section::EstimatedCommissionCorrespondence {
        update_protocol_item(
            joined_protocols
                .expect("Для EstimatedCommissionCorrespondence гарантированно будут возвращены элементы"),
            &mut messages,
            &mut recorder,
        )
        .await?;
    }

    recorder.commit().await?;

    PlanCancelMessage::Success.checked_append(&mut messages, &updated_plans);

    finalise(updated_plans, None, messages)
}

pub(crate) async fn pre_cancel_plan(
    request: PreCancelPlansReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<CancelPlansResponseData, ()>> {
    tracing::info!(
        kind = "cancel",
        "Предзапрос на аннулирование ППЗ/ДС ({get}): {request:?}\n",
        request = request,
        get = PRE_CANCEL
    );

    let (plans, _, messages) =
        pre_cancel_plan_inner(&request.item_list, request.section_id, &db_pool)
            .await?;

    finalise(plans, Some(PRE_REQUEST_RETURN_FIELDS), messages)
}

async fn pre_cancel_plan_inner(
    item_list: &[ObjectIdentifier],
    section_id: Section,
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Option<Vec<JoinedEcProtocolItem>>, Messages)> {
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;
    let mut messages = Messages::default();

    let joined_protocols = match section_id {
        Section::EstimatedCommissionInPerson
        | Section::EstimatedCommissionCorrespondence => {
            let (plan_status, protocol_type) = match section_id {
                Section::EstimatedCommissionInPerson => (
                    PlanStatus::EstimatedCommissionInPerson,
                    ProtocolType::InPersonMeeting,
                ),
                Section::EstimatedCommissionCorrespondence => (
                    PlanStatus::EstimatedCommissionCorrespondence,
                    ProtocolType::CorrespondenceMeeting,
                ),
                _ => unreachable!("Остальные варианты не могут попасть сюда"),
            };

            examine_plan_status(
                &plans,
                &[plan_status],
                PlanCancelMessage::InvalidPlanStatus,
                &mut messages,
            );
            let protocol_with_items = examine_protocol_items(
                &plans,
                Some(protocol_type),
                |protocol_item, plan| {
                    examine_protocol(section_id, protocol_item, plan)
                },
                &mut messages,
                db_pool,
            )
            .await?;

            Some(protocol_with_items)
        }
        // Ничего не проверяется
        Section::EstimatedCommissionNotRequired => None,
        invalid_section => {
            return Err(ProcessingError::Section(format!(
                "Секция {} невалидна для аннулирования ППЗ/ДС",
                invalid_section
            )))
        }
    };

    Ok((plans, joined_protocols, messages))
}

async fn update_plans(
    section_id: Section,
    mut plans: Vec<PlanOrAmendment>,
    cancel_items: &[ObjectIdentifierWithStatusNote],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<Vec<PlanOrAmendment>> {
    #[allow(unused_variables)]
    let cancel_data_map: AHashMap<uuid::Uuid, &ObjectIdentifierWithStatusNote> =
        cancel_items.iter().map(|item| (item.uuid, item)).collect();

    plans.iter_mut().for_each(|x| {
        #[cfg(feature = "advanced-cancellation-control")]
        set_cancel_reason_and_replacement(x, &cancel_data_map);

        set_status_and_commission_fields(x, section_id);
    });

    let (plan_fields, amendment_fields) = get_update_fields(section_id);

    let plans = PlanOrAmendment::update_different_fields(
        plans,
        &plan_fields,
        &amendment_fields,
        messages,
        recorder,
        handler,
    )
    .await?;

    send_to_monolith(&plans, recorder).await?;

    Ok(plans)
}

/// Устанавливает причину аннулирования и заменяющую ППЗ
#[cfg(feature = "advanced-cancellation-control")]
fn set_cancel_reason_and_replacement(
    plan: &mut PlanOrAmendment,
    cancel_data_map: &AHashMap<uuid::Uuid, &ObjectIdentifierWithStatusNote>,
) {
    if let Some(item) = cancel_data_map.get(plan.uuid()) {
        if item.inner.object_type == EntityKind::Plan {
            if let PlanOrAmendment::Plan(plan_ref) = plan {
                plan_ref.reason_cancel_id = item.plan_reason_cancel_id;
                plan_ref.replaced_id = item.plan_replaced_id;
            }
        }
    }
}

/// Устанавливает статус и комиссионные поля в зависимости от типа секции
fn set_status_and_commission_fields(
    plan: &mut PlanOrAmendment,
    section_id: Section,
) {
    match section_id {
        Section::EstimatedCommissionInPerson => {
            *plan.commission_date_mut() = None;
            *plan.commission_kind_id_mut() = CommissionKind::Undefined;
            *plan.status_id_mut() = PlanStatus::PlanCancelled;
        }
        Section::EstimatedCommissionCorrespondence
        | Section::EstimatedCommissionNotRequired => {
            *plan.commission_kind_id_mut() = CommissionKind::Undefined;
            *plan.status_id_mut() = PlanStatus::PlanCancelled;
        }
        _ => unreachable!("Проверено выше (pre_cancel_plan_inner)"),
    }
}

/// Возвращает списки полей для обновления в зависимости от типа секции
fn get_update_fields(
    section_id: Section,
) -> (Vec<&'static str>, Vec<&'static str>) {
    match section_id {
        Section::EstimatedCommissionInPerson => {
            let plan_fields = if cfg!(feature = "advanced-cancellation-control") {
                vec![
                    Plan::status_id,
                    Plan::commission_date,
                    Plan::commission_kind_id,
                    Plan::reason_cancel_id,
                    Plan::replaced_id,
                ]
            } else {
                vec![
                    Plan::status_id,
                    Plan::commission_date,
                    Plan::commission_kind_id,
                ]
            };

            let amendment_fields = vec![
                ContractAmendment::status_id,
                ContractAmendment::commission_date,
                ContractAmendment::commission_kind_id,
            ];

            (plan_fields, amendment_fields)
        }
        Section::EstimatedCommissionCorrespondence
        | Section::EstimatedCommissionNotRequired => {
            let plan_fields = if cfg!(feature = "advanced-cancellation-control") {
                vec![
                    Plan::status_id,
                    Plan::commission_kind_id,
                    Plan::reason_cancel_id,
                    Plan::replaced_id,
                ]
            } else {
                vec![Plan::status_id, Plan::commission_kind_id]
            };

            let amendment_fields = vec![
                ContractAmendment::status_id,
                ContractAmendment::commission_kind_id,
            ];

            (plan_fields, amendment_fields)
        }
        _ => unreachable!("Проверено выше (pre_cancel_plan_inner)"),
    }
}

/// Возвращает список причин аннулирования
#[cfg(feature = "advanced-cancellation-control")]
async fn fetch_reasons(
    master_data_service: &MasterDataService,
    ids: Vec<i32>,
) -> Result<ApiResponse<Vec<PlanReasonCancel>, ()>> {
    master_data_service
        .plan_reasons_cancel_search(SearchPlanReasonsCancelRabbitReq {
            ids: Some(ids),
            check_reason_id: None,
        })
        .await
        .map_err(Into::into)
}

#[cfg(feature = "advanced-cancellation-control")]
async fn examine_cancel_data(
    items: &[ObjectIdentifierWithStatusNote],
    messages: &mut Messages,
    proc_ctx: &ProcessingCtx,
    master_data_service: &MasterDataService,
) -> Result<()> {
    let plans = items
        .iter()
        .filter(|i| i.object_type == EntityKind::Plan)
        .collect::<Vec<_>>();

    if plans.is_empty() {
        return Ok(());
    }

    let reason_ids: Vec<i32> =
        plans.iter().filter_map(|i| i.plan_reason_cancel_id).collect();

    let reasons_result =
        fetch_reasons(master_data_service, reason_ids.clone()).await?;

    let reasons_map: AHashMap<i32, PlanReasonCancel> = reasons_result
        .data
        .into_iter()
        .filter_map(|reason| reason.header.id.map(|id| (id, reason)))
        .collect();

    for id in &reason_ids {
        if !reasons_map.contains_key(id) {
            return Err(ProcessingError::InternalError(format!(
                "Причина аннулирования с ID {} отсутствует в справочнике",
                id
            )));
        }
    }

    let replaced_plan_ids: AHashSet<i64> =
        plans.iter().filter_map(|i| i.plan_replaced_id).collect();

    let replaced_plans_map: AHashMap<i64, PlanOrAmendment> =
        if !replaced_plan_ids.is_empty() {
            let select = Select::full::<Plan>()
                .in_any(Plan::id, replaced_plan_ids.iter().cloned());
            let plans = PlanOrAmendment::select(&select, &proc_ctx.db_pool).await?;
            plans.into_iter().map(|p| (*p.id(), p)).collect()
        } else {
            Default::default()
        };

    for item in plans {
        // Проверка 1: Если не заполнена причина аннулирования
        let Some(reason_id) = item.plan_reason_cancel_id else {
            messages.add_prepared_message(
                    PlanCancelMessage::MissingCancelReason.singular(
                        &PlanOrAmendment::Plan(Plan {
                            id: item.id,
                            ..Default::default()
                        }),
                    ),
                );
                 continue;
        };

        let reason = match reasons_map.get(&reason_id) {
            Some(reason) => reason,
            None => {
                continue;
            }
        };

        // Проверка 3: Если причина требует новую ППЗ, а она не указана
        if reason.header.is_new_plan.unwrap_or(false)
            && item.plan_replaced_id.is_none()
        {
            messages.add_prepared_message(
                PlanCancelMessage::MissingIsNewPlan.singular(
                    &PlanOrAmendment::Plan(Plan {
                        id: item.id,
                        ..Default::default()
                    }),
                ),
            )
        }

        // Проверка 6: Если указана замещающая ППЗ, проверить ее существование
        if let Some(replaced_id) = item.plan_replaced_id {
            if !replaced_plans_map.contains_key(&replaced_id) {
                messages.add_message(
                    MessageKind::Error,
                    format!("ППЗ № {} не найдена", replaced_id),
                );
            }
        }

        // Проверки по `check_reason_id`
        match reason.header.check_reason_id.map(CheckReason::from) {
            Some(CheckReason::Publication) => {
                // Проверка 4: TODO: Реализовать проверку статуса публикации АПЗ в ЕИС.
                // необходимо проверить в таблице aggregated_plan наличие номера ППЗ plan-id = aggregated_plan-id и статуса АПЗ, должно быть опубликовано
                // aggregated_plan - таблица монолита
                // messages.add_message(MessageKind::Error, format!("ППЗ {} не опубликована в ЕИС", item.id));
            }
            Some(CheckReason::PriceSchedule) => {
                // Проверка 5: Если для выбранной причины аннулирования проверить заполнено ли поле «Прейскурантная закупка» (plan-is_list_price)
                // у ППЗ указанной в поле «Новый номер ППЗ/ДС» (plan-plan_replaced_id)
                if let Some(replaced_id) = item.plan_replaced_id {
                    if let Some(_replaced_plan) =
                        replaced_plans_map.get(&replaced_id)
                    {
                        if !_replaced_plan.is_list_price().unwrap_or(&false) {
                            messages.add_message(
                                MessageKind::Error,
                                format!("У ППЗ {} не заполнен признак «Прейскурантная закупка»", replaced_id),
                            );
                        }
                    }
                }
            }
            _ => {
                // CheckReason::Protocol, CheckReason::Unknown и None -
                // не требуют дополнительных проверок, пропускаем
            }
        }
    }

    Ok(())
}

/// По полученному списку ППЗ/ДС необходимо с помощью функции «Получение данных по Повестке» найти данные по ППЗ/ДС,
/// которые включены в крайнюю сформированную и не удаленную Повестку/is_removed = false, где по ППЗ/ДС в позиции
/// Повестки не указан признак «Удалена»/is_removed. у данной позиции Повестки по ППЗ/ДС нет связи с позицией
/// Протокола в таблице item_relation_agenda_protocol.
///
///Если Повестка найдена и находится в статусе/status_id = 100/«Сформирована»,
/// то при помощи функции «Обновление Повестки» по позиции Повестки
/// (по ППЗ/ДС) автоматически устанавливается признак «Удалено»/is_remove = true
///
/// Если Повестка найдена и находится в статусе/status_id = 200/«Отправлена»,
/// то при помощи функции «Обновление Повестки» по ППЗ/ДС автоматически
/// устанавливается признак «Снято с рассмотрения»/is_excluded = true,
/// очищается поле reviewed_at/«Время проведения» в таблице agenda_item
async fn update_agenda_items(
    plans: &[PlanOrAmendment],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let agenda_with_items =
        fetch_agenda_items(plans, Some(false), None, recorder.tx()).await?;

    common::agenda::update_agenda_items(
        agenda_with_items,
        |agenda_item| {
            let JoinedEcAgendaItem {
                mut agenda_item,
                agenda,
                item_agenda_protocol_rel,
            } = agenda_item;

            if item_agenda_protocol_rel.is_some() {
                return None;
            }

            match agenda.status_id {
                EcAgendaStatus::Formed => {
                    agenda_item.is_removed = true;
                    Some(agenda_item)
                }
                EcAgendaStatus::Sent => {
                    agenda_item.is_excluded = true;
                    agenda_item.reviewed_at = None;
                    Some(agenda_item)
                }
                _ => None,
            }
        },
        &[
            EcAgendaItem::is_removed,
            EcAgendaItem::is_excluded,
            EcAgendaItem::reviewed_at,
        ],
        messages,
        recorder,
    )
    .await?;

    Ok(())
}

/// Если ППЗ/ДС включена в Протокол (protocol_type_id = 2) с наивысшей датой создания,
/// который не удален/is_removed = false, то по позиции Протокола (тоже не удалена/is_removed = false)
/// к которой относится ППЗ/ДС установить признак снят с рассмотрения/is_excluded = true, если он ранее уже не был установлен.
async fn update_protocol_item(
    joined_protocols: Vec<JoinedEcProtocolItem>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    common::protocol::update_protocol_items(
        joined_protocols,
        |mut protocol_item| {
            protocol_item.item.is_excluded = true;
            Some(protocol_item.item)
        },
        &[EcProtocolItem::is_excluded],
        messages,
        recorder,
    )
    .await?;

    Ok(())
}

fn examine_protocol(
    section_id: Section,
    protocol_item: &JoinedEcProtocolItem,
    plan: &PlanOrAmendment,
) -> Option<Message> {
    let JoinedEcProtocolItem { item, protocol } = protocol_item;

    let msg = if section_id == Section::EstimatedCommissionCorrespondence {
        match protocol.status_id {
            EcProtocolStatus::Formed | EcProtocolStatus::AgreementPending => {
                PlanCancelMessage::AlreadyInProtocolWarn(protocol).into()
            }
            _ => PlanCancelMessage::AlreadyInProtocolErr(protocol).into(),
        }
    } else {
        match item.result_id {
            ResultId::NotAgreed => None,
            _ => PlanCancelMessage::AlreadyInProtocolErr(protocol).into(),
        }
    };

    msg.map(|m| m.singular(plan))
}

fn finalise(
    plans: Vec<PlanOrAmendment>,
    fields: Option<&[&str]>,
    messages: Messages,
) -> Result<ApiResponse<CancelPlansResponseData, ()>> {
    let data = if messages.is_error() {
        Vec::new()
    } else {
        plans
            .into_iter()
            .map(|p| {
                PlanOrAmendmentRep::from_item_with_section_mapping(
                    p,
                    SectionKind::EstimatedCommission,
                    fields,
                )
            })
            .collect::<Vec<_>>()
    };

    Ok(ApiResponse {
        status: Status::Ok,
        data: data.into(),
        messages,
        objects: vec![],
    })
}
