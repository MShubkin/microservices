use asez2_shared_db::{db_item::Select, DbAdaptor, DbItem};
use shared_essential::domain::*;
use shared_essential::presentation::dto::response_request::Messages;
use shared_essential::presentation::dto::{
    processing::price_analysis::{PricingReportReq, PricingReportResData},
    response_request::ApiResponse,
};

use crate::common::{RabbitNest, Result};

const PLAN_VERSION_FIELDS: &[&str] = &[
    PlanVersion::id,
    PlanVersion::uuid,
    PlanVersion::customer_id,
    PlanVersion::purchasing_method_id,
    PlanVersion::contract_subject,
    PlanVersion::sum_excluded_vat,
    PlanVersion::sum_included_vat,
    PlanVersion::status_id,
    PlanVersion::items_number,
    PlanVersion::changed_at,
    PlanVersion::changed_by,
    PlanVersion::pricing_expert_id,
    PlanVersion::pricing_resume,
    PlanVersion::pricing_sum_excluded_vat,
    PlanVersion::pricing_sum_included_vat,
];

pub(crate) async fn pricing_report_data(
    req: PricingReportReq,
    nest: &RabbitNest,
) -> Result<ApiResponse<PricingReportResData, ()>> {
    let PricingReportReq { select, .. } = req;
    let select = Select::with_fields(PLAN_VERSION_FIELDS)
        .set_filter_tree(select.filter_list);

    let db_pool = &*nest.db_pool;
    let versions = PlanVersion::select(&select, db_pool).await?;

    let res = versions
        .into_iter()
        .map(|x| PlanVersionRep::from_item(x, Some(PLAN_VERSION_FIELDS)))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok((PricingReportResData(res), Messages::default()).into())
}
