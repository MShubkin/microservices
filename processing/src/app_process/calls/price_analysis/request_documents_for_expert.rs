use std::sync::Arc;

use shared_essential::presentation::dto::general::ObjectIdentifier;

use asez2_shared_db::db_item::Select;
use shared_essential::{
    domain::{
        legacy::plans::PlanStatus, Plan, PlanOrAmendment, PlanOrAmendmentRep,
    },
    presentation::dto::{
        processing::price_analysis::{
            PreRequestDocumentsForExpertReq,
            PreRequestDocumentsForExpertResponseData,
        },
        response_request::ApiResponse,
    },
};
use sqlx::PgPool;

use crate::common::Result;

const PRE_REQUEST_DOCUMENTS_FOR_EXPERT_TAG: &str =
    "/pricing/v1/pre_request/request_documents_for_expert/";
// const REQUEST_DOCUMENTATION_TAG: &str = "/pricing/v1/action/request_documents_for_expert/";

const PRE_REQUEST_DOCUMENTS_FOR_EXPERT_FIELDS: &[&str] = &[
    Plan::uuid,
    "plan_id",                // DTO field duplicate
    "contract_subject_short", // DTO field duplicate
    Plan::commission_kind_id,
    Plan::commission_date,
    Plan::customer_id,
    Plan::sum_excluded_vat_rub,
];

pub(crate) async fn pa_pre_request_documents_for_expert(
    req: PreRequestDocumentsForExpertReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreRequestDocumentsForExpertResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = PRE_REQUEST_DOCUMENTS_FOR_EXPERT_TAG,
        req = req,
    );

    let plans = fetch_plans(&req.item_list, &db_pool).await?;

    let data = plans
        .into_iter()
        .map(PlanOrAmendmentRep::from_item_with_fields(
            PRE_REQUEST_DOCUMENTS_FOR_EXPERT_FIELDS,
        ))
        .collect();

    Ok(ApiResponse::default().with_data(data))
}

async fn fetch_plans(
    items: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<Vec<PlanOrAmendment>> {
    let plan_select = Select::with_fields(PRE_REQUEST_DOCUMENTS_FOR_EXPERT_FIELDS)
        .in_any(Plan::uuid, items.iter().map(|oid| oid.uuid))
        .in_any(
            Plan::status_id,
            [
                PlanStatus::ExecutorAppointmentD645,
                PlanStatus::ExecutorAppointmentD646,
                PlanStatus::ExecutorAppointmentD647,
                PlanStatus::ExecutorAppointmentMTP,
            ],
        );

    let plans = PlanOrAmendment::select(&plan_select, db_pool).await?;
    super::check_plans_selection(&plans, items)?;
    Ok(plans)
}
