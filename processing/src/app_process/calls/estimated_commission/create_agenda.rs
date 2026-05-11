//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
use std::{collections::hash_map::Entry, sync::Arc};

use ahash::AHashMap;
use sqlx::{types::Uuid, PgPool};

use asez2_shared_db::{
    db_item::{joined::JoinTo, selection::*, AsezDate, AsezTimestamp},
    DbItem, Value,
};
use shared_essential::{
    domain::{
        legacy::plans::PlanStatus,
        tables::processing::partner_type_commission::PartnerTypeCommission,
        CommissionKind, ContractAmendment, EcAgenda, EcAgendaItem, EcAgendaStatus,
        EcPartner, EcProtocol, EcProtocolItem,
        JoinedEcAgendaItemEcAgendaRelAgendaProtocolItemSelector,
        JoinedEcProtocolItemEcProtocolSelector, Plan, PlanOrAmendment,
        PlanOrAmendmentRep, PricingUnitId, ProtocolType, ResultId, SectionKind,
        StatusHistory,
    },
    presentation::dto::{
        general::ObjectIdentifier, processing::*, response_request::*,
    },
};

use crate::{
    app_process::{
        common::{self, agenda::AgendaPricingUnitCheck, plan::fetch_plans_by_ids},
        records::PlanCollectedUpdate,
        sections::mapping::SectionMapExt,
    },
    common::{op_with_numbers, EcObjectType, NumberRequest, ProcessingCtx, Result},
    presentation::business_messages::agenda::AgendaCreateMessage,
};

/// The fields which are returned from the precheck.
const PRECHECK_FIELDS: &[&str] = &[
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
    ContractAmendment::delta_sum_excluded_vat,
];

const AGENDA_CREATE: &str = "v1/action/create_agenda";

/// Client FE demands a list of ids of the created agenda items.
/// NB: It is possible that returning everything will improve the system's efficiency.
pub(crate) async fn create_agenda(
    request: CreateAgendaReq,
    proc_ctx: ProcessingCtx,
) -> Result<CreateAgendaResponse> {
    tracing::info!(
        kind = "get",
        "Получен запрос на создание Повестки СК ({get}): {req:?}\n",
        get = AGENDA_CREATE,
        req = request,
    );

    let CreateAgendaReq {
        user_id,
        is_force,
        meeting_date,
        item_list,
    } = request;

    let (plans, mut messages) =
        pre_create_agenda_inner(&item_list, &proc_ctx.db_pool).await?;

    if (messages.kind == MessageKind::Warning && !is_force)
        || messages.kind == MessageKind::Error
    {
        return Ok(ApiResponse {
            status: Status::Ok,
            messages,
            ..Default::default()
        });
    }

    messages.clear();

    let (updated_plans, agenda_items_ids, agenda) =
        insert_and_return(plans, user_id, meeting_date, &proc_ctx, &mut messages)
            .await
            .map_err(|e| {
                tracing::error!(
                    kind = "insert",
                    "Ошибка при создании Повестки СК и ее элементов: {:#?}",
                    e
                );
                e
            })?;

    AgendaCreateMessage::Success(&agenda)
        .checked_append(&mut messages, &updated_plans);

    Ok((agenda_items_ids, messages).into())
}

/// This function acts somewhat like create_agenda, but only runs checks.
/// In addition it runs a check vs the "Очная СК" section.
pub(crate) async fn pre_create_agenda(
    request: PreCreateAgendaReq,
    nest: Arc<PgPool>,
) -> Result<PreCreateAgendaResponse> {
    tracing::info!(
        kind = "get",
        "Получен предзапрос на создание Повестки СК {get}): {req:?}\n",
        get = AGENDA_CREATE,
        req = request,
    );

    let (plans, messages) =
        pre_create_agenda_inner(&request.item_list, nest.as_ref()).await?;

    if messages.kind >= MessageKind::Error {
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
                Some(PRECHECK_FIELDS),
            )
        })
        .collect::<Vec<_>>();

    Ok((data, messages).into())
}

async fn pre_create_agenda_inner(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Messages)> {
    let mut messages = Messages::default();

    let mut plans = fetch_and_examine_plans(
        item_list,
        &mut messages,
        db_pool,
        |invalid_plans| {
            AgendaCreateMessage::InvalidPlanStatus
                .resolve(&invalid_plans)
                .expect("invalid_plans гарантированно непустой")
        },
    )
    .await?;
    examine_protocols(
        &mut plans,
        &mut messages,
        db_pool,
        |protocol, protocol_item, plan| {
            AgendaCreateMessage::AlreadyInProtocol(protocol, protocol_item)
                .singular(plan)
        },
    )
    .await?;
    examine_agendas(&mut plans, &mut messages, db_pool, |agenda, item, plan| {
        if item.is_excluded {
            // При создании повестки мы игнорируем ППЗ/ДС, исключенные из других повесток
            None
        } else {
            AgendaCreateMessage::AlreadyInAgenda(agenda).singular(plan).into()
        }
    })
    .await?;

    examine_pricing_unit(&plans, &mut messages);

    let plans =
        item_list.iter().filter_map(|oid| plans.remove(&oid.uuid)).collect();

    Ok((plans, messages))
}

/// Second part of the task is to insert the required agenda items into the
/// database. Here the number range is created and these IDs are used in the
/// process of inserting the agendas.
/// if the operation fails at any point, the transaction is dropped and the
/// inserted numbers are rolled back.
///
/// Since the contract requires only that we return the ids (not even the uuid),
/// It is not necessary to return agenda items.
async fn insert_and_return(
    joined_data: Vec<PlanOrAmendment>,
    user_id: i32,
    meeting_date: AsezDate,
    proc_ctx: &ProcessingCtx,
    messages: &mut Messages,
) -> Result<(Vec<PlanOrAmendment>, Vec<i64>, EcAgenda)> {
    let base_pricing_organization_unit_id =
        joined_data[0].pricing_organization_unit_id();
    let pricing_organization_unit_id = if joined_data.iter().all(|x| {
        x.pricing_organization_unit_id() == base_pricing_organization_unit_id
    }) {
        *base_pricing_organization_unit_id
    } else {
        PricingUnitId::Undefined
    };
    let now = AsezTimestamp::now();

    let mut new_agenda = EcAgenda {
        // NB: The id *must* be set `op_with_numbers.`
        id: Default::default(),
        status_id: EcAgendaStatus::Formed,
        pricing_organization_unit_id,
        meeting_date,
        // TODO: use Recorder for these fields
        uuid: Uuid::new_v4(),
        is_removed: false,
        created_by: user_id,
        changed_by: user_id,
        changed_at: now,
        created_at: now,
    };

    let mut counter = 0;
    let mut new_agenda_items = joined_data
        .iter()
        .map(|x| {
            counter += 1;

            let (sum_excluded_vat, pricing_sum_excluded_vat) = match x {
                PlanOrAmendment::Plan(p) => {
                    (Some(p.sum_excluded_vat), Some(p.pricing_sum_excluded_vat))
                }
                PlanOrAmendment::Amendment(a) => (
                    Some(a.delta_sum_excluded_vat),
                    a.pricing_delta_sum_excluded_vat,
                ),
            };

            EcAgendaItem {
                agenda_uuid: new_agenda.uuid,
                source_uuid: *x.uuid(),
                number: counter,
                sum_excluded_vat,
                pricing_sum_excluded_vat,
                // TODO: use Recorder for these fields
                uuid: Uuid::new_v4(),
                is_excluded: false,
                is_removed: false,
                is_registered_by_d647: false,
                created_by: user_id,
                changed_by: user_id,
                changed_at: now,
                created_at: now,
                reviewed_at: None,
            }
        })
        .collect::<Vec<EcAgendaItem>>();

    // Также создаем новых участников СК
    let partners_select = Select::with_fields([
        PartnerTypeCommission::user_id,
        PartnerTypeCommission::role_id,
    ])
    .eq(PartnerTypeCommission::protocol_type_id, 1);
    let partners =
        PartnerTypeCommission::select(&partners_select, &*proc_ctx.db_pool).await?;
    let mut new_ec_partners = partners
        .into_iter()
        .map(|p| EcPartner {
            protocol_agenda_uuid: new_agenda.uuid,
            user_id: p.user_id,
            role_id: p.role_id,
            // TODO: use Recorder for these fields
            e_mail: None,
            is_checked_in: false,
            is_removed: false,
            uuid: Uuid::new_v4(),
            created_at: now,
            changed_at: now,
            created_by: user_id,
            changed_by: user_id,
        })
        .collect::<Vec<_>>();

    // TODO: use Recorder for these items
    let mut new_status_history = StatusHistory {
        uuid: Uuid::new_v4(),
        comment: String::new(),
        object_uuid: new_agenda.uuid,
        status_id: EcAgendaStatus::Formed.into(),
        created_at: now,
        created_by: user_id,
    };

    let to_update_plans = joined_data
        .into_iter()
        .map(|mut x| {
            *x.commission_date_mut() = Some(new_agenda.meeting_date);
            x.clone()
        })
        .collect();
    // We need one ID for the agenda, and N IDs for N agenda items, hence 1 + N.
    let num_requests = vec![NumberRequest::new(EcObjectType::Agenda, 1)];

    let recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let (updated_plans, agenda, item_ids) =
        op_with_numbers(recorder, num_requests, |ids, recorder| {
            Box::pin(async move {
                // Agenda must be inserted first or we can't use its uuid.
                new_agenda.id = ids.get(&EcObjectType::Agenda).unwrap()[0];

                let agenda =
                    EcAgenda::insert_returning(&mut new_agenda, recorder.tx())
                        .await?;

                let agenda_items = EcAgendaItem::insert_vec_returning(
                    &mut new_agenda_items,
                    recorder.tx(),
                )
                .await?;

                EcPartner::insert_vec(&mut new_ec_partners, recorder.tx()).await?;

                StatusHistory::insert(&mut new_status_history, recorder.tx())
                    .await?;

                let updated_plans = PlanOrAmendment::update(
                    to_update_plans,
                    &[Plan::commission_date],
                    messages,
                    recorder,
                    proc_ctx.create_rules_checker(),
                )
                .await?;

                let ids: Vec<_> = agenda_items.iter().map(|x| x.number).collect();
                Ok((updated_plans, agenda, ids))
            })
        })
        .await?;

    Ok((updated_plans, item_ids, agenda))
}

/// 1) Если в ППЗ/ДС указан статус/status_id отличный от статусов модуля АЦ (Перечень
/// статусов (и их код) в ППЗ/ДС модуля АЦ приведен в таблице 14) и от статуса модуля
/// СК «Сметная комиссия. Очная СК»/251, то формируем ошибку: «ППЗ/ДС <Номер ППЗ/ДС>
/// находится на статусе <текущий статус ППЗ/ДС>. Добавление в Повестку/Создание
/// Повестки/Изменение Повестки (в зависимости от нажимаемой кнопки) запрещено.»
///
/// `message_fn` гарантированно получает в качестве аргумента непустой массив
/// невалидных элементов
fn examine_plan_status<F>(
    plans_amendments: Vec<PlanOrAmendment>,
    messages: &mut Messages,
    message_fn: F,
) -> Vec<PlanOrAmendment>
where
    F: FnOnce(Vec<PlanOrAmendment>) -> Message,
{
    let (invalid_plans, valid_plans): (Vec<_>, Vec<_>) =
        plans_amendments.into_iter().partition(|pa| {
            !matches!(
                pa.status_id(),
                PlanStatus::PriceConfirmed
                    | PlanStatus::PriceDetermined
                    | PlanStatus::ExecutorAppointmentD646
                    | PlanStatus::ExecutorAppointedD646
                    | PlanStatus::AnalysisPerformedD646
                    | PlanStatus::AnalysisCompletedD646
                    | PlanStatus::EstimatedCommissionInPerson
                    | PlanStatus::ExecutorAppointmentD647
                    | PlanStatus::ExecutorAppointedD647
                    | PlanStatus::AnalysisPerformedD647
                    | PlanStatus::AnalysisCompletedD647
                    | PlanStatus::ExecutorAppointmentMTP
                    | PlanStatus::ExecutorAppointedMTP
                    | PlanStatus::AnalysisPerformedMTP
                    | PlanStatus::AnalysisCompletedMTP
            ) || *pa.commission_kind_id() != CommissionKind::InPerson
        });

    if !invalid_plans.is_empty() {
        messages.add_prepared_message(message_fn(invalid_plans));
    }

    valid_plans
}

pub(crate) async fn fetch_and_examine_plans<F>(
    items: &[ObjectIdentifier],
    messages: &mut Messages,
    db_pool: &PgPool,
    message_fn: F,
) -> Result<AHashMap<Uuid, PlanOrAmendment>>
where
    F: FnOnce(Vec<PlanOrAmendment>) -> Message,
{
    let plans = fetch_plans_by_ids(items, db_pool).await?;

    let valid_plans = examine_plan_status(plans, messages, message_fn);
    let plan_map =
        valid_plans.into_iter().map(|item| (*item.uuid(), item)).collect();

    Ok(plan_map)
}

fn examine_pricing_unit(
    items: &AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
) {
    if let Err(msg) = common::agenda::examine_pricing_unit(items.values()) {
        let msg = match msg {
            AgendaPricingUnitCheck::DifferentDepartment => {
                AgendaCreateMessage::different_department()
            }
            AgendaPricingUnitCheck::DifferentSections => {
                AgendaCreateMessage::different_plan_sections()
            }
        };
        messages.add_prepared_message(msg);
    }
}

/// 2) По ППЗ/ДС проверить наличие Протоколов (protocol_type_id = 1/Протокол
/// очного заседания СК). Если Протокол отсутствует, то перейти к следующей
/// проверке. Если присутствует, то проверить по ППЗ/ДС значение в поле «Решение
/// СК».
///
/// Алгоритм: Читать запись по ППЗ/ДС из таблицы protocol_item, где
/// protocol_item – uuid = uuid входного параметра ППЗ/ДС и protocol_item –
/// protocol_uuid = protocol – uuid найденной не удаленной записи Протокола
/// (prtocol - is_removed = false) с наивысшей датой создания. По найденной не
/// удаленной записи (protoocl_item - is_removed = false) в protocol_item (
/// проверяем значение поля result_id. Если result_id = 3/«Не согласовано.
/// Вернуть Эксперту», то переходим к следующей проверке, иначе выводим ошибку:
/// «ППЗ/ДС <Номер ППЗ/ДС> включена в Протокол <Номер Протокола> от <дата> с
/// решением <Решение СК>. Добавление в Повестку/Создание Повестки запрещено».
///
/// TODO: Make a method for shared selection and filtration.
///
/// ```ignore
/// select
///     protocol.id as prot_id,
///     protocol.is_removed as p_remove,
///     protocol_item.uuid as pi_uuid,
///     protocol_item.result_id,
///     protocol_item.is_excluded,
///     protocol_item.is_removed as p_i_remove
/// from
///     protocol_item
/// join protocol on
///     protocol_item.protocol_uuid = protocol.uuid
/// where
///     protocol_item.source_uuid in (...)
///     and protocol.is_removed = false
///     and protocol_item.is_removed = false
///     and protocol_item.is_excluded = false
///     and protocol_item.result_id <> 3;
/// ````
pub(crate) async fn examine_protocols<F>(
    items: &mut AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
    db_pool: &PgPool,
    message_fn: F,
) -> Result<()>
where
    F: Fn(&EcProtocol, &EcProtocolItem, &PlanOrAmendment) -> Message,
{
    let protocol_items = JoinedEcProtocolItemEcProtocolSelector::new(
        Select::default()
            .in_any(EcProtocolItem::source_uuid, items.keys().map(Value::from))
            .eq(EcProtocolItem::is_excluded, false)
            .eq(EcProtocolItem::is_removed, false)
            .ne(EcProtocolItem::result_id, ResultId::NotAgreed)
            .add_replace_order_desc(EcProtocolItem::created_at),
    )
    .set_protocol(
        EcProtocol::join_default().selecting(
            Select::default()
                .eq(EcProtocol::protocol_type_id, ProtocolType::InPersonMeeting)
                .eq(EcProtocol::is_removed, false),
        ),
    )
    .get(db_pool)
    .await?;

    for item in protocol_items {
        if let Some(plan) = items.remove(&item.item.source_uuid) {
            messages.add_prepared_message(message_fn(
                &item.protocol,
                &item.item,
                &plan,
            ))
        }
    }

    Ok(())
}
/// 3) По ППЗ/ДС проверить наличие Повесток. Если Повестка отсутствует, то
/// перейти к следующей проверке. Если присутствует, то проверить по ППЗ/ДС
/// значение в поле «Снято с рассмотрения» и что позиция Повестки по ППЗ/ДС не
/// включена в позицию Протокола .
///
/// Алгоритм: проверить не удаленную Повестку (agenda - is_removed = false) с
/// наивысшей датой создания таблицы agenda (данные заголовка Повесток, в
/// которую включена ППЗ/ДС). Если запись отсутствует, то переходим к следующей
/// проверке, иначе читаем запись по ППЗ/ДС из таблицы agenda_item, где
/// agenda_item – uuid = uuid входного параметра ППЗ/ДС и agenda_item –
/// agenda_uuid = agenda – uuid найденной записи Повестки и запись не удалена
/// agenda_item - is_removed = false и отсутствует записи по agenda_item –
/// agenda_uuid = item_relation_agenda_protocol - agenda_item_uuid. По найденной
/// записи в agenda_item проверяем значение поля is_excluded/«Снято с
/// рассмотрения». Если is_excluded = true/«Да», то переходим к следующей
/// проверке, иначе выводим ошибку: «ППЗ/ДС <Номер ППЗ/ДС> включена в Повестку
/// <Номер Повестки> на <дата заседания СК>. Добавление в Повестку/Создание
/// Повестки запрещено».
///
/// TODO: упростить условие в задаче. По новым вводным это должен быть эквивалент запроса
///
/// ```ignore
/// select
///     agesqlid as a_uuid,
///     agenda.id as a_id,
///     agenda.is_removed as a_remove,
///     agenda_item.uuid as ai_uuid,
///     agenda_item.is_excluded,
///     agenda_item.is_removed as a_i_remove
/// from
///     agenda_item
/// join agenda on
///     agenda_item.agenda_uuid = agenda.uuid
/// where
///     not exists (
///     select
///         *
///     from
///         item_relation_agenda_protocol
///     where
///         item_relation_agenda_protocol.agenda_item_uuid = agenda_item.uuid)
///     and agenda_item.source_uuid in (...)
///     and agenda.is_removed = false
///     and agenda_item.is_removed = false
///     and agenda_item.is_excluded = false;
/// ```
/// или
/// ```ignore
/// select
///     agenda.uuid as a_uuid,
///     agenda.id as a_id,
///     agenda.is_removed as a_remove,
///     agenda_item.uuid as ai_uuid,
///     agenda.id as a_id,
///     agenda_item.is_excluded,
///     agenda_item.is_removed as a_i_remove,
///     item_relation_agenda_protocol.protocol_uuid as i_r_p_uuid,
///     item_relation_agenda_protocol.protocol_item_uuid as i_r_pi_uuid
/// from
///     agenda_item
/// join agenda on
///     agenda_item.agenda_uuid = agenda.uuid
/// left join item_relation_agenda_protocol on
///     item_relation_agenda_protocol.agenda_item_uuid = agenda_item.uuid
/// where
///     agenda_item.source_uuid in (...)
///     and agenda.is_removed = false
///     and agenda_item.is_removed = false
///     and agenda_item.is_excluded = false;
/// ```
pub(crate) async fn examine_agendas<F>(
    items: &mut AHashMap<Uuid, PlanOrAmendment>,
    messages: &mut Messages,
    db_pool: &PgPool,
    message_fn: F,
) -> Result<()>
where
    F: Fn(&EcAgenda, &EcAgendaItem, &PlanOrAmendment) -> Option<Message>,
{
    let agenda_items =
        JoinedEcAgendaItemEcAgendaRelAgendaProtocolItemSelector::new(
            Select::default()
                .in_any(EcAgendaItem::source_uuid, items.keys())
                .eq(EcAgendaItem::is_excluded, false)
                .eq(EcAgendaItem::is_removed, false)
                .add_replace_order_desc(EcAgendaItem::created_at),
        )
        .set_agenda(
            EcAgenda::join_default()
                .selecting(Select::default().eq(EcAgenda::is_removed, false)),
        )
        .get(db_pool)
        .await?;

    for item in agenda_items {
        if item.item_agenda_protocol_rel.is_none() {
            if let Entry::Occupied(plan) = items.entry(item.agenda_item.source_uuid)
            {
                if let Some(msg) =
                    message_fn(&item.agenda, &item.agenda_item, plan.get())
                {
                    plan.remove();
                    messages.add_prepared_message(msg);
                }
            }
        }
    }

    Ok(())
}
