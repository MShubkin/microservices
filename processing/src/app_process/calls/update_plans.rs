//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.

use crate::app_process::records::PlanRepCollectedUpdate;
use crate::common::{ProcessingCtx, Result};

use shared_essential::{
    domain::tables::PlanOrAmendmentRep,
    presentation::dto::{processing::*, response_request::*},
};

const UPDATE_PLANS: &str = "v1/action/update_plans";

/// Client FE demands a list of ids of the created agenda items.
/// NB: It is possible that returning everything will improve the system's efficiency.
pub(crate) async fn update_plans(
    request: PrUpdatePlansReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<PrUpdatePlansResponseData, ()>> {
    trace_request(&request);
    let PrUpdatePlansReq {
        user_id,
        plans,
        fields,
    } = request;
    let mut messages = Messages::default();

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;
    let plans = PlanOrAmendmentRep::update(
        plans,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;
    recorder.commit().await?;

    let data = plans
        .into_iter()
        .map(PlanOrAmendmentRep::from_item_with_fields(&fields))
        .collect::<Vec<_>>();

    Ok((data, messages).into())
}

fn trace_request(req: &PrUpdatePlansReq) {
    if req.plans.is_empty() {
        tracing::info!(
            kind = "get",
            "Запрос на обновление ППЗ/ДС ({get}): {req:?}\n",
            get = UPDATE_PLANS,
            req = req
        );
    } else if let Some(head) = req.plans.first() {
        tracing::info!(
            kind = "get",
            "Запрос на обновление ППЗ/ДС ({get}): user_id:{user_id}, fields:{fields:?}, первая из {count} : {head:?}\n",
            get = UPDATE_PLANS,
            count = req.plans.len(),
            head = head,
            fields = req.fields,
            user_id = req.user_id,
        );
        tracing::trace!(kind = "get", "{req:?}\n", req = req.plans.get(1..));
    }
}
