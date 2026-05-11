use std::ops::RangeInclusive;

use ahash::AHashSet;
use asez2_shared_db::db_item::{Filter, FilterTree, Select};
use itertools::Itertools;
use shared_essential::{
    domain::{legacy::plans::PlanStatus, ContractAmendment, Plan, PlanOrAmendment},
    presentation::dto::{
        general::ObjectIdentifier,
        response_request::{BusinessMessage, Messages},
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

pub(crate) fn examine_plan_status<T>(
    plans: &[PlanOrAmendment],
    valid_statuses: &[PlanStatus],
    message: T,
    messages: &mut Messages,
) where
    T: BusinessMessage<Entity = PlanOrAmendment>,
{
    let invalid_plans = plans
        .iter()
        .filter(|p| !valid_statuses.contains(p.status_id()))
        .cloned()
        .collect::<Vec<_>>();

    message.checked_append(messages, &invalid_plans);
}

#[tracing::instrument(skip_all)]
pub(crate) async fn fetch_plans_by_ids<'a, I>(
    ids: I,
    conn: &PgPool,
) -> Result<Vec<PlanOrAmendment>>
where
    I: IntoIterator<Item = &'a ObjectIdentifier>,
{
    let ids = ids.into_iter().cloned().collect::<Vec<ObjectIdentifier>>();

    if ids.is_empty() {
        let msg = String::from("Был запрошен пустой массив ППЗ/ДС");
        return Err(ProcessingError::GetItemList(msg));
    }

    // OR фильтр тут не подходит. Надо ИЛИ искать по полному идентификатору типа:
    // `(id=x AND uuid=z) OR (id=x2 AND uuid=z2) OR (id=x3 AND uuid=z3)`
    // или так как id не уникальный, а uuid уникальный, чисто по uuid.
    // При этом, из некоторых функции могут приходит id, что плохо.
    let (filter_tree, unique_id_count) =
        if ids.iter().map(|i| i.uuid).all(|x| x.is_nil()) {
            tracing::warn!(kind = "infra", "Не пришли uuid, берем id: {:?}", ids);

            let unique_count = ids.iter().unique_by(|id| id.id).count();
            let tree = FilterTree::and_from_list([
                Filter::in_any(Plan::id, ids.iter().map(|i| i.id)),
                Filter::eq(Plan::is_actual, true),
            ]);

            (tree, unique_count)
        } else {
            let unique_count = ids.iter().unique_by(|id| id.uuid).count();
            let tree =
                Filter::in_any(Plan::uuid, ids.iter().map(|i| i.uuid)).into();

            (tree, unique_count)
        };

    let plan_select = Select::full::<Plan>().set_filter_tree(filter_tree.clone());
    let amendment_select =
        Select::full::<ContractAmendment>().set_filter_tree(filter_tree);

    let plans =
        PlanOrAmendment::select_dual(&plan_select, &amendment_select, conn).await?;

    if plans.len() < unique_id_count {
        let found_uuids = plans.iter().map(|x| *x.uuid()).collect::<AHashSet<_>>();
        let missing = ids
            .iter()
            .filter(|x| !found_uuids.contains(&x.uuid))
            .map(|x| x.id.to_string())
            .join(", ");

        let msg =
            format!("Записи ППЗ/ДС c идентификаторами {} не найдены", missing);
        return Err(ProcessingError::GetItemList(msg));
    }

    if plans.len() > unique_id_count {
        let double_found = plans
            .iter()
            .filter(|plan| {
                plans.iter().filter(|x| plan.id() == x.id()).count() > 1
                    || plans.iter().filter(|x| plan.uuid() == x.uuid()).count() > 1
            })
            .unique_by(|p| p.id())
            .map(|p| p.id().to_string())
            .join(", ");

        let msg =
            format!("ППЗ/ДС имеют одинаковые идентификаторы {}", double_found);
        return Err(ProcessingError::DbInconsistency(msg));
    }

    Ok(plans)
}

/// Поиск по ренджам `Plan::id` идентификаторов
#[tracing::instrument(skip_all)]
pub(crate) async fn fetch_plans_by_range_ids<I>(
    ids: I,
    conn: &PgPool,
) -> Result<Vec<PlanOrAmendment>>
where
    I: IntoIterator<Item = RangeInclusive<i64>>,
{
    let mut unique_ids = AHashSet::new();
    let filter_ids = ids.into_iter().map(|range| {
        range.clone().for_each(|id| {
            unique_ids.insert(id);
        });

        if range.start() == range.end() {
            Filter::eq(Plan::id, range.start())
        } else {
            Filter::between(Plan::id, range.start(), range.end())
        }
    });
    let id_filter_tree = FilterTree::or_from_list(filter_ids);

    if unique_ids.len() == 0 {
        let msg = String::from("Был запрошен пустой массив ППЗ/ДС");
        return Err(ProcessingError::GetItemList(msg));
    }

    let plan_select =
        Select::full::<Plan>().set_filter_tree(id_filter_tree.clone());
    let amendment_select =
        Select::full::<ContractAmendment>().set_filter_tree(id_filter_tree.clone());

    let plans =
        PlanOrAmendment::select_dual(&plan_select, &amendment_select, conn).await?;

    if plans.len() < unique_ids.len() {
        let found_ids = plans.iter().map(|x| *x.id()).collect::<AHashSet<_>>();
        let missing = unique_ids
            .iter()
            .filter(|id| !found_ids.contains(id))
            .sorted_by(|id1, id2| id1.cmp(id2))
            .map(|id| id.to_string())
            .join(", ");

        let msg =
            format!("Записи ППЗ/ДС c идентификаторами {} не найдены", missing);
        return Err(ProcessingError::GetItemList(msg));
    }

    if plans.len() > unique_ids.len() {
        let double_found = plans
            .iter()
            .filter(|plan| {
                plans.iter().filter(|x| plan.id() == x.id()).count() > 1
                    || plans.iter().filter(|x| plan.uuid() == x.uuid()).count() > 1
            })
            .unique_by(|p| p.id())
            .map(|p| p.id().to_string())
            .join(", ");

        let msg = format!(
            "Нарушение целостности БД. ППЗ/ДС имеют одинаковые идентификаторы {}",
            double_found
        );
        return Err(ProcessingError::GetItemList(msg));
    }

    Ok(plans)
}
