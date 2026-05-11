use sqlx::PgPool;
use std::sync::Arc;

use shared_essential::presentation::dto::processing::{
    GetExpertPlansCount, PlansCountRequest,
};

use crate::app_process::sections::*;
use crate::common::Result;

const GET_PLANS: &str = "/v1/get_plans_count";

/// This is the actual function.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_plans_count(
    req: PlansCountRequest,
    db_pool: Arc<PgPool>,
) -> Result<GetExpertPlansCount> {
    tracing::info!(
        kind = "get",
        "Получение ППЗ/ДС ({get}): {req:?}",
        req = req,
        get = GET_PLANS
    );

    let counts = process_count_sections(req, &db_pool).await?;

    Ok(GetExpertPlansCount::default().with_data(counts))
}
