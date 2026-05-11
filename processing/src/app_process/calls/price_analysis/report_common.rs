use crate::common::Result;
use asez2_shared_db::{
    db_item::{
        selection::filters::{Filter, FilterTree},
        Select,
    },
    DbAdaptor, DbItem, Value,
};
use shared_essential::domain::*;

use shared_essential::presentation::dto::{
    processing::price_analysis::{PricingReportRequest, PricingReportResData},
    response_request::ApiResponse,
};
use sqlx::PgPool;
use std::{collections::HashSet, sync::Arc};

const PLAN_FIELDS: &[&str] = &[
    Plan::id,
    Plan::uuid,
    Plan::customer_id,
    Plan::purchasing_method_id,
    Plan::section_id,
    Plan::supplier_id,
    Plan::contract_subject,
    Plan::status_id,
    Plan::items_number,
    Plan::commission_kind_id,
    Plan::sum_excluded_vat_rub,
    Plan::sum_included_vat_rub,
    Plan::pricing_started_at,
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::pricing_sum_excluded_vat_rub,
    Plan::pricing_sum_included_vat_rub,
    Plan::savings_sum_excluded_vat_rub,
    Plan::savings_sum_included_vat_rub,
    Plan::delivery_start_date,
    Plan::delivery_end_date,
    Plan::changed_at,
    Plan::changed_by,
];

const CONTRACT_AMENDMENT_FIELDS: &[&str] = &[
    ContractAmendment::id,
    ContractAmendment::uuid,
    ContractAmendment::customer_id,
    ContractAmendment::purchasing_method_id,
    ContractAmendment::section_id,
    ContractAmendment::supplier_id,
    ContractAmendment::contract_subject,
    ContractAmendment::status_id,
    ContractAmendment::items_number,
    ContractAmendment::commission_kind_id,
    ContractAmendment::sum_excluded_vat_rub,
    ContractAmendment::sum_included_vat_rub,
    ContractAmendment::pricing_started_at,
    ContractAmendment::pricing_expert_id,
    ContractAmendment::pricing_resume,
    ContractAmendment::pricing_sum_excluded_vat,
    ContractAmendment::pricing_sum_excluded_vat_rub,
    ContractAmendment::pricing_sum_included_vat,
    ContractAmendment::pricing_sum_included_vat_rub,
    ContractAmendment::pricing_delta_sum_excluded_vat,
    ContractAmendment::pricing_delta_sum_excluded_vat_rub,
    ContractAmendment::pricing_delta_sum_included_vat,
    ContractAmendment::pricing_delta_sum_included_vat_rub,
    ContractAmendment::savings_sum_excluded_vat,
    ContractAmendment::savings_sum_excluded_vat_rub,
    ContractAmendment::savings_sum_included_vat,
    ContractAmendment::savings_sum_included_vat_rub,
    ContractAmendment::delta_sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat_rub,
    ContractAmendment::delta_sum_included_vat,
    ContractAmendment::delta_sum_included_vat_rub,
    ContractAmendment::previous_sum_excluded_vat,
    ContractAmendment::previous_sum_excluded_vat_rub,
    ContractAmendment::previous_sum_included_vat,
    ContractAmendment::previous_sum_included_vat_rub,
    ContractAmendment::start_date,
    ContractAmendment::end_date,
    ContractAmendment::changed_at,
    ContractAmendment::changed_by,
];

pub(crate) async fn pricing_report_common_data(
    request: PricingReportRequest,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PricingReportResData, ()>> {
    let mut base_filters: FilterTree = Default::default();
    base_filters.push_filter(Filter::eq(Plan::pricing_organization_unit_id, 1));

    // Select current pricings in work
    let mut select_plans_current =
        Select::with_fields(PLAN_FIELDS).set_filter_tree(base_filters.clone());
    select_plans_current
        .filter_list
        .push_filter(Filter::eq("status_id", 222));

    let mut plans_current =
        PlanRep::select(&select_plans_current, db_pool.as_ref()).await?;

    let mut select_contract_amendments_current =
        Select::with_fields(CONTRACT_AMENDMENT_FIELDS)
            .set_filter_tree(base_filters.clone());
    select_contract_amendments_current
        .filter_list
        .push_filter(Filter::eq("status_id", 222));

    let mut contract_amendments_current = ContractAmendmentRep::select(
        &select_contract_amendments_current,
        db_pool.as_ref(),
    )
    .await?;

    // Select uuids of finished pricings
    let mut select_status_history = Select::with_fields([
        "uuid",
        "object_uuid",
        "status_id",
        "comment",
        "created_at",
        "created_by",
    ]);
    select_status_history
        .filter_list
        .push_filter(Filter::eq(Plan::status_id, 225));

    select_status_history.filter_list.push_filter(Filter::between(
        Plan::created_at,
        request.start_date,
        request.end_date,
    ));

    let document_uuids =
        StatusHistory::select(&select_status_history, db_pool.as_ref())
            .await?
            .into_iter()
            .map(|item| item.object_uuid)
            .collect::<HashSet<uuid::Uuid>>();
    println!("{:?}", document_uuids);
    let document_uuid_values: Vec<Value> =
        document_uuids.into_iter().map(Value::Uuid).collect();

    // Select finished pricings
    let mut select_plans =
        Select::with_fields(PLAN_FIELDS).set_filter_tree(base_filters.clone());
    select_plans
        .filter_list
        .push_filter(Filter::in_any(Plan::uuid, document_uuid_values.clone()));

    let mut plans: Vec<_> =
        PlanRep::select(&select_plans, db_pool.as_ref()).await?;

    let mut select_contract_amendments =
        Select::with_fields(CONTRACT_AMENDMENT_FIELDS)
            .set_filter_tree(base_filters);
    select_contract_amendments
        .filter_list
        .push_filter(Filter::in_any(Plan::uuid, document_uuid_values));

    let mut contract_amendments: Vec<_> =
        ContractAmendmentRep::select(&select_contract_amendments, db_pool.as_ref())
            .await?;

    // Select finished pricings from versions
    let plans_versions =
        PlanVersionRep::select(&select_plans, db_pool.as_ref()).await?;
    plans.append(&mut plans_current);

    let contract_amendments_versions = ContractAmendmentVersionRep::select(
        &select_contract_amendments,
        db_pool.as_ref(),
    )
    .await?;
    contract_amendments.append(&mut contract_amendments_current);

    let data = PricingReportResData {
        plans,
        plans_versions,
        contract_amendments,
        contract_amendments_versions,
    };

    Ok(ApiResponse::default().with_data(data))
}
