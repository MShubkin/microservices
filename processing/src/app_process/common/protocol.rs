use ahash::{AHashMap, AHashSet};
use asez2_shared_db::db_item::{joined::JoinTo, Select};
use sqlx::{Executor, Postgres};

use shared_essential::{
    application::records::Recorder,
    domain::{
        EcProtocol, EcProtocolItem,
        JoinedEcProtocolItemEcProtocol as JoinedProtocolItem,
        JoinedEcProtocolItemEcProtocolSelector as JoinedProtocolItemSelector,
        PlanOrAmendment, ProtocolType,
    },
    presentation::dto::response_request::{Message, Messages},
};

use crate::common::Result;

/// Проверка на существование элементов Протокола СК по переданным
/// ППЗ/ДС
///
/// Выбираются самые последние по created_at Протоколы и его элементы
///
/// При protocol_type_id = None будут проверены все Протоколы
pub(crate) async fn examine_protocol_items<T, E>(
    plans: &[PlanOrAmendment],
    protocol_type_id: Option<ProtocolType>,
    mut message_fn: T,
    messages: &mut Messages,
    db_conn: E,
) -> Result<Vec<JoinedProtocolItem>>
where
    T: FnMut(&JoinedProtocolItem, &PlanOrAmendment) -> Option<Message>,
    E: for<'a> Executor<'a, Database = Postgres>,
{
    let joined_protocols =
        fetch_protocols_items(plans, protocol_type_id, db_conn).await?;

    if !joined_protocols.is_empty() {
        let mut plan_map =
            plans.iter().map(|p| (*p.uuid(), p)).collect::<AHashMap<_, _>>();

        for j in joined_protocols.iter() {
            // Элемент Протокола точно соответствует ППЗ/ДС, так как была выборка по plans
            // remove используется чтобы обработать только один элемент из множества протоколов
            if let Some(plan) = plan_map.remove(&j.item.source_uuid) {
                if let Some(message) = message_fn(j, plan) {
                    messages.add_prepared_message(message);
                }
            }
        }
    }

    Ok(joined_protocols)
}

/// Если ППЗ/ДС включена в Протокол с наивысшей датой создания,
/// который не удален/is_removed = false, то по позиции Протокола (тоже не удалена/is_removed = false)
/// производятся изменения
pub(crate) async fn update_protocol_items<F>(
    protocol_items: Vec<JoinedProtocolItem>,
    mut modify_fn: F,
    fields: &[&'static str],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<Vec<EcProtocolItem>>
where
    F: FnMut(JoinedProtocolItem) -> Option<EcProtocolItem>,
{
    let mut to_update_protocol_items = Vec::with_capacity(protocol_items.len());

    for j in protocol_items {
        if let Some(updated_protocol_item) = modify_fn(j) {
            to_update_protocol_items.push(updated_protocol_item);
        }
    }

    if to_update_protocol_items.is_empty() {
        return Ok(Vec::new());
    }

    Ok(recorder
        .process_update(to_update_protocol_items, fields, messages)
        .await?)
}

/// Выбираются самые последние по created_at Протоколы и его элементы
///
/// При protocol_type_id = None будут запрошены все Протоколы
pub(crate) async fn fetch_protocols_items<'a, E>(
    plans: &[PlanOrAmendment],
    protocol_type_id: Option<ProtocolType>,
    db_conn: E,
) -> Result<Vec<JoinedProtocolItem>>
where
    E: Executor<'a, Database = Postgres>,
{
    let source_uuids = plans.iter().map(|p| (*p.uuid()).into());

    let protocol_item_select = Select::full_in::<_, EcProtocolItem>(
        EcProtocolItem::source_uuid,
        source_uuids,
    )
    .eq(EcProtocolItem::is_removed, false)
    .eq(EcProtocolItem::is_excluded, false);

    let mut protocol_select =
        Select::full::<EcProtocol>().eq(EcProtocol::is_removed, false);

    if let Some(protocol_type_id) = protocol_type_id {
        protocol_select =
            protocol_select.eq(EcProtocol::protocol_type_id, protocol_type_id)
    }

    let mut joined_protocol_items =
        JoinedProtocolItemSelector::new(protocol_item_select)
            .set_protocol(EcProtocol::join_default().selecting(protocol_select))
            .get(db_conn)
            .await?;
    // Нам нужна самая новая. Т.Е. цамое большое значение timestamp первым.
    // Что значит что сравниваем "на оборот"
    joined_protocol_items
        .sort_unstable_by(|a, b| b.protocol.created_at.cmp(&a.protocol.created_at));

    let mut unique_protocol_items = Vec::with_capacity(plans.len());
    let mut already_inserted = AHashSet::new();
    joined_protocol_items.into_iter().for_each(|i| {
        if already_inserted.insert(i.item.source_uuid) {
            unique_protocol_items.push(i);
        }
    });

    Ok(unique_protocol_items)
}
