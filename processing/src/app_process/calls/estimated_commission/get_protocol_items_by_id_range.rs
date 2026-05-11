use std::sync::Arc;

use ahash::AHashSet;
use asez2_shared_db::{db_item::Select, DbItem};
use shared_essential::{
    domain::{
        CommissionKind, ContractAmendment, EcProtocol, Plan, PlanOrAmendment,
        PlanOrAmendmentRep, PlanStatus, ProtocolType, ResultId, SectionKind,
    },
    presentation::dto::{
        processing::{
            GetProtocolItemsByIdRangeItem, GetProtocolItemsByIdRangeReq,
            GetProtocolItemsByIdRangeResponseData,
        },
        response_request::{ApiResponse, BusinessMessage, Message, Messages},
    },
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    app_process::{
        common::{self, plan::fetch_plans_by_range_ids},
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingError, Result},
    presentation::business_messages::protocol::ProtocolGetItemsMessage,
};

const RETURN_FIELDS: &[&str] = &[
    Plan::uuid,
    "plan_id",
    Plan::customer_id,
    Plan::supplier_id,
    Plan::contract_subject,
    Plan::currency_id,
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::section_id,
    Plan::status_id,
    Plan::number_customer,
    Plan::sum_excluded_vat,
    Plan::pricing_sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat,
    ContractAmendment::pricing_delta_sum_excluded_vat,
];

pub(crate) async fn get_protocol_items_by_id_range(
    dto: GetProtocolItemsByIdRangeReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetProtocolItemsByIdRangeResponseData, ()>> {
    let protocol_type_id = dto.protocol_type_id;

    let (plans, messages) =
        get_protocol_items_by_id_range_inner(dto, &db_pool).await?;

    if messages.is_error() {
        return Ok(ApiResponse::default().with_messages(messages));
    }

    finalise_response(plans, messages, protocol_type_id)
}

pub(crate) async fn get_protocol_items_by_id_range_inner(
    dto: GetProtocolItemsByIdRangeReq,
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Messages)> {
    let GetProtocolItemsByIdRangeReq {
        item_list,
        protocol_id,
        uuid,
        protocol_type_id,
        ..
    } = dto;

    if protocol_type_id == ProtocolType::Undefined {
        return Err(ProcessingError::GetItemList(format!(
            "Тип Протокола {} не является валидным для данного действия",
            protocol_type_id
        )));
    }

    let protocol_select = Select::full::<EcProtocol>()
        .eq(EcProtocol::uuid, uuid)
        .eq(EcProtocol::id, protocol_id);
    let protocol = EcProtocol::select(&protocol_select, db_pool)
        .await?
        .pop()
        .ok_or(ProcessingError::GetItemList(format!(
            "Протокол СК № {} не найден",
            protocol_id
        )))?;

    let ids = item_list.into_iter().filter_map(|range| {
        match (range.first(), range.get(1)) {
            (Some(&left), Some(&right)) => Some(left..=right),
            (Some(&left), None) => Some(left..=left),
            _ => None,
        }
    });
    let plans = fetch_plans_by_range_ids(ids, db_pool).await?;

    let mut messages =
        get_protocol_items_inner(&plans, protocol_type_id, db_pool).await?;

    if messages.is_error() {
        return Ok((plans, messages));
    }
    messages.clear();

    ProtocolGetItemsMessage::Success(&protocol)
        .checked_append(&mut messages, &plans);

    Ok((plans, messages))
}

pub(crate) async fn get_protocol_items_inner(
    plans: &[PlanOrAmendment],
    protocol_type: ProtocolType,
    db_pool: &PgPool,
) -> Result<Messages> {
    let mut message_buf = Messages::default();
    let mut error_plans = AHashSet::new();

    examine_protocol_items(
        plans,
        protocol_type,
        |protocol, plan| {
            ProtocolGetItemsMessage::AlreadyInProtocol(protocol)
                .singular(plan)
                .into()
        },
        &mut error_plans,
        &mut message_buf,
        db_pool,
    )
    .await?;
    examine_plan_commission_kind(
        plans,
        protocol_type,
        |invalid_plans| {
            let msg = match protocol_type {
                ProtocolType::InPersonMeeting => {
                    ProtocolGetItemsMessage::InvalidInPersonCommissionKind
                }
                ProtocolType::CorrespondenceMeeting => {
                    ProtocolGetItemsMessage::InvalidCorrespondenceCommissionKind
                }
                ProtocolType::Undefined => unreachable!("Проверено выше"),
            };

            msg.resolve(invalid_plans).expect(
                "examine_plan_commission_kind гарантирует что !invalid_plans.is_empty()",
            )
        },
        &mut error_plans,
        &mut message_buf,
    );

    if protocol_type == ProtocolType::CorrespondenceMeeting {
        examine_plan_status(plans, &mut error_plans, &mut message_buf);
    }

    Ok(message_buf)
}

pub(crate) async fn examine_protocol_items<F>(
    plans: &[PlanOrAmendment],
    protocol_type: ProtocolType,
    message_fn: F,
    error_plans: &mut AHashSet<Uuid>,
    messages: &mut Messages,
    db_pool: &PgPool,
) -> Result<()>
where
    F: Fn(&EcProtocol, &PlanOrAmendment) -> Option<Message>,
{
    common::protocol::examine_protocol_items(
        plans,
        Some(protocol_type),
        |protocol_item, plan| {
            if protocol_item.item.result_id != ResultId::NotAgreed
                && error_plans.insert(*plan.uuid())
            {
                message_fn(&protocol_item.protocol, plan)
            } else {
                None
            }
        },
        messages,
        db_pool,
    )
    .await?;

    Ok(())
}

/// Применимо только для protocol_type_id=2
fn examine_plan_status(
    plans: &[PlanOrAmendment],
    error_plans: &mut AHashSet<Uuid>,
    messages: &mut Messages,
) {
    let invalid_plans = plans
        .iter()
        .filter(|plan| {
            !matches!(
                *plan.status_id(),
                PlanStatus::PriceDetermined
                    | PlanStatus::PriceConfirmed
                    | PlanStatus::EstimatedCommissionCorrespondence
            ) && error_plans.insert(*plan.uuid())
        })
        .cloned()
        .collect::<Vec<_>>();

    ProtocolGetItemsMessage::InvalidPlanStatus
        .checked_append(messages, &invalid_plans);
}

/// Проверка на тип коммиссии у ППЗ/ДС в соответствии с типом Протокола
///
/// Гарантирует что message_fn принимает !invalid_plans.is_empty() невалидных ППЗ/ДС
pub(crate) fn examine_plan_commission_kind<F>(
    plans: &[PlanOrAmendment],
    protocol_type: ProtocolType,
    message_fn: F,
    error_plans: &mut AHashSet<Uuid>,
    messages: &mut Messages,
) where
    F: Fn(&[PlanOrAmendment]) -> Message,
{
    let required_commission_kinds: &[CommissionKind] = match protocol_type {
        ProtocolType::Undefined => unreachable!("Проверено выше"),
        ProtocolType::InPersonMeeting => {
            &[CommissionKind::Undefined, CommissionKind::InPerson]
        }
        ProtocolType::CorrespondenceMeeting => &[CommissionKind::Correspondence],
    };

    let invalid_plans = plans
        .iter()
        .filter(|plan| {
            !required_commission_kinds.contains(plan.commission_kind_id())
                && error_plans.insert(*plan.uuid())
        })
        .cloned()
        .collect::<Vec<_>>();

    if !invalid_plans.is_empty() {
        messages.add_prepared_message(message_fn(&invalid_plans));
    }
}

fn finalise_response(
    plans: Vec<PlanOrAmendment>,
    messages: Messages,
    protocol_type_id: ProtocolType,
) -> Result<ApiResponse<GetProtocolItemsByIdRangeResponseData, ()>> {
    let plans = plans
        .into_iter()
        .map(|p| {
            let actual_sum_excluded_vat = match &p {
                PlanOrAmendment::Plan(p) => p.pricing_sum_excluded_vat.into(),
                PlanOrAmendment::Amendment(a) => a.pricing_delta_sum_excluded_vat,
            };
            let plan = PlanOrAmendmentRep::from_item_with_section_mapping(
                p,
                SectionKind::EstimatedCommission,
                Some(RETURN_FIELDS),
            );

            let commission_sum_excluded_vat = match protocol_type_id {
                ProtocolType::CorrespondenceMeeting => {
                    *plan.pricing_sum_excluded_vat()
                }
                _ => None,
            };

            GetProtocolItemsByIdRangeItem {
                plan,
                actual_sum_excluded_vat,
                commission_sum_excluded_vat,
            }
        })
        .collect();

    Ok(ApiResponse::default()
        .with_data(GetProtocolItemsByIdRangeResponseData { item_list: plans })
        .with_messages(messages))
}
