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
        Attachment, ContractAmendment, ContractAmendmentItem,
        ContractAmendmentVersion, GetContractAmendmentData,
        GetContractAmendmentDataSelector,
    },
    presentation::dto::{processing::*, response_request::*},
};

use ahash::AHashSet;
use sqlx::PgPool;

const GET_COMPLETE_CONTRACT_AMENDMENTS: &str = "/v1/get_contract_amendment";

/// This is the actual function.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_complete_contract_amendments(
    req: CompletePlansRequest,
    db_pool: Arc<PgPool>,
) -> Result<GetContractAmendmentResponse> {
    let db_conn = db_pool.as_ref();

    tracing::info!(
        kind = "get",
        "Processing: Got request to send to plans on ({get}): {req:?}\n",
        req = req,
        get = GET_COMPLETE_CONTRACT_AMENDMENTS
    );

    let fields = req.select.field_list.clone();
    let item_fields =
        req.item_fields.iter().map(|x| x as &str).collect::<AHashSet<&str>>();
    let item_fields = ContractAmendmentItem::FIELDS
        .iter()
        .filter(|x| item_fields.contains(*x))
        .copied()
        .collect::<Vec<&str>>();

    // Тут фильтрация планов по ID, так что могут быть несколько версий по uuid.
    // надо брать по признаку is_actual==true.
    let actual_filter = Filter::eq(ContractAmendment::is_actual, true).into();

    let mut select = Select::with_fields(["id"]);

    select.filter_list = req.select.filter_list.and(actual_filter);
    select.field_list = fields;

    let plans = build_complete_contract_amendments(&select, db_conn).await?;
    tracing::debug!(kind = "get", "{:?}", plans);

    // Если  мы не брали currency_id, они будут везде одинаковые, так что дальнейшая проверка бессмысленная.
    let messages = select
        .field_list
        .iter()
        .any(|x| x == "currency_id")
        .then(|| check_currency(&plans))
        .unwrap_or_default();
    let data = convert_contract_amendments(plans, &select, &item_fields);
    tracing::debug!(kind = "get", "{:?}", data);

    Ok((data, messages).into())
}

/// This is essentially a copy of `GetCompleteContractAmendments::execute_inner`. It gets
/// complete plans and then organises them into a response.
async fn build_complete_contract_amendments(
    select: &Select,
    pool: &PgPool,
) -> Result<Vec<GetContractAmendmentData>> {
    let mut plan_select = select.clone();
    // We must use all fields for joined selects.
    plan_select.field_list = ContractAmendment::FIELDS
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    GetContractAmendmentDataSelector::new(plan_select)
        .set_items(
            ContractAmendmentItem::join_default()
                .selecting(
                    Select::default()
                        .eq(ContractAmendmentItem::is_removed, false)
                        .add_replace_order_asc(ContractAmendmentItem::uuid),
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
        .set_versions(ContractAmendmentVersion::join_default().distinct_aggr(true))
        .get(pool)
        .await
        .map_err(Into::into)
}

/// Converts to return DTO structures. The select is needed to determine which
/// fields are serialized.
fn convert_contract_amendments(
    plans: Vec<GetContractAmendmentData>,
    select: &Select,
    item_fields: &[&str],
) -> Vec<GetContractAmendmentDataRep> {
    let plan_set = ContractAmendment::FIELDS
        .iter()
        .chain(ContractAmendment::TOLERATED.iter().map(|(fe, _be)| fe))
        .copied()
        .collect::<AHashSet<&str>>();
    let fields = select.fields();
    let plan_fields =
        fields.iter().filter_map(|x| plan_set.get(x)).collect::<Vec<_>>();

    let from_item = from_item_with_fields(item_fields);
    let from_plan = from_item_with_fields(plan_fields);
    // TODO Extra check for replacement with tolerated fields when ready.
    plans
        .into_iter()
        .map(|x| {
            let items = x.items.into_iter().map(&from_item).collect();

            let attachments = x.attachments.into_iter().adaptors().collect();

            let mut versions = x
                .versions
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

            GetContractAmendmentDataRep {
                plan,
                items,
                attachments,
                versions,
            }
        })
        .collect()
}

/// Проверка мултивалютности.
/// Если валюты отличаются от заголовка, выдаются предупреждения.
fn check_currency(plans: &[GetContractAmendmentData]) -> Messages {
    plans.iter().filter_map(|x| {
        let header_currency = x.plan.currency_id;
        let (mut differences, mut ids): (Vec<_>, Vec<_>) = x
            .items
            .iter()
            .fold((Vec::new(), Vec::new()), |(mut cs, mut is), item| {
            if item.currency_id != header_currency {
                cs.push(item.currency_id.to_string());
                is.push(item.id.to_string());
            }
            (cs, is)
        });

        if !differences.is_empty() {
            let msg = format!(
                "ППЗ ({head_id}): Валюты ({currs}) в позициях ({ids}) отличаются от валюты заголовка ({cur_head})",
                head_id = x.plan.id,
                currs = differences.join(", "),
                ids = ids.join(", "),
                cur_head = header_currency,
            );
            differences.append(&mut ids);
            Some(Message::warn(msg).with_param_item(&x.plan))
        } else {
            None
        }
    })
    .collect::<Vec<_>>()
    .into()
}
