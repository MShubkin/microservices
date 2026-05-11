use ahash::{AHashMap, AHashSet};
use itertools::Itertools;
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{joined::JoinTo, selection::*, AsezDate},
    DbItem, Value,
};
use shared_essential::{
    application::records::Recorder,
    domain::tables::{
        processing::agenda::{
            JoinedEcAgendaEcAgendaItemPlanContractAmendment as JoinedAgenda,
            JoinedEcAgendaEcAgendaItemRelAgendaProtocolItemSelector as JoinedRelSelector,
        },
        *,
    },
    presentation::dto::{
        general::ObjectIdentifier, processing::*, response_request::*,
    },
};

use crate::{
    common::{
        number_range::{self, EcObjectType},
        ProcessingCtx, ProcessingError as PError, Result,
    },
    presentation::business_messages::protocol::ProtocolCreateMessage,
};

pub(super) struct CreateProtocolItemInPerson {
    pub(super) agenda_id: ObjectIdentifier,
    pub(super) all_items: bool,
    pub(super) agenda_items: Vec<ObjectIdentifier>,
}

/// ### Вводные с ФЕ!
/// ЕСЛИ с FE в запросе указан item_list.agenda_item.is_all_items_included = false,
/// ТО с FE придет конкретный список ППЗ/ДС из Повестки в структуре
/// item_list.agenda_item.item_list, который пойдет под создание Протокола.
///
/// ### Выбор ППЗ/ДС (agenda_items)
/// Если придет item_list.agenda_item.is_all_items_included = true, то потребуется
/// выполняться поиск ППЗ/ДС в Повестке, которые еще не включены в Протокол для
/// дальнейшего создания Протокола. По item_list.uuid Повесток найти не удаленные
/// и не реестровые ППЗ/ДС, где agenda_items.agenda_uuid = item_list.uuid Повесток
/// и agenda_items.is_removed = false и is_excluded = false и is_registered_by_d647 = false.
///
/// ### Проверки ППЗ/ДС (agenda_items)
/// По найденному списку ППЗ/ДС проверить отсутствие связей с Протоколом, где найти
/// записи в таблице item_relation_agenda_protocol по условиям:
/// item_relation_agenda_protocol.agenda_item_uuid = agenda_item.uuid найденных
/// записей ППЗ/ДС из Повестки и item_relation_agenda_protocol.agenda_uuid = item_list.uuid
/// Повесток. Исключить записи по ППЗ/ДС которые уже включены в Протокол. Оставшиеся
/// записи ППЗ/ДС включаются в формируемый Протокол.
///
/// ### Что остаётся
/// ППЗ/ДС, которые относятся к Реестру по умолчанию попадают в Протокол. У таких
/// ППЗ/ДС всегда признак is_excluded = false.
/// По списку Повесток item_list - uuid из запроса = agenda_item - agenda_uuid
/// необходимо найти в agenda_item не удаленный список ППЗ/ДС, который относится
/// к Реестру is_registered_by_d647 = true​ и is_removed = false​.
///
/// TODO: Логика по заочным протокам.
pub(super) async fn create_protocol_in_person(
    protocol_type_id: ProtocolType,
    protocol_date: AsezDate,
    selected_items: Vec<CreateProtocolItemInPerson>,
    joined_items: Vec<JoinedAgenda>,
    user_id: i32,
    messages: &mut Messages,
    proc_ctx: ProcessingCtx,
) -> Result<EcProtocol> {
    let relations =
        fetch_agenda_item_relations(&joined_items, &proc_ctx.db_pool).await?;
    let agenda_with_items = screen_items(selected_items, relations, joined_items);

    let protocol = ProtocolCreator::new_in_person(agenda_with_items)?
        .insert_update_operation(
            protocol_type_id,
            protocol_date,
            user_id,
            messages,
            proc_ctx,
        )
        .await?;

    Ok(protocol)
}

pub(super) async fn create_protocol_correspondence(
    protocol_type_id: ProtocolType,
    protocol_date: AsezDate,
    plans: Vec<PlanOrAmendment>,
    user_id: i32,
    messages: &mut Messages,
    proc_ctx: ProcessingCtx,
) -> Result<EcProtocol> {
    let protocol = ProtocolCreator::new_correspondence(plans)
        .insert_update_operation(
            protocol_type_id,
            protocol_date,
            user_id,
            messages,
            proc_ctx,
        )
        .await?;

    Ok(protocol)
}

/// Достать все связи в item_relation по Повесткам СК
pub(crate) async fn fetch_agenda_item_relations(
    joined_items: &[JoinedAgenda],
    pool: &PgPool,
) -> Result<Vec<RelAgendaProtocolItem>> {
    // По остальным проверяем но присутствие повесток через item_relation_agenda_protocol
    // И исключаем остальное.
    let negative_items = Select::full::<RelAgendaProtocolItem>().in_any(
        RelAgendaProtocolItem::agenda_uuid,
        joined_items.iter().map(|x| x.agenda.uuid),
    );
    let relations = RelAgendaProtocolItem::select(&negative_items, pool).await?;

    Ok(relations)
}

/// Преобразовываем чтобы можно было вместе обрабатывать
/// Также тут идёт проверка на включение в другие повестки.
///
/// Также идет проверка на элементы, которые должны быть включены.
///
/// При первом создании Протокола попадают все элементы Повестки, которые выбрал пользователь
/// + элементы с is_registered_by_d647=true && is_excluded=false && is_removed=false. Для этого
/// надо передать with_d647=true аргумент
fn screen_items(
    selected_items: Vec<CreateProtocolItemInPerson>,
    relations: Vec<RelAgendaProtocolItem>,
    joined_items: Vec<JoinedAgenda>,
) -> Vec<(EcAgenda, Vec<EcAgendaItem>, Vec<PlanOrAmendment>)> {
    // Пользователь может задать добавление в Протокол как все элементы Повестки,
    // так и определенные из нее
    // None устанавливается в том случае, если пользователь хочет все элементы из Повестки
    // добавить
    let selected_checker = selected_items
        .into_iter()
        .map(|i| (i.agenda_id.uuid, (!i.all_items).then_some(i.agenda_items)))
        .collect::<AHashMap<_, _>>();

    joined_items
        .into_iter()
        .filter_map(|x| {
            let selected_items = selected_checker.get(&x.agenda.uuid)?;

            let items = x
                .items
                .into_iter()
                .filter(|x| !relations.iter().any(|i| i.agenda_item_uuid == x.uuid))
                .filter(|x| {
                    if x.is_registered_by_d647 {
                        return true;
                    }

                    // Нужно взять только элементы, которые передал пользователь. Если же пользователь
                    // передал all_items_included=true, то agenda_items будет None и значит мы берем все элементы
                    selected_items.as_ref().map_or(true, |items| {
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

#[derive(Debug)]
struct ProtocolCreator {
    unit_id: PricingUnitId,
    // Гарантировано будет существовать для Очной СК
    agenda_uuids: Option<AHashSet<Uuid>>,
    items: Vec<(Option<EcAgendaItem>, PlanOrAmendment)>,
    d647_items: Vec<(Option<EcAgendaItem>, PlanOrAmendment)>,
}

impl ProtocolCreator {
    fn new_in_person(
        agenda_with_items: Vec<(EcAgenda, Vec<EcAgendaItem>, Vec<PlanOrAmendment>)>,
    ) -> Result<Self> {
        let pricing_organization_unit_id = agenda_with_items
            .first()
            .map(|(agenda, _, _)| agenda.pricing_organization_unit_id)
            .and_then(|unit| {
                agenda_with_items
                    .iter()
                    .all(|(agenda, _, _)| {
                        agenda.pricing_organization_unit_id == unit
                    })
                    .then_some(unit)
            })
            .unwrap_or(PricingUnitId::Undefined);

        let agenda_uuids = agenda_with_items
            .iter()
            .map(|(agenda, _, _)| agenda.uuid)
            .collect::<AHashSet<Uuid>>();
        // Разделить ссылки на Д647 и остальные.
        let (mut items, mut d647_items) = (Vec::new(), Vec::new());
        for (_, agenda_items, plans) in agenda_with_items {
            let mut plans_checker = plans
                .into_iter()
                .map(|p| (*p.uuid(), p))
                .collect::<AHashMap<_, _>>();
            for agenda_item in agenda_items {
                let plan = plans_checker.remove(&agenda_item.source_uuid)
                    .ok_or(PError::CreateProtocol(
                        format!("Нарушение консистентности базы данных. По элементу Повестки СК с идентификатором {} нет ППЗ/ДС", agenda_item.uuid)
                    ))?;
                if agenda_item.is_registered_by_d647 {
                    d647_items.push((Some(agenda_item), plan));
                } else {
                    items.push((Some(agenda_item), plan));
                }
            }
        }

        Ok(ProtocolCreator {
            unit_id: pricing_organization_unit_id,
            agenda_uuids: Some(agenda_uuids),
            items,
            d647_items,
        })
    }

    fn new_correspondence(plans: Vec<PlanOrAmendment>) -> Self {
        let pricing_organization_unit_id = plans
            .first()
            .map(|p| p.pricing_organization_unit_id())
            .and_then(|unit| {
                plans
                    .iter()
                    .all(|p| p.pricing_organization_unit_id() == unit)
                    .then_some(*unit)
            })
            .unwrap_or(PricingUnitId::Undefined);

        Self {
            unit_id: pricing_organization_unit_id,
            agenda_uuids: None,
            items: plans.into_iter().map(|plan| (None, plan)).collect(),
            d647_items: Vec::default(),
        }
    }

    async fn insert_update_operation(
        self,
        protocol_type_id: ProtocolType,
        protocol_date: AsezDate,
        user_id: i32,
        messages: &mut Messages,
        proc_ctx: ProcessingCtx,
    ) -> Result<EcProtocol> {
        let ProtocolCreator {
            unit_id,
            agenda_uuids,
            d647_items,
            items,
        } = self;

        let number_req =
            number_range::NumberRequest::new(EcObjectType::Protocol, 1);

        let recorder =
            proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

        // All DB operations live inside this transaction.
        number_range::op_with_numbers(
            recorder,
            vec![number_req],
            move |ids_hash, recorder| {
                Box::pin(async move {
                    let ret_protocol = insert_protocol_related(
                        unit_id,
                        protocol_type_id,
                        protocol_date,
                        ids_hash,
                        agenda_uuids.as_ref(),
                        messages,
                        recorder,
                    )
                    .await?;

                    insert_partners(
                        &ret_protocol,
                        agenda_uuids.as_ref(),
                        messages,
                        recorder,
                    )
                    .await?;

                    insert_item_related(
                        items,
                        d647_items,
                        &ret_protocol,
                        messages,
                        recorder,
                    )
                    .await?;

                    if agenda_uuids.is_some()
                        && ret_protocol.protocol_type_id
                            == ProtocolType::InPersonMeeting
                    {
                        update_agendas(agenda_uuids.unwrap(), messages, recorder)
                            .await?;
                    }

                    Ok(ret_protocol)
                })
            },
        )
        .await
    }
}

async fn insert_protocol_related(
    pricing_organization_unit_id: PricingUnitId,
    protocol_type_id: ProtocolType,
    protocol_date: AsezDate,
    ids_hash: AHashMap<EcObjectType, Vec<i64>>,
    agenda_uuids: Option<&AHashSet<Uuid>>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<EcProtocol> {
    let protocol_id = ids_hash
        .get(&EcObjectType::Protocol)
        .and_then(|x| x.first())
        .ok_or_else(|| {
            PError::CreateProtocol("Protocol ids not created".to_string())
        })?;

    let protocol = EcProtocol {
        id: *protocol_id,
        protocol_type_id,
        protocol_date,
        pricing_organization_unit_id,
        status_id: EcProtocolStatus::Formed,
        ..Default::default()
    };
    let ret_protocol = recorder
        .process_insert(vec![protocol], messages)
        .await?
        .pop()
        .ok_or(PError::CreateProtocol("No protocol created.".to_string()))?;

    if agenda_uuids.is_some()
        && ret_protocol.protocol_type_id == ProtocolType::InPersonMeeting
    {
        let mut agenda_rels = agenda_uuids
            .as_ref()
            .unwrap()
            .iter()
            .copied()
            .map(|agenda_uuid| RelAgendaProtocol {
                protocol_uuid: ret_protocol.uuid,
                agenda_uuid,
                created_by: recorder.user_id(),
                created_at: recorder.timestamp(),
            })
            .collect::<Vec<_>>();
        RelAgendaProtocol::insert_vec(&mut agenda_rels, recorder.tx()).await?;
    }

    Ok(ret_protocol)
}

/// При создании очного Протокола СК agenda_items и d647_agenda_items должны содержать
/// [`Option::Some`] с [`EcAgendaItem`]
pub(crate) async fn insert_item_related(
    items: Vec<(Option<EcAgendaItem>, PlanOrAmendment)>,
    d647_items: Vec<(Option<EcAgendaItem>, PlanOrAmendment)>,
    ret_protocol: &EcProtocol,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let mut number_sequence = 1;

    let mut assemble_protocol_item =
        |item: &Option<EcAgendaItem>,
         plan: &PlanOrAmendment,
         is_registered_by_d647| {
            let (
                commission_sum_excluded_vat,
                sum_excluded_vat,
                pricing_sum_excluded_vat,
            ) = match ret_protocol.protocol_type_id {
                ProtocolType::InPersonMeeting => {
                    let sum_excluded_vat = item
                        .as_ref()
                        .map(|i| i.sum_excluded_vat)
                        .unwrap_or_default();
                    let pricing_sum_excluded_vat = item
                        .as_ref()
                        .map(|i| i.pricing_sum_excluded_vat)
                        .unwrap_or_default();
                    (None, sum_excluded_vat, pricing_sum_excluded_vat)
                }
                ProtocolType::CorrespondenceMeeting => {
                    let sum_excluded_vat = match plan {
                        PlanOrAmendment::Plan(p) => p.sum_excluded_vat,
                        PlanOrAmendment::Amendment(a) => a.delta_sum_excluded_vat,
                    };
                    let pricing_sum_excluded_vat = match plan {
                        PlanOrAmendment::Plan(p) => {
                            Some(p.pricing_sum_excluded_vat)
                        }
                        PlanOrAmendment::Amendment(a) => {
                            a.pricing_delta_sum_excluded_vat
                        }
                    };
                    let commission_sum_excluded_vat = pricing_sum_excluded_vat;
                    (
                        commission_sum_excluded_vat,
                        Some(sum_excluded_vat),
                        pricing_sum_excluded_vat,
                    )
                }
                ProtocolType::Undefined => {
                    unreachable!("Мы создали Протокол определенного типа")
                }
            };

            let protocol_item = EcProtocolItem {
                protocol_uuid: ret_protocol.uuid,
                source_uuid: *plan.uuid(),
                number: number_sequence,
                is_registered_by_d647,
                commission_sum_excluded_vat,
                sum_excluded_vat,
                pricing_sum_excluded_vat,
                ..Default::default()
            };
            number_sequence += 1;

            protocol_item
        };

    let mut protocol_items = Vec::with_capacity(items.len() + d647_items.len());
    protocol_items.extend(
        items
            .iter()
            .map(|(item, plan)| assemble_protocol_item(item, plan, false)),
    );
    protocol_items.extend(
        d647_items
            .iter()
            .map(|(item, plan)| assemble_protocol_item(item, plan, true)),
    );

    let ret_items = recorder.process_insert(protocol_items, messages).await?;

    if ret_protocol.protocol_type_id == ProtocolType::InPersonMeeting {
        let mut item_rels = ret_items
            .iter()
            .zip(items.iter().chain(&d647_items).map(|(item, _)| item))
            .map(|(protocol_item, agenda_item)| {
                let agenda_item = agenda_item
                    .as_ref()
                    .expect("Должно быть при создании очного Протокола СК");
                RelAgendaProtocolItem {
                    protocol_item_uuid: protocol_item.uuid,
                    protocol_uuid: protocol_item.protocol_uuid,
                    agenda_item_uuid: agenda_item.uuid,
                    agenda_uuid: agenda_item.agenda_uuid,
                    created_by: recorder.user_id(),
                    created_at: recorder.timestamp(),
                }
            })
            .collect::<Vec<_>>();
        RelAgendaProtocolItem::insert_vec(&mut item_rels, recorder.tx()).await?;
    }

    Ok(())
}

pub(crate) async fn insert_partners(
    created_protocol: &EcProtocol,
    agenda_uuids: Option<&AHashSet<Uuid>>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let new_partners = if agenda_uuids.is_some()
        && created_protocol.protocol_type_id == ProtocolType::InPersonMeeting
    {
        let agenda_uuids = agenda_uuids.unwrap();

        let old_partner_select = Select::full::<EcPartner>()
            .eq(EcPartner::is_removed, false)
            .in_any(EcPartner::protocol_agenda_uuid, agenda_uuids);

        let old_partners =
            EcPartner::select(&old_partner_select, recorder.tx()).await?;

        old_partners
            .into_iter()
            // Partners should not repeat on creation.
            .unique_by(|p| (p.user_id, p.role_id))
            // NB: We no longer take uniqueness by user id as
            // partners can have multiple roles.
            .map(|p| EcPartner {
                protocol_agenda_uuid: created_protocol.uuid,
                user_id: p.user_id,
                e_mail: p.e_mail,
                role_id: p.role_id,
                ..Default::default()
            })
            .collect::<Vec<_>>()
    } else {
        let partner_type_select = Select::full::<PartnerTypeCommission>().eq(
            PartnerTypeCommission::protocol_type_id,
            created_protocol.protocol_type_id,
        );
        let partner_types =
            PartnerTypeCommission::select(&partner_type_select, recorder.tx())
                .await?;
        partner_types
            .into_iter()
            .map(|ty| EcPartner {
                protocol_agenda_uuid: created_protocol.uuid,
                user_id: ty.user_id,
                e_mail: None,
                role_id: ty.role_id,
                ..Default::default()
            })
            .collect::<Vec<_>>()
    };

    recorder.process_insert(new_partners, messages).await?;

    Ok(())
}

pub(crate) async fn update_agendas(
    agenda_uuids: AHashSet<Uuid>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    // Достать и посчитать позиции повесток. Если у каждой позиции есть
    // привязка, то надо обновить ту повестку.
    let agenda_selector = Select::full_in::<_, EcAgenda>(
        EcAgenda::uuid,
        agenda_uuids.iter().map(Value::from),
    );
    let item_sel = Select::full::<EcAgendaItem>()
        .eq(EcAgendaItem::is_removed, false)
        .eq(EcAgendaItem::is_excluded, false);

    let agendas_to_update = JoinedRelSelector::new(agenda_selector)
        .set_items(EcAgendaItem::join_default().selecting(item_sel))
        .distinct()
        .get(recorder.tx())
        .await?
        .into_iter()
        .filter(|x| {
            let rel_hash = x
                .item_rels
                .iter()
                .map(|x| x.agenda_item_uuid)
                .collect::<AHashSet<_>>();

            x.items.iter().all(|i| rel_hash.contains(&i.uuid))
        })
        .map(|x| {
            let mut agenda = x.agenda;
            agenda.status_id = EcAgendaStatus::ProtocolFormed;

            agenda
        })
        .collect::<Vec<_>>();

    recorder
        .process_update(agendas_to_update, &[EcAgenda::status_id], messages)
        .await?;
    Ok(())
}

pub(super) fn finalise(
    protocol: EcProtocol,
    mut messages: Messages,
) -> Result<CreateProtocolResponse> {
    messages.add_prepared_message(ProtocolCreateMessage::success(&protocol));

    Ok(((), messages).into())
}
