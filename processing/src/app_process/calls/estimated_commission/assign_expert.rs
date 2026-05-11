use ahash::AHashSet;
use sqlx::PgPool;

use asez2_shared_db::{db_item::selection::*, Value};
use shared_essential::{
    domain::{
        tables::{legacy::plans::PlanStatus, *},
        PlanOrAmendment,
    },
    presentation::dto::{
        general::ObjectIdentifier, processing::*, response_request::*,
    },
};

use crate::app_process::records::{send_to_monolith, PlanCollectedUpdate};
use crate::common::{ProcessingCtx, ProcessingError, Result};

/// Ручка
const ASSIGN_EXPERT: &str = "/v1/action/assign_expert/";

/// The fields which are returned.
const FIELDS: &[&str] = &["uuid", "id", "pricing_expert_id", "status_id"];

pub(crate) async fn assign_expert(
    request: AssignExpertReq,
    proc_ctx: ProcessingCtx,
) -> Result<AssignExpertResponse> {
    let db_pool = &*proc_ctx.db_pool;

    tracing::info!(
        kind = "get",
        "Processing: Got request from ({get}): {req:?}\n",
        get = ASSIGN_EXPERT,
        req = request,
    );

    let AssignExpertReq { ids, user_id } = request;

    let plans = get_plans_or_amendments(&ids, db_pool).await?;
    let mut messages = Messages::default();

    // TODO фронт хотел какую то развернутую структуру сообщений, не знаю что
    for poa in &plans {
        if poa.pricing_expert_id().is_none() {
            messages.add_message(
                MessageKind::Error,
                format!(
                    "Необходимо указать Эксперта АЦ: {} {}",
                    poa.system_name(),
                    poa.id()
                ),
            );
        }
    }

    if !messages.is_empty() {
        return Ok(ApiResponse {
            status: Status::Error,
            messages,
            objects: vec![],
            data: vec![],
        });
    }

    do_update(plans, user_id, &proc_ctx).await
}

async fn get_plans_or_amendments(
    ids: &[ObjectIdentifier],
    conn: &PgPool,
) -> Result<Vec<PlanOrAmendment>> {
    let uuids = ids.iter().map(|x| Value::from(x.uuid)).collect::<Vec<_>>();
    let select = Select::full_in::<_, ContractAmendment>("uuid", uuids);
    let plans = PlanOrAmendment::select(&select, conn).await?;

    if plans.len() != ids.len() {
        let found_uuids: AHashSet<_> = plans.iter().map(|x| *x.uuid()).collect();
        let missing = ids
            .iter()
            .filter(|x| !found_uuids.contains(&x.uuid))
            .map(|x| x.uuid.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let msg = format!("Записи ППЗ/ДС не найдены: {}", missing);
        return Err(ProcessingError::AssignExpert(msg));
    }

    Ok(plans)
}

async fn do_update(
    mut plans: Vec<PlanOrAmendment>,
    user_id: i32,
    proc_ctx: &ProcessingCtx,
) -> Result<AssignExpertResponse> {
    let mut messages = Messages::default();

    plans.iter_mut().for_each(|p| {
        *p.status_id_mut() = match p.status_id() {
            PlanStatus::ExecutorAppointmentD645 => {
                PlanStatus::ExecutorAppointedD645
            }
            PlanStatus::ExecutorAppointmentD646 => {
                PlanStatus::ExecutorAppointedD646
            }
            PlanStatus::ExecutorAppointmentD647 => {
                PlanStatus::ExecutorAppointedD647
            }
            PlanStatus::ExecutorAppointmentMTP => PlanStatus::ExecutorAppointedMTP,
            status => {
                messages.add_message(
                    MessageKind::Error,
                    format!("ППЗ/ДС находится статусе {}", status),
                );
                *status
            }
        };
    });

    if !messages.is_empty() {
        return Ok(ApiResponse {
            status: Status::Error,
            messages,
            objects: vec![],
            data: vec![],
        });
    }

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let updated_plans = PlanOrAmendment::update(
        plans,
        &["uuid", "id", "status_id"],
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    send_to_monolith(&updated_plans, &mut recorder).await?;

    messages.add_prepared_message(
        Message::success(format!(
            "Вы отправили Эксперту АЦ {} ППЗ/ДС",
            updated_plans.len()
        ))
        .with_param_items(&updated_plans),
    );

    let updated_plans = updated_plans
        .into_iter()
        .map(PlanOrAmendmentRep::from_item_with_fields(FIELDS))
        .collect();

    recorder.commit().await?;

    Ok(ApiResponse {
        status: Status::Ok,
        messages,
        objects: vec![],
        data: updated_plans,
    })
}
