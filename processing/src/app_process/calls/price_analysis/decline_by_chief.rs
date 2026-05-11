use std::sync::Arc;

use shared_essential::{
    domain::{
        legacy::plans::PlanStatus, ContractAmendment, Plan, PlanOrAmendment,
        PlanOrAmendmentRep,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::price_analysis::{
            DeclineByChiefReq, DeclineByChiefResponseData, PreDeclineByChiefReq,
            PreDeclineByChiefResponseData,
        },
        response_request::{ApiResponse, Message, Messages, Status},
    },
};
use sqlx::PgPool;

use crate::app_process::records::{send_to_monolith, PlanCollectedUpdate};
use crate::common::{ProcessingCtx, Result};
use asez2_shared_db::db_item::{Select, SelectionKind};

const PRE_DECLINE_BY_CHIEF_TAG: &str = "/pricing/v1/pre_request/decline_by_chief/";
const DECLINE_BY_CHIEF_TAG: &str = "/pricing/v1/action/decline_by_chief/";

const PRE_DECLINE_BY_CHIEF_FIELDS: &[&str] = &[
    "plan_id",
    Plan::customer_id,
    Plan::supplier_id,
    "sum_included_vat_rub",
    Plan::pricing_method_id,
    Plan::expert_conclusion_id,
    "pricing_resume_short",
    Plan::pricing_expert_id,
];

pub(crate) async fn pa_pre_decline_by_chief(
    req: PreDeclineByChiefReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreDeclineByChiefResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = PRE_DECLINE_BY_CHIEF_TAG,
        req = req,
    );

    let plans = fetch_plans(&req.item_list, &db_pool).await?;

    finalise_response(plans, Some(PRE_DECLINE_BY_CHIEF_FIELDS), Messages::default())
}

pub(crate) async fn pa_decline_by_chief(
    req: DeclineByChiefReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<DeclineByChiefResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = DECLINE_BY_CHIEF_TAG,
        req = req,
    );
    let mut messages = Messages::default();
    validate_input(&req, &mut messages);
    if messages.is_error() {
        return Ok(ApiResponse {
            data: vec![],
            messages,
            objects: vec![],
            status: Status::Ok,
        });
    }

    let identifier_list = req
        .item_list
        .iter()
        .map(|i| ObjectIdentifier::new_with_type(i.id, i.uuid, i.object_type))
        .collect::<Vec<_>>();

    let plans = fetch_plans(&identifier_list, &proc_ctx.db_pool).await?;

    let updated_plans = update_plans(req, plans, &proc_ctx, &mut messages).await?;

    if !updated_plans.is_empty() {
        messages.add_prepared_message(
            Message::success(format!(
                "{} ППЗ/ДС отправлен(-ы) Эксперту АЦ",
                updated_plans.len()
            ))
            .with_param_items(&updated_plans),
        );
    }

    finalise_response(updated_plans, None, messages)
}

fn validate_input(req: &DeclineByChiefReq, messages: &mut Messages) {
    let empty_notes = req
        .item_list
        .iter()
        .filter(|identifier| identifier.status_note.is_empty())
        .count();

    if empty_notes > 0 {
        let message =
            format!("Необходимо заполнить комментарий в {} ППЗ/ДС.", empty_notes);
        messages.add_prepared_message(Message::error(message));
    }
}

async fn fetch_plans(
    items: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<Vec<PlanOrAmendment>> {
    let plan_select = Select::full::<ContractAmendment>()
        .add_expand_filter(
            Plan::uuid,
            SelectionKind::In,
            items.iter().map(|i| i.uuid),
        )
        .add_expand_filter(
            Plan::status_id,
            SelectionKind::In,
            vec![
                PlanStatus::AnalysisPerformedD645,
                PlanStatus::AnalysisPerformedD646,
                PlanStatus::AnalysisPerformedD647,
                PlanStatus::AnalysisPerformedMTP,
            ],
        );

    let plans = PlanOrAmendment::select(&plan_select, db_pool).await?;
    super::check_plans_selection(&plans, items)?;
    Ok(plans)
}

async fn update_plans(
    req: DeclineByChiefReq,
    mut plans: Vec<PlanOrAmendment>,
    proc_ctx: &ProcessingCtx,
    messages: &mut Messages,
) -> Result<Vec<PlanOrAmendment>> {
    let DeclineByChiefReq { item_list, user_id } = req;

    plans.iter_mut().for_each(|p| {
        *p.status_id_mut() = match p.status_id() {
            PlanStatus::AnalysisPerformedD645 => PlanStatus::ExecutorAppointedD645,
            PlanStatus::AnalysisPerformedD646 => PlanStatus::ExecutorAppointedD646,
            PlanStatus::AnalysisPerformedD647 => PlanStatus::ExecutorAppointedD647,
            PlanStatus::AnalysisPerformedMTP => PlanStatus::ExecutorAppointedMTP,
            // Не должно произойти, так как выборка была в fetch_plans
            prev => *prev,
        };
    });

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(user_id)
        .with_status_notes(item_list)
        .begin()
        .await?;

    let updated_plans = PlanOrAmendment::update(
        plans,
        &[Plan::status_id],
        messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    send_to_monolith(&updated_plans, &mut recorder).await?;

    recorder.commit().await?;

    Ok(updated_plans)
}

fn finalise_response(
    plans: Vec<PlanOrAmendment>,
    fields: Option<&[&str]>,
    messages: Messages,
) -> Result<ApiResponse<Vec<PlanOrAmendmentRep>, ()>> {
    let plans_rep = plans
        .into_iter()
        .map(PlanOrAmendmentRep::from_item_with_fields_maybe(fields))
        .collect();
    let response = ApiResponse {
        data: plans_rep,
        messages,
        objects: vec![],
        status: Status::Ok,
    };

    Ok(response)
}
