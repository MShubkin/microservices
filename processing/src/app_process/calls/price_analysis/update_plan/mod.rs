//! Работа с вычислимыми полями ППЗ/ДС
//!
//! При обновлении ППЗ/ДС некоторые поля вычисляются на основании других.
//! При этом в DTO приходит только часть полей, остальные надо извлекать из
//! базы. Может так же быть ситуация, когда не приходит позиция. В этом случае ее
//! так же надо получить из базы.
//!
//! Алгоритм обновления заголовка и позиций:
//!
//! 1. По позициям из uuid заголовка из DTO мы получаем исходные позиции,
//! со всеми полями, требуемыми для исчисляемых полей. Для этого надо:
//!
//! - Прочитать из базы все позиции ППЗ, со следующим набором полей:
//!  - поля DTO
//!  - поля, необходимые для вычислений
//! - Поместить элементы в мэп по uuid
//! - Для всех позиций из DTO установить отсутствующие поля
//!   в значения, взятые из базы
//! - Преобразовать элементы DTO в DbItem
//! - Добавить недостающие позиции из БД
//!
//! 2. Вычислить поля позиций
//!
//! 3. Вычислить поля заголовка
//!
//! 4. Записать позиции в базу, используя следующий список полей:
//!  - поля, приходящие в DTO
//!  - исчесляемые поля
//!
//! 5. Записать заголовок в базу, исполюзуя следующий список полей:
//!  - поля, приходящие в DTO
//!  - исчесляемые поля
//!

use crate::app_process::common as utils;

use crate::common::{ProcessingCtx, Result};

use ahash::AHashMap;
use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::{make_bind_mask, AsezDate, Select};
use asez2_shared_db::{DbAdaptor, DbItem};
use shared_essential::{
    application::commission::is_commission_date_possible,
    domain::{maths::*, *},
    presentation::dto::{
        processing::price_analysis::*, processing::UserIdWrapper,
        response_request::*,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

const UPDATE_PLANS: &str = "price_analysis/v1/action/update_plan";
const UPDATE_CONTRACT_AMENDMENTS: &str =
    "price_analysis/v1/action/update_contract_amendment";

/// Поля заголовка, получаемые с FE.
pub(super) const PLAN_DTO_FIELDS: &[&str] = &[
    Plan::uuid,
    Plan::id,
    Plan::pricing_expert_id,
    Plan::pricing_method_id,
    Plan::expert_conclusion_id,
    Plan::pricing_resume,
    Plan::commission_kind_id,
    Plan::commission_date,
    Plan::savings_accounting_id,
    Plan::savings_sum_excluded_vat,
    Plan::savings_sum_included_vat,
    Plan::organizer_id,
    Plan::is_cooperative,
    Plan::is_list_price,
];

/// Поля, получаемые из БД.
const PLAN_DB_FIELDS: &[&str] = &[Plan::currency_id, Plan::currency_rate];

/// Вычислимые поля ППЗ.
pub(super) const PLAN_CALCULATED_FIELDS: &[&str] = &[
    Plan::savings_sum_excluded_vat_rub,
    Plan::savings_sum_included_vat_rub,
    //
    Plan::pricing_vat_id,
    Plan::pricing_sum_excluded_vat,
    Plan::pricing_sum_excluded_vat_rub,
    Plan::pricing_sum_included_vat,
    Plan::pricing_sum_included_vat_rub,
    Plan::pricing_sum_vat,
    Plan::pricing_sum_vat_rub,
    //
    Plan::pricing_transportation_vat_id,
    Plan::pricing_transportation_price,
    Plan::pricing_transportation_price_rub,
    Plan::pricing_transportation_sum_vat,
    Plan::pricing_transportation_sum_vat_rub,
    Plan::pricing_transportation_sum_included_vat,
    Plan::pricing_transportation_sum_included_vat_rub,
    //
    Plan::pricing_total_sum,
    Plan::pricing_total_sum_rub,
    //
    Plan::pricing_currency_id,
    Plan::pricing_currency_rate,
];

/// Поля, присылаемые в запросе.
const PLAN_ITEM_DTO_FIELDS: &[&str] = &[
    PlanItemFull::uuid,
    PlanItemFull::pricing_quantity,
    PlanItemFull::pricing_price,
    PlanItemFull::pricing_vat_id,
];

/// Поля, получаемые "в основном" из БД.
///
/// NB. Мы также загружаем из БД и поля DTO, для корректного обновления полей заголовка
/// при неполном списке позиций.
///
/// Некоторые из этих полей упоминаются также в DTO,
/// но не должны меняться на FE и приходят к нам "на всякий случай".
const PLAN_ITEM_DB_FIELDS: &[&str] = &[
    PlanItemFull::currency_id,
    PlanItemFull::currency_rate,
    PlanItemFull::currency_rate_date,
    PlanItemFull::unit_id,
];

/// Вычислимые поля позиций ППЗ.
const PLAN_ITEM_CALCULATED_FIELDS: &[&str] = &[
    PlanItemFull::pricing_unit_id,
    PlanItemFull::pricing_price_rub,
    PlanItemFull::pricing_currency_id,
    PlanItemFull::pricing_currency_rate,
    PlanItemFull::pricing_currency_rate_date,
    //
    PlanItemFull::pricing_sum_excluded_vat,
    PlanItemFull::pricing_sum_excluded_vat_rub,
    PlanItemFull::pricing_sum_included_vat,
    PlanItemFull::pricing_sum_included_vat_rub,
    PlanItemFull::pricing_sum_vat,
    PlanItemFull::pricing_sum_vat_rub,
    //
    PlanItemFull::pricing_transportation_vat_id,
    PlanItemFull::pricing_transportation_price,
    PlanItemFull::pricing_transportation_price_rub,
    PlanItemFull::pricing_transportation_sum_vat,
    PlanItemFull::pricing_transportation_sum_vat_rub,
    PlanItemFull::pricing_transportation_sum_included_vat,
    PlanItemFull::pricing_transportation_sum_included_vat_rub,
    //
    PlanItemFull::pricing_total_sum,
    PlanItemFull::pricing_total_sum_rub,
];

/// Поля ДС, получаемые с FE.
pub(super) const CONTRACT_AMENDMENT_DTO_FIELDS: &[&str] = &[
    ContractAmendment::uuid,
    ContractAmendment::id,
    ContractAmendment::pricing_expert_id,
    ContractAmendment::pricing_method_id,
    ContractAmendment::expert_conclusion_id,
    ContractAmendment::pricing_resume,
    ContractAmendment::commission_kind_id,
    ContractAmendment::commission_date,
    ContractAmendment::savings_accounting_id,
    ContractAmendment::savings_sum_excluded_vat,
    ContractAmendment::savings_sum_included_vat,
];

/// Поля ДС, получаемые из БД.
const CONTRACT_AMENDMENT_DB_FIELDS: &[&str] =
    &[ContractAmendment::currency_id, ContractAmendment::currency_rate];

/// Вычислимые поля ДС.
pub(super) const CONTRACT_AMENDMENT_CALCULATED_FIELDS: &[&str] = &[
    //
    ContractAmendment::savings_sum_excluded_vat_rub,
    ContractAmendment::savings_sum_included_vat_rub,
    //
    ContractAmendment::pricing_vat_id,
    ContractAmendment::pricing_sum_excluded_vat,
    ContractAmendment::pricing_sum_excluded_vat_rub,
    ContractAmendment::pricing_sum_included_vat,
    ContractAmendment::pricing_sum_included_vat_rub,
    ContractAmendment::pricing_sum_vat,
    ContractAmendment::pricing_sum_vat_rub,
    //
    ContractAmendment::pricing_transportation_vat_id,
    ContractAmendment::pricing_transportation_price,
    ContractAmendment::pricing_transportation_price_rub,
    ContractAmendment::pricing_transportation_sum_vat,
    ContractAmendment::pricing_transportation_sum_vat_rub,
    ContractAmendment::pricing_transportation_sum_included_vat,
    ContractAmendment::pricing_transportation_sum_included_vat_rub,
    //
    ContractAmendment::pricing_total_sum,
    ContractAmendment::pricing_total_sum_rub,
    //
    ContractAmendment::pricing_currency_id,
    ContractAmendment::pricing_currency_rate,
    //
    ContractAmendment::pricing_delta_sum_excluded_vat,
    ContractAmendment::pricing_delta_sum_excluded_vat_rub,
    ContractAmendment::pricing_delta_sum_included_vat,
    ContractAmendment::pricing_delta_sum_included_vat_rub,
    ContractAmendment::pricing_delta_sum_vat,
    ContractAmendment::pricing_delta_sum_vat_rub,
    //
    ContractAmendment::pricing_delta_transportation_price,
    ContractAmendment::pricing_delta_transportation_sum_included_vat,
    ContractAmendment::pricing_delta_transportation_sum_included_vat_rub,
    ContractAmendment::pricing_delta_transportation_sum_vat,
    ContractAmendment::pricing_delta_transportation_sum_vat_rub,
    //
    ContractAmendment::pricing_delta_total_sum,
    ContractAmendment::pricing_delta_total_sum_rub,
];

/// Поля позиций ДС, приходящие в запросе.
const CONTRACT_AMENDMENT_ITEM_DTO_FIELDS: &[&str] = &[
    ContractAmendmentItem::uuid,
    ContractAmendmentItem::pricing_quantity,
    ContractAmendmentItem::pricing_price,
    ContractAmendmentItem::pricing_vat_id,
];

/// Поля, получаемые "в основном" из БД.
///
/// NB. Мы также загружаем из БД и поля DTO, для корректного обновления полей заголовка
/// при неполном списке позиций.
///
/// Некоторые из этих полей упоминаются также в DTO,
/// но не должны меняться на FE и приходят к нам "на всякий случай".
const CONTRACT_AMENDMENT_ITEM_DB_FIELDS: &[&str] = &[
    ContractAmendmentItem::unit_id,
    ContractAmendmentItem::currency_id,
    ContractAmendmentItem::currency_rate,
    ContractAmendmentItem::currency_rate_date,
    ContractAmendmentItem::previous_quantity,
    ContractAmendmentItem::previous_price,
    ContractAmendmentItem::previous_sum_excluded_vat,
    ContractAmendmentItem::previous_sum_included_vat,
    ContractAmendmentItem::previous_sum_vat,
];

/// Вычислимые поля ДС.
const CONTRACT_AMENDMENT_ITEM_CALCULATED_FIELDS: &[&str] = &[
    ContractAmendmentItem::pricing_unit_id,
    ContractAmendmentItem::pricing_price_rub,
    ContractAmendmentItem::pricing_currency_id,
    ContractAmendmentItem::pricing_currency_rate,
    ContractAmendmentItem::pricing_currency_rate_date,
    //
    ContractAmendmentItem::pricing_sum_excluded_vat,
    ContractAmendmentItem::pricing_sum_excluded_vat_rub,
    ContractAmendmentItem::pricing_sum_included_vat,
    ContractAmendmentItem::pricing_sum_included_vat_rub,
    ContractAmendmentItem::pricing_sum_vat,
    ContractAmendmentItem::pricing_sum_vat_rub,
    //
    ContractAmendmentItem::pricing_transportation_vat_id,
    ContractAmendmentItem::pricing_transportation_price,
    ContractAmendmentItem::pricing_transportation_price_rub,
    ContractAmendmentItem::pricing_transportation_sum_vat,
    ContractAmendmentItem::pricing_transportation_sum_vat_rub,
    ContractAmendmentItem::pricing_transportation_sum_included_vat,
    ContractAmendmentItem::pricing_transportation_sum_included_vat_rub,
    //
    ContractAmendmentItem::pricing_total_sum,
    ContractAmendmentItem::pricing_total_sum_rub,
    //
    ContractAmendmentItem::pricing_delta_quantity,
    ContractAmendmentItem::pricing_delta_price,
    ContractAmendmentItem::pricing_delta_price_rub,
    //
    ContractAmendmentItem::pricing_delta_sum_excluded_vat,
    ContractAmendmentItem::pricing_delta_sum_excluded_vat_rub,
    ContractAmendmentItem::pricing_delta_sum_included_vat,
    ContractAmendmentItem::pricing_delta_sum_included_vat_rub,
    ContractAmendmentItem::pricing_delta_sum_vat,
    ContractAmendmentItem::pricing_delta_sum_vat_rub,
    //
    ContractAmendmentItem::pricing_delta_transportation_price,
    ContractAmendmentItem::pricing_delta_transportation_price_rub,
    ContractAmendmentItem::pricing_delta_transportation_sum_vat,
    ContractAmendmentItem::pricing_delta_transportation_sum_vat_rub,
    ContractAmendmentItem::pricing_delta_transportation_sum_included_vat,
    ContractAmendmentItem::pricing_delta_transportation_sum_included_vat_rub,
    //
    ContractAmendmentItem::pricing_delta_total_sum,
    ContractAmendmentItem::pricing_delta_total_sum_rub,
];
const ATTACHMENT_FIELDS: &[&str] = &[
    Attachment::number,
    Attachment::object_uuid,
    Attachment::category_id,
    Attachment::is_classified,
    Attachment::is_removed,
    Attachment::kind_id,
    Attachment::mime_id,
    Attachment::size,
];

#[derive(Debug, thiserror::Error)]
pub enum UpdatePlanCAError {
    #[error(
        "ППЗ/ДС с датой очной СК {commission_date} находится в прошлом. Выберите другую дату."
    )]
    OldCommissionDate { commission_date: AsezDate },
    #[error(
        "Запланировать рассмотрение ППЗ/ДС №{plan_id} на {commission_date} запрещено. Обратитесь к Секретарю СК или выберите другую дату."
    )]
    UnableToUpdateCommissionDate {
        plan_id: i64,
        commission_date: AsezDate,
    },
    #[error("Экономия не может быть больше заявленной Стоимости Заказчика.")]
    InvalidEconomy,
    #[error("В запросе отсутствует обязательное поле `{0}`")]
    MissingRequiredField(&'static str),
    #[error("ППЗ/ДС с номером {0} не найдена в базе данных")]
    NotFound(i64),
    #[error("В запросе отстутствует UUID позиции")]
    MissingItemUuid,
    #[error("Позиция с номером {0} не найдена в базе данных")]
    ItemNotFound(i64),
}

fn trace_request(req: &UserIdWrapper<UpdatePlanReq>) {
    tracing::info!(
        kind = "update",
        "Запрос на обновление ППЗ ({update}), user_id: {user_id}, заголовок: {head:?}, {item_count} позиций, {attach_count} документов.\n",
        update = UPDATE_PLANS,
        user_id = req.user_id,
        head = req.dto.plan,
        item_count = req.dto.item_list.len(),
        attach_count = req.dto.pricing_attachment_list.len(),
    );
    // Print everything if we really need to.
    tracing::trace!(
        kind = "update",
        "{items:?}\n{attachments:?}\n",
        items = req.dto.item_list,
        attachments = req.dto.pricing_attachment_list,
    );
}

pub(crate) async fn pa_update_plan(
    request: UserIdWrapper<UpdatePlanReq>,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<UpdatePlanResponseData, ()>> {
    trace_request(&request);
    let UserIdWrapper { user_id, dto } = request;
    let Some(plan_uuid) = dto.plan.uuid else {
        return Err(UpdatePlanCAError::MissingRequiredField("uuid").into());
    };
    let Some(plan_id) = dto.plan.id else {
        return Err(UpdatePlanCAError::MissingRequiredField("id").into());
    };
    let mut messages = Messages::default();

    let pool = &*proc_ctx.db_pool;

    let plan_select = Select::with_fields(PLAN_DB_FIELDS).eq(Plan::uuid, plan_uuid);
    let plan = Plan::select_option(&plan_select, pool)
        .await?
        .ok_or(UpdatePlanCAError::NotFound(plan_id))?;

    let validated_plan = apply_plan_checks(dto.plan.into(), pool)
        .await
        .map(PlanOrAmendmentRep::unwrap_plan)?;

    let plan_update_fields =
        concat_fields(PLAN_DTO_FIELDS.to_vec(), PLAN_CALCULATED_FIELDS);
    let plan_mask = make_bind_mask::<Plan>(PLAN_DTO_FIELDS);
    let mut plans =
        vec![validated_plan.into_item_merged_selected(plan, &plan_mask)?];

    // Грузим позиции с полями, нужными для вычислений.
    // NB. Для вычислимых полей заголовка нам надо загрузить
    // вычислимые поля позиций.
    let item_select_fields = concat_fields(
        concat_fields(PLAN_ITEM_DTO_FIELDS.to_vec(), PLAN_ITEM_DB_FIELDS),
        PLAN_ITEM_CALCULATED_FIELDS,
    );
    let mut db_item_map = PlanItemFull::select(
        &Select::with_fields(&item_select_fields)
            .eq(PlanItemFull::plan_uuid, plans[0].uuid),
        pool,
    )
    .await?
    .into_iter()
    .map(|x| (x.uuid, x))
    .collect::<AHashMap<_, _>>();
    let item_mask = make_bind_mask::<PlanItemFull>(PLAN_ITEM_DTO_FIELDS);

    let mut plan_item_list = dto
        .item_list
        .into_iter()
        .map(|dto_item| {
            let uuid = dto_item.uuid.ok_or(UpdatePlanCAError::MissingItemUuid)?;
            let db_item = db_item_map.remove(&uuid).ok_or(
                UpdatePlanCAError::ItemNotFound(dto_item.id.unwrap_or_default()),
            )?;
            Ok(dto_item.into_item_merged_selected(db_item, &item_mask)?)
        })
        .collect::<Result<Vec<_>>>()?;

    let attachment_list = dto
        .pricing_attachment_list
        .into_iter()
        .map(|x| update_object_uuid(x, &plans[0].uuid))
        .collect::<Result<Vec<Attachment>>>()?;

    calculate_fields_for_plans(
        &mut plans[0],
        &mut plan_item_list,
        db_item_map.values(),
    );

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;

    let handler = proc_ctx.create_rules_checker();

    recorder
        .process_update_checked(plans, &plan_update_fields, handler, &mut messages)
        .await?;

    let plan_item_update_fields =
        concat_fields(PLAN_ITEM_DTO_FIELDS.to_vec(), PLAN_ITEM_CALCULATED_FIELDS);

    recorder
        .process_update(plan_item_list, &plan_item_update_fields, &mut messages)
        .await?;

    // NB: Attachments надо обновить если они есть, а если нет, то вставить.
    utils::upsert(attachment_list, &mut messages, ATTACHMENT_FIELDS, &mut recorder)
        .await?;

    recorder.commit().await?;

    messages.add_message(MessageKind::Success, "ППЗ успешно сохранена".to_string());

    Ok(((), messages).into())
}

pub(crate) async fn pa_update_contract_amendment(
    request: UserIdWrapper<UpdateContractAmendmentReq>,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<UpdateContractAmendmentResponseData, ()>> {
    tracing::info!(
        kind = "update",
        "Запрос на обновление ДС ({update}): {req:?}\n",
        update = UPDATE_CONTRACT_AMENDMENTS,
        req = request,
    );
    let UserIdWrapper { user_id, dto } = request;
    let Some(ca_uuid) = dto.contract_amendment.uuid else {
        return Err(UpdatePlanCAError::MissingRequiredField("uuid").into());
    };
    let Some(plan_id) = dto.contract_amendment.id else {
        return Err(UpdatePlanCAError::MissingRequiredField("id").into());
    };

    let mut messages = Messages::default();
    let pool = &*proc_ctx.db_pool;

    let amendment_select = Select::with_fields(CONTRACT_AMENDMENT_DB_FIELDS)
        .eq(ContractAmendment::uuid, ca_uuid);
    let contract_amendment =
        ContractAmendment::select_option(&amendment_select, pool)
            .await?
            .ok_or(UpdatePlanCAError::NotFound(plan_id))?;

    let validated_ca = apply_plan_checks(dto.contract_amendment.into(), pool)
        .await
        .map(PlanOrAmendmentRep::unwrap_amendment)?;

    let contract_amendment_update_fields = concat_fields(
        CONTRACT_AMENDMENT_DTO_FIELDS.to_vec(),
        CONTRACT_AMENDMENT_CALCULATED_FIELDS,
    );
    let contract_amendmen_mask =
        make_bind_mask::<ContractAmendment>(CONTRACT_AMENDMENT_DTO_FIELDS);

    let mut contract_amendments = vec![validated_ca
        .into_item_merged_selected(contract_amendment, &contract_amendmen_mask)?];

    // Грузим позиции с полями, нужными для вычислений.
    // NB. Для вычислимых полей заголовка нам надо загрузить
    // вычислимые поля позиций.
    let item_select_fields = concat_fields(
        concat_fields(
            CONTRACT_AMENDMENT_ITEM_DTO_FIELDS.to_vec(),
            CONTRACT_AMENDMENT_ITEM_DB_FIELDS,
        ),
        CONTRACT_AMENDMENT_ITEM_CALCULATED_FIELDS,
    );
    let mut db_item_map = ContractAmendmentItem::select(
        &Select::with_fields(&item_select_fields)
            .eq(ContractAmendmentItem::header_uuid, contract_amendments[0].uuid),
        pool,
    )
    .await?
    .into_iter()
    .map(|x| (x.uuid, x))
    .collect::<AHashMap<_, _>>();
    let item_mask =
        make_bind_mask::<ContractAmendmentItem>(CONTRACT_AMENDMENT_ITEM_DTO_FIELDS);

    let mut contract_amendment_item_list = dto
        .item_list
        .into_iter()
        .map(|dto_item| {
            let uuid = dto_item.uuid.ok_or(UpdatePlanCAError::MissingItemUuid)?;
            let db_item = db_item_map.remove(&uuid).ok_or(
                UpdatePlanCAError::ItemNotFound(dto_item.id.unwrap_or_default()),
            )?;
            Ok(dto_item.into_item_merged_selected(db_item, &item_mask)?)
        })
        .collect::<Result<Vec<_>>>()?;

    let attachment_list = dto
        .pricing_attachment_list
        .into_iter()
        .map(|x| update_object_uuid(x, &contract_amendments[0].uuid))
        .collect::<Result<Vec<Attachment>>>()?;

    calculate_fields_for_ca(
        &mut contract_amendments[0],
        &mut contract_amendment_item_list,
        db_item_map.values(),
    );

    let mut recorder =
        proc_ctx.create_record_context().with_user_id(user_id).begin().await?;
    let handler = proc_ctx.create_rules_checker();

    recorder
        .process_update_checked(
            contract_amendments,
            &contract_amendment_update_fields,
            handler,
            &mut messages,
        )
        .await?;

    let contract_amendment_item_update_fields = concat_fields(
        CONTRACT_AMENDMENT_ITEM_DTO_FIELDS.to_vec(),
        CONTRACT_AMENDMENT_ITEM_CALCULATED_FIELDS,
    );
    recorder
        .process_update(
            contract_amendment_item_list,
            &contract_amendment_item_update_fields,
            &mut messages,
        )
        .await?;

    // NB: Attachments надо обновить если они есть, а если нет, то вставить.
    utils::upsert(attachment_list, &mut messages, ATTACHMENT_FIELDS, &mut recorder)
        .await?;

    recorder.commit().await?;

    messages.add_message(MessageKind::Success, "ДС успешно сохранено".to_string());

    Ok(((), messages).into())
}

/// Обновляем если object_uuid не пришел, или если он равен нулям.
/// Если он приходит "нормальный", мы его не меняем, хотя этот вариант
/// скорее всего не предусмотрен.
fn update_object_uuid(doc: AttachmentRep, uuid: &Uuid) -> Result<Attachment> {
    let update_uuid =
        doc.object_uuid.is_none() || doc.object_uuid == Some(Default::default());

    let mut x = doc.into_item()?;

    if update_uuid {
        x.object_uuid = *uuid;
    }

    Ok(x)
}

/// Принимает
///
/// `incoming` - входящий от пользователя ППЗ/ДС
/// `protocol_items` - позиции Протокола СК, которые относятся к этой ППЗ/ДС
///
/// Возвращает отвалидированный вариант [`PlanOrAmendmentRep`], который соответсвует
/// входимому
async fn apply_plan_checks(
    incoming: PlanOrAmendmentRep,
    pool: &PgPool,
) -> Result<PlanOrAmendmentRep> {
    let Some(plan_uuid) = *incoming.uuid() else {
        return Err(UpdatePlanCAError::MissingRequiredField("uuid").into());
    };
    let Some(plan_id) = *incoming.id() else {
        return Err(UpdatePlanCAError::MissingRequiredField("id").into());
    };

    // Проверка должна срабатывать при условии, что для текущего id,
    // в таблицах plan_version/contract_amendment_version нет записей
    // (поиск в таблицах осуществлять по id),
    let select = Select::with_fields(["uuid"]).eq(Plan::id, plan_id);

    let no_versions = match incoming.is_plan() {
        true => PlanVersion::select(&select, pool).await?.is_empty(),
        false => ContractAmendmentVersion::select(&select, pool).await?.is_empty(),
    };

    if !matches!(incoming.savings_accounting_id(), Some(SavingsAccountingId::No))
        && no_versions
        && incoming.sum_excluded_vat().unwrap_or_default()
            < incoming.savings_sum_excluded_vat().flatten().unwrap_or_default()
    {
        return Err(UpdatePlanCAError::InvalidEconomy.into());
    }

    // Если пользователь пытается обновить Дата очной СК/commission_date, то нам надо проверить некоторые
    // условия
    if let Some(commission_date) = incoming.commission_date().flatten() {
        // По ППЗ/ДС надо найти позиции Протоколда СК
        // ПРИ выполнении условий:
        // - Протокол не удален: в таблице protocol установленно значение is_removed = false.
        // - Запись в protocol_item, не удалена is_removed = false и значение в поле Решение СК/result_id = 2,
        // ТО проверку на Дату СК/commission_date не выполнять
        // Напоминание: Из утверждения protocol_item.is_removed=false, не следует, что смежный ему Протокол имеет
        // protocol.is_removed = false
        let protocol_item_select = Select::with_fields([
            EcProtocolItem::source_uuid,
            EcProtocolItem::result_id,
        ])
        .eq(EcProtocolItem::is_removed, false)
        .eq(EcProtocolItem::result_id, ResultId::AgreedWithPriceCorrection)
        .eq(EcProtocolItem::source_uuid, plan_uuid);
        let protocol_select = Select::with_fields([EcProtocol::is_removed])
            .eq(EcProtocol::is_removed, false);

        let protocol_items =
            ProtocolItemWithProtocolSelector::new(protocol_item_select)
                .set_protocol(EcProtocol::join_default().selecting(protocol_select))
                .get(pool)
                .await?;
        if protocol_items.is_empty() {
            validate_commission_date(commission_date, plan_id)?;
        }
    }

    Ok(incoming)
}

/// Валидация даты Сметной Комиссии
///
/// Если меняется/указывается Дата очной СК/commission_date, то если
/// сейчас дата с временем > понедельника 12:00 текущей недели, то поставить можно любую дату начиная со следующей недели
/// и нельзя никакие даты текущей
fn validate_commission_date(
    commission_date: AsezDate,
    plan_id: i64,
) -> std::result::Result<(), UpdatePlanCAError> {
    if commission_date < AsezDate::today() {
        return Err(UpdatePlanCAError::OldCommissionDate { commission_date });
    }

    if !is_commission_date_possible(commission_date) {
        return Err(UpdatePlanCAError::UnableToUpdateCommissionDate {
            plan_id,
            commission_date,
        });
    }

    Ok(())
}

fn concat_fields<'a>(
    mut fields1: Vec<&'a str>,
    fields2: &[&'a str],
) -> Vec<&'a str> {
    fields1.extend_from_slice(fields2);
    fields1
}

macro_rules! add_item {
    ($h:expr,$item:expr, { $($field:ident,)+ }) => {{
        $(
        if let (Some(h), Some(i)) = (&mut $h.$field, $item.$field) {
            *h += i;
        }
        )+
    }};
}

fn calculate_fields_for_plans<'a>(
    header: &mut Plan,
    items_to_update: &'a mut [PlanItemFull],
    other_items: impl IntoIterator<Item = &'a PlanItemFull>,
) {
    // first calculate fields in items
    calculate_plan_items_fields(items_to_update);
    // then fields in plans, that are calculated based on items
    calculate_plan_fields(header, (*items_to_update).iter().chain(other_items));
}

fn calculate_fields_for_ca<'a>(
    header: &mut ContractAmendment,
    items_to_update: &'a mut [ContractAmendmentItem],
    other_items: impl IntoIterator<Item = &'a ContractAmendmentItem>,
) {
    // first calculate fields in items
    calculate_ca_items_fields(items_to_update);
    // then fields in ca, that are calculated based on items
    calculate_ca_fields(header, (*items_to_update).iter().chain(other_items));
}

fn calculate_plan_fields<'a>(
    h: &mut Plan,
    items: impl IntoIterator<Item = &'a PlanItemFull>,
) {
    let currency_rate = h.currency_rate.get_conversion(h.currency_id);
    let convert_currency = |x| currency_rate.convert_value(x);

    // пересчет в рубли
    h.savings_sum_excluded_vat_rub =
        h.savings_sum_excluded_vat.map(convert_currency);
    h.savings_sum_included_vat_rub =
        h.savings_sum_included_vat.map(convert_currency);

    // начальные значения по позициям
    let mut pricing_vat_id = None;

    h.pricing_sum_excluded_vat = CurrencyValue::default();
    h.pricing_sum_excluded_vat_rub = Some(CurrencyValue::default());
    h.pricing_sum_included_vat = Some(CurrencyValue::default());
    h.pricing_sum_included_vat_rub = Some(CurrencyValue::default());
    h.pricing_sum_vat = Some(CurrencyValue::default());
    h.pricing_sum_vat_rub = Some(CurrencyValue::default());
    let mut pricing_transportation_vat_id = None;
    h.pricing_transportation_price = Some(CurrencyValue::default());
    h.pricing_transportation_price_rub = Some(CurrencyValue::default());
    h.pricing_transportation_sum_vat = Some(CurrencyValue::default());
    h.pricing_transportation_sum_vat_rub = Some(CurrencyValue::default());
    h.pricing_transportation_sum_included_vat = Some(CurrencyValue::default());
    h.pricing_transportation_sum_included_vat_rub = Some(CurrencyValue::default());

    h.pricing_total_sum = Some(CurrencyValue::default());
    h.pricing_total_sum_rub = Some(CurrencyValue::default());

    for item in items {
        if item.is_removed {
            continue;
        }

        // общее значение
        update_vat_id(&mut pricing_vat_id, item.pricing_vat_id);

        if let Some(i) = item.pricing_sum_excluded_vat {
            h.pricing_sum_excluded_vat += i;
        };
        add_item!(
            h,
            item,
            {
                pricing_sum_excluded_vat_rub,
                pricing_sum_included_vat,
                pricing_sum_included_vat_rub,
                pricing_sum_vat,
                pricing_sum_vat_rub,

                pricing_transportation_price,
                pricing_transportation_price_rub,
                pricing_transportation_sum_vat,
                pricing_transportation_sum_vat_rub,
                pricing_transportation_sum_included_vat,
                pricing_transportation_sum_included_vat_rub,

                pricing_total_sum,
                pricing_total_sum_rub,
            }
        );
        // общее значение
        update_vat_id(
            &mut pricing_transportation_vat_id,
            item.pricing_transportation_vat_id,
        );
    }
    // копируем
    h.pricing_currency_id = h.currency_id.into();
    h.pricing_currency_rate = h.currency_rate.into();

    h.pricing_vat_id = pricing_vat_id.unwrap_or_default();
    h.pricing_transportation_vat_id =
        pricing_transportation_vat_id.unwrap_or_default();
}
///
///
fn calculate_plan_items_fields(items: &mut [PlanItemFull]) {
    for item in items.iter_mut() {
        let base_price_wo_vat = item
            .pricing_quantity
            .map(|q| q.sum_value(item.pricing_price.unwrap_or_default()))
            .unwrap_or_default();
        let currency_rate = item.currency_rate.get_conversion(item.currency_id);
        let base_vat = item.pricing_vat_id.vat(base_price_wo_vat);
        let base_price = base_price_wo_vat + base_vat;
        let convert_currency = |x| currency_rate.convert_value(x);

        item.pricing_unit_id = item.unit_id.into();
        item.pricing_price_rub = item.pricing_price.map(convert_currency);

        item.pricing_currency_id = item.currency_id.into();
        item.pricing_currency_rate = item.currency_rate.into();
        item.pricing_currency_rate_date = item.currency_rate_date;

        item.pricing_sum_excluded_vat = Some(base_price_wo_vat);
        item.pricing_sum_excluded_vat_rub =
            Some(convert_currency(base_price_wo_vat));
        item.pricing_sum_included_vat = Some(base_price);
        item.pricing_sum_included_vat_rub =
            item.pricing_sum_included_vat.map(convert_currency);
        item.pricing_sum_vat = base_vat.into();
        item.pricing_sum_vat_rub = Some(convert_currency(base_vat));

        let zero = CurrencyValue::default();

        item.pricing_transportation_vat_id = VatId::Unspecified;
        item.pricing_transportation_price = Some(zero);
        item.pricing_transportation_price_rub = Some(zero);
        item.pricing_transportation_sum_vat = Some(zero);
        item.pricing_transportation_sum_vat_rub = Some(zero);
        item.pricing_transportation_sum_included_vat = Some(zero);
        item.pricing_transportation_sum_included_vat_rub = Some(zero);

        let full_sum = base_price
            + item.pricing_transportation_sum_included_vat.unwrap_or_default();
        item.pricing_total_sum = full_sum.into();
        item.pricing_total_sum_rub = Some(convert_currency(full_sum));
    }
}

fn calculate_ca_fields<'a>(
    h: &mut ContractAmendment,
    items: impl IntoIterator<Item = &'a ContractAmendmentItem>,
) {
    let currency_rate = h.currency_rate.get_conversion(h.currency_id);
    let convert_currency = |x| currency_rate.convert_value(x);

    // пересчет в рубли
    h.savings_sum_excluded_vat_rub =
        h.savings_sum_excluded_vat.map(convert_currency);
    h.savings_sum_included_vat_rub =
        h.savings_sum_included_vat.map(convert_currency);

    // начальные значения по позициям
    let mut pricing_vat_id = None;
    h.pricing_sum_excluded_vat = CurrencyValue::default();
    h.pricing_sum_excluded_vat_rub = Some(CurrencyValue::default());
    h.pricing_sum_included_vat = Some(CurrencyValue::default());
    h.pricing_sum_included_vat_rub = Some(CurrencyValue::default());
    h.pricing_sum_vat = Some(CurrencyValue::default());
    h.pricing_sum_vat_rub = Some(CurrencyValue::default());

    let mut pricing_transportation_vat_id = None;
    h.pricing_transportation_price = Some(CurrencyValue::default());
    h.pricing_transportation_price_rub = Some(CurrencyValue::default());
    h.pricing_transportation_sum_vat = Some(CurrencyValue::default());
    h.pricing_transportation_sum_vat_rub = Some(CurrencyValue::default());
    h.pricing_transportation_sum_included_vat = Some(CurrencyValue::default());
    h.pricing_transportation_sum_included_vat_rub = Some(CurrencyValue::default());

    h.pricing_total_sum = Some(CurrencyValue::default());
    h.pricing_total_sum_rub = Some(CurrencyValue::default());

    h.pricing_currency_id = h.currency_id.into();
    h.pricing_currency_rate = currency_rate.into();

    h.pricing_delta_sum_excluded_vat = Some(CurrencyValue::default());
    h.pricing_delta_sum_excluded_vat_rub = Some(CurrencyValue::default());
    h.pricing_delta_sum_included_vat = Some(CurrencyValue::default());
    h.pricing_delta_sum_included_vat_rub = Some(CurrencyValue::default());
    h.pricing_delta_sum_vat = Some(CurrencyValue::default());
    h.pricing_delta_sum_vat_rub = Some(CurrencyValue::default());

    h.pricing_delta_total_sum = Some(CurrencyValue::default());
    h.pricing_delta_total_sum_rub = Some(CurrencyValue::default());
    h.pricing_delta_transportation_price = Some(CurrencyValue::default());
    h.pricing_delta_transportation_sum_included_vat =
        Some(CurrencyValue::default());
    h.pricing_delta_transportation_sum_included_vat_rub =
        Some(CurrencyValue::default());
    h.pricing_delta_transportation_sum_vat = Some(CurrencyValue::default());
    h.pricing_delta_transportation_sum_vat_rub = Some(CurrencyValue::default());

    for item in items {
        if item.is_removed {
            continue;
        }

        update_vat_id(&mut pricing_vat_id, item.pricing_vat_id);
        if let Some(i) = item.pricing_sum_excluded_vat {
            h.pricing_sum_excluded_vat += i;
        };
        if let Some(ref mut h) = h.pricing_transportation_price {
            *h += item.pricing_transportation_price;
        }
        add_item!(
            h,
            item,
            {
                pricing_sum_excluded_vat_rub,
                pricing_sum_included_vat,
                pricing_sum_included_vat_rub,
                pricing_sum_vat,
                pricing_sum_vat_rub,

                pricing_transportation_price_rub,
                pricing_transportation_sum_vat,
                pricing_transportation_sum_vat_rub,
                pricing_transportation_sum_included_vat,
                pricing_transportation_sum_included_vat_rub,

                pricing_total_sum,
                pricing_total_sum_rub,

                pricing_delta_sum_excluded_vat,
                pricing_delta_sum_excluded_vat_rub,
                pricing_delta_sum_included_vat,
                pricing_delta_sum_included_vat_rub,
                pricing_delta_sum_vat,
                pricing_delta_sum_vat_rub,

                pricing_delta_transportation_price,
                pricing_delta_transportation_sum_included_vat,
                pricing_delta_transportation_sum_included_vat_rub,
                pricing_delta_transportation_sum_vat,
                pricing_delta_transportation_sum_vat_rub,
                pricing_delta_total_sum,
                pricing_delta_total_sum_rub,
            }
        );
        update_vat_id(
            &mut pricing_transportation_vat_id,
            item.pricing_transportation_vat_id,
        );
    }

    h.pricing_vat_id = pricing_vat_id.unwrap_or_default();
    h.pricing_transportation_vat_id =
        pricing_transportation_vat_id.unwrap_or_default();
}

fn calculate_ca_items_fields(items: &mut [ContractAmendmentItem]) {
    for item in items.iter_mut() {
        let base_price_wo_vat = item.pricing_quantity.sum_value(item.pricing_price);
        let currency_rate = item.currency_rate.get_conversion(item.currency_id);
        let base_vat = item.pricing_vat_id.vat(base_price_wo_vat);
        let base_price = base_price_wo_vat + base_vat;
        let convert_currency = |x| currency_rate.convert_value(x);

        item.pricing_unit_id = item.unit_id;
        item.pricing_price_rub = Some(convert_currency(item.pricing_price));
        item.pricing_currency_id = item.currency_id;
        item.pricing_currency_rate = item.currency_rate.into();
        item.pricing_currency_rate_date = item.currency_rate_date.into();

        item.pricing_sum_excluded_vat = Some(base_price_wo_vat);
        item.pricing_sum_excluded_vat_rub =
            Some(convert_currency(base_price_wo_vat));
        item.pricing_sum_included_vat = Some(base_price_wo_vat + base_vat);
        item.pricing_sum_included_vat_rub =
            item.pricing_sum_included_vat.map(convert_currency);
        item.pricing_sum_vat = base_vat.into();
        item.pricing_sum_vat_rub = Some(convert_currency(base_vat));

        item.pricing_transportation_vat_id = VatId::Unspecified;
        item.pricing_transportation_price = CurrencyValue::default();
        item.pricing_transportation_price_rub = Some(CurrencyValue::default());
        item.pricing_transportation_sum_vat = Some(CurrencyValue::default());
        item.pricing_transportation_sum_vat_rub = Some(CurrencyValue::default());
        item.pricing_transportation_sum_included_vat =
            Some(CurrencyValue::default());
        item.pricing_transportation_sum_included_vat_rub =
            Some(CurrencyValue::default());

        let full_sum = base_price
            + item.pricing_transportation_sum_included_vat.unwrap_or_default();
        item.pricing_total_sum = full_sum.into();
        item.pricing_total_sum_rub = Some(convert_currency(full_sum));

        item.pricing_delta_quantity =
            Some(item.pricing_quantity - item.previous_quantity);
        item.pricing_delta_price = Some(item.pricing_price - item.previous_price);
        item.pricing_delta_price_rub =
            item.pricing_delta_price.map(convert_currency);

        item.pricing_delta_sum_excluded_vat =
            Some(base_price_wo_vat - item.previous_sum_excluded_vat);
        item.pricing_delta_sum_excluded_vat_rub =
            item.pricing_delta_sum_excluded_vat.map(convert_currency);
        item.pricing_delta_sum_included_vat = item
            .pricing_sum_included_vat
            .map(|x| x - item.previous_sum_included_vat);
        item.pricing_delta_sum_included_vat_rub =
            item.pricing_delta_sum_included_vat.map(convert_currency);
        item.pricing_delta_sum_vat = Some(base_vat - item.previous_sum_vat);
        item.pricing_delta_sum_vat_rub =
            item.pricing_delta_sum_vat.map(convert_currency);

        item.pricing_delta_transportation_price = Some(CurrencyValue::default());
        item.pricing_delta_transportation_price_rub =
            Some(CurrencyValue::default());
        item.pricing_delta_transportation_sum_vat = Some(CurrencyValue::default());
        item.pricing_delta_transportation_sum_vat_rub =
            Some(CurrencyValue::default());
        item.pricing_delta_transportation_sum_included_vat =
            Some(CurrencyValue::default());
        item.pricing_delta_transportation_sum_included_vat_rub =
            Some(CurrencyValue::default());

        item.pricing_delta_total_sum = item.pricing_delta_sum_included_vat;
        item.pricing_delta_total_sum_rub = item.pricing_delta_sum_included_vat_rub;
    }
}

/// Utility to calculate common vat_id across multiple values.
fn update_vat_id(common: &mut Option<VatId>, value: VatId) {
    match *common {
        None => *common = Some(value),
        Some(v) if v == value => {}
        _ => *common = Some(VatId::Compound),
    }
}

#[cfg(test)]
mod tests;
