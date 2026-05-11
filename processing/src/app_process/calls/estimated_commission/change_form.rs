use std::{ops::Deref, sync::Arc};

use asez2_shared_db::db_item::AsezDate;
use sqlx::PgPool;

use shared_essential::{
    application::records::Recorder,
    domain::{
        tables::{legacy::plans::PlanStatus, *},
        EcAgendaItem, EcAgendaStatus, EcProtocolItem,
        JoinedEcAgendaItemEcAgendaRelAgendaProtocolItem as JoinedEcAgendaItem,
        JoinedEcProtocolItemEcProtocol as JoinedEcProtocolItem, Plan,
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
        records::{send_to_monolith, PlanCollectedUpdate, ProcessingRulesChecker},
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::plan::PlanChangeFormMessage,
};

/// Ручка
const PRE_CHANGE_FORM: &str = "/v1/pre_request/change_form/";
const CHANGE_FORM: &str = "/v1/action/change_form/";

/// The fields which are returned from the precheck.
const PRE_REQUEST_RETURN_FIELDS: &[&str] = &[
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

pub(crate) async fn pre_change_form(
    request: PreChangeFormReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreChangeFormResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Предзапрос на изменение формы СК ({get}): {req:?}\n",
        get = PRE_CHANGE_FORM,
        req = request,
    );

    let (plans, _, _, messages) =
        pre_change_form_inner(&request.item_list, request.section_id, &db_pool)
            .await?;

    finalise(plans, Some(PRE_REQUEST_RETURN_FIELDS), messages)
}

pub(crate) async fn change_form(
    request: ChangeFormReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ChangeFormResponseData, ()>> {
    let db_pool = &*proc_ctx.db_pool;

    tracing::info!(
        kind = "get",
        "Получен запрос на изменение формы СК ({get}): {req:?}\n",
        get = CHANGE_FORM,
        req = request,
    );

    if request.commission_kind_id == CommissionKind::Undefined {
        return Err(ProcessingError::ChangeForm(String::from(
            "Невалидное значение было передано для `commission_kind_id`",
        )));
    }
    let item_ids =
        request.item_list.iter().map(Deref::deref).cloned().collect::<Vec<_>>();

    let (plans, joined_agendas, joined_protocols, mut messages) =
        pre_change_form_inner(&item_ids, request.section_id, db_pool).await?;

    if messages.is_error() || (messages.is_warn() && !request.is_force) {
        return finalise(plans, None, messages);
    }
    messages.clear();

    let commission_kind_id = request.commission_kind_id;
    let section_id = request.section_id;

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .with_status_notes(request.item_list)
        .begin()
        .await?;

    let updated_plans = update_plans(
        plans,
        request.commission_kind_id,
        request.section_id,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    if section_id == Section::EstimatedCommissionInPerson {
        update_agenda_items(
            joined_agendas
                .expect("Для EstimatedCommissionInPerson гарантированно будут возвращены элементы"), 
            &mut messages,
            &mut recorder,
        )
        .await?;
    }

    if section_id == Section::EstimatedCommissionCorrespondence {
        update_protocol_item(
            joined_protocols
                .expect("Для EstimatedCommissionCorrespondence гарантированно будут возвращены элементы"), 
            &mut messages,
            &mut recorder,
        )
        .await?;
    }

    recorder.commit().await?;

    match commission_kind_id {
        CommissionKind::InPerson => PlanChangeFormMessage::InPersonSuccess
            .checked_append(&mut messages, &updated_plans),
        CommissionKind::Correspondence => {
            PlanChangeFormMessage::CorrespondenceSuccess
                .checked_append(&mut messages, &updated_plans)
        }
        CommissionKind::NotRequired => PlanChangeFormMessage::NoCommissionSuccess
            .checked_append(&mut messages, &updated_plans),
        CommissionKind::Undefined => unreachable!("Выше была проверка"),
    };

    finalise(updated_plans, None, messages)
}

async fn pre_change_form_inner(
    item_list: &[ObjectIdentifier],
    section_id: Section,
    db_pool: &PgPool,
) -> Result<(
    Vec<PlanOrAmendment>,
    Option<Vec<JoinedEcAgendaItem>>,
    Option<Vec<JoinedEcProtocolItem>>,
    Messages,
)> {
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;
    let mut messages = Messages::default();

    let (joined_agendas, joined_protocols) = match section_id {
        Section::EstimatedCommissionInPerson => {
            examine_plan_status(
                &plans,
                &[PlanStatus::EstimatedCommissionInPerson],
                PlanChangeFormMessage::InvalidPlanStatus,
                &mut messages,
            );
            let protocols_with_items = examine_protocol_items(
                &plans,
                Some(ProtocolType::InPersonMeeting),
                examine_protocol_in_person,
                &mut messages,
                db_pool,
            )
            .await?;

            let joined_agendas = examine_agenda_items(
                &plans,
                examine_agenda,
                &mut messages,
                db_pool,
            )
            .await?;

            (Some(joined_agendas), Some(protocols_with_items))
        }
        Section::EstimatedCommissionCorrespondence => {
            examine_plan_status(
                &plans,
                &[PlanStatus::EstimatedCommissionCorrespondence],
                PlanChangeFormMessage::InvalidPlanStatus,
                &mut messages,
            );
            let joined_protocols = examine_protocol_items(
                &plans,
                Some(ProtocolType::CorrespondenceMeeting),
                |protocol_item, plan| {
                    examine_protocol_correspondence(protocol_item, plan).into()
                },
                &mut messages,
                db_pool,
            )
            .await?;

            (None, Some(joined_protocols))
        }
        // Ничего не проверяется
        Section::EstimatedCommissionNotRequired => (None, None),
        invalid_section => {
            return Err(ProcessingError::Section(format!(
                "Секция {} невалидна для изменения формы СК",
                invalid_section
            )))
        }
    };

    Ok((plans, joined_agendas, joined_protocols, messages))
}

fn examine_protocol_correspondence(
    protocol_item: &JoinedEcProtocolItem,
    plan: &PlanOrAmendment,
) -> Message {
    let JoinedEcProtocolItem { protocol, .. } = protocol_item;

    let msg = match protocol.status_id {
        EcProtocolStatus::Formed | EcProtocolStatus::AgreementPending => {
            PlanChangeFormMessage::AlreadyInProtocolWarn(protocol)
        }
        _ => PlanChangeFormMessage::AlreadyInProtocolErr(protocol),
    };

    msg.singular(plan)
}

fn examine_protocol_in_person(
    protocol_item: &JoinedEcProtocolItem,
    plan: &PlanOrAmendment,
) -> Option<Message> {
    let JoinedEcProtocolItem { item, protocol } = protocol_item;

    match item.result_id {
        ResultId::NotAgreed => None,
        _ => PlanChangeFormMessage::InvalidProtocolResult(protocol, item)
            .singular(plan)
            .into(),
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

    PlanChangeFormMessage::AlreadyInAgenda(agenda).singular(plan).into()
}

async fn update_plans(
    mut plans: Vec<PlanOrAmendment>,
    commission_kind_id: CommissionKind,
    section_id: Section,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<Vec<PlanOrAmendment>> {
    let new_status_id = match commission_kind_id {
        CommissionKind::InPerson => PlanStatus::EstimatedCommissionInPerson,
        CommissionKind::Correspondence => {
            PlanStatus::EstimatedCommissionCorrespondence
        }
        CommissionKind::NotRequired => PlanStatus::EstimatedCommissionNo,
        CommissionKind::Undefined => unreachable!("Проверка была выше"),
    };

    let commission_date = if commission_kind_id == CommissionKind::InPerson
        && matches!(
            section_id,
            Section::EstimatedCommissionCorrespondence
                | Section::EstimatedCommissionNotRequired
        ) {
        AsezDate::today().with_next_weekday(time::Weekday::Friday).into()
    } else {
        None
    };

    plans.iter_mut().for_each(|p| {
        *p.status_id_mut() = new_status_id;
        *p.commission_kind_id_mut() = commission_kind_id;
        *p.commission_date_mut() = commission_date;
    });

    let updated_plans = PlanOrAmendment::update(
        plans,
        &[Plan::status_id, Plan::commission_kind_id, Plan::commission_date],
        messages,
        recorder,
        handler,
    )
    .await?;

    send_to_monolith(&updated_plans, recorder).await?;

    Ok(updated_plans)
}

/// По полученному списку ППЗ/ДС необходимо с помощью функции «Получение данных по Повестке» найти
/// данные по ППЗ/ДС, которые включены в крайнюю сформированную и не удаленную Повестку/is_removed = false,
/// где по ППЗ/ДС в позиции Повестки не указан признак «Удалена»/is_removed у данной позиции Повестки
/// по ППЗ/ДС нет связи с позицией Протокола в таблице item_relation_agenda_protocol.
async fn update_agenda_items(
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

/// Если ППЗ/ДС включена в Протокол (protocol_type_id = 2) с наивысшей датой создания,
/// который не удален/is_removed = false, то по позиции Протокола (тоже не удалена/is_removed = false)
/// к которой относится ППЗ/ДС установить признак снят с рассмотрения/is_excluded = true, если он ранее уже не был установлен.
async fn update_protocol_item(
    protocols_with_items: Vec<JoinedEcProtocolItem>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    common::protocol::update_protocol_items(
        protocols_with_items,
        |mut protocol_item| {
            protocol_item.item.is_excluded = true;
            Some(protocol_item.item)
        },
        &[EcProtocolItem::is_excluded],
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
) -> Result<ApiResponse<ReturnToCustomerResponseData, ()>> {
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

    Ok((data, messages).into())
}
