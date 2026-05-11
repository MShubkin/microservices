//! Начальный модуль до асинхронный, низкогабаритной отправки сообщений на монолит.
//! Отдельная task крутится и по одному посылает сообщения на монолит.
use super::ProcessingError as PError;
use super::NO_SEND_TO_PLANNING;
use super::{RabbitConfig, Result};
use crate::presentation::legacy_interaction::*;

use asez2_shared_db::db_item::from_item_with_fields;

use asez2_shared_db::db_item::DbItemDel;
use asez2_shared_db::db_item::{Filter, Select};
use asez2_shared_db::{DbAdaptor, DbItem, Value};
use broker::rabbit::{RabbitAdapter, RabbitMessage};
use shared_essential::application::records::Recorder;
use shared_essential::domain::*;
use shared_essential::presentation::dto::processing::legacy_interaction::*;
use shared_essential::presentation::dto::response_request::Messages;

use amqprs::channel::BasicPublishArguments;
use amqprs::BasicProperties;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Проверяем monolith_sender_object 2 раза в секунду.
const SENDER_OBJECT_WAIT_MS: u64 = 500;

#[derive(Debug, Clone, DbItem)]
#[item_table = "monolith_sender_object"]
pub(crate) struct MonolithSenderObject {
    #[item_field_pkey]
    #[item_field_autogen]
    id: i64,
    messages: Json<ObjectInner>,
}
impl DbItemDel for MonolithSenderObject {}

#[derive(Debug, Clone, DbItem)]
#[item_table = "monolith_sender_object"]
/// This is used to mark monolith sender objects as locked.
pub(crate) struct MonolithSenderObjectLocked {
    #[item_field_pkey]
    #[item_field_autogen]
    id: i64,
    locked: bool,
    last_error: Option<String>,
}

impl MonolithSenderObjectLocked {
    fn new(id: i64) -> Self {
        Self {
            id,
            locked: true,
            last_error: None,
        }
    }
    fn add_error(mut self, error: &str) -> Self {
        self.last_error = Some(error.to_string());
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub(crate) enum ObjectInner {
    UpdatePlans(InsertUpdateSrmPlansReq),
    UpdateAmendments(InsertUpdateSrmAmendmentsReq),
    #[default]
    None,
}

impl MonolithSenderObject {
    async fn inner_or_delete(
        self,
        pool: &PgPool,
    ) -> Result<Option<(&'static str, ProcessingToLegacyReq)>> {
        use ProcessingToLegacyReq::*;

        let sendable = match self.messages.0 {
            ObjectInner::UpdatePlans(x) => (SEND_TO_MONOLITH_QUEUE, UpdatePlans(x)),
            ObjectInner::UpdateAmendments(x) => {
                (SEND_TO_MONOLITH_QUEUE, UpdateAmendments(x))
            }
            ObjectInner::None => {
                let id = self.id;
                Self::delete(id, pool).await?;
                return Ok(None);
            }
        };
        Ok(Some(sendable))
    }

    pub(crate) fn new(x: ObjectInner) -> Self {
        Self {
            id: 0, //autogen
            messages: Json(x),
        }
    }

    async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        let filters = Filter::eq(Self::id, id);

        Self::delete_returning(&filters.into(), pool).await?;
        Ok(())
    }

    /// NB: Sapable MUST make sense to monolith,
    #[tracing::instrument(skip_all)]
    async fn send_and_delete(
        self,
        adapter: &RabbitAdapter,
        marker: MonolithSenderObjectLocked,
        db_pool: &PgPool,
    ) -> Result<i64> {
        const FAKE_TIMEOUT: u64 = 40_000;
        let id = self.id;

        let (queue_name, sendable) = match self.inner_or_delete(db_pool).await? {
            None => return Ok(id),
            Some(x) => x,
        };

        let props = BasicProperties::default()
            .with_content_type("application/json")
            .with_persistence(true)
            .finish();
        let args = BasicPublishArguments::new("", queue_name);
        let tag = format!("processing<-monolith-consumer-{}", Uuid::new_v4());

        // TODO: Узнать нужно ли нам ждать ответа с монолит (сильно замедляет)
        // процесс обновления если да.
        let mut direct_reply = adapter.direct_reply(props, args, &tag).await?;
        let timeout = std::time::Duration::from_millis(FAKE_TIMEOUT);

        let messages: Result<RabbitMessage<Messages>> =
            direct_reply.request(&sendable, timeout).await.map_err(Into::into);

        // We then send the object. If for whatever reason the send we revert the status
        // to what it was before, and then delete it from the table.
        // NB: This is important to avoid having an object that fails to reach teh other part
        // of the system, and at the same time cannot be displayed or worked with in our part of the system
        // due to a bad status. It allows the user to then manually check why the object does not send
        // and correct the problem if it is a real error.
        let revert = match messages {
            Err(e) => {
                tracing::error!(
                    kind = "broker",
                    "Невозможно послать объект: {:#?}",
                    e
                );
                Some(e.to_string())
            }
            // If there is an internal logic error within the process on the
            // other side, we get a success with an internal error instead.
            // In this case we must still revert.
            Ok(m) if m.content.is_error() => {
                let m = m.content;
                tracing::error!(
                    kind = "broker",
                    "Невозможно послать объект: {:#?}",
                    m
                );
                Some(
                    m.messages
                        .get(0)
                        .map(|x| x.text.as_ref())
                        .unwrap_or("Невозможно послать объект")
                        .to_string(),
                )
            }
            Ok(m) => {
                tracing::debug!(kind = "broker", "Послали объект но.{}.", id);
                tracing::debug!(kind = "broker", "{:?}", m);
                None
            }
        };

        // If we have an error, we add it to the record and keep it locked for posterity.
        // If we have no error, we delete the record.
        if let Some(e) = revert {
            marker
                .add_error(&e)
                .update(Some(&[MonolithSenderObjectLocked::last_error]), db_pool)
                .await?;

            let plans = get_plans_or_amendments(sendable)?;

            super::status_rollback::rollback_status(plans, &e, db_pool)
                .await
                .map_err(|e| {
                    tracing::error!(
                        kind = "broker",
                        "Невозможно откатить статус {e:#?}"
                    );
                    e
                })?;
        } else {
            Self::delete(id, db_pool).await?;
        }
        Ok(id)
    }
}

/// Перевести заголовки ППЗ/ДС в формат с которым мы можем работать.
/// Этот шаг нужен для того чтобы можно ныло откатить статусы.
fn get_plans_or_amendments(
    input: ProcessingToLegacyReq,
) -> Result<Vec<PlanOrAmendment>> {
    let res = match input {
        ProcessingToLegacyReq::UpdatePlans(x) => x
            .into_iter()
            .map(|x| {
                let plan =
                    PlanRep::try_from(x.header).map_err(PError::StatusRevert)?;
                let plan = plan.into_item()?;
                Ok(PlanOrAmendment::Plan(plan))
            })
            .collect::<Result<Vec<_>>>()?,
        ProcessingToLegacyReq::UpdateAmendments(x) => x
            .into_iter()
            .map(|x| {
                let plan = ContractAmendmentRep::try_from(x.header)
                    .map_err(PError::StatusRevert)?;
                let plan = plan.into_item()?;
                Ok(PlanOrAmendment::Amendment(plan))
            })
            .collect::<Result<Vec<_>>>()?,
    };
    Ok(res)
}

#[derive(Debug, Clone)]
pub(crate) struct MonolithSender {
    adapter: RabbitAdapter,
    db_pool: PgPool,
    rabbit_cfg: RabbitConfig,
    // This pool does not log.
    silent_pool: PgPool,
    handle: Arc<RwLock<bool>>,
}

impl MonolithSender {
    pub(crate) async fn new(cfg: &RabbitConfig, db_pool: &PgPool) -> Result<Self> {
        let adapter = cfg.get_rabbit().await?;
        Ok(Self {
            adapter,
            rabbit_cfg: cfg.clone(),
            db_pool: db_pool.clone(),
            // The silent pool is not silent.
            silent_pool: db_pool.clone(),
            handle: RwLock::new(true).into(),
        })
    }

    /// This function is used in reality, but not in tests.
    pub(crate) async fn add_silent_pool(mut self) -> Result<Self> {
        if std::env::var(NO_SEND_TO_PLANNING).is_err() {
            let silent_pool =
                asez2_shared_db::PgDbOptions::from_env()?.get_silent_pool();
            self.silent_pool = silent_pool;
        }
        Ok(self)
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn run(&self) {
        if std::env::var(NO_SEND_TO_PLANNING).is_ok() {
            return;
        }
        let inner = self.clone();

        tokio::task::spawn(async move { inner.run_inner().await });
    }

    pub(crate) async fn stop(self) {
        let mut guard = self.handle.write().await;
        *guard = false;
        drop(guard);
    }

    /// This is a simple sender, for simplicity it waits for each message to be sent, despite the
    /// low throughput we may end up with as a result.
    async fn run_inner(mut self) -> Result<()> {
        while *self.handle.read().await {
            // Check whether rabbit is alive.
            let ch = self.adapter.connection();
            if !ch.is_open() {
                let new_rabbit = self.rabbit_cfg.get_rabbit().await?;
                _ = std::mem::replace(&mut self.adapter, new_rabbit);
            }

            if let Err(e) = Self::run_inner_once(
                &self.adapter,
                &self.db_pool,
                &self.silent_pool,
            )
            .await
            {
                tracing::trace!(
                    kind = "broker",
                    "Ошибка при отправки на монолит {:?}",
                    e
                );
            }
            // Если не вставлять, то выжигаем процессор и постгрес.
            tokio::time::sleep(std::time::Duration::from_millis(
                SENDER_OBJECT_WAIT_MS,
            ))
            .await;
        }
        tracing::info!("Отключаем послание ППЗ/ДС на монолит.");
        Ok(())
    }

    async fn run_inner_once(
        adapter: &RabbitAdapter,
        db_pool: &PgPool,
        silent_pool: &PgPool,
    ) -> Result<()> {
        const FUTURES: usize = 20;
        const LOCK_QUERY: &str =
            "LOCK TABLE monolith_sender_object IN ACCESS EXCLUSIVE MODE";

        let mut tx = silent_pool.begin().await?;
        // We lock the rows we are working with, even for access, until the objects have been locked
        // at the level of the table.
        // NB: We cannot lock the table for the whole sending process as this takes a while and we cannot
        // afford to disrupt potential writes (by other processes) to the table for extended periods of time
        // as this will disrupt all calls that change Plan/Amendment status.
        sqlx::query(LOCK_QUERY).execute(&mut *tx).await?;
        // We retrieve the ids to all stored tasks and then send them one at a time.
        // This increases throughput, without increasing the complexity of the mechanism
        // as a task-spawing mechanism with a separate return mechanism for deletion would
        // have.
        // Since most of the time is not processing time, but waiting time, this should be fine.
        let ids =
            sqlx::query("select id from monolith_sender_object where locked=false")
                .try_map(|x| <(i64,)>::from_row(&x).map(|(x,)| x))
                .fetch_all(&mut tx)
                .await?;
        // Here we lock the objects we're retrieving.
        let markers = ids
            .into_iter()
            .map(MonolithSenderObjectLocked::new)
            .collect::<Vec<_>>();

        MonolithSenderObjectLocked::update_vec(
            &markers,
            Some(&[MonolithSenderObjectLocked::locked]),
            &mut tx,
        )
        .await?;
        tx.commit().await?;

        // Here we get individual objects and spawn the future that sends and deletes them.
        let mut futures = Vec::with_capacity(markers.len());
        for m in markers {
            // First we search for suitable objects.
            let oldest =
                Select::default().eq(MonolithSenderObject::id, m.id).take_first();
            let mut obj = MonolithSenderObject::select(&oldest, db_pool).await?;

            if let Some(x) = obj.pop() {
                futures.push(x.send_and_delete(adapter, m, db_pool));
            }
        }

        // Once the futures are gathered, they are resolved. Any deletions that take place
        // following successful sends are resolved here, before the next cycle, so we do not
        // need any extra mechanisms for returning ids.
        let mut live_handles =
            futures::stream::iter(futures).buffer_unordered(FUTURES);
        while let Some(r) = live_handles.next().await {
            match r {
                Ok(r) => tracing::info!(
                    kind = "broker",
                    "Сообщение но.{} отправлено.",
                    r
                ),
                Err(e) => tracing::error!(kind = "broker", "{:?}", e),
            }
        }
        Ok(())
    }
}

/// Get all amendments and items with appropriate uuids and make a sendable object.
pub(crate) async fn amendments_for_monolith(
    uuids: impl Iterator<Item = Value> + ExactSizeIterator,
    recorder: &mut Recorder<'_>,
) -> Result<Option<ObjectInner>> {
    if uuids.len() == 0 {
        tracing::debug!("Пустой список ДС, не отправляем");
        return Ok(None);
    }

    tracing::info!("Подготовка {} ДС для отправки на монолит", uuids.len());

    let select = Select::full_in::<_, Plan>(Plan::uuid, uuids);

    let from_ca =
        from_item_with_fields::<ContractAmendmentRep, _, _>(AMENDMENT_FIELDS);
    let from_item = from_item_with_fields::<ContractAmendmentItemRep, _, _>(
        AMENDMENT_ITEM_FIELDS,
    );
    let full = FullAmendmentSelect::new(select)
        // NB: Мы должны использовать транзакцию чтобы брать обновлённые данные.
        .get(recorder.tx())
        .await?
        .into_iter()
        .map(|x| {
            let uuid = x.amendment.uuid;
            let mut header: ContractAmendmentLegacyRep =
                from_ca(x.amendment).into();
            // Надо добавить status_note, если он есть.
            header.status_note = recorder.find_status_note(&uuid);

            let items = x
                .items
                .into_iter()
                .map(&from_item)
                .map(ContractAmendmentItemLegacyRep::from)
                .collect();
            Ok(AmendmentFromSrm {
                header,
                items,
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(ObjectInner::UpdateAmendments(full)))
}

/// Get all plans and items with appropriate uuids and make a sendable object.
pub(crate) async fn plans_for_monolith(
    uuids: impl Iterator<Item = Value> + ExactSizeIterator,
    recorder: &mut Recorder<'_>,
) -> Result<Option<ObjectInner>> {
    if uuids.len() == 0 {
        tracing::debug!("Пустой список ППЗ, не отправляем");
        return Ok(None);
    }

    tracing::info!("Подготовка {} ППЗ для отправки на монолит", uuids.len());

    let select = Select::full_in::<_, Plan>(Plan::uuid, uuids);

    let from_plan = from_item_with_fields::<PlanRep, _, _>(PLAN_FIELDS);
    let from_item =
        from_item_with_fields::<PlanItemFullRep, _, _>(PLAN_ITEM_FIELDS);
    let full = FullPlanSelect::new(select)
        // NB: Мы должны использовать транзакцию чтобы брать обновлённые данные.
        .get(recorder.tx())
        .await?
        .into_iter()
        .map(|x| {
            let uuid = x.plan.uuid;
            let mut header: PlanLegacyRep = from_plan(x.plan).into();
            // Надо добавить status_note, если он есть.
            header.status_note = recorder.find_status_note(&uuid);

            let items = x
                .items
                .into_iter()
                .map(&from_item)
                .map(PlanItemLegacyRep::from)
                .collect();
            Ok(PlanFromSrm {
                header,
                items,
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(ObjectInner::UpdatePlans(full)))
}

// Константы из задачи
// https://rcportal.inlinegroup.ru/web#id=2775&cids=1&menu_id=112&model=project.task&view_type=form
// Некоторые из заданных полей в принципе в этих структурах не существуют, но ини останутся в
// константах, так как задача так написана.
// Поля которые не существуют не будут посылаться.
const AMENDMENT_FIELDS: &[&str] = &[
    // amendment technical fields.
    "organizer_id",
    "is_cooperative",
    "is_list_price",
    "commission_date",
    "commission_kind_id",
    "price_analysis_method_id",
    "pricing_expert_id",
    "pricing_method_id",
    "pricing_resume",
    "expert_conclusion_id",
    "is_savings_accounting",
    "savings_sum_excluded_vat",
    "savings_sum_excluded_vat_rub",
    "savings_sum_included_vat",
    "savings_sum_included_vat_rub",
    "status_id",
    "status_note",
    "uuid",
    "id",
    "local_version",
    "local_version_uuid",
    "is_check_documentation",
    "check_documentation_date",
    // amendment general fields
    "pricing_currency_id",
    "pricing_currency_rate",
    "pricing_currency_id",
    "pricing_vat_id",
    "pricing_sum_vat",
    "pricing_sum_vat_rub",
    "pricing_sum_excluded_vat",
    "pricing_sum_excluded_vat_rub",
    "pricing_sum_included_vat",
    "pricing_sum_included_vat_rub",
    "pricing_transportation_vat_id",
    "pricing_transportation_sum_vat",
    "pricing_transportation_sum_vat_rub",
    "pricing_transportation_price",
    "pricing_transportation_price_rub",
    "pricing_transportation_sum_included_vat",
    "pricing_transportation_sum_included_vat_rub",
    "pricing_total_sum",
    "pricing_total_sum_rub",
    "changed_at",
    "changed_by",
];
const AMENDMENT_ITEM_FIELDS: &[&str] = &[
    // amendment technical fields.
    "pricing_expert_id",
    "pricing_resume",
    "uuid",
    "id",
    // amendment item fields
    "pricing_currency_rate_date",
    "pricing_currency_rate",
    "pricing_currency_id",
    "pricing_unit_id",
    "pricing_quantity",
    "pricing_price",
    "pricing_price_rub",
    "pricing_vat_id",
    "pricing_sum_vat",
    "pricing_sum_vat_rub",
    "pricing_sum_excluded_vat",
    "pricing_sum_excluded_vat_rub",
    "pricing_sum_included_vat",
    "pricing_sum_included_vat_rub",
    "pricing_transportation_price_rub",
    "pricing_transportation_vat_id",
    "pricing_transportation_sum_vat",
    "pricing_transportation_sum_vat_rub",
    "pricing_transportation_sum_included_vat",
    "pricing_transportation_sum_included_vat_rub",
    "pricing_total_sum",
    "pricing_total_sum_rub",
    "pricing_delta_unit_id",
    "pricing_delta_quantity",
    "pricing_delta_currency_id",
    "pricing_delta_currency_rate_date",
    "pricing_delta_price",
    "pricing_delta_price_rub",
    "pricing_delta_sum_excluded_vat",
    "pricing_delta_sum_excluded_vat_rub",
    "pricing_delta_sum_vat",
    "pricing_delta_sum_vat_rub",
    "pricing_delta_sum_included_vat",
    "pricing_delta_sum_included_vat_rub",
    "pricing_delta_transportation_price",
    "pricing_delta_transportation_sum_vat",
    "pricing_delta_transportation_sum_vat_rub",
    "pricing_delta_transportation_sum_included_vat",
    "pricing_delta_transportation_sum_included_vat_rub",
    "pricing_delta_total_sum",
    "pricing_delta_total_sum_rub",
    "changed_at",
    "changed_by",
];
const PLAN_FIELDS: &[&str] = &[
    "organizer_id",
    "is_cooperative",
    "is_list_price",
    "commission_date", // Not in monolith
    "commission_kind_id",
    "pricing_method_id", // price_analysis_method_id
    "pricing_expert_id",
    "pricing_resume",
    "expert_conclusion_id",
    "is_savings_accounting",
    "savings_sum_excluded_vat",
    "savings_sum_excluded_vat_rub",
    "savings_sum_included_vat",
    "savings_sum_included_vat_rub",
    "status_id",
    "status_note",
    "uuid",
    "id",
    "local_version",            // does not exist
    "local_version_uuid",       // does not exxist.
    "is_check_documentation",   // does not exist in monolith
    "check_documentation_date", // does not exist in monolith
    "pricing_currency_id",
    "pricing_currency_rate",
    "pricing_currency_id",
    "pricing_vat_id",
    "pricing_sum_vat",
    "pricing_sum_vat_rub",
    "pricing_sum_excluded_vat",
    "pricing_sum_excluded_vat_rub",
    "pricing_sum_included_vat",
    "pricing_sum_included_vat_rub",
    "pricing_transportation_vat_id",
    "pricing_transportation_price",
    "pricing_transportation_price_rub",
    "pricing_transportation_sum_vat",
    "pricing_transportation_sum_vat_rub",
    "pricing_transportation_sum_included_vat",
    "pricing_transportation_sum_included_vat_rub",
    "pricing_total_sum",
    "pricing_total_sum_rub",
    "pricing_currency_rate",
    "pricing_unit_id",
    "pricing_quantity",
    "pricing_price",
    "pricing_price_rub",
    "changed_at",
    "changed_by",
    "reason_cancel_id",
    "replaced_id",
];
const PLAN_ITEM_FIELDS: &[&str] = &[
    "uuid",
    "id",
    "pricing_currency_id",
    "pricing_currency_rate",
    "pricing_vat_id",
    "pricing_transportation_price",
    "pricing_transportation_price_rub",
    // Plan item fields.
    "plan_uuid",
    "pricing_currency_rate_date",
    "pricing_unit_id",
    "price_unit",
    "pricing_quantity",
    "pricing_price",
    "pricing_price_rub",
    "pricing_sum_vat",
    "pricing_sum_vat_rub",
    "pricing_sum_excluded_vat",
    "pricing_sum_excluded_vat_rub",
    "pricing_sum_included_vat",
    "pricing_sum_included_vat_rub",
    "pricing_transportation_vat_id",
    "pricing_transportation_sum_vat",
    "pricing_transportation_sum_vat_rub",
    "pricing_transportation_sum_included_vat",
    "pricing_transportation_sum_included_vat_rub",
    "pricing_total_sum",
    "pricing_total_sum_rub",
    "plan_id_lotting",
    "uuid_item_proposal",
    "changed_at",
    "changed_by",
];
