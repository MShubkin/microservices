use ahash::AHashSet;
use itertools::Itertools;

use shared_essential::presentation::dto::response_request::Status;
use shared_essential::presentation::dto::{
    general::ObjectIdentifierWithStatusNote, response_request::BusinessMessage,
};

use asez2_shared_db::db_item::{Select, SelectionKind};
use shared_essential::{
    domain::{
        legacy::plans::PlanStatus, ContractAmendment, ExpertConclusionId, Plan,
        PlanOrAmendment, PlanOrAmendmentRep,
    },
    presentation::dto::{
        processing::price_analysis::{
            ApproveByChiefReq, ApproveByChiefResponseData,
        },
        response_request::{ApiResponse, Messages},
    },
};
use sqlx::PgPool;

use crate::app_process::records::send_to_monolith;
use crate::common::{ProcessingCtx, ProcessingError, Result};
use crate::{
    app_process::records::PlanCollectedUpdate,
    presentation::business_messages::plan::PlanApproveByChiefMessage,
};
use shared_essential::domain::CommissionKind;

const APPROVED_BY_CHIEF_TAG: &str = "/pricing/v1/action/approve_by_chief/";

pub(crate) async fn pa_approve_by_chief(
    req: ApproveByChiefReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ApproveByChiefResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = APPROVED_BY_CHIEF_TAG,
        req = req,
    );
    let mut messages = Messages::default();

    let plans = fetch_plans(&req.item_list, &proc_ctx.db_pool).await?;

    check_plans(&plans, &mut messages);
    if messages.is_error() {
        return finalise_response(vec![], messages);
    }

    let updated_plans = update_plans(req, plans, &proc_ctx, &mut messages).await?;

    PlanApproveByChiefMessage::Refunded.checked_append(
        &mut messages,
        &updated_plans
            .iter()
            .filter(|p| p.status_id() == &PlanStatus::ReturnToClientRework)
            .collect::<Vec<&PlanOrAmendment>>(),
    );
    PlanApproveByChiefMessage::Success.checked_append(
        &mut messages,
        &updated_plans
            .iter()
            .filter(|p| p.status_id() != &PlanStatus::ReturnToClientRework)
            .collect::<Vec<&PlanOrAmendment>>(),
    );

    let from_item =
        PlanOrAmendmentRep::from_item_with_fields(&["uuid", "id", "status_id"]);
    let updated_plans = updated_plans.into_iter().map(from_item).collect();

    finalise_response(updated_plans, messages)
}

async fn fetch_plans(
    items: &[ObjectIdentifierWithStatusNote],
    db_pool: &PgPool,
) -> Result<Vec<PlanOrAmendment>> {
    let initial_len = items.len();

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

    if initial_len != plans.len() {
        let uuid_checker = plans.iter().map(|p| *p.uuid()).collect::<AHashSet<_>>();

        let ids = items
            .iter()
            .filter(|i| !uuid_checker.contains(&i.uuid))
            .map(|i| i.id.to_string())
            .join(", ");

        return Err(ProcessingError::GetItemList(format!(
            "ППЗ/ДС с идентификаторами {} не были найдены для данного действия",
            ids
        )));
    }

    Ok(plans)
}

fn check_plans(
    plan_or_amendment_list: &Vec<PlanOrAmendment>,
    messages: &mut Messages,
) {
    for poa in plan_or_amendment_list {
        if poa.pricing_expert_id().is_none() {
            PlanApproveByChiefMessage::FieldIsMissing("Эксперт АЦ")
                .checked_append(messages, &[poa]);
        }

        // если заключение эксперта = RefundToCustomer, то в update_plan()
        // ппз/дс будет переведен на статус ReturnToClientRework.
        // А при переводе на этот статус проверка формы и даты СК не требуется
        if poa.expert_conclusion_id() == &Some(ExpertConclusionId::RefundToCustomer)
        {
            continue;
        }

        // Проверка на незаполненную форму СК
        let is_plan_with_type_2 = poa.is_plan() && *poa.purchasing_type_id() == 2;
        let is_amendment = poa.is_amendment();
        let commission_kind = poa.commission_kind_id();
        if (is_plan_with_type_2 || is_amendment)
            && *commission_kind == CommissionKind::Undefined
        {
            PlanApproveByChiefMessage::FieldIsMissing("Форма СК")
                .checked_append(messages, &[&poa]);
        }

        // Проверка на незаполненную дату СК
        if *commission_kind == CommissionKind::InPerson
            && poa.commission_date().is_none()
        {
            PlanApproveByChiefMessage::FieldIsMissing("Дата СК")
                .checked_append(messages, &[&poa]);
        }
    }
}

async fn update_plans(
    req: ApproveByChiefReq,
    mut plans: Vec<PlanOrAmendment>,
    proc_ctx: &ProcessingCtx,
    messages: &mut Messages,
) -> Result<Vec<PlanOrAmendment>> {
    let ApproveByChiefReq { item_list, user_id } = req;

    plans.iter_mut().for_each(|p| {
        if p.expert_conclusion_id() == &Some(ExpertConclusionId::RefundToCustomer) {
            *p.status_id_mut() = match p.status_id() {
                PlanStatus::AnalysisPerformedD645
                | PlanStatus::AnalysisPerformedD646
                | PlanStatus::AnalysisPerformedD647
                | PlanStatus::AnalysisPerformedMTP => {
                    PlanStatus::ReturnToClientRework
                }
                // Не должно произойти, так как выборка была четко по статусам выше
                prev => *prev,
            };
        } else {
            *p.status_id_mut() = match p.status_id() {
                PlanStatus::AnalysisPerformedD645 => {
                    PlanStatus::AnalysisCompletedD645
                }
                PlanStatus::AnalysisPerformedD646 => {
                    PlanStatus::AnalysisCompletedD646
                }
                PlanStatus::AnalysisPerformedD647 => {
                    PlanStatus::AnalysisCompletedD647
                }
                PlanStatus::AnalysisPerformedMTP => {
                    PlanStatus::AnalysisCompletedMTP
                }
                // Не должно произойти, так как выборка была четко по статусам выше
                prev => *prev,
            };
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
    updated_plans: Vec<PlanOrAmendmentRep>,
    messages: Messages,
) -> Result<ApiResponse<ApproveByChiefResponseData, ()>> {
    let status = match messages.is_error() {
        true => Status::Error,
        false => Status::Ok,
    };
    let response = ApiResponse {
        data: updated_plans,
        messages,
        objects: vec![],
        status,
    };

    Ok(response)
}
