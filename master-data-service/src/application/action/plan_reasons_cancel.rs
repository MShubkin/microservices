use shared_essential::presentation::dto::{
    master_data::{
        error::MasterDataResult, plan_reasons_cancel::PlanReasonCancel,
        request::SearchPlanReasonsCancelRabbitReq,
    },
    response_request::ApiResponse,
};
use sqlx::PgPool;

use crate::application::master_data::base::plan_reasons_cancel;

pub async fn search(
    dto: SearchPlanReasonsCancelRabbitReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<Vec<PlanReasonCancel>, ()>> {
    plan_reasons_cancel::search_internal(dto, pool).await
}
