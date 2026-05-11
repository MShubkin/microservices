//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
use std::sync::Arc;

use crate::common::Result;

use asez2_shared_db::db_item::{from_item_with_fields, Select};
use asez2_shared_db::db_item::{joined::JoinTo, AdaptorableIter};
use asez2_shared_db::DbAdaptor;
use shared_essential::{
    domain::{
        Attachment, ContractAmendment, ContractAmendmentItemVersion,
        ContractAmendmentVersion, ContractAmendmentVersionRep,
        GetContractAmendmentData, GetContractAmendmentDataSelector,
        GetContractAmendmentVersionData, GetContractAmendmentVersionDataSelector,
    },
    presentation::dto::{processing::*, response_request::*},
};

use sqlx::PgPool;

const GET_COMPLETE_PLAN_VERSION: &str = "/v1/get_contract_amendment_version";

const PLAN_ITEM_VERSION_FIELDS: &[&str] = &[
    ContractAmendmentItemVersion::description_internal,
    ContractAmendmentItemVersion::number,
    "uuid",
    "price",
    "quantity",
    "currency_id",
    "unit_id",
    "vat_id",
    "sum_vat",
    "sum_excluded_vat",
    "sum_included_vat",
    "delta_sum_vat",
    "delta_sum_excluded_vat",
    "delta_sum_included_vat",
    "pricing_unit_id",
    "pricing_quantity",
    "pricing_price",
    "pricing_price_rub",
    "pricing_vat_id",
    "pricing_currency_id",
    "pricing_currency_rate",
    "pricing_currency_rate_date",
    "pricing_sum_excluded_vat",
    "pricing_sum_excluded_vat_rub",
    "pricing_sum_included_vat",
    "pricing_sum_included_vat_rub",
    "pricing_sum_vat",
    "pricing_sum_vat_rub",
    "pricing_transportation_price",
    "pricing_transportation_price_rub",
    "pricing_transportation_vat_id",
    "pricing_transportation_sum_vat",
    "pricing_transportation_sum_vat_rub",
    "pricing_transportation_sum_included_vat",
    "pricing_transportation_sum_included_vat_rub",
    "pricing_total_sum",
    "pricing_total_sum_rub",
];

#[tracing::instrument(skip_all)]
pub(crate) async fn get_contract_amendment_version(
    req: PlanVersionRequest,
    db_pool: Arc<PgPool>,
) -> Result<GetContractAmendmentVersionResponse> {
    let db_conn = db_pool.as_ref();
    tracing::info!(
        kind = "get",
        "Processing: Got request to send to plans on ({get}): {req:?}\n",
        req = req,
        get = GET_COMPLETE_PLAN_VERSION
    );

    let plans = build_complete_plans(req.plan_id, req.version, db_conn).await?;
    tracing::debug!(kind = "get", "{:?}", plans);

    let og_plans = GetContractAmendmentDataSelector::new(
        Select::default().eq(ContractAmendment::id, req.plan_id),
    )
    .set_versions(ContractAmendmentVersion::join_default().distinct_aggr(true))
    .get(db_conn)
    .await?;

    let data = convert_plans(plans, og_plans);
    tracing::debug!(kind = "get", "{:?}", data);

    // Ok((data, messages).into())
    Ok((data, Messages::default()).into())
}

/// This is essentially a copy of `GetCompleteContractAmendments::execute_inner`. It gets
/// complete plans and then organises them into a response.
async fn build_complete_plans(
    plan_id: i64,
    version: i16,
    pool: &PgPool,
) -> Result<Vec<GetContractAmendmentVersionData>> {
    // We must use all fields for joined selects.
    let plan_select: Select = Select::full::<ContractAmendmentVersion>()
        .eq(ContractAmendmentVersion::id, plan_id)
        .eq(ContractAmendmentVersion::pricing_version, version)
        .take_first();

    GetContractAmendmentVersionDataSelector::new(plan_select)
        .set_items(
            ContractAmendmentItemVersion::join_default()
                .selecting(
                    Select::default()
                        .eq(ContractAmendmentItemVersion::is_removed, false)
                        .eq(ContractAmendmentItemVersion::pricing_version, version)
                        .add_replace_order_asc(ContractAmendmentItemVersion::uuid),
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
        .get(pool)
        .await
        .map_err(Into::into)
}

/// Converts to return DTO structures. The select is needed to determine which
/// fields are serialized.
fn convert_plans(
    plans: Vec<GetContractAmendmentVersionData>,
    og_plans: Vec<GetContractAmendmentData>,
) -> Vec<GetContractAmendmentVersionDataRep> {
    let from_item = from_item_with_fields(PLAN_ITEM_VERSION_FIELDS);
    plans
        .into_iter()
        .map(|x| {
            let mut items = x.items;
            items.dedup();

            let items = items.into_iter().map(&from_item).collect();

            let mut attachments = x.attachments;
            attachments.dedup();

            let attachments = attachments.into_iter().adaptors().collect();

            let plan = ContractAmendmentVersionRep::from_item::<&str>(x.plan, None);

            let og_plan = og_plans.first().unwrap();
            let mut versions = og_plan.versions.clone();
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
                pricing_expert_id: og_plan.plan.pricing_expert_id,
                expert_conclusion_id: og_plan.plan.expert_conclusion_id,
                pricing_created_at: None,
                sum_excluded_vat: og_plan.plan.sum_excluded_vat,
                sum_included_vat: og_plan.plan.sum_included_vat,
                sum_excluded_vat_rub: og_plan.plan.sum_excluded_vat_rub,
                sum_included_vat_rub: og_plan.plan.sum_included_vat_rub,
            });

            GetContractAmendmentVersionDataRep {
                plan,
                items,
                attachments,
                versions,
            }
        })
        .collect()
}
