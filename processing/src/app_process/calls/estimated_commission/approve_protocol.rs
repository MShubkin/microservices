//! Бизнес логика по ручкам "/rest/estimated_commission/v1/(pre_request/action)/protocol_remove/".
use std::{ops::Deref, sync::Arc};

use sqlx::PgPool;

use asez2_shared_db::{
    db_item::{selection::*, AdaptorableIter, AsezTimestamp},
    DbItem,
};
use shared_essential::{
    domain::plan_reasons_cancel::CheckReason,
    domain::tables::{
        legacy::plans::PlanStatus,
        maths::CurrencyValue,
        processing::{
            plan::PricingUnitId,
            protocol_item::{
                JoinedEcProtocolItemEcProtocolPlanContractAmendment as JoinedProtocolItem,
                JoinedEcProtocolItemEcProtocolPlanContractAmendmentSelector as JoinedProtocolItemSelector,
                ResultId,
            },
        },
        *,
    },
    presentation::dto::{
        general::{ObjectIdentifier, ObjectIdentifierWithStatusNote},
        master_data::request::SearchPlanReasonsCancelRabbitReq,
        processing::{ApproveProtocolReq, PreApproveProtocolReq},
        response_request::*,
    },
};

use crate::{
    app_process::records::{send_to_monolith, PlanCollectedUpdate},
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::protocol::ProtocolApproveMessage,
};

use rabbit_services::master_data::MasterDataService;

#[derive(Debug, derive_more::Display)]
pub(crate) enum ApproveProtocol {
    #[display(fmt = "v1/pre_request/protocol_approve/")]
    PreRequest,
    #[display(fmt = "v1/action/protocol_approve/")]
    Action,
}

const FETCH_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    EcProtocol::id,
    EcProtocol::registration_number,
    EcProtocol::pricing_organization_unit_id,
    EcProtocol::status_id,
    EcProtocol::protocol_date,
];
const UPDATE_PROTOCOL_FIELDS: &[&str] =
    &[EcProtocol::status_id, EcProtocol::changed_at, EcProtocol::changed_by];
const RETURN_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    "protocol_id",
    EcProtocol::registration_number,
    "protocol_status_id",
    EcProtocol::protocol_date,
];

pub(crate) type ApproveProtocolResponse =
    ApiResponse<PaginatedData<EcProtocolRep>, ()>;

pub(crate) async fn approve_protocol(
    request: ApproveProtocolReq,
    proc_ctx: ProcessingCtx,
    master_data_service: MasterDataService,
) -> Result<ApproveProtocolResponse> {
    tracing::info!(
        kind = "get",
        "Запрос на утверждение Протокола СК ({what}): {req:?}\n",
        what = ApproveProtocol::Action,
        req = request,
    );

    if request.protocol_type_id == ProtocolType::Undefined {
        return Err(ProcessingError::ApproveProtocol(format!(
            "Тип Протокола СК {} является невалидным для утверждения Протокола СК",
            request.protocol_type_id
        )));
    }
    let item_ids =
        request.ids.iter().map(Deref::deref).cloned().collect::<Vec<_>>();

    let (protocols, mut messages) =
        approve_protocol_inner(&item_ids, &proc_ctx.db_pool).await?;

    if !messages.is_empty() {
        return Ok(ApiResponse::default().with_messages(messages));
    }
    messages.clear();

    let updated_protocols = approve_protocols_action(
        protocols,
        request.ids,
        request.protocol_type_id,
        request.user_id,
        &proc_ctx,
        &mut messages,
        &master_data_service,
    )
    .await?;

    match request.protocol_type_id {
        ProtocolType::InPersonMeeting => ProtocolApproveMessage::InPersonSuccess
            .checked_append(&mut messages, &updated_protocols),
        ProtocolType::CorrespondenceMeeting => {
            ProtocolApproveMessage::CorrespondenceSuccess
                .checked_append(&mut messages, &updated_protocols)
        }
        ProtocolType::Undefined => unreachable!("Проверено выше"),
    };

    finalise(messages, updated_protocols)
}

pub(crate) async fn pre_approve_protocol(
    request: PreApproveProtocolReq,
    db_pool: Arc<PgPool>,
) -> Result<ApproveProtocolResponse> {
    tracing::info!(
        kind = "get",
        "Предзапрос на утверждение Протокола СК ({what}): {req:?}\n",
        what = ApproveProtocol::PreRequest,
        req = request,
    );

    let (protocols, messages) = approve_protocol_inner(&request, &db_pool).await?;

    finalise(messages, protocols)
}

pub(crate) async fn approve_protocol_inner(
    ids: &[ObjectIdentifier],
    db_pool: &PgPool,
) -> Result<(Vec<EcProtocol>, Messages)> {
    let select = Select::with_fields(FETCH_FIELDS)
        .in_any(EcProtocol::uuid, ids.iter().map(|x| x.uuid));
    let protocols = EcProtocol::select(&select, db_pool).await?;
    let mut messages = Messages::default();

    examine_protocols(&protocols, &mut messages);

    Ok((protocols, messages))
}

async fn approve_protocols_action(
    mut protocols: Vec<EcProtocol>,
    ids: Vec<ObjectIdentifierWithStatusNote>,
    protocol_type_id: ProtocolType,
    user_id: i32,
    proc_ctx: &ProcessingCtx,
    messages: &mut Messages,
    master_data_service: &MasterDataService,
) -> Result<Vec<EcProtocol>> {
    let db_conn: &PgPool = &proc_ctx.db_pool;
    let now = AsezTimestamp::now();

    // 2. в Протоколе значение поля «Статус Протокола» меняется на 400/«Утвержден».
    protocols.iter_mut().for_each(|x| {
        x.status_id = EcProtocolStatus::Confirmed;
        x.changed_at = now;
        x.changed_by = user_id;
    });

    // 3. Записать историю изменения статуса Протокола в таблицу status_history.
    // делается автоматически

    // 4. По uuid Протокола необходимо найти позиции Протокола/protocol_item,
    // где protocol_item - is_removed = false и is_excluded = false.
    let select = Select::full_in::<_, EcProtocolItem>(
        EcProtocolItem::protocol_uuid,
        protocols.iter().map(|p| p.uuid.into()),
    )
    .eq(EcProtocolItem::is_removed, false)
    .eq(EcProtocolItem::is_excluded, false);
    let joined_protocol_items =
        JoinedProtocolItemSelector::new(select).get(db_conn).await?;

    // Проверяем, есть ли в протоколе отмененные планы
    // Даже если протокол утверждается, некоторые планы могут быть аннулированы
    // В этом случае автоматически подбираем причину аннулирования
    let cancel_reason_id = if cfg!(feature = "advanced-cancellation-control") {
        let has_cancelled_protocols = joined_protocol_items
            .iter()
            .any(|item| item.item.result_id == ResultId::Cancel);

        if has_cancelled_protocols {
            get_auto_reason(master_data_service).await?
        } else {
            None
        }
    } else {
        None
    };

    let to_update_plans = joined_protocol_items
        .into_iter()
        .filter_map(
            |JoinedProtocolItem { plan, amendment, protocol, item }| {
                if protocol.protocol_type_id != protocol_type_id {
                    tracing::warn!(kind = "update", "approve_protocol: protocol {} with type {} while action type is {}",
                                   protocol.uuid, protocol.protocol_type_id, protocol_type_id);
                }
                let plan_or_amendment =
                    PlanOrAmendment::from_either(plan, amendment)?;

                change_plan_status(
                    plan_or_amendment,
                    item,
                    protocol_type_id,
                    cancel_reason_id,
                )
            },
        )
        .collect::<Vec<_>>();

    let mut recorder = proc_ctx
        .create_record_context()
        .with_status_notes(ids)
        .with_user_id(user_id)
        .begin()
        .await?;

    let updated_protocols = recorder
        .process_update(protocols, UPDATE_PROTOCOL_FIELDS, messages)
        .await?;

    #[allow(unused_mut)]
    let mut plan_fields =
        vec![Plan::status_id, Plan::commission_date, Plan::commission_kind_id];

    #[cfg(feature = "advanced-cancellation-control")]
    if to_update_plans
        .iter()
        .any(|p| *p.status_id() == PlanStatus::PlanCancelled && p.is_plan())
    {
        plan_fields.push(Plan::reason_cancel_id);
    }

    let amendment_fields = vec![
        ContractAmendment::status_id,
        ContractAmendment::commission_date,
        ContractAmendment::commission_kind_id,
    ];

    let updated = PlanOrAmendment::update_different_fields(
        to_update_plans,
        &plan_fields,
        &amendment_fields,
        messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
    )
    .await?;

    send_to_monolith(&updated, &mut recorder).await?;

    recorder.commit().await?;

    Ok(updated_protocols)
}

/// 5. Проверить статус ППЗ/ДС
///
/// Если protocol_type_id = 1 c FE и статус ППЗ/ДС ≠ 251/Сметная комиссия. Очная
/// СК, то никаких изменений статусов по этой ППЗ/ДС не выполняем.
///
/// Если protocol_type_id = 2 c FE и статус ППЗ/ДС ≠ 252/Сметная комиссия. Очная
/// СК, то никаких изменений статусов по этой ППЗ/ДС не выполняем.
///
/// 6. Изменить статусы ППЗ/ДС в зависимости от типа протокола и
/// решений/result_id, принятых на очном заседании СК и записывается в историю
/// изменения статусов ППЗ/ДС;
///
/// Если protocol_type_id = 1 [...](status_result_change_in_person)
///
/// Если protocol_type_id = 2 [...](status_result_change_commission)
///
/// В случаях когда статус не надо обновлять будет возвращен [`Option::None`]
fn change_plan_status(
    plan_or_amendment: PlanOrAmendment,
    protocol_item: EcProtocolItem,
    protocol_type_id: ProtocolType,
    cancel_reason_id: Option<i32>,
) -> Option<PlanOrAmendment> {
    let status_id = plan_or_amendment.status_id();

    match (protocol_type_id, status_id) {
        (
            ProtocolType::InPersonMeeting,
            PlanStatus::EstimatedCommissionInPerson,
        ) => change_plan_status_in_person(
            plan_or_amendment,
            protocol_item.is_registered_by_d647,
            protocol_item.result_id,
            protocol_item.commission_sum_excluded_vat,
            cancel_reason_id,
        ),
        (
            ProtocolType::CorrespondenceMeeting,
            PlanStatus::EstimatedCommissionCorrespondence,
        ) => change_plan_status_correspondence(plan_or_amendment).into(),
        // Если protocol_type_id = 1 c FE и статус ППЗ/ДС ≠ 251/Сметная комиссия. Очная
        // СК, то никаких изменений статусов по этой ППЗ/ДС не выполняем.
        // или
        // Если protocol_type_id = 2 c FE и статус ППЗ/ДС ≠ 252/Сметная комиссия. Очная
        // СК, то никаких изменений статусов по этой ППЗ/ДС не выполняем.
        _ => None,
    }
}

/// ... смотрим в ППЗ (для ДС не релевантно) на признак «Для определения цены» =
/// true. Если признак установлен, то устанавливаем статус ППЗ = Цена определена
/// (не закупка)/160. Иначе устанавливаем статус 140/ППЗ утверждена или 140/ДС
/// утверждено.
pub(crate) fn change_plan_status_correspondence(
    mut plan_or_amendment: PlanOrAmendment,
) -> PlanOrAmendment {
    use PlanStatus::*;

    *plan_or_amendment.status_id_mut() = match &plan_or_amendment {
        PlanOrAmendment::Plan(p) => {
            if p.is_not_purchase {
                PriceDetermined
            } else {
                PriceConfirmed
            }
        }
        PlanOrAmendment::Amendment(_) => PriceConfirmed,
    };

    plan_or_amendment
}

/// - result_id = 1/Утверждено, то смотрим в ППЗ (для ДС не релевантно) на
///   признак «Для определения цены» = true. Если признак установлен, то
///   устанавливаем статус ППЗ = Цена определена (не закупка)/160. Иначе
///   устанавливаем статус 140/ППЗ утверждена или 140/ДС утверждено.
///
/// - result_id = 2/Согласовано с корректировкой стоимости при условии что:
///   Стоимость СК (без НДС) commission_sum_excluded_vat из позиции Протокола == Текущей стоимости АЦ (без НДС) actual_sum_excluded_vat из ППЗ/ДС
///   И
///   is_registered_by_d647 == false
///   То выполняем алгоритм соответсвующий Решениe СК result_id = 1/"Утверждено"
///
/// - result_id = 2/Согласовано с корректировкой стоимости
///   или 3/Не согласовано. Вернуть Эксперту.
///    Переводим ППЗ/ДС на статусы:
///    - Анализ цены Д646. Исполнитель назначен/222 если Департамент
///      (организация) АЦ/pricing_organization_unit_id = 1/Д646
///    - Анализ цены Д647. Исполнитель назначен/342 если Департамент
///    (организация) АЦ/pricing_organization_unit_id = 2/Д647
///    - Анализ цены МТР. Исполнитель назначен/352 если Департамент
///      (организация) АЦ/pricing_organization_unit_id = 3/ГПК
///
///    При result_id = 3, необходимо очистить дату очной СК/commission_date и
///    форму СК/commission_kind_id.
///
/// - result_id = 4/Аннулировать. Переводим ППЗ/ДС на статусы 150/ППЗ
///   Аннулирована или 150/ДС Аннулировано и необходимо очистить дату очной
///   СК/commission_date и форму СК/commission_kind_id .
pub(crate) fn change_plan_status_in_person(
    mut plan_or_amendment: PlanOrAmendment,
    is_registered_by_d647: bool,
    result: ResultId,
    commission_sum_excluded_vat: Option<CurrencyValue>,
    cancel_reason_id: Option<i32>,
) -> Option<PlanOrAmendment> {
    if matches!(result, ResultId::NotAgreed | ResultId::Cancel) {
        *plan_or_amendment.commission_date_mut() = None;
        *plan_or_amendment.commission_kind_id_mut() = CommissionKind::Undefined;
    }

    let actual_sum_excluded_vat = match &plan_or_amendment {
        PlanOrAmendment::Plan(p) => p.pricing_sum_excluded_vat.into(),
        PlanOrAmendment::Amendment(a) => a.pricing_delta_sum_excluded_vat,
    };
    let is_actualized_sum = actual_sum_excluded_vat == commission_sum_excluded_vat;

    match (result, is_registered_by_d647) {
        (ResultId::Approved, _) | (_, true) => {
            change_plan_status_correspondence(plan_or_amendment).into()
        }
        (ResultId::AgreedWithPriceCorrection, false) if is_actualized_sum => {
            change_plan_status_correspondence(plan_or_amendment).into()
        }
        (ResultId::AgreedWithPriceCorrection | ResultId::NotAgreed, _) => {
            let new_status = match *plan_or_amendment.pricing_organization_unit_id()
            {
                PricingUnitId::D646 => PlanStatus::ExecutorAppointedD646,
                PricingUnitId::D647 => PlanStatus::ExecutorAppointedD647,
                PricingUnitId::Gpk => PlanStatus::ExecutorAppointedMTP,
                dept_id => {
                    tracing::warn!(kind = "update", "По ППЗ/ДС {} не указан департамент, невозможно установить новый статус {dept_id:?}", plan_or_amendment.id());
                    return None;
                }
            };

            *plan_or_amendment.status_id_mut() = new_status;

            Some(plan_or_amendment)
        }
        (ResultId::Cancel, _) => {
            *plan_or_amendment.status_id_mut() = PlanStatus::PlanCancelled;

            if let Some(reason_id) = cancel_reason_id {
                if let PlanOrAmendment::Plan(p) = &mut plan_or_amendment {
                    p.reason_cancel_id = Some(reason_id);
                }
            }

            Some(plan_or_amendment)
        }
        _ => None,
    }
}

pub(crate) async fn get_auto_reason(
    master_data_service: &MasterDataService,
) -> Result<Option<i32>> {
    let reason_response = master_data_service
        .plan_reasons_cancel_search(SearchPlanReasonsCancelRabbitReq {
            ids: None,
            check_reason_id: Some(CheckReason::Protocol.into()),
        })
        .await
        .map_err(|e| {
            ProcessingError::InternalError(format!(
                "Не удалось получить причину аннулирования: {:?}",
                e
            ))
        })?;

    Ok(reason_response.data.iter().find_map(|reason| reason.header.id))
}

/// - Если в Протоколе указан статус/status_id ≠ 300/На подписании, то формируем ошибку:
///
/// «Перевести Протокол <Системный номер Протокола> на статус "Утвержден" невозможно.
/// Текущий статус Протокола "…"».
fn examine_protocols(protocols: &[EcProtocol], messages: &mut Messages) {
    use EcProtocolStatus::*;

    protocols
        .iter()
        .filter(|p| !matches!(p.status_id, Formed | SignaturePending))
        .for_each(|p| {
            messages.add_prepared_message(
                ProtocolApproveMessage::InvalidProtocolStatus.singular(p),
            );
        });
}

fn finalise(
    messages: Messages,
    data: Vec<EcProtocol>,
) -> Result<ApproveProtocolResponse> {
    let data = if messages.is_error() {
        Vec::new()
    } else {
        data.into_iter().adaptors_with_fields(RETURN_FIELDS).collect()
    };

    Ok((data, messages).into())
}
