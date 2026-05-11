use asez2_shared_db::{db_item::Select, DbItem};
use shared_essential::{
    domain::{Plan, PlanOrAmendment, PlanStatus, StatusHistory},
    presentation::dto::{
        general::ObjectIdentifier,
        processing::price_analysis::{
            ReviewProgressItem, ReviewProgressReq, ReviewProgressResp,
        },
        response_request::{ApiResponse, EntityKind, Message, Messages},
    },
};

use crate::common::{ProcessingCtx, Result as ProcessingResult};

pub(crate) async fn pa_review_progress(
    ReviewProgressReq { uuid, .. }: ReviewProgressReq,
    proc_ctx: ProcessingCtx,
) -> ProcessingResult<ApiResponse<ReviewProgressResp, ()>> {
    const VALID_STATUSES: &[i16] = &[
        // assign_expert
        PlanStatus::ExecutorAppointmentD646 as _,
        PlanStatus::ExecutorAppointmentD647 as _,
        PlanStatus::ExecutorAppointmentMTP as _,
        // primary_expert_control или determine_price
        PlanStatus::ExecutorAppointedD646 as _,
        PlanStatus::ExecutorAppointedD647 as _,
        PlanStatus::ExecutorAppointedMTP as _,
        // approve_price
        PlanStatus::AnalysisPerformedD646 as _,
        PlanStatus::AnalysisPerformedD647 as _,
        PlanStatus::AnalysisPerformedMTP as _,
    ];
    const APPROVE_STATUS_ID: &[i16] = &[
        PlanStatus::AnalysisPerformedD646 as _,
        PlanStatus::AnalysisPerformedD647 as _,
        PlanStatus::AnalysisPerformedMTP as _,
    ];

    let plan_or_amendment = PlanOrAmendment::select_single(
        &Select::full::<Plan>().eq(Plan::uuid, uuid),
        &proc_ctx.db_pool,
    )
    .await?;

    let status_history = StatusHistory::select(
        &Select::full::<StatusHistory>()
            .eq(StatusHistory::object_uuid, uuid)
            .in_any(StatusHistory::status_id, VALID_STATUSES)
            .add_replace_order_asc(StatusHistory::created_at),
        &*proc_ctx.db_pool,
    )
    .await?;
    if status_history.len() < 2 {
        return Ok((
            ReviewProgressResp::default(),
            Message::info("Недостаточное количество правок"),
        )
            .into());
    }

    let receipt_date = status_history[0].created_at;
    let result: Vec<_> = status_history
        .into_iter()
        .filter(|sh| APPROVE_STATUS_ID.contains(&sh.status_id))
        .filter_map(|sh| {
            match (
                *plan_or_amendment.pricing_expert_id(),
                *plan_or_amendment.expert_conclusion_id(),
            ) {
                (Some(pricing_expert_id), Some(expert_conclusion_id)) => {
                    Some(ReviewProgressItem {
                        object: ObjectIdentifier::new_with_type(
                            *plan_or_amendment.id(),
                            *plan_or_amendment.uuid(),
                            if plan_or_amendment.is_plan() {
                                EntityKind::Plan
                            } else {
                                EntityKind::ContractAmendment
                            },
                        ),
                        consideration_date: sh.created_at,
                        comment: sh.comment,

                        pricing_expert_id,
                        receipt_date,
                        expert_conclusion_id,
                    })
                }
                _ => None,
            }
        })
        .collect();

    Ok((result, Messages::default()).into())
}
