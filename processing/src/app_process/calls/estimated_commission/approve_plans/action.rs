use shared_essential::{
    domain::tables::{legacy::plans::PlanStatus, *},
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{ApprovePlansReq, ApprovePlansResponseData},
        response_request::{ApiResponse, BusinessMessage},
    },
};

use crate::{
    app_process::{
        estimated_commission::approve_plans::pre_request::pre_approve_inner,
        records::{send_to_monolith, PlanCollectedUpdate},
    },
    common::{ProcessingCtx, Result},
    presentation::business_messages::plan::PlanApproveMessage,
};

const ACTION_APPROVE_PLANS: &str = "/v1/action/approve/";

const FIELDS_TO_UPDATE: &[&str] = &[Plan::status_id, Plan::changed_by];

pub(crate) async fn action_approve(
    request: ApprovePlansReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ApprovePlansResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Processing: Got request from ({get}): {req:?}\n",
        get = ACTION_APPROVE_PLANS,
        req = request,
    );

    let pseudo_item_list = request
        .item_list
        .iter()
        .map(|x| ObjectIdentifier {
            uuid: x.uuid,
            id: x.id,
            ..Default::default()
        })
        .collect::<Vec<_>>();

    let (mut plans, mut messages) =
        pre_approve_inner(&pseudo_item_list, request.section_id, &proc_ctx.db_pool)
            .await?;
    if messages.is_error() || (messages.is_warn() && !request.is_force) {
        return Ok(ApiResponse::default().with_messages(messages));
    }

    change_status(&mut plans);

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .with_status_notes(request.item_list)
        .begin()
        .await?;

    let updated_plans = PlanOrAmendment::update(
        plans,
        FIELDS_TO_UPDATE,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    send_to_monolith(&updated_plans, &mut recorder).await?;

    recorder.commit().await?;

    PlanApproveMessage::Success.checked_append(&mut messages, &updated_plans);

    Ok(ApiResponse::default().with_messages(messages))
}

fn change_status(items: &mut [PlanOrAmendment]) {
    items.iter_mut().for_each(|item| {
        let new_status_id = match item {
            PlanOrAmendment::Plan(plan) => {
                if plan.is_not_purchase {
                    PlanStatus::PriceDetermined
                } else {
                    PlanStatus::PriceConfirmed
                }
            }
            PlanOrAmendment::Amendment(_) => PlanStatus::PriceConfirmed,
        };
        *item.status_id_mut() = new_status_id;
    })
}
