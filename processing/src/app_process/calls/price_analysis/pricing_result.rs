use std::sync::Arc;

use crate::{
    app_process::common::plan::fetch_plans_by_ids,
    common::{ProcessingError, Result},
};

use asez2_shared_db::db_item::{AsezDate, Select};
use asez2_shared_db::{DbItem, Value};
use shared_essential::{
    domain::{
        maths::CurrencyValue,
        processing::regulatory_deadline_price::RegulatoryDeadlinePrice, Plan,
        PlanOrAmendment, PlanOrAmendmentRep, SavingsAccountingId, Section,
        StatusHistory,
    },
    presentation::dto::{
        general::ObjectIdentifier,
        processing::{
            price_analysis::{PricingResultReq, PricingResultResponseData},
            CalculatedPlanRep, ColorScheme, ColorThreshold,
        },
        response_request::{ApiResponse, Messages, Status},
    },
};
use sqlx::PgPool;

const PRICING_RESULT_TAG: &str = "/pricing/v1/get/pricing_result/";

const RELEVANT_STATUSES: [i16; 3] = [222, 342, 352];

const RESPONSE_FIELDS: &[&str] = &[
    Plan::uuid,
    Plan::id,
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::expert_conclusion_id,
    Plan::customer_id,
    Plan::contract_subject,
    Plan::supplier_id,
    Plan::section_id,
    Plan::single_supplier_reason_id,
    Plan::sum_excluded_vat_rub,
    Plan::sum_included_vat_rub,
    Plan::pricing_sum_excluded_vat_rub,
    Plan::pricing_sum_included_vat_rub,
];

pub(crate) async fn pa_pricing_result(
    req: PricingResultReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<PricingResultResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Процессинг получил запрос от {get}: {req:?}\n",
        get = PRICING_RESULT_TAG,
        req = req,
    );

    let plan = fetch_plan(&req, &db_pool).await?;
    let savings_in_percent = calculate_savings_in_percent(&plan);
    let days_with_expert =
        calculate_number_of_days_with_expert(&plan, &db_pool).await?;

    let calculated_data = CalculatedPlanRep::new(PlanOrAmendmentRep::from_item(
        plan,
        Some(RESPONSE_FIELDS),
    ))
    .set_number_of_days_with_expert_threshold_unconditional(
        days_with_expert.unwrap_or_default(),
    )
    .set_savings_in_percent_unconditional(savings_in_percent.unwrap_or_default());

    Ok(ApiResponse {
        data: calculated_data,
        messages: Messages::default(),
        objects: vec![],
        status: Status::Ok,
    })
}

async fn fetch_plan(
    item: &ObjectIdentifier,
    db_pool: &PgPool,
) -> Result<PlanOrAmendment> {
    let mut plans = fetch_plans_by_ids(vec![item], db_pool).await?;

    if plans.len() > 1 {
        return Err(ProcessingError::GetItemList(format!(
            "Найдено несколько записей с идентификатором {}",
            item.uuid
        )));
    }

    Ok(plans.remove(0))
}

/// Если в поле savings_accounting_id указано значение 2 или 3, то вывести разницу между значениями полей.
/// Расчет экономии в процентах производится по следующей формуле:
///
/// savings_sum_included_vat_rub / (savings_sum_included_vat_rub + pricing_sum_included_vat_rub)
fn calculate_savings_in_percent(plan: &PlanOrAmendment) -> Option<String> {
    if *plan.savings_accounting_id() == SavingsAccountingId::No {
        return Some("Экономия не учитывается".to_string());
    }

    let savings_sum_included_vat_rub = (*plan.savings_sum_included_vat_rub())?;
    let pricing_sum_included_vat_rub = (*plan.pricing_sum_included_vat_rub())?;
    let sum_included_vat_rub_max =
        savings_sum_included_vat_rub + pricing_sum_included_vat_rub;

    if sum_included_vat_rub_max == CurrencyValue::from(0) {
        return None;
    }

    let percent_diff = f64::from(savings_sum_included_vat_rub)
        / f64::from(sum_included_vat_rub_max)
        * 100.0;

    Some(format!("{:.0}%", percent_diff.round()))
}

/// Необходимо посчитать количество рабочих дней от даты перевода ППЗ/ДС на статус 222, 342, 352 до текущей даты (включительно).
/// Для этого найти для UUID ППЗ/ДС актуальные записи с переводами на эти статусы.
/// Далее выбрать крайнюю по дате создания запись и посчитать количество рабочих дней между текущей датой и датой создания найденной крайней записи.
/// Также нужно проверить в STATUS_HISTORY для UUID ППЗ/ДС наличие других записей с более поздней датой создания (changed_at).
/// Если более поздние записи не найдены, то посчитать количество рабочих дней между текущей датой (включительно) и датой создания (changed_at) найденной крайней записи со  STATUS_ID=222, 342, 352.
/// Если записи с более поздними датами создания найдены, то взять самую раннюю из них по дате создания (changed_at) (STATUS_ID в этой записи может быть любой)
/// и посчитать количество рабочих дней между датой создания этой записи и ранее найденной датой создания записи со STATUS_ID=222, 342, 352.
async fn calculate_number_of_days_with_expert(
    plan: &PlanOrAmendment,
    db_pool: &PgPool,
) -> Result<Option<ColorThreshold>> {
    let select = Select::full_in::<_, StatusHistory>(
        StatusHistory::object_uuid,
        vec![Value::from(plan.uuid())],
    );
    let history_items = StatusHistory::select(&select, db_pool).await?;

    if history_items.is_empty() {
        return Ok(None);
    }

    let relevant_items: Vec<_> = history_items
        .iter()
        .filter(|entry| RELEVANT_STATUSES.contains(&entry.status_id))
        .collect();

    // Выбираем крайнюю по дате создания
    let latest_relevant =
        relevant_items.iter().max_by_key(|entry| entry.created_at);

    if let Some(latest_relevant) = latest_relevant {
        let start_date = AsezDate(latest_relevant.created_at.date());

        // Выбираем первую запись (по времени), которая идет сразу после записи с нужным статусом
        let next_status = history_items
            .iter()
            .filter(|entry| entry.created_at > latest_relevant.created_at)
            .min_by_key(|entry| entry.created_at);

        let end_date = next_status
            .map(|entry| AsezDate(entry.created_at.date()))
            .unwrap_or_else(AsezDate::today); // Если записи нет, используем текущую дату
        let working_days = start_date.working_days_between(end_date);

        let color_threshold = get_scheme_id(
            working_days as i32,
            Section::PriceAnalysisDeterminePrice,
            db_pool,
        )
        .await?
        .map(|scheme_id| ColorThreshold {
            value: working_days as usize,
            color_scheme_id: scheme_id,
        });

        return Ok(color_threshold);
    }

    Ok(None)
}

async fn get_scheme_id(
    days: i32,
    section: Section,
    db_pool: &PgPool,
) -> Result<Option<ColorScheme>> {
    let select_deadlines = Select::with_fields([
        RegulatoryDeadlinePrice::start_day,
        RegulatoryDeadlinePrice::end_day,
        RegulatoryDeadlinePrice::color_scheme_id,
    ])
    .eq(RegulatoryDeadlinePrice::section, section as i32)
    .eq(RegulatoryDeadlinePrice::status, Some(false));

    let deadlines: Vec<RegulatoryDeadlinePrice> =
        RegulatoryDeadlinePrice::select(&select_deadlines, db_pool).await?;

    if deadlines.is_empty() {
        return Ok(None);
    }

    let color_scheme_id = deadlines
        .iter()
        .find(|deadline| days >= deadline.start_day && days <= deadline.end_day)
        .map(|deadline| deadline.color_scheme_id)
        // Все что выходит за значение end, считаем красным
        .unwrap_or(ColorScheme::Red as i32);

    Ok(Some(ColorScheme::from(color_scheme_id as u8)))
}
