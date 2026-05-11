use std::sync::Arc;

use crate::common::Result;
use asez2_shared_db::{
    db_item::{
        selection::filters::{Filter, FilterTree},
        Select,
    },
    DbAdaptor,
};
use shared_essential::domain::*;
use shared_essential::presentation::dto::response_request::Messages;
use shared_essential::presentation::dto::{
    processing::price_analysis::{PricingReportRequest, PricingReportResData},
    response_request::ApiResponse,
};
use sqlx::PgPool;

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
    Plan::savings_sum_excluded_vat,
    Plan::savings_sum_excluded_vat_rub,
    Plan::savings_sum_included_vat,
    Plan::savings_sum_included_vat_rub,
    Plan::currency_id,
    Plan::items_number,
    Plan::section_id,
    Plan::supplier_id,
    Plan::single_supplier_reason_id,
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
    ContractAmendment::currency_id,
    ContractAmendment::items_number,
    ContractAmendment::section_id,
    ContractAmendment::supplier_id,
    ContractAmendment::single_supplier_reason_id,
    ContractAmendment::savings_accounting_id,
    ContractAmendment::commission_kind_id,
    ContractAmendment::changed_at,
    ContractAmendment::changed_by,
];

pub(crate) async fn pricing_report_commission_data(
    _request: PricingReportRequest,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PricingReportResData, ()>> {
    let db_pool = db_pool.as_ref();

    let mut base_filters: FilterTree = Default::default();
    base_filters.push_filter(Filter::in_any(
        Plan::pricing_organization_unit_id,
        vec![PricingUnitId::D646, PricingUnitId::D645],
    ));
    base_filters.push_filter(Filter::in_any(Plan::status_id, vec![251, 252, 253]));

    // Select pricings
    let select_plans =
        Select::with_fields(PLAN_FIELDS).set_filter_tree(base_filters.clone());

    let plans = PlanRep::select(&select_plans, db_pool).await?;

    let select_contract_amendments = Select::with_fields(CONTRACT_AMENDMENT_FIELDS)
        .set_filter_tree(base_filters);

    let contract_amendments =
        ContractAmendmentRep::select(&select_contract_amendments, db_pool).await?;

    // Select pricings from versions
    let plans_versions = PlanVersionRep::select(&select_plans, db_pool).await?;

    let contract_amendments_versions =
        ContractAmendmentVersionRep::select(&select_contract_amendments, db_pool)
            .await?;

    Ok((
        PricingReportResData {
            plans,
            plans_versions,
            contract_amendments,
            contract_amendments_versions,
        },
        Messages::default(),
    )
        .into())
}
