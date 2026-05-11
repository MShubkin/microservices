use ahash::{AHashMap, AHashSet};
use itertools::{Either, Itertools};
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{Filter, FilterTree, Select},
    DbItem,
};
use shared_essential::{
    application::records::Recorder,
    domain::{
        maths::CurrencyValue,
        processing::protocol::{
            JoinedEcProtocolEcProtocolItem as ProtocolWithItems,
            JoinedEcProtocolEcProtocolItemSelector as ProtocolWithItemsSelector,
            ProtocolType,
        },
        EcAgenda, EcAgendaItem, EcAgendaStatus, EcPartner, EcProtocol,
        EcProtocolItem, JoinedEcAgendaEcAgendaItem as AgendaWithItems,
        JoinedEcAgendaEcAgendaItemPlanContractAmendment as JoinedAgenda,
        PlanOrAmendment, RelAgendaProtocol, RelAgendaProtocolItem,
    },
    presentation::dto::{general::ObjectIdentifier, response_request::Messages},
};

use crate::common::{ProcessingError, Result};
use crate::{
    app_process::estimated_commission::create_protocol, common::ProcessingCtx,
};

const OLD_ITEM_UPDATE_FIELDS: &[&str] = &[
    EcProtocolItem::is_removed,
    EcProtocolItem::is_excluded,
    EcProtocolItem::number,
    EcProtocolItem::changed_by,
    EcProtocolItem::changed_at,
    EcProtocolItem::sum_excluded_vat,
    EcProtocolItem::pricing_sum_excluded_vat,
    EcProtocolItem::commission_sum_excluded_vat,
];

pub(super) struct AddPlansProtocolItemInPerson {
    pub(super) agenda_id: ObjectIdentifier,
    pub(super) all_items: bool,
    pub(super) agenda_items: Vec<ObjectIdentifier>,
}

pub(super) async fn add_plans_protocol_in_person(
    selected_items: Vec<AddPlansProtocolItemInPerson>,
    protocol_id: ObjectIdentifier,
    user_id: i32,
    joined_agendas: Vec<JoinedAgenda>,
    messages: &mut Messages,
    proc_ctx: &ProcessingCtx,
) -> Result<(EcProtocol, Vec<PlanOrAmendment>)> {
    let old_relations = create_protocol::action::fetch_agenda_item_relations(
        &joined_agendas,
        &proc_ctx.db_pool,
    )
    .await?;

    let agenda_with_all_items = joined_agendas
        .iter()
        .map(|joined_agenda| AgendaWithItems {
            agenda: joined_agenda.agenda.clone(),
            agenda_items: joined_agenda.items.clone(),
        })
        .collect::<Vec<_>>();
    let agenda_with_selected_items =
        screen_items(selected_items, &old_relations, joined_agendas);
    let agenda_uuids = agenda_with_selected_items
        .iter()
        .map(|(agenda, _, _)| agenda.uuid)
        .collect::<Vec<_>>();

    let ProtocolWithItems {
        protocol,
        protocol_items,
    } = fetch_protocol_with_items(
        protocol_id.id,
        protocol_id.uuid,
        &proc_ctx.db_pool,
    )
    .await?;

    let items_to_add = agenda_with_selected_items.into_iter().flat_map(|(_, agenda_items, plans)| {
        let mut plans_checker =
            plans.into_iter().map(|p| (*p.uuid(), p)).collect::<AHashMap<_, _>>();

        agenda_items.into_iter().map(move |agenda_item| {
            plans_checker.remove(&agenda_item.source_uuid)
                .ok_or(ProcessingError::AddPlansProtocol(
                    format!("Нарушение консистентности базы данных. По элементу Повестки СК с идентификатором {} нет ППЗ/ДС", agenda_item.uuid)
                )).map(|plan| (plan, Some(agenda_item)))
        })
    })
    .collect::<Result<Vec<_>>>()?;

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let new_agenda_protocol_item_rels = insert_protocol_items(
        &protocol,
        protocol_items,
        &items_to_add,
        messages,
        &mut recorder,
    )
    .await?;

    update_partners(agenda_uuids, protocol_id.uuid, messages, &mut recorder)
        .await?;

    if let Some(new_relations) = new_agenda_protocol_item_rels {
        update_agendas_with_relation(
            agenda_with_all_items,
            &old_relations,
            &new_relations,
            protocol_id.uuid,
            messages,
            &mut recorder,
        )
        .await?;
    }

    let updated_protocol =
        update_protocol(protocol, messages, &mut recorder).await?;

    recorder.commit().await?;

    let added_plans = items_to_add.into_iter().map(|(p, _)| p).collect::<Vec<_>>();
    Ok((updated_protocol, added_plans))
}

pub(crate) async fn add_plans_protocol_correspondence(
    protocol_id: ObjectIdentifier,
    plans: Vec<PlanOrAmendment>,
    user_id: i32,
    messages: &mut Messages,
    proc_ctx: &ProcessingCtx,
) -> Result<(EcProtocol, Vec<PlanOrAmendment>)> {
    let ProtocolWithItems {
        protocol,
        protocol_items,
    } = fetch_protocol_with_items(
        protocol_id.id,
        protocol_id.uuid,
        &proc_ctx.db_pool,
    )
    .await?;

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let items_to_add =
        plans.into_iter().map(|plan| (plan, None)).collect::<Vec<_>>();
    insert_protocol_items(
        &protocol,
        protocol_items,
        &items_to_add,
        messages,
        &mut recorder,
    )
    .await?;

    let updated_protocol =
        update_protocol(protocol, messages, &mut recorder).await?;

    recorder.commit().await?;

    let added_plans = items_to_add.into_iter().map(|(p, _)| p).collect();
    Ok((updated_protocol, added_plans))
}

/// Выборка только тех элементов Повесток, которые выбрал пользователь
fn screen_items(
    selected_items: Vec<AddPlansProtocolItemInPerson>,
    old_relations: &[RelAgendaProtocolItem],
    joined_items: Vec<JoinedAgenda>,
) -> Vec<(EcAgenda, Vec<EcAgendaItem>, Vec<PlanOrAmendment>)> {
    // Пользователь может задать добавление в Протокол как все элементы Повестки,
    // так и определенные из нее. None устанавливается в том случае, если пользователь хочет добавить
    // все элементы
    let selected_checker = selected_items
        .into_iter()
        .map(|i| (i.agenda_id.uuid, (!i.all_items).then_some(i.agenda_items)))
        .collect::<AHashMap<_, _>>();

    joined_items
        .into_iter()
        .filter_map(|x| {
            let selected_agenda_items = selected_checker.get(&x.agenda.uuid)?;

            let items = x
                .items
                .into_iter()
                .filter(|x| {
                    !old_relations.iter().any(|i| i.agenda_item_uuid == x.uuid)
                })
                .filter(|x| {
                    // Нужно взять только элементы, которые передал пользователь. Если же пользователь
                    // передал all_items_included=true, то agenda_items будет None и значит мы берем все элементы
                    selected_agenda_items.as_ref().map_or(true, |items| {
                        items.iter().any(|i| i.uuid == x.uuid)
                    })
                })
                .collect::<Vec<_>>();

            if items.is_empty() {
                return None;
            }

            let plans = PlanOrAmendment::collect(x.plans, x.amendments);
            Some((x.agenda, items, plans))
        })
        .collect()
}

/// Добавление новых protocol_item с учетом реордеринга старых элементов
/// Порядок должен быть такой как в Повестке, то есть элементы идут в порядке
/// 1. is_registered_by_d647=false, is_removed=false
/// 2. is_registered_by_d647=false, is_removed=true
/// 3. is_registered_by_d647=true, is_removed=false
/// 4. is_registered_by_d647=true, is_removed=true
async fn insert_protocol_items(
    protocol: &EcProtocol,
    mut old_protocol_items: Vec<EcProtocolItem>,
    items: &[(PlanOrAmendment, Option<EcAgendaItem>)],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<Option<Vec<RelAgendaProtocolItem>>> {
    let mut old_item_checker = old_protocol_items
        .iter_mut()
        .map(|item| (item.source_uuid, item))
        .collect::<AHashMap<_, _>>();

    let mut new_protocol_items = Vec::with_capacity(items.len());
    let mut new_agenda_protocol_item_rels = (protocol.protocol_type_id
        == ProtocolType::InPersonMeeting)
        .then(|| Vec::with_capacity(items.len()));

    let mut append_new_item_rel =
        |protocol_item: &EcProtocolItem, agenda_item: Option<&EcAgendaItem>| {
            if let (Some(rels), Some(agenda_item)) =
                (new_agenda_protocol_item_rels.as_mut(), agenda_item.as_ref())
            {
                let new_agenda_protocol_item_rel = RelAgendaProtocolItem {
                    protocol_item_uuid: protocol_item.uuid,
                    protocol_uuid: protocol_item.protocol_uuid,
                    agenda_item_uuid: agenda_item.uuid,
                    agenda_uuid: agenda_item.agenda_uuid,
                    // TODO: implement for recorder
                    created_at: recorder.timestamp(),
                    created_by: recorder.user_id(),
                };

                rels.push(new_agenda_protocol_item_rel);
            }
        };

    for (plan, agenda_item) in items
        .iter()
        .filter(|(plan, _)| !old_item_checker.contains_key(plan.uuid()))
    {
        let (
            commission_sum_excluded_vat,
            sum_excluded_vat,
            pricing_sum_excluded_vat,
        ) = find_protocol_item_sums(
            plan,
            agenda_item.as_ref(),
            protocol.protocol_type_id,
        );
        let is_registered_by_d647 =
            agenda_item.as_ref().map(|i| i.is_registered_by_d647).unwrap_or(false);

        let new_protocol_item = EcProtocolItem {
            protocol_uuid: protocol.uuid,
            source_uuid: *plan.uuid(),
            // Будет заполнен ниже
            number: -1,
            is_registered_by_d647,
            sum_excluded_vat,
            pricing_sum_excluded_vat,
            commission_sum_excluded_vat,
            ..Default::default()
        };

        append_new_item_rel(&new_protocol_item, agenda_item.as_ref());
        new_protocol_items.push(new_protocol_item);
    }

    // Старые элементы которые были с i.is_removed=true нужно перевести на is_removed=false и подтянуть
    // суммы из ППЗ/ДС
    // Также по этим элементам нужно создать item_relation_agenda_protocol записи
    for (plan, agenda_item) in items {
        if let Some(old_protocol_item) = old_item_checker.get_mut(plan.uuid()) {
            let (
                commission_sum_excluded_vat,
                sum_excluded_vat,
                pricing_sum_excluded_vat,
            ) = find_protocol_item_sums(
                plan,
                agenda_item.as_ref(),
                protocol.protocol_type_id,
            );

            old_protocol_item.is_removed = false;
            old_protocol_item.is_excluded = false;
            old_protocol_item.sum_excluded_vat = sum_excluded_vat;
            old_protocol_item.pricing_sum_excluded_vat = pricing_sum_excluded_vat;
            old_protocol_item.commission_sum_excluded_vat =
                commission_sum_excluded_vat;

            append_new_item_rel(old_protocol_item, agenda_item.as_ref());
        }
    }

    let mut new_number_sequence = 1;
    let mut update_number =
        |items: &mut [EcProtocolItem], is_registered_by_d647, is_removed| {
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

    update_number(&mut old_protocol_items, false, false);
    update_number(&mut new_protocol_items, false, false);
    update_number(&mut old_protocol_items, false, true);
    update_number(&mut old_protocol_items, true, false);
    update_number(&mut new_protocol_items, true, false);
    update_number(&mut old_protocol_items, true, true);

    new_protocol_items.extend(old_protocol_items);
    recorder
        .process_upsert(new_protocol_items, OLD_ITEM_UPDATE_FIELDS, messages)
        .await?;

    let new_agenda_protocol_item_rels =
        if let Some(mut rels) = new_agenda_protocol_item_rels {
            RelAgendaProtocolItem::insert_vec_returning(&mut rels, recorder.tx())
                .await?
                .into()
        } else {
            None
        };

    Ok(new_agenda_protocol_item_rels)
}

fn find_protocol_item_sums(
    plan: &PlanOrAmendment,
    agenda_item: Option<&EcAgendaItem>,
    protocol_type: ProtocolType,
) -> (Option<CurrencyValue>, Option<CurrencyValue>, Option<CurrencyValue>) {
    match protocol_type {
        ProtocolType::CorrespondenceMeeting => {
            let sum_excluded_vat = match &plan {
                PlanOrAmendment::Plan(p) => p.sum_excluded_vat,
                PlanOrAmendment::Amendment(a) => a.delta_sum_excluded_vat,
            };
            let pricing_sum_excluded_vat = match &plan {
                PlanOrAmendment::Plan(p) => Some(p.pricing_sum_excluded_vat),
                PlanOrAmendment::Amendment(a) => a.pricing_delta_sum_excluded_vat,
            };
            let commission_sum_excluded_vat = pricing_sum_excluded_vat;
            (
                commission_sum_excluded_vat,
                Some(sum_excluded_vat),
                pricing_sum_excluded_vat,
            )
        }
        ProtocolType::InPersonMeeting => {
            let sum_excluded_vat = agenda_item
                .as_ref()
                .map(|i| i.sum_excluded_vat)
                .unwrap_or_default();
            let pricing_sum_excluded_vat = agenda_item
                .as_ref()
                .map(|i| i.pricing_sum_excluded_vat)
                .unwrap_or_default();
            (None, sum_excluded_vat, pricing_sum_excluded_vat)
        }
        _ => (None, None, None),
    }
}

/// Необходимо по uuid Протокола из запроса найти всех партнеров из таблицы estimated_commission_partner.
/// Необходимо по item_list - uuid Повестки из запроса найти всех партнеров из таблицы estimated_commission_partner, где is_removed = false.
/// Затем читать найденных партнеров по Повестке и по user_id искать партнера Протокола.  
/// Если такой найден с признаком is_removed = false, то переходим к следующему партнеру из Повестки,
/// если найден в Протоколе партнер с признаком is_removed = true, то данному партнеру ставим  
/// is_removed = false и устанавливаем changed_at и changed_by Код логина пользователя из запроса FE и
async fn update_partners<I>(
    agenda_uuids: I,
    protocol_uuid: Uuid,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()>
where
    I: IntoIterator<Item = Uuid>,
{
    let protocol_partner_filters = FilterTree::And(vec![
        FilterTree::filter(Filter::eq(
            EcPartner::protocol_agenda_uuid,
            protocol_uuid,
        )),
        FilterTree::filter(Filter::eq(EcPartner::is_removed, true)),
    ]);
    let agenda_partner_filters = FilterTree::And(vec![
        FilterTree::filter(Filter::in_any(
            EcPartner::protocol_agenda_uuid,
            agenda_uuids,
        )),
        FilterTree::filter(Filter::eq(EcPartner::is_removed, false)),
    ]);
    let final_filters = protocol_partner_filters.or(agenda_partner_filters);
    let partner_select = Select::full::<EcPartner>().set_filter_tree(final_filters);

    let (protocol_partners, agenda_partners): (Vec<_>, Vec<_>) =
        EcPartner::select(&partner_select, recorder.tx())
            .await?
            .into_iter()
            .partition_map(|i| {
                if i.protocol_agenda_uuid == protocol_uuid {
                    Either::Left(i)
                } else {
                    Either::Right(i)
                }
            });
    // Затем читать найденных партнеров по Повестке и по user_id искать партнера Протокола.
    // Если такой найден с признаком is_removed = false, то переходим к следующему партнеру из Повестки,
    // если найден в Протоколе партнер с признаком is_removed = true,
    // то данному партнеру ставим  is_removed = false и устанавливаем changed_at
    // и changed_by Код логина пользователя из запроса FE и записываем историю изменения полей в field_history.
    let to_update_partners = protocol_partners
        .into_iter()
        .filter(|protocol_partner| {
            agenda_partners.iter().any(|agenda_partner| {
                agenda_partner.user_id == protocol_partner.user_id
            })
        })
        .map(|mut protocol_partner| {
            protocol_partner.is_removed = false;
            protocol_partner
        })
        .collect::<Vec<_>>();

    recorder
        .process_update(
            to_update_partners,
            &[EcPartner::is_removed, EcPartner::changed_at, EcPartner::changed_by],
            messages,
        )
        .await?;

    Ok(())
}

async fn update_protocol(
    protocol: EcProtocol,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<EcProtocol> {
    let updated_protocol = recorder
        .process_update(
            vec![protocol],
            &[EcProtocol::changed_at, EcProtocol::changed_by],
            messages,
        )
        .await?
        .pop()
        .expect("Мы явно передали один протокол");

    Ok(updated_protocol)
}

/// Если у всех действующих позиций Повестки (is_removed = false) включены по итогу действия в
/// Протокол (есть записи в таблице item_relation_agenda_protocol), то необходимо в Повестке
/// установить статус 300/Сформирован Протокол и записать в историю изменения статусов/status_history.  
///
/// Принимает Повестки с ее действующими элементами, все связи их элементов в item_relation_agenda_protocol таблице
#[allow(clippy::too_many_arguments)]
async fn update_agendas_with_relation(
    agendas_with_items: Vec<AgendaWithItems>,
    old_relations: &[RelAgendaProtocolItem],
    new_relations: &[RelAgendaProtocolItem],
    protocol_uuid: Uuid,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let item_relation_checker = old_relations
        .iter()
        .chain(new_relations)
        .map(|rel| rel.agenda_item_uuid)
        .collect::<AHashSet<_>>();
    let agenda_protocol_checker = old_relations
        .iter()
        .map(|rel| (rel.agenda_uuid, rel.protocol_uuid))
        .collect::<AHashSet<_>>();

    // По Повестке и Протоколу может быть только одна запись в agenda_protocol_relation,
    // поэтому мы здесь проверяем чтобы связь сформировалась только в том случае, если
    // по Повестке появились только новые записи в item_relation
    let mut new_agenda_protocol_relations = agendas_with_items
        .iter()
        .filter(|i| {
            !agenda_protocol_checker.contains(&(i.agenda.uuid, protocol_uuid))
        })
        .map(|i| RelAgendaProtocol {
            protocol_uuid,
            agenda_uuid: i.agenda.uuid,
            created_at: recorder.timestamp(),
            created_by: recorder.user_id(),
        })
        .collect::<Vec<_>>();

    let agendas_to_update = agendas_with_items
        .into_iter()
        .filter_map(|i| {
            i.agenda_items
                .iter()
                .filter(|agenda_item| !agenda_item.is_removed)
                .all(|agenda_item| {
                    item_relation_checker.contains(&agenda_item.uuid)
                })
                .then_some(i.agenda)
        })
        .map(|mut agenda| {
            agenda.status_id = EcAgendaStatus::ProtocolFormed;
            agenda
        })
        .collect::<Vec<_>>();

    RelAgendaProtocol::insert_vec(
        &mut new_agenda_protocol_relations,
        recorder.tx(),
    )
    .await?;

    recorder
        .process_update(agendas_to_update, &[EcAgenda::status_id], messages)
        .await?;

    Ok(())
}

/// Получение протокола со ВСЕМИ его элементами, даже с is_removed=true
/// для дальнейшего реордеринга элементов
pub(crate) async fn fetch_protocol_with_items(
    protocol_id: i64,
    protocol_uuid: Uuid,
    db_conn: &PgPool,
) -> Result<ProtocolWithItems> {
    let protocol_select = Select::full::<EcProtocol>()
        .eq(EcProtocol::uuid, protocol_uuid)
        .eq(EcProtocol::id, protocol_id);
    let protocol_with_items = ProtocolWithItemsSelector::new(protocol_select)
        .get(db_conn)
        .await?
        .pop()
        .ok_or(ProcessingError::AddPlansProtocol(format!(
            "Протокол СК с идентификатором {} не найден",
            protocol_id
        )))?;

    Ok(protocol_with_items)
}
