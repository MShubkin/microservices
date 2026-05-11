use shared_essential::presentation::dto::general::{
    ObjectIdentifier, ObjectIdentifierWithStatusNote,
};
use shared_essential::presentation::dto::response_request::Status;

use asez2_shared_db::db_item::{Select, SelectionKind};
use shared_essential::{
    domain::{
        legacy::plans::PlanStatus, ContractAmendment, ExpertConclusionId, Plan,
        PlanOrAmendment, PlanOrAmendmentRep, SavingsAccountingId,
    },
    presentation::dto::{
        processing::price_analysis::{
            PriceDeterminedReq, PriceDeterminedResponseData,
        },
        response_request::{ApiResponse, Message, Messages},
    },
};
use sqlx::PgPool;

use crate::app_process::records::{send_to_monolith, PlanCollectedUpdate};
use crate::common::{ProcessingCtx, Result};

const PRICE_DETERMINED_TAG: &str = "/pricing/v1/action/price_determined/";

pub(crate) async fn pa_price_determined(
    req: PriceDeterminedReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<PriceDeterminedResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = PRICE_DETERMINED_TAG,
        req = req,
    );
    let mut messages = Messages::default();

    let plans = fetch_plans(&req.item_list, &proc_ctx.db_pool).await?;

    check_plan_fields(&plans, &mut messages);
    check_conclusion(&plans, &mut messages);

    if messages.is_error() {
        return finalise_response(vec![], messages);
    }

    let updated_plans = update_plans(req, plans, &proc_ctx, &mut messages).await?;

    messages.add_prepared_message(PriceDeterminedMessage::success(&updated_plans));

    let updated_plans = updated_plans
        .into_iter()
        .map(PlanOrAmendmentRep::from_item_with_fields(&[
            "uuid",
            "id",
            "status_id",
        ]))
        .collect();

    finalise_response(updated_plans, messages)
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

fn check_plan_fields(
    plan_or_amendment_list: &[PlanOrAmendment],
    messages: &mut Messages,
) {
    for poa in plan_or_amendment_list {
        [
            (
                poa.expert_conclusion_id()
                    .map_or(true, |c| c == ExpertConclusionId::Undefined),
                "Решение Эксперта АЦ",
            ),
            (
                poa.pricing_resume().as_ref().map_or(true, |r| r.is_empty()),
                "Заключение Эксперта АЦ",
            ),
            (*poa.pricing_method_id() == 0, "Метод ценообразования"),
            (poa.pricing_expert_id().is_none(), "Эксперт АЦ"),
            (
                // Eсли expert_conclusion_id = 1 или 2 или 3 И savings_accounting_id пустое, то формируем
                // ошибку
                poa.expert_conclusion_id().map_or(false, |conclusion_id| {
                    matches!(
                        conclusion_id,
                        ExpertConclusionId::AgreedWithDeclaredPrice
                            | ExpertConclusionId::AgreedWithDecreasingPrice
                            | ExpertConclusionId::AgreedWithIncreasingPrice
                    )
                }) && poa.savings_accounting_id()
                    == &SavingsAccountingId::Undefined,
                "\"Учитывать экономию\"",
            ),
        ]
        .into_iter()
        .filter(|(is_empty, _)| *is_empty)
        .for_each(|(_, field)| {
            messages.add_prepared_message(PriceDeterminedMessage::missing_field(
                field, poa,
            ));
        });
    }
}

/// Если в ППЗ или ДС стоит Решение Эксперта АЦ = Запрос документации (expert_conclusion_id = 5),
/// а status_id = 222 и is_check_documentation = true, то формируем ошибку по ППЗ/ДС
fn check_conclusion(plans: &[PlanOrAmendment], messages: &mut Messages) {
    plans
        .iter()
        .filter(|p| {
            p.expert_conclusion_id()
                .map(|conclusion| {
                    matches!(conclusion, ExpertConclusionId::DocumentationRequest)
                })
                .unwrap_or(false)
                && *p.is_check_documentation()
        })
        .for_each(|invalid_plan| {
            messages.add_prepared_message(
                PriceDeterminedMessage::on_documentation_conclusion(invalid_plan),
            )
        })
}

async fn update_plans(
    req: PriceDeterminedReq,
    mut plans: Vec<PlanOrAmendment>,
    proc_ctx: &ProcessingCtx,
    messages: &mut Messages,
) -> Result<Vec<PlanOrAmendment>> {
    let PriceDeterminedReq { item_list, user_id } = req;

    plans.iter_mut().for_each(|p| {
        *p.status_id_mut() = match p.status_id() {
            PlanStatus::ExecutorAppointedD645 => PlanStatus::AnalysisPerformedD645,
            PlanStatus::ExecutorAppointedD646 => PlanStatus::AnalysisPerformedD646,
            PlanStatus::ExecutorAppointedD647 => PlanStatus::AnalysisPerformedD647,
            PlanStatus::ExecutorAppointedMTP => PlanStatus::AnalysisPerformedMTP,
            // Не должно произойти, так как выборка была четко по статусам выше
            prev => *prev,
        };
    });

    let status_notes = item_list
        .into_iter()
        .map(|i| {
            ObjectIdentifierWithStatusNote::new_with_type(
                i.id,
                i.uuid,
                i.object_type,
                String::new(),
            )
        })
        .collect::<Vec<_>>();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(user_id)
        .with_status_notes(status_notes)
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
) -> Result<ApiResponse<PriceDeterminedResponseData, ()>> {
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

pub(crate) struct PriceDeterminedMessage;

impl PriceDeterminedMessage {
    pub(crate) fn missing_field(field: &str, poa: &PlanOrAmendment) -> Message {
        Message::error(format!("В ППЗ/ДС {} не заполнено поле {}", poa.id(), field))
            .with_param_item(poa)
    }

    pub(crate) fn on_documentation_conclusion(poa: &PlanOrAmendment) -> Message {
        Message::error(String::from("Измените Решение Эксперта АЦ. При «Запросе документации» цена закупки не может быть определена"))
            .with_param_item(poa)
    }

    pub(crate) fn success(plans: &[PlanOrAmendment]) -> Message {
        Message::success(format!("По {} ППЗ/ДС цена определена", plans.len()))
            .with_param_items(plans)
    }
}
