//! Бизнес логика по ручкам "/rest/estimated_commission/v1/(pre_request/action)/change_commission_date/".
use std::sync::Arc;

use sqlx::PgPool;

use shared_essential::{
    application::records::Recorder,
    domain::{
        tables::{legacy::plans::PlanStatus, *},
        JoinedEcAgendaItemEcAgendaRelAgendaProtocolItem as JoinedEcAgendaItem,
        JoinedEcProtocolItemEcProtocol as JoinedEcProtocolItem,
    },
    presentation::dto::{
        general::ObjectIdentifier, processing::*, response_request::*,
    },
};

use crate::{
    app_process::{
        common::{
            self,
            agenda::examine_agenda_items,
            plan::{examine_plan_status, fetch_plans_by_ids},
            protocol::examine_protocol_items,
        },
        records::{PlanCollectedUpdate, ProcessingRulesChecker},
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingCtx, Result},
    presentation::business_messages::plan::PlanChangeCommissionDateMessage,
};

const CHANGE_COMMISSION_DATE: &str = "v1/action/change_commission_date/";
const PRE_CHANGE_COMMISSION_DATE: &str = "v1/pre_request/change_commission_date/";

const RESPONSE_FIELD_LIST: &[&str] = &[
    Plan::uuid,
    "plan_id",
    Plan::customer_id,
    Plan::contract_subject,
    Plan::pricing_expert_id,
    Plan::supplier_id,
    Plan::sum_excluded_vat,
    ContractAmendment::delta_sum_excluded_vat,
    Plan::currency_id,
    Plan::pricing_organization_unit_id,
    Plan::commission_date,
    Plan::status_id,
];

pub(crate) async fn pre_change_commission_date(
    request: PreChangeCommissionDateReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreChangeCommissionDateResponse, ()>> {
    tracing::info!(
        kind = "get",
        "Предзапрос на изменение Даты Очной СК ({get}): {req:?}\n",
        get = PRE_CHANGE_COMMISSION_DATE,
        req = request,
    );

    let (plans, _, messages) =
        pre_change_commission_date_inner(&request.item_list, &db_pool).await?;

    finalise(plans, Some(RESPONSE_FIELD_LIST), messages)
}

pub(crate) async fn change_commission_date(
    request: ChangeCommissionDateReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ChangeCommissionDateResponse, ()>> {
    tracing::info!(
        kind = "get",
        "Запрос на изменение Даты Очной СК ({get}): {req:?}\n",
        get = CHANGE_COMMISSION_DATE,
        req = request,
    );

    let item_list =
        request.item_list.iter().map(|i| i.item.clone()).collect::<Vec<_>>();
    let (plans, agendas_with_items, mut messages) =
        pre_change_commission_date_inner(&item_list, &proc_ctx.db_pool).await?;

    if messages.is_error()
        || (messages.kind == MessageKind::Warning && !request.is_force)
    {
        return Ok(messages.into());
    }
    messages.clear();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .begin()
        .await?;

    let updated_plans = update_commission_date(
        plans,
        request.item_list,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;
    update_agenda_item(agendas_with_items, &mut messages, &mut recorder).await?;

    recorder.commit().await?;

    PlanChangeCommissionDateMessage::Success
        .checked_append(&mut messages, &updated_plans);

    finalise(updated_plans, None, messages)
}

async fn pre_change_commission_date_inner(
    item_list: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Vec<JoinedEcAgendaItem>, Messages)> {
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;
    let mut messages = Messages::default();

    examine_plan_status(
        &plans,
        &[PlanStatus::EstimatedCommissionInPerson],
        PlanChangeCommissionDateMessage::InvalidPlanStatus,
        &mut messages,
    );
    examine_protocol_items(
        &plans,
        Some(ProtocolType::InPersonMeeting),
        examine_protocol_item,
        &mut messages,
        db_pool,
    )
    .await?;

    let joined_agendas =
        examine_agenda_items(&plans, examine_agenda, &mut messages, db_pool)
            .await?;

    Ok((plans, joined_agendas, messages))
}

fn examine_protocol_item(
    protocol_item: &JoinedEcProtocolItem,
    plan: &PlanOrAmendment,
) -> Option<Message> {
    let JoinedEcProtocolItem { item, protocol } = protocol_item;

    if item.result_id != ResultId::NotAgreed {
        PlanChangeCommissionDateMessage::AlreadyInProtocol(protocol)
            .singular(plan)
            .into()
    } else {
        None
    }
}

fn examine_agenda(
    agenda_item: &JoinedEcAgendaItem,
    plan: &PlanOrAmendment,
) -> Option<Message> {
    let JoinedEcAgendaItem {
        agenda,
        item_agenda_protocol_rel,
        ..
    } = agenda_item;

    if item_agenda_protocol_rel.is_some() {
        return None;
    }

    PlanChangeCommissionDateMessage::AlreadyInAgenda(agenda)
        .singular(plan)
        .into()
}

async fn update_commission_date(
    mut plans: Vec<PlanOrAmendment>,
    ids: Vec<ChangeCommissionDateItem>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<Vec<PlanOrAmendment>> {
    plans.iter_mut().for_each(|p| {
        let commission_date = ids.iter().find(|i| {
            i.item.uuid == *p.uuid()
        })
        .map(|i| i.commission_date)
        .expect("Должно найти, так как fetch_plans проверяет наличие всех данных, что пользователь хочет запросить");
        *p.commission_date_mut() = Some(commission_date);
    });

    let updated_plans = PlanOrAmendment::update(
        plans,
        &[Plan::commission_date],
        messages,
        recorder,
        handler,
    )
    .await?;

    Ok(updated_plans)
}

/// По полученному списку ППЗ/ДС необходимо с помощью функции «Получение данных по Повестке»
/// найти данные по ППЗ/ДС, которые включены в крайнюю сформированную и не удаленную Повестку/is_removed = false,
/// где по ППЗ/ДС в позиции Повестки не указан признак «Удалена»/is_removed.  у данной позиции Повестки по ППЗ/ДС
/// нет связи с позицией Протокола в таблице item_relation_agenda_protocol.
///
///
/// Если Повестка найдена и находится в статусе/status_id = 100/«Сформирована»,
/// то при помощи функции «Обновление Повестки» по ППЗ/ДС автоматически устанавливается признак «Удалено»/is_remove = true
///
/// Если Повестка найдена и находится в статусе/status_id = 200/«Отправлена», то при помощи функции «Обновление Повестки» по ППЗ/ДС автоматически устанавливается:
/// «Снято с рассмотрения»/is_excluded = true,
/// «Время проведения»/reviewed_at = None.
async fn update_agenda_item(
    agenda_with_items: Vec<JoinedEcAgendaItem>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    common::agenda::update_agenda_items(
        agenda_with_items,
        |agenda_item| {
            let JoinedEcAgendaItem {
                mut agenda_item,
                agenda,
                item_agenda_protocol_rel,
            } = agenda_item;

            if item_agenda_protocol_rel.is_some() {
                return None;
            }

            match agenda.status_id {
                EcAgendaStatus::Formed => {
                    agenda_item.is_removed = true;
                    Some(agenda_item)
                }
                EcAgendaStatus::Sent => {
                    agenda_item.is_excluded = true;
                    agenda_item.reviewed_at = None;
                    Some(agenda_item)
                }
                _ => None,
            }
        },
        &[
            EcAgendaItem::is_removed,
            EcAgendaItem::is_excluded,
            EcAgendaItem::reviewed_at,
        ],
        messages,
        recorder,
    )
    .await?;

    Ok(())
}

fn finalise(
    plans: Vec<PlanOrAmendment>,
    fields: Option<&[&str]>,
    messages: Messages,
) -> Result<ApiResponse<ChangeCommissionDateResponse, ()>> {
    let data = if messages.is_error() {
        Vec::new()
    } else {
        plans
            .into_iter()
            .map(|p| {
                PlanOrAmendmentRep::from_item_with_section_mapping(
                    p,
                    SectionKind::EstimatedCommission,
                    fields,
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
