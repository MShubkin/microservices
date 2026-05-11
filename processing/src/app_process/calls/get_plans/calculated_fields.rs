use crate::common::Result;
use ahash::AHashMap;
use asez2_shared_db::db_item::{
    AsezDate, AsezTimestamp, Filter, FilterTree, Select,
};
use asez2_shared_db::DbItem;
use rabbit_services::specialized_departments::SpecializedDepartmentsService;
use shared_essential::domain::{
    legacy::plans::PlanStatus,
    processing::regulatory_deadline_price::RegulatoryDeadlinePrice,
    DocumentApproverRep, Section, StatusHistory,
};
use shared_essential::presentation::dto::processing::{
    ColorScheme, ColorThreshold, *,
};
use shared_essential::presentation::dto::response_request::ApiResponse;
use shared_essential::presentation::dto::specialized_departments::request::GetApproversForPlansReq;
use sqlx::PgPool;
use uuid::Uuid;

use super::GetPlansItem;

#[derive(Default)]
struct CalculationFlags {
    pricing_process_count: bool,
    start_received_date: bool,
    start_primary_expert_control_date: bool,
    start_determine_price_date: bool,
    start_approved_date: bool,
    number_of_days_with_expert_threshold: bool,
    pricing_working_days_count_threshold: bool,
    commission_percent_economy: bool,
    commission_economy_sum_excluded_vat: bool,
    vote_iteraction_price: bool,
    approvers: bool,
}

impl CalculationFlags {
    fn from_field_list(field_list: &[String]) -> Self {
        let mut flags = CalculationFlags::default();
        field_list.iter().for_each(|field| match field.as_str() {
            PRICING_PROCESS_COUNT => flags.pricing_process_count = true,
            START_RECEIVED_DATE => flags.start_received_date = true,
            START_PRIMARY_EXPERT_CONTROL_DATE => {
                flags.start_primary_expert_control_date = true
            }
            START_DETERMINE_PRICE_DATE => flags.start_determine_price_date = true,
            NUMBER_OF_DAYS_WITH_EXPERT_THRESHOLD => {
                flags.number_of_days_with_expert_threshold = true
            }
            PRICING_WORKING_DAYS_COUNT_THRESHOLD => {
                flags.pricing_working_days_count_threshold = true
            }
            START_APPROVED_DATE => flags.start_approved_date = true,
            COMMISSION_PERCENT_ECONOMY => flags.commission_percent_economy = true,
            COMMISSION_ECONOMY_SUM_EXCLUDED_VAT => {
                flags.commission_economy_sum_excluded_vat = true
            }
            VOTE_ITERACTION_PRICE => flags.vote_iteraction_price = true,
            APPROVERS => flags.approvers = true,
            _ => {}
        });

        flags
    }
}

pub(crate) async fn add_calculated_fields(
    items: &mut Vec<GetPlansItem>,
    db_conn: &PgPool,
    spec_deps: &SpecializedDepartmentsService,
    select: &Select,
    section: &Section,
) -> Result<()> {
    let calculation_flags = CalculationFlags::from_field_list(&select.field_list);

    let plan_info: Vec<(Uuid, PlanStatus)> = items
        .iter()
        .map(|item| (*item.plan.item.uuid(), *item.plan.item.status_id()))
        .collect();

    if calculation_flags.pricing_process_count {
        let pricing_counts =
            get_pricing_process_counts(db_conn, &plan_info).await?;
        items.iter_mut().map(|i| &mut i.plan).zip(pricing_counts).for_each(
            |(plan, count)| {
                if let Some(count) = count {
                    plan.calculated.pricing_process_count = Some(count);
                }
            },
        );
    }

    if calculation_flags.start_received_date {
        let received_dates = get_start_received_dates(db_conn, &plan_info).await?;
        items.iter_mut().map(|i| &mut i.plan).zip(received_dates).for_each(
            |(plan, received_date)| {
                if let Some(received_date) = received_date {
                    plan.calculated.start_received_date = Some(received_date);
                }
            },
        );
    }

    if calculation_flags.start_primary_expert_control_date {
        let expert_control_dates =
            get_start_primary_expert_control_dates(db_conn, &plan_info).await?;
        items
            .iter_mut()
            .map(|i| &mut i.plan)
            .zip(expert_control_dates)
            .for_each(|(plan, control_date)| {
                if let Some(control_date) = control_date {
                    plan.calculated.start_primary_expert_control_date =
                        Some(control_date);
                }
            });
    }

    if calculation_flags.start_determine_price_date {
        let determine_price_dates =
            get_start_determine_price_dates(db_conn, &plan_info).await?;
        items
            .iter_mut()
            .map(|i| &mut i.plan)
            .zip(determine_price_dates)
            .for_each(|(plan, price)| {
                if let Some(price) = price {
                    plan.calculated.start_determine_price_date = Some(price);
                }
            });
    }

    if calculation_flags.start_approved_date {
        let approved_dates = get_start_approved_dates(db_conn, &plan_info).await?;
        items.iter_mut().map(|i| &mut i.plan).zip(approved_dates).for_each(
            |(plan, approved_date)| {
                if let Some(approved_date) = approved_date {
                    plan.calculated.start_approved_date = Some(approved_date);
                }
            },
        );
    }

    if calculation_flags.number_of_days_with_expert_threshold {
        let number_of_days =
            get_number_of_days_with_expert(db_conn, &plan_info, section).await?;
        items.iter_mut().map(|i| &mut i.plan).zip(number_of_days).for_each(
            |(plan, date)| {
                if let Some(date) = date {
                    plan.calculated.number_of_days_with_expert_threshold =
                        Some(date);
                }
            },
        );
    }

    if calculation_flags.pricing_working_days_count_threshold {
        let plan_info: Vec<(Uuid, AsezDate)> = items
            .iter()
            .map(|item| &item.plan.item)
            .map(|plan| {
                let uuid = plan.uuid();
                let pricing_started_at = plan.pricing_started_at();
                (*uuid, AsezDate(pricing_started_at.date()))
            })
            .collect();

        let pricing_working_days =
            calculate_pricing_working_days_count(db_conn, &plan_info, section)
                .await?;

        items
            .iter_mut()
            .map(|i| &mut i.plan)
            .zip(pricing_working_days)
            .for_each(|(plan, threshold)| {
                if let Some(threshold) = threshold {
                    plan.calculated.pricing_working_days_count_threshold =
                        Some(threshold);
                }
            });
    }

    // TODO: По позиции Протокола дублируется логика по вычисляемым полям, нужно будет объединить
    if calculation_flags.commission_economy_sum_excluded_vat {
        items.iter_mut().filter_map(|i| i.protocol_item.as_mut()).for_each(
            |item| {
                if let (Some(sum_excluded_vat), Some(commission_sum_excluded_vat)) = (item.item.sum_excluded_vat, item.item.commission_sum_excluded_vat) {
                    let commission_economy = sum_excluded_vat - commission_sum_excluded_vat;
                    item.calculated.commission_economy_sum_excluded_vat = Some(commission_economy);
                }
            },
        );
    }

    if calculation_flags.commission_percent_economy {
        items.iter_mut().filter_map(|i| i.protocol_item.as_mut()).for_each(
            |item| {
                if let (Some(sum_excluded_vat), Some(commission_sum_excluded_vat)) = (
                    item.item.sum_excluded_vat,
                    item.item.commission_sum_excluded_vat,
                ) {
                    let (sum_excluded_vat, commission_sum_excluded_vat): (i64, i64) = (
                        sum_excluded_vat.into(),
                        commission_sum_excluded_vat.into(),
                    );
                    let commission_economy =
                        sum_excluded_vat - commission_sum_excluded_vat;

                    let percent_economy = if commission_sum_excluded_vat < 0
                        || sum_excluded_vat <= 0
                    {
                        "-".to_string()
                    } else {
                        const C: f64 = 100.;
                        let x = commission_economy as f64 / sum_excluded_vat as f64
                            * C.powi(2);
                        let x = x.round() / C;
                        format!("{x:.2}").replace('.', ",")
                    };

                    item.calculated.commission_percent_economy =
                        Some(percent_economy);
                }
            },
        );
    }

    if calculation_flags.vote_iteraction_price {
        let plan_info: Vec<i64> =
            items.iter().map(|item| *item.plan.item.id()).collect();

        let vote_iteraction_price =
            get_vote_iteraction_prices(db_conn, &plan_info).await?;

        let price_map: AHashMap<i64, i64> =
            vote_iteraction_price.into_iter().collect();

        items.iter_mut().map(|i| &mut i.plan).for_each(|plan| {
            plan.calculated.vote_iteraction_price =
                price_map.get(plan.item.id()).copied();
        });
    }

    if calculation_flags.approvers {
        let approvers_map = get_approvers_map(
            items.iter().map(|item| *item.plan.item.id()).collect(),
            spec_deps,
        )
        .await?;
        items.iter_mut().map(|i| &mut i.plan).for_each(|plan| {
            plan.calculated.approvers = Some(
                approvers_map.get(plan.item.id()).cloned().unwrap_or_default(),
            );
        });
    }

    filter_calculated_data(items, &select.filter_list)?;

    Ok(())
}

// Количество переводов ППЗ/ДС на статус.
// В таблице status_history для ID обрабатываемой ППЗ/ДC найти записи, в которых status_id= status_id ППЗ/ДС.
async fn get_pricing_process_counts(
    db_conn: &PgPool,
    plan_info: &[(Uuid, PlanStatus)],
) -> Result<Vec<Option<u16>>> {
    let select = Select::full::<StatusHistory>().in_any(
        StatusHistory::object_uuid,
        plan_info.iter().map(|(uuid, _)| *uuid),
    );
    let items = StatusHistory::select(&select, db_conn).await?;

    let mut counts = vec![None; plan_info.len()];
    for (i, (object_uuid, status_id)) in plan_info.iter().enumerate() {
        let count = items
            .iter()
            .filter(|item| {
                item.object_uuid == *object_uuid
                    && item.status_id == *status_id as i16
            })
            .count() as u16;

        counts[i] = (count > 0).then_some(count);
    }

    Ok(counts)
}

// В таблице status_history для uuid обрабатываемой ППЗ/ДC найти записи,
// в которых status_id= status_id ППЗ/ДС (Возможные значения поля status_id: 221, 341, 351)
// Из полученных записей выбрать запись крайнюю по дате создания.
async fn get_start_received_dates(
    db_conn: &PgPool,
    plan_info: &[(Uuid, PlanStatus)],
) -> Result<Vec<Option<AsezTimestamp>>> {
    let select = Select::full::<StatusHistory>()
        .in_any(StatusHistory::object_uuid, plan_info.iter().map(|(uuid, _)| *uuid))
        .in_any(
            StatusHistory::status_id,
            vec![
                PlanStatus::ExecutorAppointmentD645,
                PlanStatus::ExecutorAppointmentD646,
                PlanStatus::ExecutorAppointmentD647,
                PlanStatus::ExecutorAppointmentMTP,
            ],
        )
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid]);

    let items = StatusHistory::select(&select, db_conn).await?;

    Ok(plan_info
        .iter()
        .map(|(uuid, _)| {
            items
                .iter()
                .find(|item| item.object_uuid == *uuid)
                .map(|item| AsezTimestamp(*item.created_at))
        })
        .collect())
}

// В таблице status_history для uuid обрабатываемой ППЗ/ДC найти записи,
// в которых status_id= status_id ППЗ/ДС (Возможные значения поля status_id: 222, 342, 352)
// Из полученных записей выбрать запись крайнюю по дате создания.
async fn get_start_primary_expert_control_dates(
    db_conn: &PgPool,
    plan_info: &[(Uuid, PlanStatus)],
) -> Result<Vec<Option<AsezTimestamp>>> {
    let select = Select::full::<StatusHistory>()
        .in_any(StatusHistory::object_uuid, plan_info.iter().map(|(uuid, _)| *uuid))
        .in_any(
            StatusHistory::status_id,
            vec![
                PlanStatus::ExecutorAppointedD645,
                PlanStatus::ExecutorAppointedD646,
                PlanStatus::ExecutorAppointedD647,
                PlanStatus::ExecutorAppointedMTP,
            ],
        )
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid]);

    let items = StatusHistory::select(&select, db_conn).await?;

    Ok(plan_info
        .iter()
        .map(|(uuid, _)| {
            items
                .iter()
                .find(|item| item.object_uuid == *uuid)
                .map(|item| AsezTimestamp(*item.created_at))
        })
        .collect())
}

// В таблицах plan/contract_amendment для uuid обрабатываемой ППЗ/ДC найти записи,
// в которых status_id (Возможные значения поля status_id : 222, 342, 352).
// Из полученных записей выбрать запись крайнюю по дате создания.
async fn get_start_determine_price_dates(
    db_conn: &PgPool,
    plan_info: &[(Uuid, PlanStatus)],
) -> Result<Vec<Option<AsezTimestamp>>> {
    let select = Select::full::<StatusHistory>()
        .in_any(StatusHistory::object_uuid, plan_info.iter().map(|(uuid, _)| *uuid))
        .in_any(
            StatusHistory::status_id,
            vec![
                PlanStatus::ExecutorAppointedD645,
                PlanStatus::ExecutorAppointedD646,
                PlanStatus::ExecutorAppointedD647,
                PlanStatus::ExecutorAppointedMTP,
            ],
        )
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid]);

    let items = StatusHistory::select(&select, db_conn).await?;

    Ok(plan_info
        .iter()
        .map(|(uuid, _)| {
            items
                .iter()
                .find(|item| item.object_uuid == *uuid)
                .map(|item| AsezTimestamp(*item.created_at))
        })
        .collect())
}

// В таблице status_history для uuid обрабатываемой ППЗ/ДC найти записи, в которых status_id= status_id ППЗ/ДС
// (Возможные значения поля status_id: 223, 343, 353)
// Из полученных записей выбрать запись крайнюю по дате создания.
async fn get_start_approved_dates(
    db_conn: &PgPool,
    plan_info: &[(Uuid, PlanStatus)],
) -> Result<Vec<Option<AsezTimestamp>>> {
    let select = Select::full::<StatusHistory>()
        .in_any(StatusHistory::object_uuid, plan_info.iter().map(|(uuid, _)| *uuid))
        .in_any(
            StatusHistory::status_id,
            vec![
                PlanStatus::AnalysisPerformedD645,
                PlanStatus::AnalysisPerformedD646,
                PlanStatus::AnalysisPerformedD647,
                PlanStatus::AnalysisPerformedMTP,
            ],
        )
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid]);

    let items = StatusHistory::select(&select, db_conn).await?;
    Ok(plan_info
        .iter()
        .map(|(uuid, _)| {
            items
                .iter()
                .find(|item| item.object_uuid == *uuid)
                .map(|item| AsezTimestamp(*item.created_at))
        })
        .collect())
}

// В таблице status_history для uuid обрабатываемой ППЗ/ДC найти записи,
// в которых status_id= status_id ППЗ/ДС (Возможные значения поля status_id: 222, 342, 352).
// Из полученных записей выбрать запись крайнюю по дате создания. Определить количество рабочих дней между датами.
// Для вычисления цветовой схемы нужно посмотреть таблицу regulatory_deadline_price (color_scheme_id), где проверить, входит ли полученное значение
// в интервал "start_day" и "end_day" для соответствующей секции и status = false. Значения которые выходят за рамки "end" считаются красным.
async fn get_number_of_days_with_expert(
    db_conn: &PgPool,
    plan_info: &[(Uuid, PlanStatus)],
    section: &Section,
) -> Result<Vec<Option<ColorThreshold>>> {
    let select = Select::full::<StatusHistory>()
        .in_any(StatusHistory::object_uuid, plan_info.iter().map(|(uuid, _)| *uuid))
        .in_any(
            StatusHistory::status_id,
            vec![
                PlanStatus::ExecutorAppointedD645,
                PlanStatus::ExecutorAppointedD646,
                PlanStatus::ExecutorAppointedD647,
                PlanStatus::ExecutorAppointedMTP,
            ],
        )
        .add_replace_order_desc(StatusHistory::object_uuid)
        .add_replace_order_desc(StatusHistory::created_at)
        .distinct_on(&[StatusHistory::object_uuid]);

    let items = StatusHistory::select(&select, db_conn).await?;

    let working_days: Vec<Option<u16>> = plan_info
        .iter()
        .map(|(uuid, _)| {
            items.iter().find(|item| item.object_uuid == *uuid).map(|item| {
                AsezDate(item.created_at.date())
                    .working_days_between(AsezDate::today())
            })
        })
        .collect();

    if working_days.iter().all(|days| days.is_none()) {
        return Ok(vec![None; working_days.len()]);
    }

    let select_deadlines = Select::with_fields([
        RegulatoryDeadlinePrice::start_day,
        RegulatoryDeadlinePrice::end_day,
        RegulatoryDeadlinePrice::color_scheme_id,
    ])
    .eq(RegulatoryDeadlinePrice::section, *section as i32)
    .eq(RegulatoryDeadlinePrice::status, Some(false));

    let deadlines: Vec<RegulatoryDeadlinePrice> =
        RegulatoryDeadlinePrice::select(&select_deadlines, db_conn).await?;

    //TODO: доработать логику, сейчас неизвестно, что делать, если в таблице нет нужных данных
    if deadlines.is_empty() {
        return Ok(working_days
            .iter()
            .map(|&days| {
                days.map(|value| ColorThreshold {
                    value: value as usize,
                    color_scheme_id: ColorScheme::Undefined,
                })
            })
            .collect());
    }

    let deadline_ranges: Vec<(i32, i32, i32)> = deadlines
        .iter()
        .map(|deadline| {
            (deadline.start_day, deadline.end_day, deadline.color_scheme_id)
        })
        .collect();

    let thresholds: Vec<Option<ColorThreshold>> = working_days
        .into_iter()
        .map(|days| {
            days.map(|value| {
                let color_scheme_id = deadline_ranges
                    .iter()
                    .find(|(start, end, _)| {
                        value as i32 >= *start && value as i32 <= *end
                    })
                    .map(|(_, _, color)| *color)
                    // Все что выходит за значение end, считаем красным
                    .unwrap_or(ColorScheme::Red as i32);

                let color_scheme = ColorScheme::from(color_scheme_id as u8);

                ColorThreshold {
                    value: value as usize,
                    color_scheme_id: color_scheme,
                }
            })
        })
        .collect();

    Ok(thresholds)
}

// В ППЗ/ДС в соответствующей таблице plan или contract_amendment найти значение pricing_started_at
// Определить кол-во рабочих дней между датами.
// Для вычисления цветовой схемы нужно посмотреть таблицу regulatory_deadline_price (color_scheme_id), где проверить, входит ли полученное значение
// в интервал "start_day" и "end_day" для соответствующей секции и status = false. Значения которые выходят за рамки "end" считаются красным.
async fn calculate_pricing_working_days_count(
    db_conn: &PgPool,
    plan_info: &[(Uuid, AsezDate)],
    section: &Section,
) -> Result<Vec<Option<ColorThreshold>>> {
    let select_deadlines = Select::with_fields([
        RegulatoryDeadlinePrice::start_day,
        RegulatoryDeadlinePrice::end_day,
        RegulatoryDeadlinePrice::color_scheme_id,
    ])
    .eq(RegulatoryDeadlinePrice::section, *section as i32)
    .eq(RegulatoryDeadlinePrice::status, Some(false));

    let deadlines: Vec<RegulatoryDeadlinePrice> =
        RegulatoryDeadlinePrice::select(&select_deadlines, db_conn).await?;

    if deadlines.is_empty() {
        return Ok(plan_info.iter().map(|_| None).collect());
    }

    let deadline_ranges: Vec<(i32, i32, i32)> = deadlines
        .into_iter()
        .map(|deadline| {
            (deadline.start_day, deadline.end_day, deadline.color_scheme_id)
        })
        .collect();

    let results: Vec<Option<ColorThreshold>> = plan_info
        .iter()
        .map(|(_, pricing_started_at)| {
            // Считаем количество рабочих дней
            let working_days_count =
                pricing_started_at.working_days_between(AsezDate::today());

            // Сопоставляем с диапазонами
            let color_scheme_id = deadline_ranges
                .iter()
                .find(|(start, end, _)| {
                    working_days_count as i32 >= *start
                        && working_days_count as i32 <= *end
                })
                .map(|(_, _, color)| *color)
                // Все что выходит за значение end, считаем красным
                .unwrap_or(ColorScheme::Red as i32);

            let color_scheme = ColorScheme::from(color_scheme_id as u8);

            Some(ColorThreshold {
                value: working_days_count as usize,
                color_scheme_id: color_scheme,
            })
        })
        .collect();

    Ok(results)
}

async fn get_vote_iteraction_prices(
    db_conn: &PgPool,
    plan_info: &[i64],
) -> Result<Vec<(i64, i64)>> {
    let votes: Vec<(i64, i64)> = sqlx::query_as(
        "
        WITH plan_uuids AS (
            SELECT DISTINCT uuid, id
            FROM plan
            WHERE id = ANY($1)
            UNION
            SELECT DISTINCT uuid, id
            FROM contract_amendment
            WHERE id = ANY($1)
        ),
        protocol_items AS (
            SELECT pi.source_uuid, pi.uuid
            FROM protocol_item pi
            INNER JOIN protocol p ON pi.protocol_uuid = p.uuid
            WHERE pi.source_uuid IN (SELECT uuid FROM plan_uuids)
              AND p.is_removed = false
              AND p.protocol_type_id = 2
              AND pi.is_removed = false
        ),
        agenda_items AS (
            SELECT ai.source_uuid, ai.uuid
            FROM agenda_item ai
            INNER JOIN agenda a ON ai.agenda_uuid = a.uuid
            WHERE ai.source_uuid IN (SELECT uuid FROM plan_uuids)
              AND a.is_removed = false
              AND ai.is_removed = false
        ),
        p_counts AS (
            SELECT pv.id, COUNT(DISTINCT pi.uuid) AS p_count
            FROM plan_uuids pv
            LEFT JOIN protocol_items pi ON pv.uuid = pi.source_uuid
            GROUP BY pv.id
        ),
        a_counts AS (
            SELECT pv.id, COUNT(DISTINCT ai.uuid) AS a_count
            FROM plan_uuids pv
            LEFT JOIN agenda_items ai ON pv.uuid = ai.source_uuid
            GROUP BY pv.id
        )
        SELECT plan_uuids.id as id, coalesce(p_count, 0) + coalesce(a_count, 0) AS vote_iteraction_price
        FROM plan_uuids 
        left JOIN p_counts p ON p.id = plan_uuids.id
        left JOIN a_counts a ON a.id = plan_uuids.id;
    ",
    )
    .bind(plan_info)
    .fetch_all(db_conn)
    .await?;

    Ok(votes)
}

async fn get_approvers_map(
    plan_ids: Vec<i64>,
    spec_deps: &SpecializedDepartmentsService,
) -> Result<AHashMap<i64, Vec<Approver>>> {
    let req = GetApproversForPlansReq {
        plan_ids,
        is_actual: Some(true),
    };
    let ApiResponse { data, .. } = spec_deps.get_approvers_for_plans(req).await?;
    let to_approver = |x: DocumentApproverRep| Approver {
        department_id: x.department_id,
        response_id: x.response_id.flatten(),
    };
    let approvers_map = data
        .into_iter()
        .map(|x| (x.plan_id, x.item_list.into_iter().map(to_approver).collect()))
        .collect();
    Ok(approvers_map)
}

fn filter_calculated_data(
    data: &mut Vec<GetPlansItem>,
    filter_list: &FilterTree,
) -> Result<()> {
    let filters = filter_list.slice();

    data.retain(|item| {
        filters.iter().all(|filter| {
            let calculated = &item.plan.calculated;
            match filter.field.as_str() {
                NUMBER_OF_DAYS_WITH_EXPERT_THRESHOLD => matches_threshold(
                    filter,
                    calculated.number_of_days_with_expert_threshold.as_ref(),
                ),
                PRICING_WORKING_DAYS_COUNT_THRESHOLD => matches_threshold(
                    filter,
                    calculated.pricing_working_days_count_threshold.as_ref(),
                ),
                PRICING_PROCESS_COUNT => {
                    matches_number(filter, calculated.pricing_process_count)
                }
                START_PRIMARY_EXPERT_CONTROL_DATE => matches_date(
                    filter,
                    calculated.start_primary_expert_control_date.as_ref(),
                ),
                START_RECEIVED_DATE => {
                    matches_date(filter, calculated.start_received_date.as_ref())
                }
                START_DETERMINE_PRICE_DATE => matches_date(
                    filter,
                    calculated.start_determine_price_date.as_ref(),
                ),
                START_APPROVED_DATE => {
                    matches_date(filter, calculated.start_approved_date.as_ref())
                }
                _ => true, // Пропускаем фильтры, не относящиеся к расчетным полям
            }
        })
    });

    Ok(())
}

fn matches_threshold(filter: &Filter, threshold: Option<&ColorThreshold>) -> bool {
    threshold.map_or(false, |t| filter.matches_number(t.value as i64))
}

fn matches_number(filter: &Filter, value: Option<u16>) -> bool {
    value.map_or(false, |v| filter.matches_number(v as i64))
}

fn matches_date(filter: &Filter, date: Option<&AsezTimestamp>) -> bool {
    date.map_or(false, |d| filter.matches_date(d))
}
