//! Бизнес логика по ручкам "/rest/estimated_commission/v1/action/return_to_customer/".

use std::ops::Deref;
use std::sync::Arc;

use ahash::AHashMap;
use shared_essential::application::records::Recorder;
use shared_essential::presentation::dto::general::ObjectIdentifierWithStatusNote;
use sqlx::PgPool;
use uuid::Uuid;

use shared_essential::domain::tables::{
    legacy::plans::PlanStatus,
    JoinedEcAgendaItemEcAgendaRelAgendaProtocolItem as JoinedEcAgendaItem,
    JoinedEcProtocolItemEcProtocol as JoinedEcProtocolItem, *,
};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier, processing::*, response_request::*,
};

use crate::app_process::common::protocol::fetch_protocols_items;
use crate::app_process::records::{send_to_monolith, ProcessingRulesChecker};
use crate::app_process::sections::mapping::SectionMapExt;
use crate::app_process::{
    common::{
        self,
        agenda::fetch_agenda_items,
        plan::{examine_plan_status, fetch_plans_by_ids},
        protocol::examine_protocol_items,
    },
    records::PlanCollectedUpdate,
};
use crate::common::{ProcessingCtx, ProcessingError, Result};
use crate::presentation::business_messages::plan::PlanReturnToCustomerMessage;

const RETURN_TO_CUSTOMER: &str = "v1/action/return_to_customer/";
const PRE_RETURN_TO_CUSTOMER: &str = "v1/pre_request/return_to_customer/";

const PRE_REQUEST_RETURN_FIELDS: &[&str] = &[
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

struct UpdateKind {
    /// Нужно ли снять с рассмотрения элемент Повестки или Протокола
    to_exclude_item: Option<bool>,
    /// Нужно ли обновить commission поля
    update_commission: bool,
}

pub(crate) async fn return_to_customer(
    request: ReturnToCustomerReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ReturnToCustomerResponseData, ()>> {
    let db_pool = &*proc_ctx.db_pool;

    tracing::info!(
        kind = "get",
        "Получен запрос на возврат заказчику ({get}): {req:?}\n",
        get = RETURN_TO_CUSTOMER,
        req = request,
    );

    let ReturnToCustomerReq {
        section_id,
        action_type,
        is_force,
        item_list,
        user_id,
    } = request;

    // Многие обновления зависят от сторонних факторов, поэтому по ним
    // надо сохранять информацию
    //
    // По дефолту надо обновлять commission поля, но в некоторых случаях
    // их обновлять не надо
    let (identifier_list, mut update_checker): (Vec<_>, AHashMap<_, _>) = item_list
        .into_iter()
        .map(|i| {
            let oid_with_note = ObjectIdentifierWithStatusNote::new_with_type(
                i.id,
                i.uuid,
                i.object_type,
                i.status_note,
            );
            let kind = UpdateKind {
                to_exclude_item: i.is_excluded,
                update_commission: true,
            };
            (oid_with_note, (i.uuid, kind))
        })
        .unzip();

    let (plans, joined_protocols, mut messages) = pre_return_to_customer_inner(
        identifier_list.iter().map(Deref::deref),
        section_id,
        db_pool,
    )
    .await?;

    if messages.is_error() || (messages.kind == MessageKind::Warning && !is_force) {
        return finalise(plans, None, messages);
    }
    messages.clear();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(user_id)
        .with_status_notes(identifier_list)
        .begin()
        .await?;

    if section_id == Section::EstimatedCommissionInPerson {
        update_agenda_item(
            &plans,
            &mut update_checker,
            &mut messages,
            &mut recorder,
        )
        .await?;
    }

    match section_id {
        Section::EstimatedCommissionCorrespondence => {
            update_protocol_item(
                joined_protocols.expect("Для EstimatedCommissionCorrespondence гарантированно будут возвращены элементы"),
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

            // Если по ППЗ/ДС найден Протокол с решением комиссии/result_id = 3/Не согласовано. Вернуть Эксперту АЦ, в таблицах plan - ППЗ и contract_amendment - ДС необходимо очистить поля:
            // - «Дата очной СК»/commission_date
            // - признак «Очная СК»/commission_kind_id
            update_checker.iter_mut().for_each(|(source_uuid, check_item)| {
                if let Some(result_id) = protocol_item_checker.get(source_uuid) {
                    if !matches!(result_id, ResultId::NotAgreed) {
                        check_item.update_commission = false
                    }
                }
            })
        }
        _ => {}
    }

    let updated_plans = update_plans(
        plans,
        action_type,
        section_id,
        update_checker,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    recorder.commit().await?;

    PlanReturnToCustomerMessage::Success(action_type)
        .checked_append(&mut messages, &updated_plans);

    finalise(updated_plans, None, messages)
}

pub(crate) async fn pre_return_to_customer(
    request: PreReturnToCustomerReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PreReturnToCustomerResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Получен предзапрос на возврат заказчику ({get}): {req:?}\n",
        get = PRE_RETURN_TO_CUSTOMER,
        req = request,
    );

    let (plans, _, messages) = pre_return_to_customer_inner(
        &request.item_list,
        request.section_id,
        &db_pool,
    )
    .await?;

    finalise(plans, Some(PRE_REQUEST_RETURN_FIELDS), messages)
}

pub(crate) async fn pre_return_to_customer_inner<'a, I>(
    item_list: I,
    section_id: Section,
    db_pool: &PgPool,
) -> Result<(Vec<PlanOrAmendment>, Option<Vec<JoinedEcProtocolItem>>, Messages)>
where
    I: IntoIterator<Item = &'a ObjectIdentifier> + 'a,
{
    let plans = fetch_plans_by_ids(item_list, db_pool).await?;
    let mut messages = Messages::default();

    let joined_protocols = match section_id {
        Section::EstimatedCommissionInPerson
        | Section::EstimatedCommissionCorrespondence => {
            let (plan_status, protocol_type) = match section_id {
                Section::EstimatedCommissionInPerson => (
                    PlanStatus::EstimatedCommissionInPerson,
                    ProtocolType::InPersonMeeting,
                ),
                Section::EstimatedCommissionCorrespondence => (
                    PlanStatus::EstimatedCommissionCorrespondence,
                    ProtocolType::CorrespondenceMeeting,
                ),
                _ => unreachable!("Остальные варианты не могут попасть сюда"),
            };
            examine_plan_status(
                &plans,
                &[plan_status],
                PlanReturnToCustomerMessage::InvalidPlanStatus,
                &mut messages,
            );
            let joined_protocols = examine_protocol_items(
                &plans,
                Some(protocol_type),
                |protocol_item, plan| {
                    examine_protocol(section_id, protocol_item, plan)
                },
                &mut messages,
                db_pool,
            )
            .await?;

            Some(joined_protocols)
        }
        // Ничего не проверяется
        Section::EstimatedCommissionNotRequired => None,
        invalid_section => {
            return Err(ProcessingError::Section(format!(
                "Секция {} невалидна для возврата заказчику",
                invalid_section
            )))
        }
    };

    Ok((plans, joined_protocols, messages))
}

fn examine_protocol(
    section_id: Section,
    protocol_item: &JoinedEcProtocolItem,
    plan: &PlanOrAmendment,
) -> Option<Message> {
    let JoinedEcProtocolItem { item, protocol } = protocol_item;

    let msg = if section_id == Section::EstimatedCommissionCorrespondence {
        match protocol.status_id {
            EcProtocolStatus::Formed | EcProtocolStatus::AgreementPending => {
                PlanReturnToCustomerMessage::AlreadyInProtocolWarn(protocol).into()
            }
            _ => PlanReturnToCustomerMessage::AlreadyInProtocolErr(protocol).into(),
        }
    } else {
        match item.result_id {
            ResultId::NotAgreed => None,
            _ => PlanReturnToCustomerMessage::AlreadyInProtocolErr(protocol).into(),
        }
    };

    msg.map(|m| m.singular(plan))
}

#[allow(clippy::too_many_arguments)]
async fn update_plans(
    mut plans: Vec<PlanOrAmendment>,
    action_type: ActionType,
    section_id: Section,
    update_checker: AHashMap<Uuid, UpdateKind>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
) -> Result<Vec<PlanOrAmendment>> {
    let new_status = if action_type == ActionType::Revision {
        PlanStatus::ReturnToClientRework
    } else {
        PlanStatus::RequestClientDocumentation
    };

    let to_update_fields = match section_id {
        Section::EstimatedCommissionInPerson => {
            plans.iter_mut().for_each(|p| {
                *p.status_id_mut() = new_status;

                let check_item = update_checker
                    .get(p.uuid())
                    .expect("По всем ППЗ/ДС точно есть запись в update_checker");
                if check_item.update_commission {
                    *p.commission_date_mut() = None;
                    *p.commission_kind_id_mut() = CommissionKind::Undefined;
                }
            });

            vec![Plan::status_id, Plan::commission_date, Plan::commission_kind_id]
        }
        Section::EstimatedCommissionCorrespondence => {
            plans.iter_mut().for_each(|p| {
                *p.status_id_mut() = new_status;

                let check_item = update_checker
                    .get(p.uuid())
                    .expect("По всем ППЗ/ДС точно есть запись в update_checker");
                if check_item.update_commission {
                    *p.commission_kind_id_mut() = CommissionKind::Undefined;
                }
            });

            vec![Plan::status_id, Plan::commission_kind_id]
        }
        Section::EstimatedCommissionNotRequired => {
            plans.iter_mut().for_each(|p| {
                *p.status_id_mut() = new_status;
                *p.commission_kind_id_mut() = CommissionKind::Undefined;
            });

            vec![Plan::status_id, Plan::commission_kind_id]
        }
        _ => unreachable!("Проверка происходит в самом начале процесса"),
    };

    let updated_plans = PlanOrAmendment::update(
        plans,
        &to_update_fields,
        messages,
        recorder,
        handler,
    )
    .await?;

    send_to_monolith(&updated_plans, recorder).await?;

    Ok(updated_plans)
}

/// По полученному списку ППЗ/ДС необходимо с помощью функции «Получение данных по Повестке»
/// найти данные по ППЗ/ДС, которые включены в крайнюю сформированную и не удаленную Повестку/is_removed = false,
/// где по ППЗ/ДС в позиции Повестки не указан признак «Удалена»/is_removed у данной позиции Повестки по ППЗ/ДС нет связи
/// с позицией Протокола в таблице item_relation_agenda_protocol..
///
/// Если Повестка найдена и находится в статусе/status_id = 100/«Сформирована»,
/// то при помощи функции «Обновление Повестки» по позиции Повестки (по ППЗ/ДС)
/// автоматически устанавливается признак «Удалено»/is_remove = true
///
///Если Повестка найдена и находится в статусе/status_id = 200/«Отправлена»,
/// то если с FE по ППЗ/ДС пришел признак «Снято с рассмотрения»/is_excluded = true:
///- указать признак «Снято с рассмотрения»/is_excluded = true в таблице agenda_item;
///- очистить поля «Дата очной СК»/comission_date  "Форма СК"/commission_kind_id в таблицах plan - ППЗ и contract_amendment – ДС
///- очистить поле  reviewed_at/«Время проведения» в таблице agenda_item
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

/// Для заочной СК
///
/// Если ППЗ/ДС включена в Протокол (protocol_type_id = 2) с наивысшей датой создания,
/// который не удален/is_removed = false, то по позиции Протокола (тоже не удалена/is_removed = false)
/// к которой относится ППЗ/ДС установить признак снят с рассмотрения/is_excluded = true, если он ранее уже не был установлен.
async fn update_protocol_item(
    joined_protocols: Vec<JoinedEcProtocolItem>,
    update_checker: &mut AHashMap<Uuid, UpdateKind>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<()> {
    common::protocol::update_protocol_items(
        joined_protocols,
        |mut protocol_item| match update_checker
            .get_mut(&protocol_item.item.source_uuid)
        {
            Some(check_item) => {
                if check_item.to_exclude_item == Some(true) {
                    protocol_item.item.is_excluded = true;
                    Some(protocol_item.item)
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
