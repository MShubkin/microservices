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
    Plan::sum_excluded_vat,
    Plan::sum_included_vat,
    Plan::sum_excluded_vat_rub,
    Plan::sum_included_vat_rub,
    Plan::customer_id,
    Plan::purchasing_type_id,
    Plan::purchasing_method_id,
    Plan::contract_subject,
    Plan::pricing_started_at,
    Plan::pricing_expert_id,
    Plan::pricing_sum_excluded_vat,
    Plan::pricing_sum_included_vat,
    Plan::pricing_sum_excluded_vat_rub,
    Plan::pricing_sum_included_vat_rub,
    Plan::pricing_resume,
    Plan::status_id,
    Plan::savings_sum_excluded_vat_rub,
    Plan::savings_sum_included_vat_rub,
    Plan::items_number,
    Plan::section_id,
    Plan::savings_accounting_id,
    Plan::commission_kind_id,
    Plan::changed_at,
    Plan::changed_by,
];

const CONTRACT_AMENDMENT_FIELDS: &[&str] = &[
    ContractAmendment::id,
    ContractAmendment::uuid,
    ContractAmendment::sum_excluded_vat,
    ContractAmendment::sum_included_vat,
    ContractAmendment::sum_excluded_vat_rub,
    ContractAmendment::sum_included_vat_rub,
    ContractAmendment::previous_sum_excluded_vat,
    ContractAmendment::previous_sum_excluded_vat_rub,
    ContractAmendment::previous_sum_included_vat,
    ContractAmendment::previous_sum_included_vat_rub,
    ContractAmendment::customer_id,
    ContractAmendment::purchasing_type_id,
    ContractAmendment::purchasing_method_id,
    ContractAmendment::contract_subject,
    ContractAmendment::pricing_started_at,
    ContractAmendment::pricing_expert_id,
    ContractAmendment::pricing_sum_excluded_vat,
    ContractAmendment::pricing_sum_included_vat,
    ContractAmendment::pricing_sum_excluded_vat_rub,
    ContractAmendment::pricing_sum_included_vat_rub,
    ContractAmendment::pricing_resume,
    ContractAmendment::status_id,
    ContractAmendment::savings_sum_excluded_vat,
    ContractAmendment::savings_sum_excluded_vat_rub,
    ContractAmendment::savings_sum_included_vat,
    ContractAmendment::savings_sum_included_vat_rub,
    ContractAmendment::items_number,
    ContractAmendment::section_id,
    ContractAmendment::savings_accounting_id,
    ContractAmendment::commission_kind_id,
    ContractAmendment::changed_at,
    ContractAmendment::changed_by,
];

pub(crate) async fn pricing_report_savings_data(
    request: PricingReportRequest,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PricingReportResData, ()>> {
    let db_pool = db_pool.as_ref();

    let mut base_filters: FilterTree = Default::default();
    base_filters.push_filter(Filter::eq(Plan::pricing_organization_unit_id, 1));

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

    let document_uuids = StatusHistory::select(&select_status_history, db_pool)
        .await?
        .into_iter()
        .map(|item| item.object_uuid)
        .collect::<HashSet<uuid::Uuid>>();
    let document_uuid_values: Vec<Value> =
        document_uuids.into_iter().map(Value::Uuid).collect();

    // Select finished pricings
    let mut select_plans =
        Select::with_fields(PLAN_FIELDS).set_filter_tree(base_filters.clone());
    select_plans
        .filter_list
        .push_filter(Filter::in_any(Plan::uuid, document_uuid_values.clone()));

    let plans = PlanRep::select(&select_plans, db_pool).await?;

    let mut select_contract_amendments =
        Select::with_fields(CONTRACT_AMENDMENT_FIELDS)
            .set_filter_tree(base_filters);
    select_contract_amendments
        .filter_list
        .push_filter(Filter::in_any(Plan::uuid, document_uuid_values));

    let contract_amendments =
        ContractAmendmentRep::select(&select_contract_amendments, db_pool).await?;

    // Select finished pricings from versions
    let plans_versions = PlanVersionRep::select(&select_plans, db_pool).await?;

    let contract_amendments_versions =
        ContractAmendmentVersionRep::select(&select_contract_amendments, db_pool)
            .await?;

    let data = PricingReportResData {
        plans,
        plans_versions,
        contract_amendments,
        contract_amendments_versions,
    };

    Ok(ApiResponse::default().with_data(data))
}
