//! This module contains the historian functionality.
//!
//! The historian scans changed fields in an item to be inserted and then
//! Inserts changed fields into the "field_history" table. It then alters
//! entries in this table as it goes.
use super::rules_lawyer::RulesLawyer;
use super::{Error, ProcessUpsert, Result, StatusHandler, UpdateCtx};
use crate::presentation::dto::response_request::{Message, Messages};

use crate::domain::StatusHistory;

use asez2_shared_db::db_item::selection::SelectionKind;
use asez2_shared_db::db_item::{DbItemExt, Field, Select};
use asez2_shared_db::{DbItem, Value};
use asez2_tables::{FieldChange, HistoryStatus};

use ahash::AHashMap;
use itertools::Itertools;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HistorianMode {
    Insert,
    Update,
    Upsert,
}

/// NB: Historian requires `#[sqlx(default)]` to be derived on subject items to run.
/// TODO: We may need to change the trait bounds if we are going to be splitting
/// update process at some point.
#[derive(Debug)]
pub(super) struct Historian<I> {
    /// The records being inserted or modified are temporarily held here,
    /// optionally along with corresponding old ones.
    items: Vec<I>,
    unchanged_items: Vec<I>,
    old_items: Vec<I>,
    fields_to_update: Vec<&'static str>,
    /// We also store the fields we are processing.
    field_changes: Vec<FieldChange>,
    /// We operate differently in update and insert mode.
    mode: HistorianMode,
}

impl<I: ProcessUpsert> Historian<I> {
    pub(super) fn new(
        items: Vec<I>,
        fields_to_update: &[&'static str],
        mode: HistorianMode,
    ) -> Self {
        let fields_to_update = fields_to_update.to_vec();
        Historian {
            items,
            unchanged_items: Vec::new(),
            old_items: Vec::new(),
            fields_to_update,
            field_changes: Vec::new(),
            mode,
        }
    }

    pub(super) async fn pre_update(
        &mut self,
        messages: &mut Messages,
        tx: &mut Transaction<'_, Postgres>,
        db_pool: &PgPool,
        status_notes: &AHashMap<Uuid, String>,
        ctx: &UpdateCtx,
    ) -> Result<()> {
        let mut items = std::mem::take(&mut self.items);

        items.iter_mut().for_each(ProcessUpsert::generate_uuid_if_needed);

        // Для записи истории изменений полей мы используем отдельную транзакцию,
        // чтобы записи оставались и в случае, когда сами изменения не были выполнены.
        //let pool: &PgConnection = (&*tx).deref();
        let mut t = db_pool.begin().await?;

        let old_items = if !matches!(self.mode, HistorianMode::Insert) {
            // TODO: Decide whether we need to get items also from "SAP" for the sake
            // of comparing which fields need to be updated (aka at this stage!).
            let select = make_select_by_uuids(&items, &self.fields_to_update);
            let mut old = I::select(&select, &mut *t).await?;
            // This is needed for a definite historic sort.
            items.sort_by_key(|a| a.record_uuid());
            old.sort_by_key(|a| a.record_uuid());
            old
        } else {
            Vec::new()
        };

        // TODO: Decide whether an empty update is an error case or not.
        let CrossCheckResult {
            items,
            unchanged_items,
            old_items,
            changes,
            new_uuids,
        } = crosscheck(old_items, items, &self.fields_to_update, ctx);

        let mut field_changes = changes
            .into_iter()
            .map(|x| NewFieldChange::field_change(x, ctx))
            .collect::<Vec<_>>();

        // TODO: Do we need to filter out completely unchanged DbItems?

        // since id (BIGSERIAL) is not generated on our end, we must get fields like this.
        field_changes =
            FieldChange::insert_vec_returning(&mut field_changes, &mut t).await?;

        t.commit().await?;

        // If there are items that do not exist, we cannot reasonably complete the update.
        // However we should still record the attempt, hence this is done after histories
        // are recorded.
        if matches!(self.mode, HistorianMode::Update) && !new_uuids.is_empty() {
            messages.add_prepared_message(Message::stop(format!(
                "Строки с UUID {} не существует.",
                new_uuids.into_iter().join(", ")
            )));
            return Err(Error::UpdateFailed(I::TABLE, messages.to_owned()));
        }

        // Мы записываем истории статусов на этом этапе.
        update_status_history::<I>(&field_changes, status_notes, tx).await?;

        self.items = items;
        self.unchanged_items = unchanged_items;
        self.old_items = old_items;
        self.field_changes = field_changes;

        Ok(())
    }

    /// NB: For testing purposes, this "works" but give warnings.
    pub(super) async fn complete(
        self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<I>> {
        let Historian {
            mut items,
            unchanged_items,
            mut fields_to_update,
            mut field_changes,
            mode,
            ..
        } = self;

        if !matches!(mode, HistorianMode::Insert) {
            fields_to_update.extend(I::CTX_UPDATE_FIELDS);
        };

        let mut ret = match mode {
            HistorianMode::Insert => {
                I::insert_vec_returning(&mut items, tx).await?
            }
            HistorianMode::Update => {
                I::update_vec_returning(
                    &items,
                    Some(&fields_to_update),
                    Some(I::FIELDS),
                    tx,
                )
                .await?
            }
            HistorianMode::Upsert => {
                I::upsert_returning(&mut items, Some(&fields_to_update), tx).await?
            }
        };
        // NB: This must be part of the main transaction, since rolling back the update
        // to the main table should also mean that the update is never completed. Thus
        // this is rolled back together with the actual table update.
        // (This is why we do not use `complete_fields_update` here)
        for x in field_changes.iter_mut() {
            x.record_status = HistoryStatus::Finished;
        }
        FieldChange::mass_update_status(
            &field_changes,
            HistoryStatus::Finished,
            tx,
        )
        .await?;
        ret.extend(unchanged_items);
        Ok(ret)
    }
}

impl<I: ProcessUpsert + RulesLawyer> Historian<I> {
    /// NB: For testing purposes, this "works" but give warnings.
    pub(super) async fn check_rules<T: StatusHandler>(
        &mut self,
        msgs: &mut Messages,
        status_handler: T,
        db_pool: &PgPool,
    ) -> Result<()> {
        // Start with a check of the rules.
        let pass = (match self.mode {
            HistorianMode::Insert => {
                status_handler.check_insert(&self.items, msgs).await
            }
            HistorianMode::Update => {
                status_handler
                    .check_update(
                        &self.fields_to_update,
                        &self.items,
                        &self.old_items,
                        msgs,
                    )
                    .await
            }
            HistorianMode::Upsert => {
                status_handler
                    .check_upsert(
                        &self.fields_to_update,
                        &self.items,
                        &self.old_items,
                        msgs,
                    )
                    .await
            }
        })
        .map_err(Error::status_error)?;

        if pass {
            complete_fields_update(
                &mut self.field_changes,
                HistoryStatus::Checked,
                db_pool,
            )
            .await?;
            Ok(())
        } else {
            msgs.add_prepared_message(Message::stop(
                "Rules check for plan failed.".to_string(),
            ));
            Err(Error::Rules(I::TABLE, msgs.to_owned()))
        }
    }
}

// Convenience fn for updating the status of the FieldChange records.
async fn inner_field_change(
    fields: &mut [FieldChange],
    new_status: HistoryStatus,
    t: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    for x in fields.iter_mut() {
        x.record_status = new_status;
    }
    FieldChange::mass_update_status(fields, new_status, t).await?;
    Ok(())
}

// Convenience fn for updating the status of the FieldChange records.
async fn complete_fields_update(
    fields: &mut [FieldChange],
    new_status: HistoryStatus,
    pool: &PgPool,
) -> Result<()> {
    let mut t = pool.begin().await?;
    inner_field_change(fields, new_status, &mut t).await?;
    t.commit().await?;

    Ok(())
}

#[derive(Debug, PartialEq)]
pub(crate) struct NewFieldChange {
    pub(crate) record_uuid: uuid::Uuid,
    pub(crate) table_name: &'static str,
    pub(crate) field_name: &'static str,
    pub(crate) field_value: Option<Value>,
}
impl NewFieldChange {
    fn from_field(f: Field, uuid: Uuid, table_name: &'static str) -> Self {
        Self {
            record_uuid: uuid,
            table_name,
            field_name: f.field(),
            field_value: f.value,
        }
    }

    fn field_change(self, ctx: &UpdateCtx) -> FieldChange {
        FieldChange {
            // Id is not inserted anyway.
            id: 0,
            record_uuid: self.record_uuid,
            table_name: self.table_name.to_owned(),
            field_name: self.field_name.to_owned(),
            field_value: self.field_value.map(sqlx::types::Json),
            record_status: HistoryStatus::Proposed,
            created_by: ctx.user_id,
            created_at: ctx.timestamp,
        }
    }
}

/// This is a convenience function for making a selection based on Uuids of items.
fn make_select_by_uuids<I: DbItemExt>(items: &[I], fields: &[&str]) -> Select {
    let uuids = items.iter().map(|x| Value::Uuid(x.record_uuid()));
    // pkey MUST be present for subsequent check.
    // HashSet is used for extra safety (dedup duplicate keys).
    let fields =
        fields.iter().copied().chain(I::PRIMARY_KEYS.iter().copied()).unique();

    Select::with_fields(fields).add_expand_filter("uuid", SelectionKind::In, uuids)
}

// Создаём историю статусов на базе лишь изменённый полей.
async fn update_status_history<I: ProcessUpsert>(
    field_changes: &[FieldChange],
    status_notes: &AHashMap<Uuid, String>,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    if let Some(f) = I::STATUS_FIELD {
        let mut status_histories = field_changes
            .iter()
            .filter_map(|x| {
                let comment = status_notes
                    .get(&x.record_uuid)
                    .map(|x| x as &str)
                    .unwrap_or_default();
                x.as_status_history(f, comment)
            })
            .collect::<Vec<_>>();
        StatusHistory::insert_vec(&mut status_histories, tx).await?;
    }
    Ok(())
}

struct CrossCheckResult<I> {
    /// Новые и обновленные сущности.
    items: Vec<I>,
    /// Сущности, не требующие обновления.
    unchanged_items: Vec<I>,
    /// Сущности до обновления.
    old_items: Vec<I>,
    /// UUIDы новых сущностей.
    new_uuids: Vec<Uuid>,
    /// Изменения полей сущностей.
    changes: Vec<NewFieldChange>,
}

/// Returns field for update and orphan keys (records not already in DB).
fn crosscheck<I>(
    old: Vec<I>,
    new: Vec<I>,
    fields: &[&str],
    ctx: &UpdateCtx,
) -> CrossCheckResult<I>
where
    I: ProcessUpsert,
{
    let mut old_with_keys = old
        .into_iter()
        .map(|n| (n.record_uuid(), n))
        .collect::<AHashMap<Uuid, I>>();
    // This is used to filter out fields which are not marked for update.
    let valid_fields = fields.iter().copied().collect::<HashSet<_>>();

    let mut items = Vec::with_capacity(new.len());
    let mut unchanged_items = Vec::new();
    let mut old_items = Vec::with_capacity(old_with_keys.len());
    let mut new_uuids = Vec::with_capacity(new.len());
    let mut all_changes = Vec::with_capacity(new.len() * valid_fields.len()); //???

    for mut new in new {
        let uuid = new.record_uuid();

        if let Some(old) = old_with_keys.remove(&uuid) {
            // If we find that the new vector contains keys corresponding to the old
            // one, we add changed fields for the item while removing the item.
            let changes = old
                .differing_fields(&new)
                .into_iter()
                .filter(|f| valid_fields.contains(f.field()))
                .map(|f| NewFieldChange::from_field(f, uuid, I::TABLE))
                .collect::<Vec<_>>();

            if !changes.is_empty() {
                new.apply_update_ctx(ctx);
                all_changes.extend(
                    changes
                        .into_iter()
                        .filter(|fc| !I::is_ctx_update_field(fc.field_name)),
                );
                items.push(new);
                old_items.push(old);
            } else {
                unchanged_items.push(new);
            }
        } else {
            // first, apply context as it might change uuid
            new.apply_insert_ctx(ctx);
            let changes = new
                .fields_with_values()
                .into_iter()
                .filter(|f| {
                    !I::is_ctx_update_field(f.field())
                        && valid_fields.contains(f.field())
                })
                .map(move |f| NewFieldChange::from_field(f, uuid, I::TABLE));
            all_changes.extend(changes);
            new_uuids.push(new.record_uuid());
            items.push(new);
        }
    }

    CrossCheckResult {
        items,
        unchanged_items,
        old_items,
        changes: all_changes,
        new_uuids,
    }
}

#[cfg(test)]
mod tests;
