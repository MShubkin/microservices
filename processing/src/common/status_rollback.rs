use crate::common::ProcessingError as PError;
use crate::common::Result;

use asez2_shared_db::db_item::{Filter, FilterTree, Select};
use asez2_shared_db::DbItem;
use shared_essential::domain::{
    ContractAmendment, Plan, PlanOrAmendment, PlanStatus, StatusHistory,
};

use ahash::AHashMap;
use sqlx::PgPool;

const UPDATE_FIELDS: Option<&[&str]> = Some(&[Plan::status_id]);

/// This function overrides many higher functions and rule checks and rolls back
/// the status to the previous status, but also records it into the status history
/// with a status note indicating the error.
pub(crate) async fn rollback_status(
    plans: Vec<PlanOrAmendment>,
    note: &str,
    pool: &PgPool,
) -> Result<()> {
    // We create a filter that retrieves status history records for this plan
    // which DO NOT have the CURRENT status.
    let tree = plans
        .iter()
        .map(|obj| {
            FilterTree::from(Filter::eq(StatusHistory::object_uuid, obj.uuid()))
                .and(
                    Filter::not_eq(StatusHistory::status_id, *obj.status_id())
                        .into(),
                )
        })
        .collect::<Vec<_>>();

    let tree = FilterTree::or_from_list(tree);
    // TODO: It is strange that we need asc and not desc to return the newest record.
    let history_select = Select::full::<StatusHistory>()
        .set_filter_tree(tree)
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_asc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid, StatusHistory::created_at]);

    let histories = StatusHistory::select(&history_select, pool)
        .await?
        .into_iter()
        .map(|h| (h.object_uuid, h))
        .collect::<AHashMap<_, _>>();

    let mut new_histories = Vec::with_capacity(histories.len());
    let mut plans_to_update = Vec::with_capacity(histories.len());
    let mut amendments_to_update = Vec::with_capacity(histories.len());

    // TODO: Decide whether this is suitable.
    let note = format!("Автоматический откат статуса системой: {note}");

    for mut plan in plans.into_iter() {
        let old_entry = histories.get(plan.uuid()).ok_or_else(|| {
            PError::StatusRevert({
                let record = format!(
                    "Не найден предыдущий статус для ППЗ/ДС: ({}, {})",
                    plan.id(),
                    plan.uuid()
                );
                tracing::error!(
                    kind = "broker",
                    "Откат статуса невозможен: {}",
                    record
                );
                record
            })
        })?;
        let previous_status = PlanStatus::from(old_entry.status_id);
        let new_history = StatusHistory::new(
            *plan.uuid(),
            previous_status,
            &note,
            *plan.changed_by_mut(),
        );
        new_histories.push(new_history);
        *plan.status_id_mut() = previous_status;
        match plan {
            PlanOrAmendment::Plan(x) => plans_to_update.push(x),
            PlanOrAmendment::Amendment(x) => amendments_to_update.push(x),
        }
    }
    // NB: We don't use the normal mechanism for two reasons:
    // 1. We need to skip the usual checks for status transitions
    //   (though we should be able to do that anyway)
    // 2. Not sure whether we need to do a field record for an automatic reversion,
    //    especially if it's in status notes anyway.
    // 3. Don't want to crosspollinate app_process and common.
    let mut tx = pool.begin().await?;

    Plan::update_vec(&plans_to_update, UPDATE_FIELDS, &mut tx).await?;
    ContractAmendment::update_vec(&amendments_to_update, UPDATE_FIELDS, &mut tx)
        .await?;
    StatusHistory::insert_vec(&mut new_histories, &mut tx).await?;

    tx.commit().await?;

    let ids = plans_to_update
        .iter()
        .map(|x| x.id)
        .chain(amendments_to_update.iter().map(|x| x.id))
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    tracing::warn!(
        kind = "broker",
        "Проведён откат статусов следующих ППЗ/ДС: {}.",
        ids
    );
    Ok(())
}
