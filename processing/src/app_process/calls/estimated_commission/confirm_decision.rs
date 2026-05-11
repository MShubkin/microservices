use ahash::{AHashMap, AHashSet};
use itertools::Itertools;
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::db_item::{Filter, FilterTree, Select};
use shared_essential::{
    application::records::Recorder,
    domain::{
        tables::processing::protocol_item::ResultId, ContractAmendment,
        EcProtocolItem,
        JoinedEcProtocolItemPlanContractAmendmentSelector as ProtocolItemWithPlanSelector,
        Plan, PlanOrAmendment,
    },
    presentation::dto::{
        general::ObjectIdentifierWithStatusNote,
        processing::{ConfirmDecisionItem, ConfirmDecisionReq},
        response_request::{ApiResponse, BusinessMessage, Messages},
    },
};

use crate::{
    app_process::records::{
        send_to_monolith, PlanCollectedUpdate, ProcessingRulesChecker,
    },
    common::{ProcessingCtx, ProcessingError, Result},
    presentation::business_messages::protocol::ConfirmDecisionMessage,
};

use super::approve_protocol;
use rabbit_services::master_data::MasterDataService;

const CONFIRM_DECISION: &str = "/v1/action/confirm_decision/";

pub(crate) async fn confirm_decision(
    req: ConfirmDecisionReq,
    proc_ctx: ProcessingCtx,
    master_data_service: MasterDataService,
) -> Result<ApiResponse<(), ()>> {
    tracing::info!(
        kind = "update",
        "Получен запрос на Подтверждение решения СК в Протоколе очной СК ({get}): {req:?}\n",
        get = CONFIRM_DECISION,
        req = req,
    );

    let ConfirmDecisionReq {
        protocol_uuid,
        is_registered_by_d647,
        user_id,
        item_list,
        ..
    } = req;
    let mut messages = Messages::default();

    let protocol_items = fetch_protol_items(
        is_registered_by_d647,
        protocol_uuid,
        &item_list,
        &proc_ctx.db_pool,
    )
    .await?;

    let mut recorder = proc_ctx
        .create_record_context()
        .with_user_id(user_id)
        .with_status_notes(item_list.iter().map(|i| {
            ObjectIdentifierWithStatusNote::new(
                i.plan_id,
                i.source_uuid,
                i.status_note.clone(),
            )
        }))
        .begin()
        .await?;

    let updated_plans = update_plans(
        is_registered_by_d647,
        item_list,
        protocol_items,
        &mut messages,
        &mut recorder,
        proc_ctx.create_rules_checker(),
        &master_data_service,
    )
    .await?;

    recorder.commit().await?;

    ConfirmDecisionMessage::Success.checked_append(&mut messages, &updated_plans);

    Ok(ApiResponse::default().with_messages(messages))
}

/// При is_registered_by_d647 = true
/// Выполняется следующая логика: смотрим в ППЗ (для ДС не релевантно) на признак is_not_purchase/«не закупка» = true.
/// Если признак установлен, то устанавливаем статус ППЗ = Цена определена (не закупка)/160. Иначе устанавливаем статус 140/ППЗ утверждена.  
/// Если у нас ДС(contract_amendment) то устанавливаем статус 140/ДС утверждено.
///
/// При is_registered_by_d647 = false
/// result_id = 1/Утверждено, то смотрим в ППЗ (для ДС не релевантно) на признак is_not_purchase/«не закупка».
/// Если признак установлен, то устанавливаем статус ППЗ = Цена определена (не закупка)/160. Иначе устанавливаем статус 140/ППЗ утверждена.  
/// Если у нас ДС(contract_amendment) то устанавливаем статус 140/ДС утверждено.
/// - result_id = 2/Согласовано с корректировкой стоимости или 3/Не согласовано. Вернуть Эксперту.
/// Переводим ППЗ/ДС на статусы:
/// – Анализ цены Д646. Исполнитель назначен/222 если Департамент (организация) АЦ/pricing_organization_unit_id = 1/Д646
/// – Анализ цены Д647. Исполнитель назначен/342 если Департамент (организация) АЦ/pricing_organization_unit_id = 2/Д647
/// – Анализ цены МТР. Исполнитель назначен/352 если Департамент (организация) АЦ/pricing_organization_unit_id = 3/ГПК
/// При result_id = 3, необходимо очистить дату очной СК/commission_date и форму СК/commission_kind_id.
/// - result_id = 4/Аннулировать. Переводим ППЗ/ДС на статусы 150/ППЗ Аннулирована или 150/ДС Аннулировано и необходимо очистить дату очной СК/commission_date и форму СК/commission_kind_id .
///
/// Обновление данных такое же как и в approve_protocol
async fn update_plans(
    is_registered_by_d647: bool,
    items: Vec<ConfirmDecisionItem>,
    protocol_items: Vec<(EcProtocolItem, PlanOrAmendment)>,
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
    handler: ProcessingRulesChecker,
    master_data_service: &MasterDataService,
) -> Result<Vec<PlanOrAmendment>> {
    let cancel_reason_id = if cfg!(feature = "advanced-cancellation-control") {
        let has_cancelled_protocols =
            items.iter().any(|item| item.result_id == ResultId::Cancel);

        if has_cancelled_protocols {
            approve_protocol::get_auto_reason(master_data_service).await?
        } else {
            None
        }
    } else {
        None
    };

    #[allow(unused_mut)]
    let (to_update_plans, mut plan_fields, amendment_fields) =
        if is_registered_by_d647 {
            (
                protocol_items
                    .into_iter()
                    .map(|(_, plan)| {
                        approve_protocol::change_plan_status_correspondence(plan)
                    })
                    .collect::<Vec<_>>(),
                vec![Plan::status_id],
                vec![ContractAmendment::status_id],
            )
        } else {
            let result_id_checker = items
                .into_iter()
                .map(|i| (i.source_uuid, i.result_id))
                .collect::<AHashMap<_, _>>();

            (
            protocol_items
                .into_iter()
                .filter_map(|(protocol_item, plan)| {
                    let result_id = result_id_checker.get(plan.uuid()).expect("Выборка в fetch_protol_items гарантирует возвращение всех данных");

                    approve_protocol::change_plan_status_in_person(
                        plan,
                        false,
                        *result_id,
                        protocol_item.commission_sum_excluded_vat,
                        cancel_reason_id
                    )
                })
                .collect(),
            vec![Plan::status_id, Plan::commission_kind_id, Plan::commission_date],
            vec![ContractAmendment::status_id, ContractAmendment::commission_kind_id, ContractAmendment::commission_date],
        )
        };

    #[cfg(feature = "advanced-cancellation-control")]
    {
        use shared_essential::domain::legacy::plans::PlanStatus;
        let has_cancelled_plans = to_update_plans
            .iter()
            .any(|p| *p.status_id() == PlanStatus::PlanCancelled && p.is_plan());

        if has_cancelled_plans {
            plan_fields.push(Plan::reason_cancel_id);
        }
    }

    let updated_plans = PlanOrAmendment::update_different_fields(
        to_update_plans,
        &plan_fields,
        &amendment_fields,
        messages,
        recorder,
        handler,
    )
    .await?;

    send_to_monolith(&updated_plans, recorder).await?;

    Ok(updated_plans)
}

/// Пользователь может передать uuid элемента Протокола и source_uuid,
/// который в действительности не будет относиться к этому элементу Протокола.
/// Такая выборка (uuid=$1 AnD source_uuid=$1) проверяет валидность переданных данных.
async fn fetch_protol_items(
    is_registered_by_d647: bool,
    protocol_uuid: Uuid,
    item_list: &[ConfirmDecisionItem],
    db_pool: &PgPool,
) -> Result<Vec<(EcProtocolItem, PlanOrAmendment)>> {
    if item_list.is_empty() {
        return Err(ProcessingError::GetItemList(String::from(
            "Был передан пустой список ППЗ/ДС на подтверждение решения",
        )));
    }

    let item_filter_list = item_list.iter().map(|i| {
        FilterTree::and_from_list([
            FilterTree::filter(Filter::eq(EcProtocolItem::uuid, i.uuid)),
            FilterTree::filter(Filter::eq(
                EcProtocolItem::source_uuid,
                i.source_uuid,
            )),
        ])
    });
    let other_filter_tree = FilterTree::and_from_list([
        FilterTree::filter(Filter::eq(
            EcProtocolItem::is_registered_by_d647,
            is_registered_by_d647,
        )),
        FilterTree::filter(Filter::eq(
            EcProtocolItem::protocol_uuid,
            protocol_uuid,
        )),
    ]);
    let final_filter_tree = FilterTree::and_from_list([
        other_filter_tree,
        FilterTree::or_from_list(item_filter_list),
    ]);

    let protocol_item_select =
        Select::full::<EcProtocolItem>().set_filter_tree(final_filter_tree);
    let protocol_items = ProtocolItemWithPlanSelector::new(protocol_item_select)
        .get(db_pool)
        .await?
        .into_iter()
        .filter_map(|i| {
            PlanOrAmendment::from_either(i.plan, i.amendment).map(|p| (i.item, p))
        })
        .collect::<Vec<_>>();

    if item_list.len() != protocol_items.len() {
        let found_ids = protocol_items
            .iter()
            .map(|(_, plan)| *plan.id())
            .collect::<AHashSet<_>>();
        let missing = item_list
            .iter()
            .filter(|i| !found_ids.contains(&i.plan_id))
            .map(|i| i.plan_id.to_string())
            .join(", ");

        let msg =
            format!("Записи ППЗ/ДС c идентификаторами {} не найдены или не имеют смежного с ними элемента Протокола", missing);
        return Err(ProcessingError::GetItemList(msg));
    }

    Ok(protocol_items)
}
