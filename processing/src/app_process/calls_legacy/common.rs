use super::traits::*;
use crate::common::{ProcessingCtx, ProcessingError as PError, Result};

use asez2_shared_db::db_item::{
    make_bind_mask, selection::*, AsezTimestamp, DbFieldMask, DbItemDel, DbUpsert,
    DbVersioned, Select,
};
use asez2_shared_db::{DbAdaptor, DbItem};
use itertools::Itertools;
use shared_essential::application::records::{ProcessUpsert, Recorder, UpdateCtx};
use shared_essential::domain::traits::*;
use shared_essential::domain::{
    CommissionKind, ContractAmendment, ContractAmendmentItem,
    ContractAmendmentItemRep, ContractAmendmentRep, DocumentApprover,
    DocumentApproverRep, HasIsActual, Plan, PlanItemFull, PlanItemFullRep, PlanRep,
    PlanRetrospective, PlanRetrospectiveRep,
};
use shared_essential::presentation::dto::response_request::Messages;

use ahash::{AHashMap, AHashSet};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

type PlanComparator = Comparator<PlanItemFull, PlanItemFullRep, Plan, PlanRep>;
type AmendmentComparator = Comparator<
    ContractAmendmentItem,
    ContractAmendmentItemRep,
    ContractAmendment,
    ContractAmendmentRep,
>;

/// Create a bind mask for the fields that must be set to the default value
/// when updating certain plans.
fn make_mask<T: DbItem>(masks: &[&[&str]]) -> DbFieldMask<T> {
    let mut fields: Vec<&str> = vec![];
    for mask in masks {
        fields.extend(*mask);
    }
    let mut output = make_bind_mask::<T>(&fields);
    // We must deactivate the uuid (see how `make_bind_mask` works)
    output[0] = false;
    output
}

impl<T: DbItem> Masks<T> {
    fn new() -> Self {
        let drop_if_old_v = make_mask(&[IGNORE_IF_SAME_VERSION]);
        let drop2_if_old_v_unit_same = make_mask(&[
            CLEAR_NEW_VERSION_FIELDS,
            NEW_VERSION_AND_PRICING_UNIT,
            EXTRA_IGNORE_FIELDS,
        ]);
        let zero2_if_old_v_unit_changed = make_mask::<T>(&[
            CLEAR_NEW_VERSION_FIELDS,
            NEW_VERSION_AND_PRICING_UNIT,
        ]);
        let zero_if_new_v = make_mask(&[CLEAR_NEW_VERSION_FIELDS]);
        let zero2_if_new_v_unit_changed =
            make_mask(&[NEW_VERSION_AND_PRICING_UNIT]);

        Self {
            drop_if_old_v,
            drop2_if_old_v_unit_same,
            zero2_if_old_v_unit_changed,
            zero_if_new_v,
            zero2_if_new_v_unit_changed,
        }
    }
}

// This exists for DRY.
fn merge_header<Rep, Db, IRep, IDb>(
    mut rep: Rep,
    old: &Db,
    masks: &Masks<Db>,
    ctx: &UpdateCtx,
) -> Result<(Db, bool, bool)>
where
    Rep: UpdateHeaderRep<Db, IRep, IDb>,
    Db: UpdateHeader<IRep, IDb> + Compare<IDb, IRep, Rep>,
    IRep: UpdateItemRep<IDb>,
    IDb: UpdateItem<Db, IRep>,
{
    // If we are updating (and only then), then if pricing organisation unit differs
    // we reset fields. If, when updating, they are the same, we take the values from
    // the DB.
    let changed_org_unit = rep.pricing_organization_unit_id().is_some()
        && Some(old.pricing_organization_unit_id())
            != rep.pricing_organization_unit_id();
    let new_version = rep.maybe_pricing_started_at().is_some()
        && Some(old.pricing_started_at()) != rep.maybe_pricing_started_at();

    let reset = changed_org_unit || new_version;

    let reset_by_status = rep.maybe_status_id().map(|x| !x.is_ec()).unwrap_or(true);
    let backup_commission_kind_id = rep.commission_kind_id();

    rep = match new_version {
        false => rep.unset_fields(&masks.drop_if_old_v),
        true => rep.zero_fields(&masks.zero_if_new_v),
    };
    rep = match (new_version, changed_org_unit) {
        (false, false) => rep.unset_fields(&masks.drop2_if_old_v_unit_same),
        (false, true) => rep.zero_fields(&masks.zero2_if_old_v_unit_changed),
        (true, false) => rep,
        (true, true) => rep.zero_fields(&masks.zero2_if_new_v_unit_changed),
    };

    let mut new = rep.into_item_merged(old.clone())?;
    // If we are changing ot estimated commission status and there is no
    // commission kind in the DB, we use what is sent to us from SRM.
    if !reset_by_status && old.commission_kind_id() == CommissionKind::Undefined {
        *new.commission_kind_id_mut() = backup_commission_kind_id;
    }
    new.set_pricing_changed_at(ctx.timestamp);

    Ok((new, reset, new_version))
}

impl Compare<PlanItemFull, PlanItemFullRep, PlanRep> for Plan {
    fn new_insert(mut self, ctx: &UpdateCtx) -> PlanComparator {
        self.pricing_created_at = ctx.timestamp;
        self.pricing_changed_at = ctx.timestamp;

        Comparator {
            h: self,
            reset_pricing_fields: None,
            new_version: true,
            ..Comparator::default()
        }
    }
    fn new_update(
        rep: PlanRep,
        old: &Plan,
        masks: &Masks<Plan>,
        ctx: &UpdateCtx,
    ) -> Result<PlanComparator> {
        let (new, reset, new_version) = merge_header(rep, old, masks, ctx)?;

        Ok(Comparator {
            h: new,
            reset_pricing_fields: Some(reset),
            new_version,
            ..Comparator::default()
        })
    }

    // Adjust item in accordance with parameters in the comparator.
    fn complete(
        c: &PlanComparator,
        mut new_item: PlanItemFullRep,
        item: Option<PlanItemFull>,
        masks: &Masks<PlanItemFull>,
    ) -> Result<Option<PlanItemFull>> {
        // If there is no old item, and the new is removed, we don't add it to the DB.
        if item.is_none() && new_item.is_removed.unwrap_or(false) {
            return Ok(None);
        }
        // If we are updating (and only then), then we reset fields if conditions are met.
        // (see the `merge_header` function above for details).
        new_item = match c.reset_pricing_fields {
            Some(true) => new_item.zero_fields(&masks.zero2_if_old_v_unit_changed),
            Some(false) => new_item
                .unset_fields(&masks.drop_if_old_v)
                .unset_fields(&masks.drop2_if_old_v_unit_same),
            None => new_item,
        };
        if !c.new_version {
            new_item = new_item.unset_fields(&masks.drop_if_old_v);
        }
        let mut new = match &item {
            Some(i) => new_item.into_item_merged(i.clone())?,
            None => new_item.into_item()?,
        };
        new.pricing_created_at = c.h.pricing_created_at;
        new.pricing_changed_at = c.h.pricing_changed_at;

        Ok(Some(new))
    }
}

impl Compare<ContractAmendmentItem, ContractAmendmentItemRep, ContractAmendmentRep>
    for ContractAmendment
{
    fn new_insert(mut self, ctx: &UpdateCtx) -> AmendmentComparator {
        self.pricing_created_at = ctx.timestamp;
        self.pricing_changed_at = ctx.timestamp;

        Comparator {
            h: self,
            reset_pricing_fields: None,
            new_version: true,
            ..Comparator::default()
        }
    }
    fn new_update(
        rep: ContractAmendmentRep,
        old: &ContractAmendment,
        masks: &Masks<ContractAmendment>,
        ctx: &UpdateCtx,
    ) -> Result<AmendmentComparator> {
        let (new, reset, new_version) = merge_header(rep, old, masks, ctx)?;

        Ok(Comparator {
            h: new,
            reset_pricing_fields: Some(reset),
            new_version,
            ..Comparator::default()
        })
    }

    // Adjust item in accordance with parameters in the comparator.
    fn complete(
        c: &AmendmentComparator,
        mut new_item: ContractAmendmentItemRep,
        item: Option<ContractAmendmentItem>,
        masks: &Masks<ContractAmendmentItem>,
    ) -> Result<Option<ContractAmendmentItem>> {
        // If there is no old item, and the new is removed, we don't add it to the DB.
        if item.is_none() && new_item.is_removed.unwrap_or(false) {
            return Ok(None);
        }
        // If we are updating (and only then), then we reset fields if conditions are met.
        // (see the `merge_header` function above for details).
        new_item = match c.reset_pricing_fields {
            Some(true) => new_item.zero_fields(&masks.zero2_if_old_v_unit_changed),
            Some(false) => new_item
                .unset_fields(&masks.drop_if_old_v)
                .unset_fields(&masks.drop2_if_old_v_unit_same),
            None => new_item,
        };
        if !c.new_version {
            new_item = new_item.unset_fields(&masks.drop_if_old_v);
        }
        // Convert to items.
        let mut new = match &item {
            Some(i) => new_item.into_item_merged(i.clone())?,
            None => new_item.into_item()?,
        };
        new.pricing_created_at = c.h.pricing_created_at;
        new.pricing_changed_at = c.h.pricing_changed_at;
        // We do not calculate deltas if values for price, quantity and vat_id
        // have not changed.
        if let Some(old) = item {
            if new.price == old.price
                && new.quantity == old.quantity
                && new.vat_id == old.vat_id
            {
                return Ok(Some(new));
            };
        }
        let new_item_currency_rate = new.currency_rate;
        Ok(Some(new.calculate_deltas(new_item_currency_rate)))
    }
}

/// Последнии заголовки для ППЗ/ДС которые есть в БД
async fn get_existing_headers<Db, IRep, IDb>(
    ids: &[i64],
    pool: &PgPool,
) -> Result<Vec<Db>>
where
    Db: UpdateHeader<IRep, IDb>,
    IRep: UpdateItemRep<IDb>,
    IDb: UpdateItem<Db, IRep>,
{
    let select = Select::full::<Db>()
        .in_any(Plan::id, ids)
        .eq(Plan::is_actual, true)
        .add_replace_order_desc(Plan::id)
        .add_replace_order_desc(Plan::changed_at)
        .distinct_on(&[Plan::id]);

    Db::select(&select, pool).await.map_err(Into::into)
}

/// Функция должна создать версии существующих заголовках если:
/// ЛИБО если pricing_started_at не соответствует, ЛИБО если если новый UUID
/// Функция должна вернуть мэппинг UUID заголовка к его текущий версии для
/// создания версий позиций.
async fn version_existing_headers<Rep, Db, IRep, IDb>(
    headers_to_update: &[Rep],
    existing_headers: &AHashMap<i64, &Db>,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<AHashMap<Uuid, i16>>
where
    Rep: UpdateHeaderRep<Db, IRep, IDb>,
    Db: UpdateHeader<IRep, IDb>,
    IRep: UpdateItemRep<IDb>,
    IDb: UpdateItem<Db, IRep>,
    <Db as DbVersioned>::Versioned: UpdateVersion,
{
    // Create a list of headers that need new versions.
    let existing_to_version = headers_to_update
        .iter()
        .filter_map(|upd| {
            let id = upd.maybe_id().expect("Exists (see above).");
            let uuid = upd.maybe_uuid().expect("Exists (see above).");
            let started_at = upd.maybe_pricing_started_at().unwrap_or_default();

            match existing_headers.get(&id) {
                // Новая версия ЛИБО если pricing_started_at не соответствует, ЛИБО если если новый UUID/
                Some(e)
                    if e.pricing_started_at() != started_at
                        || e.record_uuid() != uuid =>
                {
                    Some((*e).to_owned())
                }
                // Версии не создаются для новый записей
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    let header_version_map =
        Db::insert_version_vec_returning(&existing_to_version, tx)
            .await?
            .into_iter()
            .map(|x| {
                let version = x.pricing_version();
                (x.uuid(), version)
            })
            .collect::<AHashMap<_, _>>();
    Ok(header_version_map)
}

/// Цель функции достать все существующии позиции по ППЗ/ДС которые могут обновится,
/// И создать версию для них если такова создаётся для их заголовка.
/// Функция должна вернуть существующие позиции так как мы ими ещё воспользуемся.
///
/// NB: We need uuids of EXISTING headers to get EXISTING items by, which may be
/// different to uuids of the headers that come.
async fn get_and_version_existing_items<Db, IRep, IDb>(
    header_uuids: AHashSet<Uuid>,
    header_version_map: &AHashMap<Uuid, i16>,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<AHashMap<Uuid, IDb>>
where
    Db: UpdateHeader<IRep, IDb>,
    IRep: UpdateItemRep<IDb>,
    IDb: UpdateItem<Db, IRep>,
{
    // Here we need to retrieve all plans again and compare against the pricing
    // started at fields...
    // We select a full item and then integrate them.
    let item_select = Select::full::<IDb>();

    let item_select = if IDb::FIELDS.iter().any(|x| *x == PlanItemFull::plan_uuid) {
        item_select.in_any(PlanItemFull::plan_uuid, header_uuids)
    } else {
        item_select.in_any(ContractAmendmentItem::header_uuid, header_uuids)
    };
    let all_existing_items = IDb::select(&item_select, &mut *tx).await?;

    // Create item versions.
    let mut all_existing_item_map = AHashMap::new();
    let mut items_to_version = Vec::with_capacity(all_existing_items.len());

    for item in all_existing_items {
        if let Some(v) = header_version_map.get(&item.source_uuid()) {
            items_to_version.push(item.to_versioned(*v));
        };
        all_existing_item_map.insert(item.record_uuid(), item);
    }
    // By versioning the items we hereby conclude the versioning.
    <IDb as DbVersioned>::Versioned::insert_vec(&mut items_to_version, tx).await?;

    Ok(all_existing_item_map)
}

/// Цель функции проставить is_actual=false на каждом заголовки с порядковым номером
/// который обновляется, но не является новейшей версии по системе монолита
/// (те, пришёл заголовок с подобным id, но не uuid).
async fn deactivate_old_headers<T: ProcessUpsert + HasIsActual>(
    is_actual_by_id: Vec<i64>,
    is_actual_by_uuid: Vec<Uuid>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let plan_select = Select::with_fields([Plan::is_actual, Plan::id, Plan::uuid])
        .in_any(Plan::id, is_actual_by_id)
        .not_in_any(Plan::uuid, is_actual_by_uuid)
        .eq(Plan::is_actual, true);

    let is_actual_to_deactivate = T::select(&plan_select, recorder.tx())
        .await?
        .into_iter()
        .map(|mut x| {
            x.set_is_actual(false);
            x
        })
        .collect::<Vec<_>>();

    // Deactivate old headers.
    recorder
        .process_update(is_actual_to_deactivate, &[Plan::is_actual], messages)
        .await?;
    Ok(())
}

/// This structure exists for debugging of received information only.
pub(crate) struct ReceivedList {
    pub(crate) headers: Vec<(Uuid, i64)>,
    pub(crate) items: Vec<(Uuid, Uuid, i64)>,
}

async fn do_final_upserts<Rep, Db, IRep, IDb>(
    headers: AHashMap<Uuid, Comparator<IDb, IRep, Db, Rep>>,
    items: Vec<IDb>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<ReceivedList>
where
    Rep: UpdateHeaderRep<Db, IRep, IDb>,
    Db: UpdateHeader<IRep, IDb>,
    IRep: UpdateItemRep<IDb>,
    IDb: UpdateItem<Db, IRep>,
{
    // general update of headers.
    let headers = headers.into_iter().map(|(_, x)| x.h).collect::<Vec<_>>();

    let fields = Db::FIELDS;
    let headers = recorder.process_upsert(headers, fields, messages).await?;
    // Мы доверяем монолиту. Иначе надо заниматься опасными делами.
    // (т.е. замыкать всю таблицу, что подействует на другие процессы.)
    let fields = IDb::FIELDS;
    let items = recorder.process_upsert(items, fields, messages).await?;

    let headers = headers
        .into_iter()
        .map(|x| (x.uuid(), HasId::id(&x)))
        .collect::<Vec<_>>();
    let items = items
        .into_iter()
        .map(|x| (x.uuid(), x.source_uuid(), x.id()))
        .collect::<Vec<_>>();
    Ok(ReceivedList { headers, items })
}

/// Сама функция добавки/обновления ППЗ/ДС которые пришли с монолита в нашу БД.
/// Тут происходит строго этот процесс. Он весь висит в транзакции, чтобы не было
/// на половину выполненного процесса.
pub(super) async fn upsert<Rep, Db, IRep, IDb>(
    plans_to_update: Vec<Rep>,
    items_to_update: Vec<IRep>,
    retrospectives_to_update: Vec<PlanRetrospectiveRep>,
    spec_deps_to_upsert: Vec<DocumentApproverRep>,
    proc_ctx: &ProcessingCtx,
) -> Result<(ReceivedList, Messages)>
where
    Rep: UpdateHeaderRep<Db, IRep, IDb>,
    Db: UpdateHeader<IRep, IDb> + Compare<IDb, IRep, Rep>,
    IRep: UpdateItemRep<IDb>,
    IDb: UpdateItem<Db, IRep>,
    <Db as DbVersioned>::Versioned: UpdateVersion,
{
    let mut messages = Messages::default();
    let mut item_uuids = Vec::with_capacity(items_to_update.len());

    for x in items_to_update.iter() {
        let uuid = x.maybe_source_uuid().ok_or_else(|| {
            PError::Import("У позиции не указан UUIDа заголовка".to_string())
        })?;
        item_uuids.push(uuid);
    }

    // Build a list of plan ids and uuids.
    let mut header_ids = Vec::with_capacity(plans_to_update.len());
    for x in plans_to_update.iter() {
        let id = x.maybe_id().ok_or_else(|| {
            PError::Import("У позиции не указан порядковый номер".to_string())
        })?;
        header_ids.push(id);
    }
    // Here we get existing headers and merge them with what we receive.
    let existing_headers =
        get_existing_headers::<Db, IRep, IDb>(&header_ids, &proc_ctx.db_pool)
            .await?;

    let mut existing_h_by_id = AHashMap::new();
    let mut existing_h_by_uuid = AHashMap::new();
    let mut existing_h_uuids = AHashSet::new();
    for h in existing_headers.iter() {
        existing_h_by_id.insert(HasId::id(h), h);
        existing_h_by_uuid.insert(h.record_uuid(), h);
        existing_h_uuids.insert(h.record_uuid());
    }

    // TODO: Change the type to add the user ID at some point.
    let mut recorder = proc_ctx.create_external_context().begin().await?;

    let header_version_map = version_existing_headers(
        &plans_to_update,
        &existing_h_by_id,
        recorder.tx(),
    )
    .await?;

    let mut all_existing_item_map =
        get_and_version_existing_items::<Db, IRep, IDb>(
            existing_h_uuids,
            &header_version_map,
            recorder.tx(),
        )
        .await?;

    let ctx = recorder.ctx();

    // Now we must insert/update the records that have come. Everything can be done
    // in a single upsert operation, but we do have to make the decision between updating an
    // old record with new information, or simply inserting a new record. This is handled by
    // the following code.
    //
    // lastly we update records that did not come and switch is_actual to false.
    //
    // TODO: The final logic on which fields need to be cleared and updated will be
    // added in the final stage.
    let mut is_actual_by_id = Vec::with_capacity(plans_to_update.len());
    let mut is_actual_by_uuid = Vec::with_capacity(plans_to_update.len());
    let mut header_map = AHashMap::new();
    let masks = Masks::<Db>::new();
    for rep in plans_to_update {
        let uuid = rep.maybe_uuid().expect("Checked above");
        // Merging of Headers with DbHeaders occurs here.
        // This merge ensures that when variable fields come, we wipe nothing.
        let header = match existing_h_by_uuid.remove(&uuid) {
            Some(h) => Comparator::new_update(rep, h, &masks, &ctx)?,
            None => rep.into_item().map(|x| Comparator::new_insert(x, &ctx))?,
        };
        // We update is_actual in existing plans here. If there are existing
        // plans in our database with IDs that we are updating, we then
        // set is_actual to false in those plans.
        // Because we must track changed fields we cannot use a simple
        // `UPDATE plan SET is_actual=false WHERE id=ANY($)'
        if header.h.is_actual() {
            is_actual_by_id.push(HasId::id(&header.h));
            is_actual_by_uuid.push(header.h.record_uuid());
        }
        header_map.insert(header.h.record_uuid(), header);
    }

    // Here items that are already in the DB are merged for updating, while
    // incoming items not in the DB are prepared as new items.
    //
    // TODO: The final logic on which fields need to be cleared and updated will be
    // added in the final stage.
    let mut items = Vec::with_capacity(items_to_update.len());
    let masks = Masks::<IDb>::new();
    for item_rep in items_to_update {
        let source_uuid = item_rep.maybe_source_uuid().expect("See above.");

        let comp = header_map.get(&source_uuid).ok_or_else(|| {
            let uuid = item_rep.maybe_uuid().expect("See above.");
            PError::Import(format!("У позиции \"{uuid}\" неt заголовка"))
        })?;
        let old_item = all_existing_item_map
            .remove(&item_rep.maybe_uuid().expect("See above."));

        if let Some(item) = comp.complete(item_rep, old_item, &masks)? {
            items.push(item);
        }
    }

    deactivate_old_headers::<Db>(
        is_actual_by_id,
        is_actual_by_uuid,
        &mut messages,
        &mut recorder,
    )
    .await?;

    let mut retrospectives = retrospectives_to_update
        .into_iter()
        .map(|x| x.into_item().map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;

    let updated_retro_ids = PlanRetrospective::upsert_returning(
        &mut retrospectives,
        Some(&[PlanRetrospective::is_removed]),
        recorder.tx(),
    )
    .await?
    .into_iter()
    .map(|x| x.id)
    .collect::<Vec<_>>();

    let plan_id_filter: FilterTree =
        Filter::in_any(PlanRetrospective::plan_id, header_ids.iter()).into();
    let filter_out_updated =
        Filter::not_in_any(PlanRetrospective::id, updated_retro_ids).into();
    let final_filter = plan_id_filter.and(filter_out_updated);

    PlanRetrospective::delete_returning(&final_filter, recorder.tx()).await?;

    upsert_specialized_departments(
        spec_deps_to_upsert,
        &mut messages,
        &mut recorder,
    )
    .await?;

    let debug_list =
        do_final_upserts(header_map, items, &mut messages, &mut recorder).await?;
    recorder.commit().await?;
    Ok((debug_list, messages))
}

// Новая запись
fn prepare_new_doc_appr(x: &mut DocumentApprover, now: AsezTimestamp) {
    if x.division_id.is_some() {
        x.division_assigned_at = Some(now);
    }
    x.send_date_1 = Some(now);
    x.created_at = now;
}

async fn upsert_specialized_departments(
    spec_deps: Vec<DocumentApproverRep>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    if spec_deps.is_empty() {
        return Ok(());
    }

    let existing_spec_deps = DocumentApprover::select(
        &Select::with_fields([
            DocumentApprover::uuid,
            DocumentApprover::document_uuid,
            DocumentApprover::is_actual,
            DocumentApprover::is_removed,
            DocumentApprover::department_id,
            DocumentApprover::division_id,
            DocumentApprover::number,
        ])
        .in_any(
            DocumentApprover::document_uuid,
            spec_deps.iter().map(|x| x.document_uuid).collect::<AHashSet<_>>(),
        ),
        recorder.tx(),
    )
    .await?;

    let (incoming_uuids, incoming_doc_uuids): (AHashSet<_>, AHashSet<_>) =
        spec_deps
            .iter()
            .filter_map(|x| {
                x.uuid.and_then(|uuid| {
                    x.document_uuid.map(|document_uuid| (uuid, document_uuid))
                })
            })
            .unzip();
    // у назначений, относящихся к входящим ППЗ/ДС, но не попавших во входящие,
    // is_actual должно быть установлено в false
    let should_be_deactualized = move |doc_appr: &DocumentApprover| {
        !incoming_uuids.contains(&doc_appr.uuid)
            && incoming_doc_uuids.contains(&doc_appr.document_uuid)
    };

    let existing_uuids =
        existing_spec_deps.iter().map(|x| x.uuid).collect::<AHashSet<_>>();
    // Новые назначения ПД
    let is_new =
        move |doc_appr: &DocumentApprover| !existing_uuids.contains(&doc_appr.uuid);

    let incoming_items = spec_deps
        .into_iter()
        .map(DocumentApproverRep::into_item)
        .map_ok(|mut x| {
            if is_new(&x) {
                prepare_new_doc_appr(&mut x, recorder.timestamp())
            }
            x
        });
    let existing_items_to_deactualize = existing_spec_deps
        .into_iter()
        .filter(|x| should_be_deactualized(x))
        .map(|mut x| {
            x.is_actual = false;
            Ok(x)
        });

    let items = incoming_items
        .chain(existing_items_to_deactualize)
        .collect::<std::result::Result<_, _>>()?;

    recorder
        .process_upsert(
            items,
            &[DocumentApprover::is_actual, DocumentApprover::is_removed],
            messages,
        )
        .await?;

    Ok(())
}

/// When updating the same version of a plan, we ignore these fields if the "planning"
/// module sends them to us. If we are making a new version of a plan, we set these fields to zero.
///
/// This list should contain:
/// - all pricing_* and savings_* (except pricing_organization_unit_id, pricing_started_at, pricing_created_at, pricing_changed_at)
/// - commission_kind_id and commission_date
/// - is_check_documentation and check_documentation_date
/// - expert_conclusion_id
pub(super) const CLEAR_NEW_VERSION_FIELDS: &[&str] = &[
    ContractAmendmentItem::pricing_quantity,
    ContractAmendmentItem::pricing_unit_id,
    ContractAmendmentItem::pricing_price,
    ContractAmendmentItem::pricing_price_rub,
    ContractAmendmentItem::pricing_department_id,
    ContractAmendmentItem::pricing_method_id,
    ContractAmendmentItem::pricing_resume,
    ContractAmendmentItem::pricing_vat_id,
    ContractAmendmentItem::pricing_currency_id,
    ContractAmendmentItem::pricing_currency_rate,
    ContractAmendmentItem::pricing_currency_rate_date,
    ContractAmendmentItem::pricing_sum_excluded_vat,
    ContractAmendmentItem::pricing_sum_excluded_vat_rub,
    ContractAmendmentItem::pricing_sum_included_vat,
    ContractAmendmentItem::pricing_sum_included_vat_rub,
    ContractAmendmentItem::pricing_sum_vat,
    ContractAmendmentItem::pricing_sum_vat_rub,
    ContractAmendmentItem::pricing_transportation_vat_id,
    ContractAmendmentItem::pricing_transportation_price,
    ContractAmendmentItem::pricing_transportation_price_rub,
    ContractAmendmentItem::pricing_transportation_sum_vat,
    ContractAmendmentItem::pricing_transportation_sum_vat_rub,
    ContractAmendmentItem::pricing_transportation_sum_included_vat,
    ContractAmendmentItem::pricing_transportation_sum_included_vat_rub,
    ContractAmendmentItem::pricing_total_sum,
    ContractAmendmentItem::pricing_total_sum_rub,
    ContractAmendmentItem::pricing_delta_unit_id,
    ContractAmendmentItem::pricing_delta_quantity,
    ContractAmendmentItem::pricing_delta_currency_id,
    ContractAmendmentItem::pricing_delta_currency_rate,
    ContractAmendmentItem::pricing_delta_currency_rate_date,
    ContractAmendmentItem::pricing_delta_price,
    ContractAmendmentItem::pricing_delta_price_rub,
    ContractAmendmentItem::pricing_delta_sum_excluded_vat,
    ContractAmendmentItem::pricing_delta_sum_excluded_vat_rub,
    ContractAmendmentItem::pricing_delta_sum_vat,
    ContractAmendmentItem::pricing_delta_sum_vat_rub,
    ContractAmendmentItem::pricing_delta_sum_included_vat,
    ContractAmendmentItem::pricing_delta_sum_included_vat_rub,
    ContractAmendmentItem::pricing_delta_transportation_price,
    ContractAmendmentItem::pricing_delta_transportation_price_rub,
    ContractAmendmentItem::pricing_delta_transportation_sum_vat,
    ContractAmendmentItem::pricing_delta_transportation_sum_vat_rub,
    ContractAmendmentItem::pricing_delta_transportation_sum_included_vat,
    ContractAmendmentItem::pricing_delta_transportation_sum_included_vat_rub,
    ContractAmendmentItem::pricing_delta_total_sum,
    ContractAmendmentItem::pricing_delta_total_sum_rub,
    Plan::savings_accounting_id,
    Plan::savings_sum_excluded_vat,
    Plan::savings_sum_excluded_vat_rub,
    Plan::savings_sum_included_vat,
    Plan::savings_sum_included_vat_rub,
    Plan::commission_kind_id,
    Plan::commission_date,
    Plan::expert_conclusion_id,
];

pub(super) const EXTRA_IGNORE_FIELDS: &[&str] =
    &[ContractAmendmentItem::uuid_item_proposal, PlanItemFull::plan_id_lotting];

/// Список полей которые переписываются с монолита при создании новый версии.
/// в остальных случаях они игнорируются.
pub(super) const IGNORE_IF_SAME_VERSION: &[&str] = &[
    Plan::sum_vat,
    Plan::sum_vat_rub,
    Plan::sum_excluded_vat,
    Plan::sum_excluded_vat_rub,
    Plan::sum_included_vat,
    Plan::sum_included_vat_rub,
    ContractAmendment::delta_sum_vat,
    ContractAmendment::delta_sum_vat_rub,
    ContractAmendment::delta_sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat_rub,
    ContractAmendment::delta_sum_included_vat,
    ContractAmendment::delta_sum_included_vat_rub,
    PlanItemFull::price,
    PlanItemFull::quantity,
    Plan::vat_id,
];

/// Эти поля мы затираем если PricingUnit отличается при создании новый версии.
/// также мы их игнорируем когда новые версии не создаются
pub(super) const NEW_VERSION_AND_PRICING_UNIT: &[&str] = &[
    Plan::pricing_expert_id,
    Plan::is_check_documentation,
    Plan::check_documentation_date,
];
