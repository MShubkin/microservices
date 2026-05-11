//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
use std::sync::Arc;

use crate::common::Result;

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::{
    from_item_with_fields, AdaptorableIter, FieldTolerance, Filter, Select,
};
use asez2_shared_db::DbItem;
use shared_essential::{
    domain::{
        Attachment, GetPlanData, GetPlanDataSelector, Plan, PlanItemFull,
        PlanVersion,
    },
    presentation::dto::{processing::*, response_request::*},
};

use ahash::AHashSet;
use sqlx::PgPool;

const GET_COMPLETE_PLANS: &str = "/v1/get_plan";

/// Функция по передаче полных планов с документами.
/// НБ: Пока что тут не нужны, секции (так как везде None).
#[tracing::instrument(skip_all)]
pub(crate) async fn get_complete_plans(
    req: CompletePlansRequest,
    db_pool: Arc<PgPool>,
) -> Result<GetPlanResponse> {
    let db_conn = db_pool.as_ref();

    tracing::info!(
        kind = "get",
        "Processing: Got request to send to plans on ({get}): {req:?}\n",
        req = req,
        get = GET_COMPLETE_PLANS
    );

    let fields = req.select.field_list.clone();
    let item_fields =
        req.item_fields.iter().map(|x| x as &str).collect::<AHashSet<&str>>();
    let item_fields = PlanItemFull::FIELDS
        .iter()
        .filter(|x| item_fields.contains(*x))
        .copied()
        .collect::<Vec<&str>>();

    // Тут фильтрация планов по ID, так что могут быть несколько версий по uuid.
    // надо брать по признаку is_actual==true.
    let actual_filter = Filter::eq(Plan::is_actual, true).into();

    let mut select = Select::with_fields(["id"]);
    select.filter_list = req.select.filter_list.and(actual_filter);
    select.field_list = fields;

    let plans = build_complete_plans(&select, db_conn).await?;
    tracing::debug!(kind = "get", "{:?}", plans);

    // Если  мы не брали currency_id, они будут везде одинаковые, так что дальнейшая проверка бессмысленная.
    let messages = select
        .field_list
        .iter()
        .any(|x| x == "currency_id")
        .then(|| check_currency(&plans))
        .unwrap_or_default();
    let data = convert_plans(plans, &select, &item_fields)?;
    tracing::debug!(kind = "get", "{:?}", data);

    Ok((data, messages).into())
}

/// This is essentially a copy of `GetCompletePlans::execute_inner`. It gets
/// complete plans and then organises them into a response.
async fn build_complete_plans(
    select: &Select,
    pool: &PgPool,
) -> Result<Vec<GetPlanData>> {
    let mut plan_select = select.clone();
    // We must use all fields for joined selects.
    plan_select.field_list =
        Plan::FIELDS.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    GetPlanDataSelector::new(plan_select)
        .set_items(
            PlanItemFull::join_default()
                .selecting(
                    Select::default()
                        .eq(PlanItemFull::is_removed, false)
                        .add_replace_order_asc(PlanItemFull::uuid),
                )
                .distinct_aggr(true),
        )
        .set_attachments(
            Attachment::join_default()
                .selecting(
                    Select::default()
                        .add_replace_order_asc(Attachment::category_id)
                        .add_replace_order_asc(Attachment::number)
                        .add_replace_order_asc(Attachment::uuid),
                )
                .distinct_aggr(true),
        )
        .set_versions(PlanVersion::join_default().distinct_aggr(true))
        .get(pool)
        .await
        .map_err(Into::into)
}

/// Converts to return DTO structures. The select is needed to determine which
/// fields are serialized.
fn convert_plans(
    plans: Vec<GetPlanData>,
    select: &Select,
    item_fields: &[&str],
) -> Result<Vec<GetPlanDataRep>> {
    let plan_set = Plan::FIELDS
        .iter()
        .chain(Plan::TOLERATED.iter().map(|(fe, _be)| fe))
        .copied()
        .collect::<AHashSet<&str>>();
    let fields = select.fields();
    let plan_fields =
        fields.iter().filter_map(|x| plan_set.get(x)).collect::<Vec<_>>();

    // TODO Extra check for replacement with tolerated fields when ready.

    let from_item = from_item_with_fields(item_fields);
    let from_plan = from_item_with_fields(plan_fields);
    plans
        .into_iter()
        .map(|x| {
            let mut items = x.items;
            items.dedup();

            let items = items.into_iter().map(&from_item).collect();

            let mut attachments = x.attachments;
            attachments.dedup();

            let attachments = attachments.into_iter().adaptors().collect();

            let mut versions = x.versions;
            versions.dedup();

            let mut versions = versions
                .into_iter()
                .map(|x| VersionInfo {
                    pricing_version: Some(x.pricing_version),
                    is_active: false,
                    pricing_expert_id: x.pricing_expert_id,
                    expert_conclusion_id: x.expert_conclusion_id,
                    pricing_created_at: Some(x.pricing_created_at),
                    sum_excluded_vat: x.sum_excluded_vat,
                    sum_included_vat: x.sum_included_vat,
                    sum_excluded_vat_rub: x.sum_excluded_vat_rub,
                    sum_included_vat_rub: x.sum_included_vat_rub,
                })
                .collect::<Vec<_>>();
            versions.push(VersionInfo {
                pricing_version: None,
                is_active: true,
                pricing_expert_id: x.plan.pricing_expert_id,
                expert_conclusion_id: x.plan.expert_conclusion_id,
                pricing_created_at: None,
                sum_excluded_vat: x.plan.sum_excluded_vat,
                sum_included_vat: x.plan.sum_included_vat,
                sum_excluded_vat_rub: x.plan.sum_excluded_vat_rub,
                sum_included_vat_rub: x.plan.sum_included_vat_rub,
            });

            let plan = from_plan(x.plan);

            Ok(GetPlanDataRep {
                plan,
                items,
                attachments,
                versions,
            })
        })
        .collect::<Result<Vec<_>>>()
}

/// Проверка мултивалютности.
/// Если валюты отличаются от заголовка, выдаются предупреждения.
fn check_currency(plans: &[GetPlanData]) -> Messages {
    plans.iter().filter_map(|x| {
        let header_currency = x.plan.currency_id;
        let (differences, numbers): (Vec<_>, Vec<_>) = x
            .items
            .iter()
            .fold((Vec::new(), Vec::new()), |(mut cs, mut nbs), item| {
            if item.currency_id != header_currency {
                cs.push(item.currency_id.to_string());
                nbs.push(item.number.to_string());
            }
            (cs, nbs)
        });

        if !differences.is_empty() {
            let msg = format!(
                "ППЗ ({head_id}): Валюты ({currs}) в позициях ({numbers}) отличаются от валюты заголовка ({cur_head})",
                head_id = x.plan.id,
                currs = differences.join(", "),
                numbers = numbers.join(", "),
                cur_head = header_currency,
            );
            Some(Message::warn(msg).with_param_item(&x.plan))
        } else {
            None
        }
    })
    .collect::<Vec<_>>()
    .into()
}
