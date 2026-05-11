use ahash::AHashMap;
use itertools::Itertools;

use shared_essential::presentation::dto::general::ObjectIdentifier;
use sqlx::PgPool;
use uuid::Uuid;

use crate::app_process::records::{send_to_monolith, PlanCollectedUpdate};
use crate::common::{ProcessingCtx, Result};
use asez2_shared_db::db_item::selection::*;
use shared_essential::{
    domain::{
        tables::{legacy::plans::PlanStatus, *},
        PlanOrAmendment,
    },
    presentation::dto::{processing::*, response_request::*},
};

/// Ручка
const ASSIGN_EXPERT: &str = "/v1/action/assign_expert_mass/";

/// The fields which are returned.
const FIELDS: &[&str] = &[
    Plan::uuid,
    Plan::id,
    Plan::status_id,
    Plan::commission_date,
    Plan::commission_kind_id,
    Plan::pricing_expert_id,
    Plan::pricing_competitive_note_for_expert,
];

/// The fields which are returned.
const ADAPTOR_FIELDS: &[&str] = &[
    Plan::uuid,
    Plan::status_id,
    Plan::commission_date,
    Plan::commission_kind_id,
    Plan::pricing_expert_id,
    Plan::pricing_competitive_note_for_expert,
    "plan_id",
];

enum AssignExpertMassMessage<'a> {
    ExpertNotSet(&'static str, i64),
    Success(&'a [PlanOrAmendment]),
}

impl<'a> AssignExpertMassMessage<'a> {
    fn into_message(self) -> Message {
        match self {
            AssignExpertMassMessage::ExpertNotSet(kind, id) => Message::error(
                format!("Необходимо указать Эксперта АЦ: {kind} {id}",),
            ),
            AssignExpertMassMessage::Success(plans) => Message::success(format!(
                "Вы отправили Эксперту АЦ {} ППЗ/ДС",
                plans.len()
            ))
            .with_param_items(plans),
        }
    }
}

pub(crate) async fn assign_expert_mass(
    request: AssignExpertMassReq,
    proc_ctx: ProcessingCtx,
) -> Result<AssignExpertMassResponse> {
    let db_pool = &*proc_ctx.db_pool;

    tracing::info!(
        kind = "get",
        "Processing: Got request from ({get}): {req:?}\n",
        get = ASSIGN_EXPERT,
        req = request,
    );

    if let Err(messages) = validate_request(&request) {
        return Ok(ApiResponse::default().with_messages(messages));
    }

    let AssignExpertMassReq { plans, user_id } = request;

    let ids_to_plans: AHashMap<_, _> = plans
        .into_iter()
        .flat_map(|plan| plan.uuid().map(|uuid| (uuid, plan)))
        .collect();

    let plans = get_plans_or_amendments(&ids_to_plans, db_pool).await?;
    let (plans, messages) =
        update_plans(plans, ids_to_plans, user_id, &proc_ctx).await?;

    Ok(ApiResponse::default().with_data(plans).with_messages(messages))
}

fn validate_request(
    req: &AssignExpertMassReq,
) -> std::result::Result<(), Messages> {
    let AssignExpertMassReq { plans, .. } = req;
    let mut messages = Messages::default();
    plans.iter().for_each(|plan| {
        if plan.pricing_expert_id().is_none() {
            messages.add_prepared_message(
                AssignExpertMassMessage::ExpertNotSet(
                    plan.kind_str(),
                    plan.id().unwrap_or(0),
                )
                .into_message(),
            );
        }
    });

    if !messages.is_empty() {
        Err(messages)
    } else {
        Ok(())
    }
}

async fn get_plans_or_amendments(
    plans_map: &AHashMap<Uuid, PlanOrAmendmentRep>,
    conn: &PgPool,
) -> Result<Vec<PlanOrAmendment>> {
    let select = Select::with_fields(FIELDS).in_any(Plan::uuid, plans_map.keys());
    let plans = PlanOrAmendment::select(&select, conn).await?;
    let oids: Vec<_> = plans_map
        .iter()
        .map(|(uuid, plan)| ObjectIdentifier::new(plan.id().unwrap_or(0), *uuid))
        .collect();
    super::check_plans_selection(&plans, &oids)?;
    Ok(plans)
}

fn update_status_id(mut plan: PlanOrAmendment) -> PlanOrAmendment {
    let status = match plan.status_id() {
        PlanStatus::ExecutorAppointmentD645 => PlanStatus::ExecutorAppointedD645,
        PlanStatus::ExecutorAppointmentD646 => PlanStatus::ExecutorAppointedD646,
        PlanStatus::ExecutorAppointmentD647 => PlanStatus::ExecutorAppointedD647,
        PlanStatus::ExecutorAppointmentMTP => PlanStatus::ExecutorAppointedMTP,
        _ => return plan,
    };
    *plan.status_id_mut() = status;
    plan
}

async fn update_plans(
    plans: Vec<PlanOrAmendment>,
    mut ids: AHashMap<Uuid, PlanOrAmendmentRep>,
    user_id: i32,
    proc_ctx: &ProcessingCtx,
) -> Result<(Vec<PlanOrAmendmentRep>, Messages)> {
    let mut messages = Messages::default();

    let plans = plans
        .into_iter()
        .filter_map(|plan| {
            ids.remove(plan.uuid()).map(|dto| Ok(dto.into_item_merged(plan)?))
        })
        .map_ok(update_status_id)
        .collect::<Result<_>>()?;

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let updated_plans = PlanOrAmendment::update(
        plans,
        FIELDS,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    send_to_monolith(&updated_plans, &mut recorder).await?;

    if !messages.is_error() {
        messages.add_prepared_message(
            AssignExpertMassMessage::Success(&updated_plans).into_message(),
        )
    }
    let updated_plans = updated_plans
        .into_iter()
        .map(PlanOrAmendmentRep::from_item_with_fields(ADAPTOR_FIELDS))
        .collect();

    recorder.commit().await?;

    Ok((updated_plans, messages))
}
