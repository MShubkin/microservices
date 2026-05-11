//! Повестка в этй ручке не должна удалятся.
//! Позиции повестки можно удалять ТОЛЬКО при условие, что они не включены в
//! протокол (нет связи в таблице item_relation_agenda_protocol).

use ahash::{AHashMap, AHashSet};
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::{db_item::Select, result::SharedDbError, DbItem};
use shared_essential::{
    domain::*,
    presentation::dto::{
        estimated_commission::{
            UpdateAgendaHeader, UpdateAgendaItem, UpdateAgendaReqWithUser,
        },
        general::ObjectIdentifier,
        processing::UpdateAgendaRes,
        response_request::*,
    },
};

use tables::processing::rel_agenda_protocol_item::JoinedRelAgendaProtocolItemEcProtocolSelector as RelationWithProtocolSelect;

use crate::{
    app_process::records::{send_to_monolith, PlanCollectedUpdate},
    presentation::business_messages::agenda::AgendaUpdateMessage,
};
use crate::{
    app_process::{
        calls::items_common,
        common::plan::fetch_plans_by_ids,
        estimated_commission::{create_agenda, get_agenda_items_by_id_range},
    },
    common::{ProcessingCtx, Result},
};

mod constants;
mod direct_update;
mod simple_details;

use constants::*;
use direct_update::UpdateReq;

type PrepareItemContext = items_common::PrepareItemContext<EcAgendaItem>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum UpdateAgendaError {
    #[error("Ошибка при обновлении позиций Повестки СК: {0}.")]
    Items(#[from] items_common::ItemsError),
    #[error("Повестка СК с идентификатором {0} не найдена в базе данных.")]
    NoAgenda(i64),
    #[error("Попытка обновить удаленную повестку с идентификатором {0}.")]
    RemovedAgenda(i64),
    #[error("см. Сообщения")]
    Messages(Messages),
    #[error("Элемент Повестки СК {0} не имеет смежного ППЗ/ДС.")]
    NoSource(i64),
    #[error("Выполнить сохранение невозможно. Укажите дату заседания СК.")]
    MissingMeetingDate,
    #[error(transparent)]
    Db(#[from] SharedDbError),
}

#[tracing::instrument(skip_all)]
pub(crate) async fn update_agenda(
    req: UpdateAgendaReqWithUser,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<UpdateAgendaRes, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг: Обновление повестки СК ({get}): {req:?}\n",
        req = req,
        get = UPDATE_AGENDA_DETAILS
    );

    update_agenda_inner(req, &proc_ctx).await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn update_agenda_inner(
    UpdateAgendaReqWithUser {
        user: user_id,
        header,
        items,
        items_d647,
        mut partner_list,
        mut attachment_list,
    }: UpdateAgendaReqWithUser,
    proc_ctx: &ProcessingCtx,
) -> Result<ApiResponse<UpdateAgendaRes, ()>> {
    let header = prepare_header(header)?;

    check_relations(&header, items.iter().chain(items_d647.iter()), proc_ctx)
        .await?;

    let mut uctx = {
        let items = EcAgendaItem::select(
            &Select::with_fields(AGENDA_OLD_ITEM_FIELDS)
                .eq(EcAgendaItem::agenda_uuid, header.uuid),
            &*proc_ctx.db_pool,
        )
        .await?;
        PrepareItemContext::new(header.uuid, items, user_id)
    };

    // порядок итерации (сначала items, потом items_d647) важен для нумерации позиций.
    let to_check_items = items
        .into_iter()
        .map(|item| (item, false))
        .chain(items_d647.into_iter().map(|item| (item, true)))
        .map(|(item, d647)| prepare_agenda_item(item, d647, &mut uctx))
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>>>()?;

    let to_check_plans = {
        let select = Select::full::<Plan>()
            .in_any(Plan::uuid, to_check_items.iter().map(|i| i.source_uuid));
        PlanOrAmendment::select(&select, &proc_ctx.db_pool)
            .await?
            .into_iter()
            .map(|p| (*p.uuid(), p))
            .collect::<AHashMap<_, _>>()
    };

    uctx.validate_compatability(
        &to_check_items,
        to_check_plans.iter().map(|(uuid, plan)| (uuid, *plan.id())).collect(),
    )
    .map_err(UpdateAgendaError::from)?;
    let to_upsert_items = uctx.upsertable_items(to_check_items, has_changes);

    let included_items = uctx.included_items(&to_upsert_items);
    if !included_items.is_empty() {
        let messages = examine_items(
            &to_check_plans,
            included_items,
            Some(header.id),
            proc_ctx,
        )
        .await?;
        if !messages.is_empty() {
            return Err(UpdateAgendaError::Messages(messages).into());
        }
    }

    let new_items = uctx.new_items(&to_upsert_items);
    if !new_items.is_empty() {
        let messages =
            examine_items(&to_check_plans, new_items, None, proc_ctx).await?;
        if !messages.is_empty() {
            return Err(UpdateAgendaError::Messages(messages).into());
        }
    }

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(user_id)
        .with_timestamp(uctx.timestamp())
        .begin()
        .await?;

    partner_list.iter_mut().for_each(|partner| {
        partner.protocol_agenda_uuid = Some(header.uuid);
    });
    attachment_list.iter_mut().for_each(|a| {
        a.object_uuid = Some(header.uuid);
    });

    let db_req =
        UpdateReq::new(header, to_upsert_items, partner_list, attachment_list)?;
    let mut messages = Messages::default();

    let mut pre_reply = db_req.update_direct(&mut messages, &mut recorder).await?;

    let plans_to_update = get_plans_to_update(
        &pre_reply.agenda,
        pre_reply.items.iter(),
        &proc_ctx.db_pool,
    )
    .await?;

    if !plans_to_update.is_empty() {
        let updated_plans = PlanOrAmendment::update(
            plans_to_update,
            &[Plan::commission_date, Plan::commission_kind_id],
            &mut messages,
            &mut recorder,
            proc_ctx.create_rules_checker(),
        )
        .await?;

        send_to_monolith(&updated_plans, &mut recorder).await?;
    }

    pre_reply.get_extras(&proc_ctx.db_pool).await?;

    recorder.commit().await?;

    messages.add_prepared_message(Message::success(format!(
        "Повестка № {agenda_id} на {meeting_date} сохранена",
        agenda_id = pre_reply.agenda.id,
        meeting_date = pre_reply.agenda.meeting_date,
    )));

    pre_reply.into_response().map(|r| (r, messages).into())
}

async fn examine_items(
    to_check_plans: &AHashMap<Uuid, PlanOrAmendment>,
    items: Vec<&EcAgendaItem>,
    agenda_id: Option<i64>,
    proc_ctx: &ProcessingCtx,
) -> Result<Messages> {
    let mut plans = {
        let uuids =
            items.into_iter().map(|item| item.source_uuid).collect::<AHashSet<_>>();

        to_check_plans
            .iter()
            .filter(|(uuid, _)| uuids.contains(uuid))
            .map(|(a, b)| (*a, b.clone()))
            .collect::<AHashMap<_, _>>()
    };
    let mut messages = Messages::default();

    create_agenda::examine_protocols(
        &mut plans,
        &mut messages,
        &proc_ctx.db_pool,
        |protocol, item, plan| {
            if agenda_id.is_some() {
                AgendaUpdateMessage::ExclusionAlreadyInProtocol(protocol, item)
                    .singular(plan)
            } else {
                AgendaUpdateMessage::AlreadyInProtocol(protocol, item)
                    .singular(plan)
            }
        },
    )
    .await?;
    create_agenda::examine_agendas(
        &mut plans,
        &mut messages,
        &proc_ctx.db_pool,
        |agenda, _, plan| match agenda_id {
            Some(aid) if aid != agenda.id => {
                AgendaUpdateMessage::ExclusionAlreadyInAgenda(agenda)
                    .singular(plan)
                    .into()
            }
            Some(_) => None,
            None => {
                AgendaUpdateMessage::AlreadyInAgenda(agenda).singular(plan).into()
            }
        },
    )
    .await?;
    if agenda_id.is_some() {
        get_agenda_items_by_id_range::examine_commission_kind(
            &mut plans,
            &mut messages,
            |invalid_plans| {
                AgendaUpdateMessage::ExclusionInvalidCommissionKind
                .resolve(invalid_plans)
                .expect("examine_commission_kind гарантирует что !invalid_plans.is_empty()")
            },
        );
    }

    Ok(messages)
}

/// Проверяет возможность сохранения повестки:
/// - Повестка не удалена
/// - ППЗ/ДС по удаленным позициям не находятся в Протоколе
async fn check_relations<'a, I>(
    agenda: &EcAgenda,
    items: I,
    proc_ctx: &ProcessingCtx,
) -> Result<()>
where
    I: IntoIterator<Item = &'a UpdateAgendaItem>,
{
    let db_agenda = EcAgenda::select_option(
        &Select::with_fields([EcAgenda::is_removed, EcAgenda::status_id])
            .eq(EcAgenda::uuid, agenda.uuid),
        &*proc_ctx.db_pool,
    )
    .await?
    .ok_or(UpdateAgendaError::NoAgenda(agenda.id))?;

    if db_agenda.is_removed {
        return Err(UpdateAgendaError::RemovedAgenda(agenda.id))?;
    }

    let to_remove_items = items
        .into_iter()
        .filter_map(|item| {
            if let (Some(true), Some(uuid)) = (item.is_removed, item.uuid) {
                Some(uuid)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if to_remove_items.is_empty() {
        return Ok(());
    }

    let rel_select = Select::full::<RelAgendaProtocolItem>()
        .in_any(RelAgendaProtocolItem::agenda_item_uuid, to_remove_items);
    // "По найденным ППЗ/ДС необходимо проверить наличие связи с позицией Протокола.
    // Если запись найдена,.. ..сообщения.. ..и передачи на FE.
    let relations = RelationWithProtocolSelect::new(rel_select)
        .distinct()
        .get(&*proc_ctx.db_pool)
        .await?;

    if relations.is_empty() {
        return Ok(());
    }

    let mut relation_checker = AHashMap::new();
    relations.into_iter().for_each(|rel| {
        relation_checker
            .entry(rel.protocol.uuid)
            .and_modify(|(_, count)| *count += 1)
            .or_insert((rel.protocol, 1));
    });

    let messages = relation_checker.values().map(|(protocol, count)| {
        Message::error(format!("{} ППЗ/ДС включена(ы) в Протокол № {} от {}. Удаление выполнить невозможно.", count, protocol.id, protocol.protocol_date))
    }).collect();

    Err(UpdateAgendaError::Messages(Messages {
        messages,
        kind: MessageKind::Error,
    }))?
}

fn has_changes(new: &EcAgendaItem, old: &EcAgendaItem) -> bool {
    new.number != old.number
        || new.is_excluded != old.is_excluded
        || new.is_removed != old.is_removed
        || new.sum_excluded_vat != old.sum_excluded_vat
        || new.pricing_sum_excluded_vat != old.pricing_sum_excluded_vat
        || new.reviewed_at != old.reviewed_at
}

fn prepare_header(header: UpdateAgendaHeader) -> Result<EcAgenda> {
    if header.meeting_date.is_none() {
        return Err(UpdateAgendaError::MissingMeetingDate.into());
    }

    let UpdateAgendaHeader {
        id,
        uuid,
        meeting_date,
        pricing_organization_unit_id,
    } = header;

    Ok(EcAgenda {
        id,
        uuid,
        meeting_date: meeting_date.expect("Выше проверено"),
        pricing_organization_unit_id: pricing_organization_unit_id
            .unwrap_or_default(),
        ..Default::default()
    })
}

fn prepare_agenda_item(
    item: UpdateAgendaItem,
    d647: bool,
    uctx: &mut PrepareItemContext,
) -> Result<Option<EcAgendaItem>> {
    let UpdateAgendaItem {
        uuid,
        source_uuid,
        is_excluded,
        sum_excluded_vat,
        pricing_sum_excluded_vat,
        reviewed_at,
        is_removed,
    } = item;

    if is_removed == Some(true) && uuid.is_none() {
        // позиция добавлена и удалена в FE без сохранения...
        return Ok(None);
    }

    let number = uctx.next_number(is_removed.unwrap_or(false), d647);
    let (uuid, created_by, created_at) = uctx
        .find_existing_item(uuid, source_uuid, d647)
        .map(|old_item| (old_item.uuid, old_item.created_by, old_item.created_at))
        .unwrap_or((Uuid::new_v4(), uctx.user_id(), uctx.timestamp()));

    let item = EcAgendaItem {
        uuid,
        agenda_uuid: uctx.container_uuid(),
        source_uuid,
        number,
        is_registered_by_d647: d647,
        is_excluded,
        is_removed: is_removed.unwrap_or(false),
        reviewed_at,
        sum_excluded_vat,
        pricing_sum_excluded_vat,
        created_at,
        created_by,
        changed_at: created_at,
        changed_by: created_by,
    };

    Ok(Some(item))
}

async fn get_plans_to_update<'a, I>(
    agenda: &EcAgenda,
    items: I,
    pool: &PgPool,
) -> Result<Vec<PlanOrAmendment>>
where
    I: 'a + Iterator<Item = &'a EcAgendaItem>,
{
    let excluded_map = agenda_items_excluded(items);
    if excluded_map.is_empty() {
        return Ok(Vec::new());
    }
    let ids = excluded_map
        .keys()
        .map(|uuid| ObjectIdentifier::new_with_type(0, *uuid, EntityKind::Unknown))
        .collect::<Vec<_>>();
    let mut plans = fetch_plans_by_ids(&ids, pool).await?;
    plans
        .iter_mut()
        .for_each(|plan| modify_plan(plan, agenda, &excluded_map));
    Ok(plans)
}

/// Строит отображение Uuid ППЗ/ДС в признак is_excluded.
///
/// Внимание! ППЗ/ДС может присутствовать в повестке дважды, один раз как
/// позиция списка, один как позиция реестра. Минимум одна из них будет
/// is_excluded = true. Конечный результат is_excluded должен быть a.is_excluded && b.is_excluded
fn agenda_items_excluded<'a, I>(items: I) -> AHashMap<Uuid, bool>
where
    I: 'a + Iterator<Item = &'a EcAgendaItem>,
{
    let mut excluded_map = AHashMap::new();
    // один и тот же ППЗ/ДС может быть исключен из списка, но включен в реестр
    items.for_each(|item| {
        *excluded_map.entry(item.source_uuid).or_insert(true) &= item.is_excluded;
    });
    excluded_map
}

// Установка признака is_excluded:
//
// Если в существующей записи выполняется установка признака
// is_excluded = true, то необходимо в модуле АЦ в таблицах plan -
// ППЗ и contract_amendment - ДС очистить поле «Дата очной
// СК»/commission_date и очистить признак «Очная
// СК»/commission_kind_id = 1, где plan/contract_amendment - uuid =
// agenda_item - source_uuid если статус ППЗ/ДС ≠ 251.
//
// Снятие признака is_excluded:
//
// Если в существующей записи выполняется снятие признака
// is_excluded = false, то необходимо в модуле АЦ в таблицах plan -
// ППЗ и contract_amendment - ДС поле «Дата очной
// СК»/commission_date заполнить датой Заседания/agenda -
// meeting_date и установить признак «Очная СК»/commission_kind_id =
// 1, где plan/contract_amendment - uuid = agenda_item -
// source_uuid.
fn modify_plan(
    plan: &mut PlanOrAmendment,
    agenda: &EcAgenda,
    excluded_map: &AHashMap<Uuid, bool>,
) {
    match excluded_map.get(plan.uuid()) {
        Some(true)
            if plan.status_id() != &PlanStatus::EstimatedCommissionInPerson =>
        {
            *plan.commission_date_mut() = None;
            *plan.commission_kind_id_mut() = CommissionKind::Undefined;
        }
        Some(true) => {}
        Some(false) => {
            *plan.commission_date_mut() = Some(agenda.meeting_date);
            *plan.commission_kind_id_mut() = CommissionKind::InPerson;
        }
        _ => {
            tracing::warn!(
                kind = "update",
                "ППЗ/ДС {} отсутствует в Повестке",
                plan.uuid()
            );
        }
    }
}
