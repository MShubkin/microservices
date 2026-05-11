//! Бизнес логика по ручкам "/rest/estimated_commission/v1/(action/pre_request)/return_to_expert/".
use std::sync::Arc;

use ahash::AHashMap;
use sqlx::PgPool;
use uuid::Uuid;

use shared_essential::{
    application::records::Recorder,
    domain::tables::{
        legacy::plans::PlanStatus, CommissionKind, ContractAmendment, EcAgendaItem,
        EcAgendaStatus, EcProtocolItem, EcProtocolStatus,
        JoinedEcAgendaItemEcAgendaRelAgendaProtocolItem as JoinedEcAgendaItem,
        JoinedEcProtocolItemEcProtocol as JoinedEcProtocolItem, Plan,
        PlanOrAmendment, PlanOrAmendmentRep, PricingUnitId, ProtocolType, ResultId,
        Section, SectionKind,
    },
    presentation::dto::{
        general::ObjectIdentifier, processing::*, response_request::*,
    },
};

use crate::{
    app_process::{
        common::{
            self,
            agenda::fetch_agenda_items,
            plan::{examine_plan_status, fetch_plans_by_ids},
            protocol::{examine_protocol_items, fetch_protocols_items},
        },
        records::{send_to_monolith, PlanCollectedUpdate, ProcessingRulesChecker},
        sections::mapping::SectionMapExt,
    },
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::plan::PlanReturnToExpertMessage,
};

const PRE_RETURN_TO_EXPERT: &str = "v1/pre_request/return_to_expert/";
const RETURN_TO_EXPERT: &str = "v1/action/return_to_expert/";

const RESPONSE_FIELD_LIST: &[&str] = &[
    Plan::uuid,
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
    Plan::number_customer,
    "plan_id",
];

struct UpdateKind {
    /// Нужно ли снять с рассмотрения элемент Повестки или Протокола
    to_exclude_item: Option<bool>,
    /// Нужно ли обновить commission поля
    update_commission: bool,
}

pub(crate) async fn pre_return_to_expert(
    request: PreReturnToExpertReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreReturnToExpertResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Получен предзапрос на возврат эксперта ({get}): {req:?}\n",
        get = PRE_RETURN_TO_EXPERT,
        req = request,
    );

    let (plans, _, messages) = pre_return_to_expert_inner(
        &request.item_list,
        request.section_id,
        &db_pool,
    )
    .await?;

    finalise(plans, Some(RESPONSE_FIELD_LIST), messages)
}

pub(crate) async fn return_to_expert(
    request: ReturnToExpertReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ReturnToExpertResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Получен запрос на возврат эксперту ({get}): {req:?}\n",
        get = RETURN_TO_EXPERT,
        req = request,
    );

    let identifier_list = request
        .item_list
        .iter()
        .map(|i| ObjectIdentifier::new_with_type(i.id, i.uuid, i.object_type))
        .collect::<Vec<_>>();

    let (plans, maybe_protocol_items, mut messages) = pre_return_to_expert_inner(
        &identifier_list,
        request.section_id,
        &proc_ctx.db_pool,
    )
    .await?;

    if messages.is_error() || (messages.is_warn() && !request.is_force) {
        return finalise(plans, Some(RESPONSE_FIELD_LIST), messages);
    }
    messages.clear();

    // Многие обновления зависят от сторонних факторов, поэтому по ним
    // надо сохранять информацию
    //
    // По дефолту надо обновлять commission поля, но в некоторых случаях
    // их обновлять не надо
    let mut update_checker: AHashMap<Uuid, UpdateKind> = request
        .item_list
        .iter()
        .map(|x| {
            let kind = UpdateKind {
                to_exclude_item: x.is_excluded,
                update_commission: true,
            };
            (x.uuid, kind)
        })
        .collect();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(request.user_id)
        .with_status_notes(request.item_list.iter().cloned().map(Into::into))
        .begin()
        .await?;

    match request.section_id {
        Section::EstimatedCommissionCorrespondence => {
            update_protocol_item(
                maybe_protocol_items
                    .expect("Заочная СК гарантированно вернет Протоколы"),
                &mut update_checker,
                &mut messages,
                &mut recorder,
            )
            .await?;
        }
        Section::EstimatedCommissionInPerson => {
            let protocol_item_checker =
                fetch_protocols_items(&plans, None, recorder.tx())
                    .await?
                    .into_iter()
                    .map(|i| (i.item.source_uuid, i.item.result_id))
                    .collect::<AHashMap<_, _>>();

            //Проверить решение комиссии/result_id в позиции Протокола по ППЗ/ДС. Если указано решение result_id:
            //- 3/Не согласовано. Вернуть Эксперту АЦ,
            //- 4/Аннулировать, то в модуле АЦ
            //То в таблицах plan - ППЗ и contract_amendment – ДС очистить поля
            //- «Дата очной СК»/comission_date
            //- "Форма СК"/commission_kind_id
            update_checker.iter_mut().for_each(|(source_uuid, check_item)| {
                if let Some(result_id) = protocol_item_checker.get(source_uuid) {
                    if !matches!(result_id, ResultId::Cancel | ResultId::NotAgreed)
                    {
                        check_item.update_commission = false
                    }
                }
            })
        }
        _ => {}
    };

    if request.section_id == Section::EstimatedCommissionInPerson {
        update_agenda_item(
            &plans,
            &mut update_checker,
            &mut messages,
            &mut recorder,
        )
        .await?;
    };

    let updated_plans = update_plan(
        plans,
        update_checker,
        request.section_id,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    recorder.commit().await?;

    PlanReturnToExpertMessage::Success
        .checked_append(&mut messages, &updated_plans);

    finalise(updated_plans, Some(RESPONSE_FIELD_LIST), messages)
}

/// Гарантирует, что при section_id = [`Section::EstimatedCommissionCorrespondence`] будут возращены
///  Vec<JoinedEcProtocolItem>
async fn pre_return_to_expert_inner(
    item_list: &[ObjectIdentifier],
    section_id: Section,
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Option<Vec<JoinedEcProtocolItem>>, Messages)> {
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;
    let mut messages = Messages::default();

    let protocol_items = match section_id {
        Section::EstimatedCommissionInPerson => {
            examine_plan_status(
                &plans,
                &[PlanStatus::EstimatedCommissionInPerson],
                PlanReturnToExpertMessage::InvalidPlanStatus,
                &mut messages,
            );
            None
        }
        Section::EstimatedCommissionCorrespondence => {
            examine_plan_status(
                &plans,
                &[PlanStatus::EstimatedCommissionCorrespondence],
                PlanReturnToExpertMessage::InvalidPlanStatus,
                &mut messages,
            );
            let protocol_items = examine_protocol_items(
                &plans,
                Some(ProtocolType::CorrespondenceMeeting),
                |protocol_item, plan| examine_protocol(protocol_item, plan).into(),
                &mut messages,
                db_pool,
            )
            .await?;
            Some(protocol_items)
        }
        // Ничего не проверяется
        Section::EstimatedCommissionNotRequired => None,
        invalid_section => {
            return Err(ProcessingError::Section(format!(
                "Секция {} невалидна для возврата эксперту",
                invalid_section
            )))
        }
    };

    Ok((plans, protocol_items, messages))
}

fn examine_protocol(
    protocol_item: &JoinedEcProtocolItem,
    plan: &PlanOrAmendment,
) -> Message {
    let msg = match protocol_item.protocol.status_id {
        EcProtocolStatus::Formed | EcProtocolStatus::AgreementPending => {
            PlanReturnToExpertMessage::AlreadyInProtocolWarn(
                &protocol_item.protocol,
            )
        }
        _ => {
            PlanReturnToExpertMessage::AlreadyInProtocolErr(&protocol_item.protocol)
        }
    };

    msg.singular(plan)
}

/// Обновление статусов и других полей у ППЗ/ДС
async fn update_plan(
    plans: Vec<PlanOrAmendment>,
    update_checker: AHashMap<Uuid, UpdateKind>,
    section_id: Section,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<Vec<PlanOrAmendment>> {
    let plan_status_transition = |plan: &PlanOrAmendment| -> Option<PlanStatus> {
        match *plan.pricing_organization_unit_id() {
            PricingUnitId::D645 => Some(PlanStatus::ExecutorAppointedD645),
            PricingUnitId::D646 => Some(PlanStatus::ExecutorAppointedD646),
            PricingUnitId::D647 => Some(PlanStatus::ExecutorAppointedD647),
            PricingUnitId::Gpk => Some(PlanStatus::ExecutorAppointedMTP),
            _ => None,
        }
    };

    let (to_update_plans, to_update_fields) = match section_id {
        Section::EstimatedCommissionInPerson => {
            let to_update_plans = plans
                .into_iter()
                .filter_map(|mut plan| {
                    let new_status = plan_status_transition(&plan)?;

                    *plan.status_id_mut() = new_status;

                    let check_item = update_checker.get(plan.uuid()).expect(
                        "По всем ППЗ/ДС точно есть запись в update_checker",
                    );
                    if check_item.update_commission {
                        *plan.commission_date_mut() = None;
                        *plan.commission_kind_id_mut() = CommissionKind::Undefined;
                    }

                    Some(plan)
                })
                .collect();
            (
                to_update_plans,
                vec![
                    Plan::status_id,
                    Plan::commission_date,
                    Plan::commission_kind_id,
                ],
            )
        }
        Section::EstimatedCommissionCorrespondence => {
            let to_update_plans = plans
                .into_iter()
                .filter_map(|mut plan| {
                    let new_status = plan_status_transition(&plan)?;

                    *plan.status_id_mut() = new_status;

                    let check_item = update_checker.get(plan.uuid()).expect(
                        "По всем ППЗ/ДС точно есть запись в update_checker",
                    );
                    if check_item.update_commission {
                        *plan.commission_kind_id_mut() = CommissionKind::Undefined;
                    }

                    Some(plan)
                })
                .collect();

            (to_update_plans, vec![Plan::status_id, Plan::commission_kind_id])
        }
        Section::EstimatedCommissionNotRequired => {
            let to_update_plans = plans
                .into_iter()
                .filter_map(|mut plan| {
                    let new_status = plan_status_transition(&plan)?;

                    *plan.status_id_mut() = new_status;
                    *plan.commission_kind_id_mut() = CommissionKind::Undefined;

                    Some(plan)
                })
                .collect();

            (to_update_plans, vec![Plan::status_id, Plan::commission_kind_id])
        }
        _ => unreachable!("Проверено выше"),
    };

    let updated_data = PlanOrAmendment::update(
        to_update_plans,
        &to_update_fields,
        messages,
        recorder,
        handler,
    )
    .await?;

    send_to_monolith(&updated_data, recorder).await?;

    Ok(updated_data)
}

/// При очной СК
///
/// Если Протокол не найден, то по полученному списку ППЗ/ДС необходимо найти данные по ППЗ/ДС,
/// которые включены в крайнюю сформированную и не удаленную/is_removed = false Повестку,
/// где по ППЗ/ДС в Повестке не указан признак «Удалена»/is_removed = false, у данной позиции
/// Повестки по ППЗ/ДС нет связи с позицией Протокола в таблице item_relation_agenda_protocol.
///
/// Если статус Повестки СК 100
/// - указать признак «Удалено»/is_removed = true в таблице agenda_item;
/// - очистить поля «Дата очной СК»/comission_date  "Форма СК"/commission_kind_id в таблицах plan - ППЗ и contract_amendment – ДС
///
/// Если статус Повестки СК 200
/// Если с FE по ППЗ/ДС пришел признак «Снято с рассмотрения»/is_excluded = true:
/// - указать признак «Снято с рассмотрения»/is_excluded = true в таблице agenda_item;
/// - очистить поля «Дата очной СК»/comission_date  "Форма СК"/commission_kind_id в таблицах plan - ППЗ и contract_amendment – ДС
/// - очистить поле  reviewed_at/ «Время проведения» в таблице agenda_item
async fn update_agenda_item(
    plans: &[PlanOrAmendment],
    update_checker: &mut AHashMap<Uuid, UpdateKind>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    let agenda_with_items =
        fetch_agenda_items(plans, Some(false), None, recorder.tx()).await?;

    common::agenda::update_agenda_items(
        agenda_with_items,
        |agenda_item| {
            let JoinedEcAgendaItem {
                mut agenda_item,
                agenda,
                item_agenda_protocol_rel,
            } = agenda_item;

            let check_item = update_checker.get_mut(&agenda_item.source_uuid)?;
            if item_agenda_protocol_rel.is_some() {
                return None;
            }

            match agenda.status_id {
                EcAgendaStatus::Formed => {
                    agenda_item.is_removed = true;
                    Some(agenda_item)
                }
                EcAgendaStatus::Sent => {
                    // Поля комиссии обновляются только если пользователь по
                    // ППЗ/ДС передал is_excluded=true признак
                    match check_item.to_exclude_item {
                        Some(true) => {
                            agenda_item.is_excluded = true;
                            agenda_item.reviewed_at = None;
                            Some(agenda_item)
                        }
                        Some(false) => {
                            // Для ППЗ/ДС не имеющих признака снято с рассмотрения в Повестке (agenda_item.is_excluded = false):
                            // поля «Дата очной СК»/comission_date "Форма СК"/commission_kind_id в таблицах plan - ППЗ и contract_amendment – ДС оставить без изменений.
                            if !agenda_item.is_excluded {
                                check_item.update_commission = false;
                            }
                            None
                        }
                        None => {
                            check_item.update_commission = false;
                            None
                        }
                    }
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

/// При заочной СК
///
/// Если ППЗ/ДС включена в Протокол (protocol_type_id = 2) с наивысшей датой создания,
/// который не удален/is_removed = false, то по позиции Протокола (тоже не удалена/is_removed = false)
/// к которой относится ППЗ/ДС установить признак снят с рассмотрения/is_excluded = true,
/// если он ранее уже не был установлен.
async fn update_protocol_item(
    protocols_with_items: Vec<JoinedEcProtocolItem>,
    update_checker: &mut AHashMap<Uuid, UpdateKind>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    common::protocol::update_protocol_items(
        protocols_with_items,
        |mut p| match update_checker.get_mut(&p.item.source_uuid) {
            Some(check_item) => {
                if check_item.to_exclude_item == Some(true) {
                    p.item.is_excluded = true;
                    Some(p.item)
                } else {
                    check_item.update_commission = false;

                    None
                }
            }
            _ => None,
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
) -> Result<ApiResponse<ReturnToExpertResponseData, ()>> {
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
