use crate::app_process::get_price_analysis_user;
use crate::app_process::records::send_plans_to_monolith;
use crate::common::{ProcessingCtx, Result};

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::{AdaptorableIter, Select};

use shared_essential::domain::{
    Plan, PlanStatus, PlanWithLastStatus, PlanWithLastStatusSelector,
    StatusHistory, UserType,
};
use shared_essential::presentation::dto::{
    general::ObjectIdentifierList,
    price_analysis::CompleteLottingRes,
    processing::price_analysis::{
        CompleteLottingData, CompleteLottingRequest, GetPriceAnalysisUsersReq,
    },
    response_request::{BusinessMessage, Message, Messages},
};

use ahash::AHashMap;
use sqlx::PgPool;
use uuid::Uuid;

const COMPLETE_LOTTING: &str = "pricing/v1/action/complete_lotting";

/// TODO amend the fields when the task becomes clearer.
const PLAN_FIELDS: &[&str] = &[
    Plan::uuid,
    Plan::id,
    Plan::customer_id,
    Plan::contract_subject,
    Plan::sum_excluded_vat_rub,
    Plan::pricing_sum_excluded_vat_rub,
];

#[tracing::instrument(skip_all)]
pub(crate) async fn pa_complete_lotting(
    req: CompleteLottingRequest,
    proc_ctx: ProcessingCtx,
) -> Result<CompleteLottingRes> {
    tracing::info!(
        kind = "update",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = COMPLETE_LOTTING,
        req = req,
    );
    let pool = &*proc_ctx.db_pool;
    let CompleteLottingRequest {
        user_id,
        dto: ObjectIdentifierList { item_list },
    } = req;

    let uuids = item_list.iter().map(|x| x.uuid).collect();
    let plans = get_plans_with_statuses(uuids, pool).await?;
    let plans = prepare_status_updates(plans);

    let mut messages = Messages::default();
    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let plans = recorder
        .process_update(plans, &[Plan::status_id], &mut messages)
        .await?;

    send_plans_to_monolith(&plans, &mut recorder).await?;

    recorder.commit().await?;

    let user_req = make_user_req(&plans);
    fill_messages(&plans, &mut messages);

    let plans = plans.into_iter().adaptors_with_fields(PLAN_FIELDS).collect();

    // We will need the user data for sending the final e-mail notifications.
    let users_to_notify =
        get_price_analysis_user(user_req, proc_ctx.db_pool.clone()).await?;
    let data = CompleteLottingData {
        users: users_to_notify.data,
        plan_data: plans,
    };
    Ok((data, messages).into())
}

/// TODO rename to `inner_XXX` when we find the pre_request task.
#[tracing::instrument(skip_all)]
async fn get_plans_with_statuses(
    uuids: Vec<Uuid>,
    pool: &PgPool,
) -> Result<Vec<PlanWithLastStatus>> {
    let plan_select = Select::full::<Plan>().in_any(Plan::uuid, uuids);
    let status_select = Select::full::<StatusHistory>()
        .add_replace_order_asc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_by)
        .distinct_on(&[StatusHistory::object_uuid]);
    let status_select = StatusHistory::join_default().selecting(status_select);

    PlanWithLastStatusSelector::new(plan_select)
        .set_status(status_select)
        .get(pool)
        .await
        .map_err(Into::into)
}

#[tracing::instrument(skip_all)]
fn prepare_status_updates(joined_plans: Vec<PlanWithLastStatus>) -> Vec<Plan> {
    use PlanStatus::*;
    joined_plans
        .into_iter()
        .filter_map(|x| {
            let mut plan = x.plan;
            if !matches!(plan.status_id, LottingMTP) {
                return None;
            }
            let old_status = x.status.map(|x| PlanStatus::from(x.status_id));
            plan.status_id = match old_status {
                None | Some(ExecutorAppointmentMTP) => ExecutorAppointmentMTP,
                Some(ExecutorAppointedMTP) => ExecutorAppointedMTP,
                Some(AnalysisPerformedMTP) => AnalysisPerformedMTP,
                _ => return None,
            };
            Some(plan)
        })
        .collect()
}

fn fill_messages(plans: &[Plan], messages: &mut Messages) {
    let mut plan_map = AHashMap::new();
    for plan in plans {
        plan_map.entry(plan.status_id).or_insert(vec![]).push(plan);
    }
    for (status, plans) in plan_map.into_iter() {
        let global = CompleteLottingMTR(status).plural(&plans);
        messages.add_prepared_message(global);
    }
}

fn make_user_req(plans: &[Plan]) -> GetPriceAnalysisUsersReq {
    let mut unit_ids = Vec::with_capacity(4);

    for plan in plans {
        if !unit_ids.contains(&plan.pricing_organization_unit_id) {
            unit_ids.push(plan.pricing_organization_unit_id);
        }
    }
    GetPriceAnalysisUsersReq {
        user_ids: None,
        unit_ids: Some(unit_ids),
        user_types: Some(vec![UserType::Director, UserType::Expert]),
    }
}

#[derive(Debug)]
struct CompleteLottingMTR(PlanStatus);

impl BusinessMessage for CompleteLottingMTR {
    type Entity = Plan;
    fn singular(&self, e: &Self::Entity) -> Message {
        let msg = format!("1 ППЗ переведены на статус {}.", self.0);
        Message::success(msg).with_param_item(e)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let l = entities.len();
        let case = match l {
            1 => "переведен",
            _ => "переведены",
        };
        let msg = format!("{l} ППЗ {case} на статус \"{}\".", self.0);
        Message::success(msg).with_param_items(entities)
    }
}
