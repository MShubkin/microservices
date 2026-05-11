//! Бизнес логика по ручке "/rest/estimated_commission/v1/pre_request/approve/".

use std::sync::Arc;

use shared_essential::{
    domain::tables::{JoinedEcProtocolItemEcProtocol as JoinedEcProtocolItem, *},
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{PreApprovePlansReq, PreApprovePlansResponseData},
        response_request::*,
    },
};
use sqlx::PgPool;

use crate::{
    app_process::{
        common::{
            plan::{examine_plan_status, fetch_plans_by_ids},
            protocol::examine_protocol_items,
        },
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingError, Result},
    presentation::business_messages::plan::PlanApproveMessage,
};

use super::PRE_REQUEST_RESPONSE_FIELDS;

const PRE_APPROVE_PLANS: &str = "/v1/pre_request/approve/";

pub(crate) async fn pre_approve(
    request: PreApprovePlansReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreApprovePlansResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Получен предзапрос на утверждение ППЗ/ДС ({get}): {req:?}\n",
        get = PRE_APPROVE_PLANS,
        req = request,
    );

    let (plans, messages) =
        pre_approve_inner(&request.item_list, request.section_id, &db_pool).await?;

    finalise(plans, messages)
}

pub(super) async fn pre_approve_inner(
    item_list: &[ObjectIdentifier],
    section_id: Section,
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Messages)> {
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;
    let mut messages = Messages::default();

    match section_id {
        Section::EstimatedCommissionInPerson => {
            examine_plan_status(
                &plans,
                &[PlanStatus::EstimatedCommissionInPerson],
                PlanApproveMessage::InvalidPlanStatus,
                &mut messages,
            );
        }
        Section::EstimatedCommissionCorrespondence => {
            examine_plan_status(
                &plans,
                &[PlanStatus::EstimatedCommissionCorrespondence],
                PlanApproveMessage::InvalidPlanStatus,
                &mut messages,
            );
            examine_protocol_items(
                &plans,
                Some(ProtocolType::CorrespondenceMeeting),
                |protocol_item, plan| examine_protocol(protocol_item, plan).into(),
                &mut messages,
                db_pool,
            )
            .await?;
        }
        // Ничего не проверяется
        Section::EstimatedCommissionNotRequired => {}
        invalid_section => {
            return Err(ProcessingError::Section(format!(
                "Секция {} невалидна для утверждения ППЗ/ДС",
                invalid_section
            )))
        }
    };

    Ok((plans, messages))
}

fn examine_protocol(
    protocol_item: &JoinedEcProtocolItem,
    plan: &PlanOrAmendment,
) -> Message {
    let JoinedEcProtocolItem { protocol, .. } = protocol_item;

    let msg = match protocol.status_id {
        EcProtocolStatus::Formed | EcProtocolStatus::AgreementPending => {
            PlanApproveMessage::AlreadyInProtocolWarn(protocol)
        }
        _ => PlanApproveMessage::AlreadyInProtocolErr(protocol),
    };

    msg.singular(plan)
}

fn finalise(
    plans: Vec<PlanOrAmendment>,
    messages: Messages,
) -> Result<ApiResponse<PreApprovePlansResponseData, ()>> {
    let data = if messages.is_error() {
        Vec::new()
    } else {
        plans
            .into_iter()
            .map(|p| {
                PlanOrAmendmentRep::from_item_with_section_mapping(
                    p,
                    SectionKind::EstimatedCommission,
                    Some(PRE_REQUEST_RESPONSE_FIELDS),
                )
            })
            .collect::<Vec<_>>()
    };

    Ok(ApiResponse {
        status: Status::Ok,
        data: data.into(),
        messages,
        objects: vec![],
    })
}
