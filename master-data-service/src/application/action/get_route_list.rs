use ahash::AHashSet;
use asez2_shared_db::{
    db_item::{
        joined::JoinTo, selection::filters::convert_filters_equal_to_value,
        DbAdaptorFieldMask, FieldTolerance, Filter, FilterTree, Select,
    },
    DbAdaptor, Value,
};
use shared_essential::{
    domain::{routes::*, ObjectTypeId},
    presentation::dto::{
        master_data::{
            error::{MasterDataError, MasterDataResult},
            request::RouteListReq,
            response::*,
            routes::{try_into_route_criteria, RouteCriterion},
        },
        response_request::{ApiResponse, MessageKind, Messages, PaginatedData},
    },
};
use sqlx::PgPool;

use crate::application::routes::{route_predicate, EvalContext};

const ROUTE_HEADER_FIELDS: &[&str] = &[
    RouteHeader::type_id,
    RouteHeader::uuid,
    RouteHeader::id,
    RouteHeader::name_short,
    RouteHeader::is_exception,
    RouteHeader::is_removed,
    RouteHeader::is_active,
    RouteHeader::created_at,
    RouteHeader::created_by,
    RouteHeader::changed_at,
    RouteHeader::changed_by,
];

const MANDATORY_CRITERIA: &[&str] = &["object_type_id"];

/// Функция получение маршрутов согласования в разрезе Департамента
///
/// # Аргументы
/// - `dto`: Параметры запроса, включая секцию, пользователя и фильтры.
/// - `pool`: Пул соединений с БД.
///
/// # Возвращает
/// - `ApiResponse` с данными о маршрутах или сообщением об ошибке.
///
/// TODO: добавить фильтрацию и сортировку
pub async fn get_route_list(
    dto: RouteListReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteListResponse, ()>> {
    let RouteListReq {
        select, type_id, ..
    } = dto;

    let (
        selector,
        Select {
            field_list: crit_fields,
            filter_list: mut crit_filters,
            ..
        },
    ) = prepare_select(select.clone(), type_id);

    let (is_plan, is_ca) = is_plan_ca_from_filters(&mut crit_filters)?;

    let routes = selector.get(pool).await?;

    let ctx = EvalContext::create().await?;
    let mut messages = Messages::default();
    let mut messages2 = Messages::default();

    let values = criteria_filtering::filters_to_values(crit_filters)?;
    let predicate = route_predicate(&values, ctx, &mut messages);
    let mask = DbAdaptorFieldMask::with_fields_and_pkeys(ROUTE_HEADER_FIELDS);

    let data = routes
        .into_iter()
        .filter(predicate)
        .filter_map(|route| {
            into_route_item(
                route,
                &crit_fields,
                is_plan,
                is_ca,
                &mask,
                &mut messages2,
            )
            .transpose()
        })
        .collect::<Result<Vec<_>, MasterDataError>>()?;
    let data = PaginatedData::new(&select, data);

    messages.add_messages(messages2);

    Ok((data, messages).into())
}

mod criteria_filtering;

fn into_route_item(
    route: RouteFull,
    crit_list: &[String],
    is_plan_filter: Option<bool>,
    is_contract_amendment_filter: Option<bool>,
    mask: &DbAdaptorFieldMask<RouteHeaderRep>,
    messages: &mut Messages,
) -> MasterDataResult<Option<RouteItem>> {
    let RouteFull { route, crits, data } = route;
    let header = RouteHeaderRep::from_item_masked(route, mask);
    let (is_plan, is_contract_amendment) = is_plan_ca_from_crits(&crits);
    if !(is_plan_filter.map_or(true, |v| is_plan == v)
        && is_contract_amendment_filter
            .map_or(true, |v| is_contract_amendment == v))
    {
        return Ok(None);
    }
    let crit_set = crit_list.iter().collect::<AHashSet<_>>();
    let criteria = crits
        .into_iter()
        .filter_map(|crit| {
            if MANDATORY_CRITERIA.contains(&crit.field_name.as_str())
                || crit_set.contains(&crit.field_name)
            {
                into_criteria_list(crit, messages)
            } else {
                None
            }
        })
        .collect();
    let data = data.data.0;
    Ok(Some(RouteItem {
        header,
        criteria,
        is_plan,
        is_contract_amendment,
        data,
    }))
}

fn is_plan_ca_from_filters(
    filter_list: &mut FilterTree,
) -> MasterDataResult<(Option<bool>, Option<bool>)> {
    let is_plan = convert_equal_bool_to_value(
        "is_plan",
        &filter_list.remove_by_field("is_plan"),
    )?;
    let is_contract_amendment = convert_equal_bool_to_value(
        "is_contract_amendment",
        &filter_list.remove_by_field("is_contract_amendment"),
    )?;
    Ok((is_plan, is_contract_amendment))
}

fn is_plan_ca_from_crits(crits: &[RouteCrit]) -> (bool, bool) {
    let is_plan = |value: &_| matches!(value, CritValue::Int(x) if *x == ObjectTypeId::Plan as i64);
    let is_ca = |value: &_| matches!(value, CritValue::Int(x) if *x == ObjectTypeId::ContractAmendment as i64);
    crits
        .iter()
        .find_map(|crit| {
            if &crit.field_name == "object_type_id" {
                match &crit.predicate.0 {
                    CritPredicate::Equal { value } => {
                        Some((is_plan(value), is_ca(value)))
                    }
                    CritPredicate::In { values } => {
                        Some(values.iter().fold((false, false), |(p, ca), v| {
                            (p || is_plan(v), ca || is_ca(v))
                        }))
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn into_criteria_list(
    crit: RouteCrit,
    messages: &mut Messages,
) -> Option<(String, Vec<RouteCriterion>)> {
    match try_into_route_criteria(crit.predicate.0) {
        Ok(v) => Some((crit.field_name, v)),
        Err(err) => {
            messages.add_message(
                MessageKind::Error,
                format!(
                    "Неподдерживаемый критерий для поля {}: {err}",
                    crit.field_name
                ),
            );
            None
        }
    }
}

fn prepare_select(
    mut input: Select,
    type_id: RouteApprType,
) -> (RouteFullSelector, Select) {
    // Фронт присылает фильтр, в котором вперемешку используются:
    // - поля заголовка маршрута
    // - поля данных маршрута
    // - именя критериев
    //
    // Нам надо разделить все это многообразие по индивидуальным Select'ам,
    // чтобы скормить их потом в JoinSelector, а Select с оставшимися полями
    // использовать для фильтрации по критериям.
    RouteHeader::apply_tolerance_to_select(&mut input);
    let (header_select, rest) = input.split_for::<RouteHeaderRep>();
    let (data_select, rest) = rest.split_for::<RouteDataRep>();
    let (crit_select, rest) = rest.split_for::<RouteCritRep>();

    let header_select = header_select
        .eq(RouteHeader::type_id, type_id)
        .eq(RouteHeader::is_removed, false);

    let crit_select = crit_select.eq(RouteCrit::is_removed, false);

    let selector = RouteFullSelector::new_with_order(header_select)
        .set_data(RouteData::join_default().selecting(data_select))
        .set_crits(
            RouteCrit::join_default().selecting(crit_select).distinct_aggr(true),
        );
    (selector, rest)
}

fn convert_equal_bool_to_value(
    name: &str,
    filters: &[Filter],
) -> MasterDataResult<Option<bool>> {
    if filters.is_empty() {
        return Ok(None);
    }
    match convert_filters_equal_to_value(filters) {
        Ok(Value::Bool(b)) => Ok(Some(*b)),
        Ok(v) => Err(MasterDataError::InternalError(format!(
            "неверное значение в фильтре `{name}`: `{v:?}`"
        ))),
        Err(e) => Err(MasterDataError::InternalError(format!(
            "неверный фильтр `{name}`: `{e}`"
        ))),
    }
}
