use std::sync::Arc;

use shared_essential::presentation::dto::general::ObjectIdentifier;
use shared_essential::presentation::dto::response_request::Status;

use asez2_shared_db::db_item::{Select, SelectionKind};
use shared_essential::{
    domain::{
        legacy::plans::PlanStatus, ContractAmendment, ExpertConclusionId, Plan,
        PlanOrAmendment, PlanOrAmendmentRep,
    },
    presentation::dto::{
        processing::price_analysis::{
            PreReturnToCustomerReq, PreReturnToCustomerResponseData,
            ReturnToCustomerReq, ReturnToCustomerResponseData,
        },
        response_request::{ApiResponse, Message, Messages},
    },
};
use sqlx::PgPool;

use crate::app_process::records::{send_to_monolith, PlanCollectedUpdate};
use crate::common::{ProcessingCtx, Result};

const RETURN_TO_CUSTOMER_TAG: &str = "/pricing/v1/action/return_to_customer/";
const PRE_RETURN_TO_CUSTOMER_TAG: &str =
    "/pricing/v1/pre_request/return_to_customer/";

const PRE_RETURN_TO_CUSTOMER_FIELDS: &[&str] = &[
    Plan::uuid,
    "plan_id",
    Plan::contract_subject,
    Plan::commission_kind_id,
    Plan::commission_date,
    Plan::customer_id,
    Plan::supplier_id,
];

pub(crate) async fn pa_pre_return_to_customer(
    req: PreReturnToCustomerReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreReturnToCustomerResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = PRE_RETURN_TO_CUSTOMER_TAG,
        req = req,
    );

    let plans = fetch_plans(&req.item_list, &db_pool).await?;

    finalise_response(
        plans,
        Some(PRE_RETURN_TO_CUSTOMER_FIELDS),
        Messages::default(),
    )
}

pub(crate) async fn pa_return_to_customer(
    req: ReturnToCustomerReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ReturnToCustomerResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = RETURN_TO_CUSTOMER_TAG,
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

    messages.add_prepared_message(Message::info(format!(
        "{} ППЗ/ДС будет отправлено Заказчику на доработку \
            после подтверждения Руководителем АЦ",
        updated_plans.len()
    )));

    finalise_response(updated_plans, Some(&["uuid", "status_id"]), messages)
}

fn validate_input(req: &ReturnToCustomerReq, messages: &mut Messages) {
    for identifier in &req.item_list {
        if identifier.status_note.is_empty() {
            messages.add_prepared_message(Message::error(format!(
                "Для ППЗ/ДС {} требуется комментарий Эксперта АЦ",
                identifier.id
            )));
        }
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
                PlanStatus::ExecutorAppointedD645,
                PlanStatus::ExecutorAppointedD646,
                PlanStatus::ExecutorAppointedD647,
                PlanStatus::ExecutorAppointedMTP,
            ],
        );

    let plans = PlanOrAmendment::select(&plan_select, db_pool).await?;
    super::check_plans_selection(&plans, items)?;
    Ok(plans)
}

async fn update_plans(
    req: ReturnToCustomerReq,
    mut plans: Vec<PlanOrAmendment>,
    proc_ctx: &ProcessingCtx,
    messages: &mut Messages,
) -> Result<Vec<PlanOrAmendment>> {
    let ReturnToCustomerReq { item_list, user_id } = req;

    plans.iter_mut().for_each(|p| {
        *p.status_id_mut() = match p.status_id() {
            PlanStatus::ExecutorAppointedD645 => PlanStatus::AnalysisPerformedD645,
            PlanStatus::ExecutorAppointedD646 => PlanStatus::AnalysisPerformedD646,
            PlanStatus::ExecutorAppointedD647 => PlanStatus::AnalysisPerformedD647,
            PlanStatus::ExecutorAppointedMTP => PlanStatus::AnalysisPerformedMTP,
            // Не должно произойти, так как выборка была четко по статусам выше
            prev => *prev,
        };
        *p.expert_conclusion_id_mut() = Some(ExpertConclusionId::RefundToCustomer);
    });

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(user_id)
        .with_status_notes(item_list)
        .begin()
        .await?;

    let updated_plans = PlanOrAmendment::update(
        plans,
        &[Plan::status_id, Plan::expert_conclusion_id],
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
