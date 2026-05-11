use std::sync::Arc;

use ahash::AHashMap;
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::db_item::{joined::JoinTo, Select};
use shared_essential::{
    application::records::{ProcessUpsert, Recorder},
    domain::{
        CommissionKind, ContractAmendment, EcAgenda, EcAgendaItem, EcAgendaStatus,
        JoinedEcAgendaEcAgendaItem as AgendaWithItems,
        JoinedEcAgendaEcAgendaItemSelector as AgendaWithItemsSelect, Plan,
        PlanOrAmendment, PlanOrAmendmentRep, PricingUnitId, SectionKind,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{
            AddPlansAgendaReq, AddPlansAgendaResponse, PreAddPlansAgendaReq,
            PreAddPlansAgendaResponse,
        },
        response_request::*,
    },
};

use crate::{
    app_process::{
        records::{PlanCollectedUpdate, ProcessingRulesChecker},
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::agenda::AgendaAddPlansMessage,
};

use super::create_agenda;

const PRE_ADD_PLANS_AGENDA: &str = "v1/pre_request/add_plans_agenda";
const ADD_PLANS_AGENDA: &str = "v1/action/add_plans_agenda";

const RESPONSE_FIELD_LIST: &[&str] = &[
    "plan_id",
    Plan::customer_id,
    Plan::contract_subject,
    Plan::pricing_expert_id,
    Plan::supplier_id,
    Plan::sum_excluded_vat,
    Plan::currency_id,
    Plan::pricing_organization_unit_id,
    Plan::commission_date,
    Plan::status_id,
    Plan::section_id,
    ContractAmendment::delta_sum_excluded_vat,
];

const OLD_ITEM_UPDATE_FIELDS: &[&str] = &[
    EcAgendaItem::is_registered_by_d647,
    EcAgendaItem::is_removed,
    EcAgendaItem::is_excluded,
    EcAgendaItem::changed_by,
    EcAgendaItem::changed_at,
    EcAgendaItem::number,
    EcAgendaItem::sum_excluded_vat,
    EcAgendaItem::pricing_sum_excluded_vat,
];

/// # Описание
///
/// Проверка возможности добавления ППЗ/ДС в Повестку СК
///
/// # Возвращает
/// * Ok([`ApiResponse<PreAddPlansAgendaResponse, ()>`]) - Ответ ППЗ/ДС, которые можно добавить в Повестку СК
/// * Ok([`ApiResponse<PreAddPlansAgendaResponse, ()>`]) c пустым `data` - Ошибка при валидации запроса
/// * Err([`ProcessingError`]) - Ошибка при процессинге запроса
pub(crate) async fn pre_add_plans_agenda(
    request: PreAddPlansAgendaReq,
    db_pool: Arc<PgPool>,
) -> Result<PreAddPlansAgendaResponse> {
    let (plans, messages) =
        pre_add_plans_agenda_inner(request, None, PRE_ADD_PLANS_AGENDA, &db_pool)
            .await?;

    let response = finalise_response(plans, messages)?;
    Ok(response)
}

/// # Описание
///
/// Добавление ППЗ/ДС в Повестку СК
///
/// # Возвращает
/// * Ok([`ApiResponse<AddPlansAgendaResponse, ()>`]) - Успешное добавление ППЗ/ДС в Повестку СК
/// и возвращение списка добавленных ППЗ/ДС
/// * Ok([`ApiResponse<AddPlansAgendaResponse, ()>`]) c пустым `data` - Ошибка при валидации запроса
/// * Err([`ProcessingError`]) - Ошибка при процессинге запроса
pub(crate) async fn add_plans_agenda(
    request: AddPlansAgendaReq,
    proc_ctx: ProcessingCtx,
) -> Result<AddPlansAgendaResponse> {
    let agenda_with_items =
        fetch_agenda_with_all_items(request.agenda_id, &proc_ctx.db_pool).await?;
    let AgendaWithItems {
        agenda,
        agenda_items: old_agenda_items,
    } = agenda_with_items;

    let (plans, mut messages) = pre_add_plans_agenda_inner(
        request.item_list,
        Some(&agenda),
        ADD_PLANS_AGENDA,
        &proc_ctx.db_pool,
    )
    .await?;

    if messages.is_error()
        || (messages.kind == MessageKind::Warning && !request.is_force)
    {
        return Ok(ApiResponse::default().with_messages(messages));
    }
    messages.clear();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .begin()
        .await?;

    let (updated_plans, agenda, _, _) = insert_agenda_items(
        agenda,
        old_agenda_items,
        plans,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    recorder.commit().await?;

    AgendaAddPlansMessage::Success(&agenda)
        .checked_append(&mut messages, &updated_plans);

    let ids = updated_plans.into_iter().map(|p| *p.id()).collect::<Vec<_>>();
    Ok((ids, messages).into())
}

pub(crate) async fn pre_add_plans_agenda_inner(
    request: Vec<ObjectIdentifier>,
    agenda: Option<&EcAgenda>,
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
    examine_agenda_items(&mut valid_plan_map, agenda, &mut message_buf, db_pool)
        .await?;

    if let Some(agenda) = agenda {
        examine_agenda_status(agenda, &mut message_buf);
        examine_agenda_pricing_unit(
            &valid_plan_map,
            agenda,
            &mut message_buf,
            |plans_with_warn, agenda| {
                AgendaAddPlansMessage::DifferentDepartment(agenda)
                    .resolve(plans_with_warn)
                    .expect("examine_agenda_pricing_unit гарантирует непустой plans_with_warn")
            },
        );
    }

    let plans = request
        .iter()
        .filter_map(|oid| valid_plan_map.remove(&oid.uuid))
        .collect();

    Ok((plans, message_buf))
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
            AgendaAddPlansMessage::InvalidPlanStatus
                .resolve(&invalid_plans)
                .expect("invalid_plans гарантированно непустой")
        },
    )
    .await
}

/// По ППЗ/ДС проверить наличие Протоколов (protocol_type_id = 1/Протокол очного заседания СК).
/// Если Протокол отсутствует, то перейти к следующей проверке. Если присутствует,
/// то проверить по ППЗ/ДС значение в поле «Решение СК».
pub(crate) async fn examine_protocol_items(
    plan_map: &mut AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    create_agenda::examine_protocols(
        plan_map,
        messages,
        db_pool,
        |protocol, protocol_item, plan| {
            AgendaAddPlansMessage::AlreadyInProtocol(protocol, protocol_item)
                .singular(plan)
        },
    )
    .await
}

/// По ППЗ/ДС проверить наличие Повесток. Если Повестка отсутствует,
/// то перейти к следующей проверке. Если присутствует,
/// то проверить по ППЗ/ДС значение в поле «Снято с рассмотрения».
pub(crate) async fn examine_agenda_items(
    plan_map: &mut AHashMap<Uuid, PlanOrAmendment>,
    agenda_to_add: Option<&EcAgenda>,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    create_agenda::examine_agendas(
        plan_map,
        messages,
        db_pool,
        |agenda, _, plan| match agenda_to_add {
            Some(agenda_to_add) if agenda_to_add.id == agenda.id => {
                AgendaAddPlansMessage::AlreadyInCurrentAgenda(agenda)
                    .singular(plan)
                    .into()
            }
            _ => {
                AgendaAddPlansMessage::AlreadyInAgenda(agenda).singular(plan).into()
            }
        },
    )
    .await
}

/// Проверка на совпадение pricing_organization_unit_id ППЗ/ДС и Повестки
///
/// Гарантирует что `message_fn` принимает непустой массив ППЗ/ДС
pub(super) fn examine_agenda_pricing_unit<F>(
    plan_map: &AHashMap<Uuid, PlanOrAmendment>,
    agenda: &EcAgenda,
    messages: &mut Messages,
    message_fn: F,
) where
    F: FnOnce(&[&PlanOrAmendment], &EcAgenda) -> Message,
{
    // Если значение в полях pricing_organization_unit_id у выбранной позиции
    // ППЗ/ДС и пришедшем с FE значения поля Повестки не совпадают, то переходим
    // к следующей проверке.
    if plan_map.values().all(|plan| {
        *plan.pricing_organization_unit_id() == agenda.pricing_organization_unit_id
    }) {
        return;
    }

    let plans_with_warn = plan_map
        .values()
        .filter(|plan| {
            match (
                plan.pricing_organization_unit_id(),
                agenda.pricing_organization_unit_id,
            ) {
                (PricingUnitId::D646, PricingUnitId::D647)
                | (PricingUnitId::D647, PricingUnitId::D646) => true,
                // По позиции ППЗ/ДС выполняем проверку на значение поля "Раздел плана"/section_id в таблицах plan
                //  - для ППЗ/ contract_amendment - для ДС. Если значение поля "Раздел плана"/section_id = 4.3 (11)
                // или 4.6 (14), то переходим к следующей проверке, иначе формируем предупреждающее сообщение
                (PricingUnitId::Gpk, PricingUnitId::D646) => {
                    !matches!(plan.section_id(), 11 | 14)
                }
                // По позиции ППЗ/ДС выполняем проверку на значение поля "Раздел плана"/section_id
                // в таблицах plan - для ППЗ/ contract_amendment - для ДС. Если значение поля
                // "Раздел плана"/section_id ≠ 4.3 (11) или 4.6 (14), то переходим к следующей
                // проверке, иначе формируем предупреждающее сообщение
                (PricingUnitId::Gpk, PricingUnitId::D647) => {
                    matches!(plan.section_id(), 11 | 14)
                }
                _ => false,
            }
        })
        .collect::<Vec<_>>();

    if !plans_with_warn.is_empty() {
        messages.add_prepared_message(message_fn(&plans_with_warn, agenda));
    }
}

/// По полученному значению в запросе  agenda_id, найти значение в
/// таблице agenda и проверить статус/status_id. Если статус/status_id = 300/400,
/// то сформировать сообщение
fn examine_agenda_status(agenda: &EcAgenda, messages: &mut Messages) {
    if matches!(
        agenda.status_id,
        EcAgendaStatus::Deleted | EcAgendaStatus::ProtocolFormed
    ) {
        messages.add_prepared_message(
            AgendaAddPlansMessage::invalid_agenda_status(agenda),
        );
    }
}

/// # Описание
///
/// Добавление ППЗ/ДС в Повестку СК в базу данных
///
/// # Возвращает
/// * Ok(`(обновленные ППЗ/ДС, обновленную Повестку СК, новые элементы Повестки СК, старые элементы Повестки СК)`) - Успешное добавление ППЗ/ДС в Повестку СК и возвращение
/// списка добавленных ППЗ/ДС
/// * Err([`ProcessingError`]) - Внутренняя ошибка сервиса
pub(crate) async fn insert_agenda_items(
    agenda: EcAgenda,
    mut old_agenda_items: Vec<EcAgendaItem>,
    plans: Vec<PlanOrAmendment>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<(Vec<PlanOrAmendment>, EcAgenda, Vec<EcAgendaItem>, Vec<EcAgendaItem>)>
{
    let mut old_item_checker = old_agenda_items
        .iter_mut()
        .map(|item| (item.source_uuid, item))
        .collect::<AHashMap<_, _>>();

    // Новые элементы все с is_registered_by_d647=false, поэтому вставляются
    // после старых с is_registered_by_d647=false
    let new_agenda_items = plans
        .iter()
        .filter(|item| !old_item_checker.contains_key(item.uuid()))
        .map(|item| {
            let (sum_excluded_vat, pricing_sum_excluded_vat) = match item {
                PlanOrAmendment::Plan(p) => {
                    (Some(p.sum_excluded_vat), Some(p.pricing_sum_excluded_vat))
                }
                PlanOrAmendment::Amendment(a) => (
                    Some(a.delta_sum_excluded_vat),
                    a.pricing_delta_sum_excluded_vat,
                ),
            };

            EcAgendaItem {
                // Будет установлено в insert_agenda_items_inner
                number: 0,
                agenda_uuid: agenda.uuid,
                source_uuid: *item.uuid(),
                // TODO: удалить?
                pricing_sum_excluded_vat,
                sum_excluded_vat,
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();

    // Старые элементы которые были с i.is_removed=true нужно перевести на is_removed=false и подтянуть
    // суммы из ППЗ/ДС
    for plan in plans.iter() {
        if let Some(old_agenda_item) = old_item_checker.get_mut(plan.uuid()) {
            let (sum_excluded_vat, pricing_sum_excluded_vat) = match plan {
                PlanOrAmendment::Plan(p) => {
                    (Some(p.sum_excluded_vat), Some(p.pricing_sum_excluded_vat))
                }
                PlanOrAmendment::Amendment(a) => (
                    Some(a.delta_sum_excluded_vat),
                    a.pricing_delta_sum_excluded_vat,
                ),
            };

            old_agenda_item.is_removed = false;
            old_agenda_item.is_excluded = false;
            old_agenda_item.is_registered_by_d647 = false;
            old_agenda_item.sum_excluded_vat = sum_excluded_vat;
            old_agenda_item.pricing_sum_excluded_vat = pricing_sum_excluded_vat;
        }
    }

    insert_agenda_items_inner(
        agenda,
        new_agenda_items,
        old_agenda_items,
        plans,
        messages,
        recorder,
        handler,
    )
    .await
}

/// Внутрення функция для добавления ППЗ/ДС в Повестку СК
///
/// Возвращает (обновленные ППЗ/ДС, обновленную Повестку СК, новые элементы Повестки СК, старые элементы Повестки СК)
#[allow(clippy::too_many_arguments)]
pub(crate) async fn insert_agenda_items_inner(
    mut agenda: EcAgenda,
    mut new_agenda_items: Vec<EcAgendaItem>,
    mut old_agenda_items: Vec<EcAgendaItem>,
    mut plans: Vec<PlanOrAmendment>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<(Vec<PlanOrAmendment>, EcAgenda, Vec<EcAgendaItem>, Vec<EcAgendaItem>)>
{
    let mut new_number_sequence = 1;
    let mut update_number =
        |items: &mut [EcAgendaItem], is_registered_by_d647, is_removed| {
            items
                .iter_mut()
                .filter(|i| {
                    i.is_registered_by_d647 == is_registered_by_d647
                        && i.is_removed == is_removed
                })
                .for_each(|item| {
                    item.number = new_number_sequence;
                    new_number_sequence += 1;
                })
        };

    update_number(&mut old_agenda_items, false, false);
    update_number(&mut new_agenda_items, false, false);
    update_number(&mut old_agenda_items, false, true);
    update_number(&mut old_agenda_items, true, false);
    update_number(&mut new_agenda_items, true, false);
    update_number(&mut old_agenda_items, true, true);

    // Обновление `comission_date` для `plans` и `contract_amendment`
    plans.iter_mut().for_each(|plan| {
        *plan.commission_date_mut() = Some(agenda.meeting_date);
        *plan.commission_kind_id_mut() = CommissionKind::InPerson;
    });

    let new_agenda_items =
        recorder.process_insert(new_agenda_items, messages).await?;

    let updated_old_agenda_items = recorder
        .process_update(old_agenda_items, OLD_ITEM_UPDATE_FIELDS, messages)
        .await?;

    let updated_plans = PlanOrAmendment::update(
        plans,
        &[Plan::commission_date, Plan::commission_kind_id],
        messages,
        recorder,
        handler,
    )
    .await?;

    // других изменений в повестке нет, но надо вручную пометить как измененную.
    agenda.apply_update_ctx(&recorder.ctx());
    let agenda = recorder
        .process_update(
            vec![agenda],
            &[EcAgenda::changed_by, EcAgenda::changed_at],
            messages,
        )
        .await?
        .remove(0);

    Ok((updated_plans, agenda, new_agenda_items, updated_old_agenda_items))
}

fn finalise_response(
    plans: Vec<PlanOrAmendment>,
    messages: Messages,
) -> Result<PreAddPlansAgendaResponse> {
    let plans = if messages.is_error() {
        Vec::new()
    } else {
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

    Ok((plans, messages).into())
}

/// Получение Повестки и всех принадлежащих к ней agenda_item,
/// даже с is_removed=true
pub(crate) async fn fetch_agenda_with_all_items(
    id: i64,
    db_pool: &PgPool,
) -> Result<AgendaWithItems> {
    let agenda_select =
        Select::with_fields([EcAgenda::id, EcAgenda::uuid, EcAgenda::meeting_date])
            .eq(EcAgenda::id, id);

    let agenda_item_select = Select::full::<EcAgendaItem>();

    let mut joined_agenda = AgendaWithItemsSelect::new(agenda_select)
        .set_agenda_items(
            EcAgendaItem::join_default().selecting(agenda_item_select),
        )
        .get(db_pool)
        .await?;

    if joined_agenda.is_empty() {
        return Err(ProcessingError::AddPlansAgenda(format!(
            "Повестка СК с идентификатором {} не найдена",
            id
        )));
    }

    Ok(joined_agenda.remove(0))
}
