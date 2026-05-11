#![allow(clippy::type_complexity)]

use ahash::{AHashMap, AHashSet};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{joined::JoinTo, selection::*, DbItemDel, Select},
    result::SharedDbError,
    DbItem,
};
use shared_essential::{
    application::records::Recorder,
    domain::{
        JoinedEcAgendaEcAgendaItemRelAgendaProtocolItemSelector as AgendaWithItemsAndItemsRelsSelector,
        *,
    },
    presentation::dto::{
        estimated_commission::UpdateProtocolReqWithUser,
        processing::UpdateProtocolRes, response_request::*,
    },
};

use crate::{
    app_process::{
        calls::items_common,
        records::{send_to_monolith, PlanCollectedUpdate, ProcessingRulesChecker},
    },
    common::{ProcessingCtx, Result},
    presentation::business_messages::protocol::ProtocolUpdateMessage,
};

mod constants;
mod direct_update;
mod types;

use constants::*;
use direct_update::UpdateReq;
pub(crate) use types::*;

use super::get_protocol_items_by_id_range;

type PrepareItemContext = items_common::PrepareItemContext<EcProtocolItem>;

#[derive(Debug, thiserror::Error)]
pub enum UpdateProtocolError {
    #[error("Ошибка при обновлении позиций Протокола: {0}.")]
    Items(#[from] items_common::ItemsError),
    #[error("Отсутствует обязательное поле `{0}`")]
    MissingField(&'static str),
    #[error("Выполнить сохранение невозможно. Укажите дату Протокола.")]
    MissingProtocolDate,
    #[error(transparent)]
    Db(#[from] SharedDbError),
}

#[tracing::instrument(skip_all)]
pub(crate) async fn update_protocol(
    request: UpdateProtocolReqWithUser,
    proc_ctx: ProcessingCtx,
) -> crate::common::Result<ApiResponse<UpdateProtocolRes, ()>> {
    tracing::info!(
        kind = "update",
        "Получен запрос на обновление Протокола СК ({path}): {req:?}\n",
        req = request,
        path = UPDATE_PROTOCOL_DETAILS
    );
    let res = update_protocol_inner(request.try_into()?, &proc_ctx).await?;
    Ok(res.into())
}

pub(crate) async fn update_protocol_inner(
    UpdateProtocolReqInner {
        user_id,
        header,
        items,
        items_d647,
        partner_list,
        attachment_list,
    }: UpdateProtocolReqInner,
    proc_ctx: &ProcessingCtx,
) -> Result<(UpdateProtocolRes, Messages)> {
    let mut uctx = {
        let items = EcProtocolItem::select(
            &Select::with_fields(ITEM_FIELDS_TO_LOAD)
                .eq(EcProtocolItem::protocol_uuid, header.uuid),
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
        .map(|(item, d647)| prepare_protocol_item(item, d647, &mut uctx))
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
    .map_err(UpdateProtocolError::from)?;
    let to_upsert_items = uctx.upsertable_items(to_check_items, has_changes);

    let mut check_messages = Messages::default();

    if header.protocol_type_id == ProtocolType::CorrespondenceMeeting {
        let included_items = uctx.included_items(&to_upsert_items);
        if !included_items.is_empty() {
            let messages = examine_included_items(
                included_items,
                &to_check_plans,
                header.id,
                proc_ctx,
            )
            .await?;
            check_messages.add_messages(messages);
        }
    }

    let new_items = uctx.new_items(&to_upsert_items);
    let new_plans = if !new_items.is_empty() {
        let (plans, messages) = examine_new_items(
            new_items,
            &to_check_plans,
            header.protocol_type_id,
            proc_ctx,
        )
        .await?;
        check_messages.add_messages(messages);

        plans
    } else {
        Vec::new()
    };

    if !check_messages.is_empty() {
        return Ok(((), check_messages));
    }

    let mut database_request = UpdateReq::new(
        header.into(),
        to_upsert_items,
        partner_list,
        attachment_list,
    )?;
    let mut messages = Messages::default();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(user_id)
        .with_timestamp(uctx.timestamp())
        .begin()
        .await?;

    let protocol = database_request
        .protocol
        .items
        .first()
        .expect("Протокол должен быть передан от пользователя");

    // Нам нужно заполнить некоторые поля у сущностей, для
    // их правильного отображения в БД
    fill_new_items(
        protocol,
        &mut database_request.protocol_items.items,
        &new_plans,
        &uctx,
        recorder.tx(),
    )
    .await?;
    database_request.attachments.items.iter_mut().for_each(|a| {
        a.object_uuid = protocol.uuid;
    });
    database_request
        .estimated_commission_partners
        .items
        .iter_mut()
        .for_each(|p| {
            p.protocol_agenda_uuid = protocol.uuid;
        });

    let (protocol, protocol_items) =
        database_request.update_direct(&mut messages, &mut recorder).await?;

    let removed_items = uctx.removed_items(&protocol_items);
    update_relations_with_status(
        &protocol,
        &removed_items,
        &mut messages,
        &mut recorder,
    )
    .await?;

    let included_items = uctx.included_items(&protocol_items);
    let new_items = uctx.new_items(&protocol_items);
    if !included_items.is_empty()
        || (!new_items.is_empty() && !new_plans.is_empty())
    {
        update_plans(
            &protocol,
            &included_items,
            new_plans,
            &mut messages,
            &mut recorder,
            proc_ctx.create_rules_checker(),
        )
        .await?;
    }

    recorder.commit().await?;

    messages.add_prepared_message(ProtocolUpdateMessage::success(&protocol));

    Ok(((), messages))
}

/// Проверка на валидность добавляемых/восстанавливаемых с is_removed=true на is_removed=false позиций Протокола
async fn examine_new_items(
    items: Vec<&EcProtocolItem>,
    to_check_plans: &AHashMap<Uuid, PlanOrAmendment>,
    protocol_type: ProtocolType,
    proc_ctx: &ProcessingCtx,
) -> Result<(Vec<PlanOrAmendment>, Messages)> {
    let plans = {
        let uuids: AHashSet<Uuid> =
            items.into_iter().map(|item| item.source_uuid).collect::<AHashSet<_>>();
        to_check_plans
            .iter()
            .filter(|(uuid, _)| uuids.contains(uuid))
            .map(|(_, b)| b)
            .cloned()
            .collect::<Vec<_>>()
    };
    let messages = get_protocol_items_by_id_range::get_protocol_items_inner(
        &plans,
        protocol_type,
        &proc_ctx.db_pool,
    )
    .await?;

    Ok((plans, messages))
}

/// При изменении по позиции признака is_excluded  с true на false,
/// выполнить проверки релевантные для заочного протокола
async fn examine_included_items(
    items: Vec<&EcProtocolItem>,
    to_check_plans: &AHashMap<Uuid, PlanOrAmendment>,
    protocol_id: i64,
    proc_ctx: &ProcessingCtx,
) -> Result<Messages> {
    let plans = {
        let uuids: AHashSet<Uuid> =
            items.into_iter().map(|item| item.source_uuid).collect::<AHashSet<_>>();
        to_check_plans
            .iter()
            .filter(|(uuid, _)| uuids.contains(uuid))
            .map(|(_, b)| b)
            .cloned()
            .collect::<Vec<_>>()
    };
    let mut messages = Messages::default();
    let mut error_plans = AHashSet::new();

    get_protocol_items_by_id_range::examine_protocol_items(
        &plans,
        ProtocolType::CorrespondenceMeeting,
        |protocol, plan| {
            if protocol.id != protocol_id {
                ProtocolUpdateMessage::ExclusionAlreadyInProtocol(protocol)
                    .singular(plan)
                    .into()
            } else {
                None
            }
        },
        &mut error_plans,
        &mut messages,
        &proc_ctx.db_pool,
    )
    .await?;
    get_protocol_items_by_id_range::examine_plan_commission_kind(
        &plans,
        ProtocolType::CorrespondenceMeeting,
        |invalid_plans| {
            ProtocolUpdateMessage::ExclusionInvalidCommissionKind
                .resolve(invalid_plans)
                .expect("examine_plan_commission_kind гарантирует что !invalid_plans.is_empty()")
        },
        &mut error_plans,
        &mut messages,
    );

    Ok(messages)
}

/// Новые элементы могут не содержать некоторые поля, например такие как суммовые, поэтому
/// если пользователь их не передал, то их надо вручную заполнить
async fn fill_new_items(
    protocol: &EcProtocol,
    to_upsert_items: &mut [EcProtocolItem],
    new_plans: &[PlanOrAmendment],
    uctx: &PrepareItemContext,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    let mut new_items_to_fill = to_upsert_items
        .iter_mut()
        .filter(|i| {
            uctx.find_existing_item(
                Some(i.uuid),
                i.source_uuid,
                i.is_registered_by_d647,
            )
            .is_none()
        })
        .filter(|i| {
            i.sum_excluded_vat.is_none()
                || i.pricing_sum_excluded_vat.is_none()
                || i.commission_sum_excluded_vat.is_none()
        })
        .collect::<Vec<_>>();

    if new_items_to_fill.is_empty() {
        return Ok(());
    }

    let sums_to_fill = match protocol.protocol_type_id {
        ProtocolType::InPersonMeeting => EcAgendaItem::select(
            &Select::with_fields([
                EcAgendaItem::source_uuid,
                EcAgendaItem::sum_excluded_vat,
                EcAgendaItem::pricing_sum_excluded_vat,
            ])
            .in_any(
                EcAgendaItem::source_uuid,
                new_items_to_fill.iter().map(|i| i.source_uuid),
            ),
            tx,
        )
        .await?
        .into_iter()
        .fold(
            AHashMap::with_capacity(new_items_to_fill.len()),
            |mut acc, agenda_item| {
                acc.entry(agenda_item.source_uuid).or_insert((
                    agenda_item.sum_excluded_vat,
                    agenda_item.pricing_sum_excluded_vat,
                    None,
                ));
                acc
            },
        ),
        _ => new_plans.iter().fold(
            AHashMap::with_capacity(new_items_to_fill.len()),
            |mut acc, plan| {
                let sum_excluded_vat = match &plan {
                    PlanOrAmendment::Plan(p) => p.sum_excluded_vat,
                    PlanOrAmendment::Amendment(a) => a.delta_sum_excluded_vat,
                };
                let pricing_sum_excluded_vat = match &plan {
                    PlanOrAmendment::Plan(p) => Some(p.pricing_sum_excluded_vat),
                    PlanOrAmendment::Amendment(a) => {
                        a.pricing_delta_sum_excluded_vat
                    }
                };
                let commission_sum_excluded_vat = pricing_sum_excluded_vat;

                acc.entry(*plan.uuid()).or_insert((
                    Some(sum_excluded_vat),
                    pricing_sum_excluded_vat,
                    commission_sum_excluded_vat,
                ));
                acc
            },
        ),
    };

    new_items_to_fill.iter_mut().for_each(|i| {
        if let Some((
            sum_excluded_vat,
            pricing_sum_excluded_vat,
            commission_sum_excluded_vat,
        )) = sums_to_fill.get(&i.source_uuid)
        {
            i.sum_excluded_vat = i.sum_excluded_vat.or(*sum_excluded_vat);
            i.pricing_sum_excluded_vat =
                i.pricing_sum_excluded_vat.or(*pricing_sum_excluded_vat);
            i.commission_sum_excluded_vat =
                i.commission_sum_excluded_vat.or(*commission_sum_excluded_vat);
        }
    });

    Ok(())
}

/// Если есть новые позиции Протокола по ППЗ/ДС или снят признак is_removed, то необходимо в модуле АЦ в таблицах plan - ППЗ и contract_amendment - ДС поле
///  «Дата очной СК»/commission_date заполнить датой Заседания/ protocol_date и установить признак «Очная СК»/commission_kind_id = 1,
/// где plan/contract_amendment - uuid = protocol_item - source_uuid.
///
/// Релевантно только для Заочного Протокола
/// Если в существующей записи выполняется установка признака is_excluded , то необходимо в модуле АЦ в таблицах plan - ППЗ и contract_amendment - ДС поле
/// установить признак «Заочная СК»/commission_kind_id = 2, где plan/contract_amendment - uuid = agenda_item - source_uuid.
/// Записать историю изменения полей, если она была произведена.
async fn update_plans(
    protocol: &EcProtocol,
    included_items: &[&EcProtocolItem],
    mut new_plans: Vec<PlanOrAmendment>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<()> {
    let included_updated_plans = if !included_items.is_empty()
        && protocol.protocol_type_id == ProtocolType::CorrespondenceMeeting
    {
        let plan_select = Select::with_fields([
            Plan::uuid,
            Plan::id,
            Plan::commission_kind_id,
            Plan::commission_date,
        ])
        .in_any(Plan::uuid, included_items.iter().map(|i| i.source_uuid));
        let mut to_update_plans =
            PlanOrAmendment::select(&plan_select, recorder.db_pool()).await?;

        to_update_plans.iter_mut().for_each(|plan| {
            *plan.commission_kind_id_mut() = CommissionKind::Correspondence;
        });
        to_update_plans
    } else {
        Vec::new()
    };

    let new_plans_to_update =
        if protocol.protocol_type_id == ProtocolType::InPersonMeeting {
            new_plans.iter_mut().for_each(|plan| {
                *plan.commission_kind_id_mut() = CommissionKind::InPerson;
                *plan.commission_date_mut() = Some(protocol.protocol_date);
            });
            new_plans
        } else {
            Vec::new()
        };

    let to_update_plans = included_updated_plans
        .into_iter()
        .chain(new_plans_to_update)
        .collect::<Vec<_>>();

    if !to_update_plans.is_empty() {
        let updated_plans = PlanOrAmendment::update(
            to_update_plans,
            &[Plan::commission_date, Plan::commission_kind_id],
            messages,
            recorder,
            handler,
        )
        .await?;

        send_to_monolith(&updated_plans, recorder).await?;
    }

    Ok(())
}

/// Тут ведётся работа над удалением связей между протоколами и повестками.
///
/// При получении с FE признак удаления/is_remove по позиции Протокола (ППЗ/ДС)
/// при установке признака в таблице protocol_item, необходимо удалить запись из
/// таблицы item_relation_agenda_protocol. Если из Протокола удаляются все
/// позиции, которые связаны с Повесткой, то так же удаляется и запись
///  в agenda_protocol_relation.
///
/// Если все ППЗ/ДС включенные в Повестку были удалены из Протокола(-ов)
/// очного заседания СК.
///
/// Действие релевантно, если protocol.protocol_type_id = 1:
///
/// - Если удаляется запись из item_relation_agenda_protocol, то по
///   item_relation_agenda_protocol.agenda_uuid проверить статус Повестки
///   agenda.status_id, где item_relation_agenda_protocol.agenda_uuid = agenda.uuid
/// - Если статус 300/Сформирован Протокол, то необходимо установить
///   предыдущий статус из истории изменения статусов Повестки / status_history.
async fn update_relations_with_status(
    protocol: &EcProtocol,
    removed_items: &[&EcProtocolItem],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let removed_item_rels =
        delete_item_protocol_relations(removed_items, recorder.tx()).await?;

    update_agenda_protocol_relations(
        protocol,
        &removed_item_rels,
        messages,
        recorder,
    )
    .await?;

    Ok(())
}

/// Если из Протокола удаляются все позиции, которые связаны с Повесткой, то так же удаляется и запись в agenda_protocol_relation.
/// Если в Протокола все позиции включены, то нужно создать agenda_protocol_relation
///
/// Если удаляется запись из item_relation_agenda_protocol, то по item_relation_agenda_protocol - agenda_uuid проверить
/// статус Повестки agenda - status_id, где item_relation_agenda_protocol - agenda_uuid = agenda - uuid.
/// Если статус 300/Сформирован Протокол, то необходимо установить предыдущий статус из истории изменения статусов Повестки / status_history.
async fn update_agenda_protocol_relations(
    protocol: &EcProtocol,
    removed_item_rels: &[RelAgendaProtocolItem],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let agenda_select = Select::default().in_any(
        EcAgenda::uuid,
        removed_item_rels.iter().map(|item_rel| item_rel.agenda_uuid),
    );
    let item_select = Select::default().eq(EcAgendaItem::is_removed, false);
    let item_rel_select = Select::default();

    // Связи джойнятся по agenda_uuid
    let joined_agendas = AgendaWithItemsAndItemsRelsSelector::new(agenda_select)
        .set_items(EcAgendaItem::join_default().selecting(item_select))
        .set_item_rels(
            RelAgendaProtocolItem::join_default().selecting(item_rel_select),
        )
        .get(recorder.tx())
        .await?;

    // Здесь нам нужно конкретно проверить отношения по текущему Протоколу, откуда
    // удаляются позиции
    // Если же есть Повестки, у которых ни одна позиция больше не состоит в Протоколе,
    // то ее надо удалить из agenda_protocol_relation
    let agenda_uuids_not_in_current_protocol = joined_agendas
        .iter()
        .filter(|joined_agenda| {
            joined_agenda
                .item_rels
                .iter()
                .filter(|rel| rel.protocol_uuid == protocol.uuid)
                .count()
                == 0
        })
        .map(|joined_agenda| joined_agenda.agenda.uuid)
        .collect::<Vec<_>>();
    if !agenda_uuids_not_in_current_protocol.is_empty() {
        let filter_tree =
            Filter::eq(RelAgendaProtocol::protocol_uuid, protocol.uuid)
                & Filter::in_any(
                    RelAgendaProtocol::agenda_uuid,
                    agenda_uuids_not_in_current_protocol,
                );
        RelAgendaProtocol::delete_returning(&filter_tree, recorder.tx()).await?;
    }

    if protocol.protocol_type_id == ProtocolType::CorrespondenceMeeting {
        return Ok(());
    }

    // Те Повестки, у которых хотя бы одна позиция не включена в Протокол, должны перейти на прошлый статус
    let agendas_without_some_rels = joined_agendas
        .into_iter()
        // По факту все Повестки имеют EcAgendaStatus::ProtocolFormed статус, но
        // все равно проверка на всякий случай
        .filter(|joined_agenda| {
            joined_agenda.agenda.status_id == EcAgendaStatus::ProtocolFormed
        })
        .filter(|joined_agenda| {
            joined_agenda.items.iter().any(|item| {
                !joined_agenda
                    .item_rels
                    .iter()
                    .any(|item_rel| item_rel.agenda_item_uuid == item.uuid)
            })
        })
        .map(|agenda_with_items| agenda_with_items.agenda)
        .collect::<Vec<_>>();

    if agendas_without_some_rels.is_empty() {
        return Ok(());
    }

    let histories_select = Select::default()
        .in_any(
            StatusHistory::object_uuid,
            agendas_without_some_rels.iter().map(|i| i.uuid),
        )
        .ne(StatusHistory::status_id, EcAgendaStatus::ProtocolFormed)
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid]);
    let prev_histories =
        StatusHistory::select(&histories_select, recorder.tx()).await?;

    let to_update_agendas = agendas_without_some_rels
        .into_iter()
        .filter_map(|mut agenda| {
            // Нам нужно статус из StatusHistory, который был до EcAgendaStatus::ProtocolFormed
            // Селект нам вернет все записи отсортированными по created_at и без записей с status_id=EcAgendaStatus::ProtocolFormed
            // поэтому нам достаточно найти первую запись с таким же uuid
            if let Some(prev_status) = prev_histories
                .iter()
                .find(|history| history.object_uuid == agenda.uuid)
            {
                agenda.status_id = prev_status.status_id.into();
                Some(agenda)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    recorder
        .process_update(to_update_agendas, &[EcAgenda::status_id], messages)
        .await?;

    Ok(())
}

/// Удаление связей в item_agenda_protocol_relation
async fn delete_item_protocol_relations(
    removed_protocol_items: &[&EcProtocolItem],
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<RelAgendaProtocolItem>> {
    if removed_protocol_items.is_empty() {
        return Ok(Vec::new());
    }

    let agenda_protocol_item_deletion_filters = Filter::in_any(
        RelAgendaProtocolItem::protocol_item_uuid,
        removed_protocol_items.iter().map(|i| i.uuid),
    )
    .into();

    let deleted_relations = RelAgendaProtocolItem::delete_returning(
        &agenda_protocol_item_deletion_filters,
        &mut *tx,
    )
    .await?;

    Ok(deleted_relations)
}

fn has_changes(new: &EcProtocolItem, old: &EcProtocolItem) -> bool {
    new.number != old.number
        || new.is_excluded != old.is_excluded
        || new.is_removed != old.is_removed
        || new.sum_excluded_vat != old.sum_excluded_vat
        || new.pricing_sum_excluded_vat != old.pricing_sum_excluded_vat
        || new.commission_sum_excluded_vat != old.commission_sum_excluded_vat
        || new.result_id != old.result_id
}

fn prepare_protocol_item(
    item: UpdateProtocolItem,
    d647: bool,
    uctx: &mut PrepareItemContext,
) -> Result<Option<EcProtocolItem>> {
    let UpdateProtocolItem {
        uuid,
        source_uuid,
        is_removed,
        is_excluded,
        sum_excluded_vat,
        pricing_sum_excluded_vat,
        commission_sum_excluded_vat,
        result_id,
    } = item;

    let number = uctx.next_number(is_removed, d647);

    if is_removed && uuid.is_none() {
        // позиция добавлена и удалена в FE без сохранения...
        return Ok(None);
    }

    let (uuid, result_id, created_at, created_by) = uctx
        .find_existing_item(uuid, source_uuid, d647)
        .map(|old_item| {
            (
                old_item.uuid,
                result_id.unwrap_or(ResultId::Undefined),
                old_item.created_at,
                old_item.created_by,
            )
        })
        .unwrap_or((
            Uuid::new_v4(),
            result_id.unwrap_or(ResultId::Undefined),
            uctx.timestamp(),
            uctx.user_id(),
        ));

    let item = EcProtocolItem {
        uuid,
        protocol_uuid: uctx.container_uuid(),
        source_uuid,
        number,
        is_registered_by_d647: d647,
        is_removed,
        is_excluded,
        result_id,
        sum_excluded_vat,
        pricing_sum_excluded_vat,
        commission_sum_excluded_vat,
        created_by,
        created_at,
        changed_at: created_at,
        changed_by: created_by,
    };

    Ok(Some(item))
}
