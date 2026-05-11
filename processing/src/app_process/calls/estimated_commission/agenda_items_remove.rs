//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
use std::sync::Arc;

use ahash::AHashMap;
use sqlx::PgPool;

use asez2_shared_db::db_item::Select;
use shared_essential::{
    domain::tables::{self, *},
    presentation::dto::{processing::*, response_request::*},
};

use tables::processing::rel_agenda_protocol_item::{
    JoinedRelAgendaProtocolItemEcProtocol as RelationWithProtocol,
    JoinedRelAgendaProtocolItemEcProtocolSelector as RelationWithProtocolSelect,
};

use crate::{
    app_process::sections::mapping::SectionMapExt,
    common::ProcessingError as PError,
};
use crate::{
    common::Result,
    presentation::business_messages::agenda::AgendaRemoveItemsMessage,
};

const PRE_REMOVE_ITEMS: &str =
    "/rest/estimated_commission/v1/pre_request/pre_request_agenda_items_remove/";

const REQUEST_FIELDS: &[&str] = &[
    Plan::uuid,
    Plan::id,
    Plan::customer_id,
    Plan::contract_subject,
    Plan::pricing_expert_id,
    Plan::supplier_id,
    Plan::sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat,
    Plan::currency_id,
    Plan::commission_date,
    Plan::status_id,
];
const RESPONSE_FIELDS: &[&str] = &[
    Plan::uuid,
    "plan_id",
    Plan::customer_id,
    Plan::contract_subject,
    Plan::pricing_expert_id,
    Plan::supplier_id,
    Plan::sum_excluded_vat,
    Plan::currency_id,
    Plan::commission_date,
    Plan::status_id,
];

pub(crate) async fn pre_remove_agenda_items(
    req: PreRemoveAgendaItemsReq,
    db_pool: Arc<PgPool>,
) -> Result<PreRemoveAgendaItemsResponse> {
    tracing::info!(
        kind = "get",
        "Получен предзапрос на удаление элементов Повестки СК ({get}): {req:?}\n",
        get = PRE_REMOVE_ITEMS,
        req = req,
    );

    let PreRemoveAgendaItemsReq { item_list, .. } = req;
    let mut messages = Messages::default();

    examine_agenda_item_relations(&item_list, &mut messages, &db_pool).await?;

    if messages.is_error() {
        return Ok(messages.into());
    }

    let plan_select = Select::with_fields(REQUEST_FIELDS)
        .in_any(Plan::uuid, item_list.iter().map(|i| i.source_uuid));
    let plans = PlanOrAmendment::select(&plan_select, &db_pool)
        .await?
        .into_iter()
        .map(|p| {
            PlanOrAmendmentRep::from_item_with_section_mapping(
                p,
                SectionKind::EstimatedCommission,
                Some(RESPONSE_FIELDS),
            )
        })
        .collect::<Vec<_>>();

    Ok((plans, messages).into())
}

/// Проверка на то что по элементам Повестки нет записей в item_relation_agenda_protocol
pub(crate) async fn examine_agenda_item_relations(
    item_list: &[PreRemoveAgendaItem],
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()> {
    let agenda_item_uuids =
        item_list.iter().filter_map(|x| x.uuid).collect::<Vec<_>>();

    if agenda_item_uuids.is_empty() {
        return Ok(());
    }

    let plan_map = item_list
        .iter()
        .filter_map(|x| x.uuid.map(|uuid| (uuid, (x.id, x.object_type))))
        .collect::<AHashMap<_, _>>();

    let rel_select = Select::full::<RelAgendaProtocolItem>()
        .in_any(RelAgendaProtocolItem::agenda_item_uuid, agenda_item_uuids);
    // "По найденным ППЗ/ДС необходимо проверить наличие связи с позицией Протокола.
    // Если запись найдена,.. ..сообщения.. ..и передачи на FE.
    let relations = RelationWithProtocolSelect::new(rel_select)
        .distinct()
        .get(db_pool)
        .await?;

    for relation in relations {
        let RelationWithProtocol { rel, protocol } = relation;

        let (plan_id, ty) = *plan_map.get(&rel.agenda_item_uuid).ok_or(
            PError::AgendaItemsRemove(format!(
                "Элемент Повестки {} не имеет связанного с ним ППЗ/ДС",
                rel.agenda_item_uuid
            )),
        )?;
        let plan = match ty {
            EntityKind::ContractAmendment => {
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: plan_id,
                    ..Default::default()
                })
            }
            EntityKind::Plan => PlanOrAmendment::Plan(Plan {
                id: plan_id,
                ..Default::default()
            }),
            _ => {
                return Err(PError::AgendaItemsRemove(format!(
                    "Невалидная сущность ({}) для проверки при удалении позиций Повестки.",
                    ty
                )))
            }
        };
        messages.add_prepared_message(
            AgendaRemoveItemsMessage::AlreadyInProtocol(&protocol).singular(&plan),
        );
    }

    Ok(())
}
