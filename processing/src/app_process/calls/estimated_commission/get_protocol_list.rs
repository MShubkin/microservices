use std::sync::Arc;

use ahash::AHashSet;
use itertools::Itertools;

use asez2_shared_db::{
    db_item::{
        from_item_with_fields,
        joined::JoinTo,
        selection::{Filter, FilterTree},
        AsezDate, FieldTolerance, Select,
    },
    Value,
};
use shared_essential::{
    domain::{
        legacy::plans::PlanStatus, ContractAmendment, EcProtocol, EcProtocolItem,
        EcProtocolStatus,
        JoinedEcProtocolEcProtocolItemPlanContractAmendment as JoinedProtocol,
        JoinedEcProtocolEcProtocolItemPlanContractAmendmentSelector as JoinedProtocolSelector,
        Plan, PlanOrAmendment, ProtocolType, ResultId,
    },
    presentation::dto::{
        processing::{
            ColorScheme, ColorThreshold, GetProtocolListReq,
            GetProtocolListResponseData, GetProtocolListResponseItem,
        },
        response_request::{ApiResponse, Messages, PaginatedData, Status},
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

const GET_PROTOCOL_LIST: &str = "/rest/estimated_commission/v1/get/protocol_list";

const PROTOCOL_FIELDS: &[&str] = &[
    "protocol_id",
    EcProtocol::uuid,
    EcProtocol::registration_number,
    EcProtocol::protocol_date,
    EcProtocol::is_secret,
    EcProtocol::pricing_organization_unit_id,
    "protocol_status_id",
    EcProtocol::created_by,
    EcProtocol::changed_by,
    EcProtocol::created_at,
    EcProtocol::changed_at,
];

#[derive(Debug)]
struct ValidatableProtocol {
    protocol: EcProtocol,
    items: Vec<ValidatableProtocolItem>,
}

#[derive(Debug)]
struct ValidatableProtocolItem {
    protocol_item: EcProtocolItem,
    plan: PlanOrAmendment,
}

pub(crate) async fn get_protocol_list(
    dto: GetProtocolListReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetProtocolListResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Процессинг: Получение списка Протоколов СК ({get}): {req:?}\n",
        req = dto,
        get = GET_PROTOCOL_LIST
    );

    let protocol_select = dto.select.clone();

    let data = get_protocol_list_inner(dto, &db_pool).await?;

    let res = ApiResponse {
        status: Status::Ok,
        data: PaginatedData::new(&protocol_select, data),
        messages: Messages::default(),
        objects: vec![],
    };

    Ok(res)
}

pub(crate) async fn get_protocol_list_inner(
    dto: GetProtocolListReq,
    db_pool: &sqlx::PgPool,
) -> Result<Vec<GetProtocolListResponseItem>> {
    let protocol_type = dto.protocol_type_id;

    let (protocol_select, extra_validations) = normalize_protocol_select(dto)?;

    let joined_protocols =
        fetch_protocols(protocol_select.clone(), db_pool).await?;
    let validatable = construct_validatable(joined_protocols)?;
    let validated = validate(validatable, &extra_validations);

    finalise_response_data(validated, extra_validations, protocol_type)
}

fn validate(
    validateable: Vec<ValidatableProtocol>,
    extra_validations: &ExtraValidations,
) -> Vec<ValidatableProtocol> {
    let ExtraValidations {
        plan_customer_ids,
        plan_ids,
        ..
    } = extra_validations;

    if plan_customer_ids.is_some() || plan_ids.is_some() {
        validateable
            .into_iter()
            .filter(|protocol| {
                plan_customer_ids
                    .as_ref()
                    .map(|h| {
                        protocol
                            .items
                            .iter()
                            .any(|p| h.contains(&(*p.plan.customer_id() as i64)))
                    })
                    .unwrap_or(true)
                    && plan_ids
                        .as_ref()
                        .map(|h| {
                            protocol.items.iter().any(|p| h.contains(p.plan.id()))
                        })
                        .unwrap_or(true)
            })
            .collect()
    } else {
        validateable
    }
}

fn finalise_response_data(
    items: Vec<ValidatableProtocol>,
    extra_validations: ExtraValidations,
    protocol_type: ProtocolType,
) -> Result<Vec<GetProtocolListResponseItem>> {
    let ExtraValidations {
        protocol_item_quantity,
        protocol_item_d647_quantity,
        ..
    } = extra_validations;

    let from_protocol = from_item_with_fields(PROTOCOL_FIELDS);
    let res = items
        .into_iter()
        .map(|p| {
            let protocol = from_protocol(p.protocol);

            let protocol_item_quantity_threshold =
                protocol_item_quantity.then(|| calculate_field(&p.items, false));
            let protocol_item_d647_quantity_threshold =
                (protocol_item_d647_quantity
                    && protocol_type == ProtocolType::InPersonMeeting)
                    .then(|| calculate_field(&p.items, true));

            let item = GetProtocolListResponseItem {
                protocol,
                protocol_item_d647_quantity_threshold,
                protocol_item_quantity_threshold,
            };
            Ok(item)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(res)
}

/// Если во всех записях по ППЗ/ДС в Протоколе решение СК, которое анализируется, не совпадает с необходимым статусом ППЗ/ДС:
/// result_id = 1/Утверждено или 2/Согласовано с корректировкой стоимости и status_id по ППЗ/ДС = 140/Утверждено или 160/Цена определена (не закупка) (160)
/// result_id = 4/Аннулировать и status_id по ППЗ/ДС = 150/Аннулировано
/// то дополнительным параметром/color_scheme_id к кол-ву передавать значение 3.
///
/// Если хоть в одной записи решение СК совпадает с необходимым статусом ППЗ/ДС, то дополнительным параметром/color_scheme_id к кол-ву передавать значение 2.
///
/// Если во всех записях решение СК совпадает с необходимым статусом ППЗ/ДС, то дополнительным параметром/color_scheme_id к кол-ву передавать значение 1.
fn calculate_field(
    items: &[ValidatableProtocolItem],
    is_registered_by_d647: bool,
) -> ColorThreshold {
    let mut matches = 0;
    let mut items_count = 0;

    items
        .iter()
        .filter(|item| {
            item.protocol_item.is_registered_by_d647 == is_registered_by_d647
        })
        .for_each(|item| {
            items_count += 1;
            match (item.protocol_item.result_id, *item.plan.status_id()) {
                (
                    ResultId::Approved | ResultId::AgreedWithPriceCorrection,
                    PlanStatus::PriceConfirmed | PlanStatus::PriceDetermined,
                )
                | (ResultId::Cancel, PlanStatus::PlanCancelled) => {
                    matches += 1;
                }
                _ => {}
            }
        });

    let color_scheme_id = match matches {
        n if items_count == n => ColorScheme::Green,
        0 => ColorScheme::Red,
        _ => ColorScheme::Yellow,
    };

    ColorThreshold {
        value: items_count,
        color_scheme_id,
    }
}

fn construct_validatable(
    joined_protocols: Vec<JoinedProtocol>,
) -> Result<Vec<ValidatableProtocol>> {
    joined_protocols
        .into_iter()
        .map(|joined_protocol| {
            let mut plans = PlanOrAmendment::collect_map_by_uuid(joined_protocol.plans,joined_protocol.amendments);

            let items = joined_protocol.items
                .into_iter()
                            // distinct не так хорошо работает
                .unique_by(|i| i.uuid)
                .map(|protocol_item| {
                    let plan = plans.remove(&protocol_item.source_uuid)
                        .ok_or(ProcessingError::GetItemList(format!("Нарушение консистентности базы данных. Элемент Протокола СК {} не имеет смежного ППЗ/ДС", protocol_item.uuid)))?;

                    Ok(ValidatableProtocolItem {
                        protocol_item,
                        plan,
                    })
                }).collect::<Result<Vec<_>>>()?;

            Ok(ValidatableProtocol {
                protocol: joined_protocol.protocol,
                items,
            })
        })
        .collect::<Result<Vec<_>>>()
}

async fn fetch_protocols(
    normalized_select: Select,
    db_pool: &sqlx::PgPool,
) -> Result<Vec<JoinedProtocol>> {
    let mut protocol_select = normalized_select;
    let protocol_orderings = std::mem::take(&mut protocol_select.order_list);

    let protocol_item_select =
        Select::full::<EcProtocolItem>().eq(EcProtocolItem::is_removed, false);
    let plan_select =
        Select::with_fields([Plan::id, Plan::customer_id, Plan::status_id]);

    let mut joined_select = JoinedProtocolSelector::new(protocol_select)
        .set_items(EcProtocolItem::join_default().selecting(protocol_item_select))
        .set_plans(Plan::join_default().selecting(plan_select.clone()))
        .set_amendments(ContractAmendment::join_default().selecting(plan_select));
    for ordering in protocol_orderings {
        joined_select = joined_select.add_order(
            EcProtocol::actual_fieldname(&ordering.field),
            ordering.order,
        );
    }

    let joined_protocols = joined_select.get(db_pool).await?;

    Ok(joined_protocols)
}

struct ExtraValidations {
    /// Фильтр Протоколов, у которых смежные ППЗ/ДС
    /// имеют нужный customer_id
    plan_customer_ids: Option<AHashSet<i64>>,
    /// Фильтр Протоколов, у которых смежные ППЗ/ДС
    /// имеют нужный id
    plan_ids: Option<AHashSet<i64>>,
    /// Нужно ли вернуть calculated поле по protocol_item
    /// с is_registered_by_d647 == false
    protocol_item_quantity: bool,
    /// Нужно ли вернуть calculated поле по protocol_item
    /// с is_registered_by_d647 == true
    protocol_item_d647_quantity: bool,
}

/// В запросе приходят неизвестные для БД поля, поэтому их придется
/// нормализовать
fn normalize_protocol_select(
    dto: GetProtocolListReq,
) -> Result<(Select, ExtraValidations)> {
    let GetProtocolListReq { mut select, .. } = dto;

    let mut extra_validations = ExtraValidations {
        plan_customer_ids: None,
        plan_ids: None,
        protocol_item_quantity: false,
        protocol_item_d647_quantity: false,
    };

    let mut new_column_list = Vec::with_capacity(select.field_list.len());
    for field in select.field_list {
        match field.as_str() {
            "protocol_item_quantity_threshold" => {
                extra_validations.protocol_item_quantity = true
            }
            "protocol_item_d647_quantity_threshold" => {
                extra_validations.protocol_item_d647_quantity = true
            }
            _ => new_column_list.push(field),
        }
    }

    let mut cleared_filter_tree = Vec::new();
    let mut has_status_500_filter = false;

    for filter in select.filter_list.into_filters() {
        match filter.field.as_str() {
            "protocol_date_year" => {
                let dates = filter.values
                    .iter()
                    .map(|v| {
                        if let Value::Int(year) = v {
                            let year: i32 = (*year).try_into().map_err(|_| {
                                ProcessingError::GetItemList(String::from(
                                    "Невалидное значение для `protocol_date_year` фильтра",
                                ))
                            })?;

                            // Является проверкой для всех остальных
                            let date =
                                AsezDate::try_from_yo(year, 1).map_err(|_| {
                                    ProcessingError::GetItemList(String::from(
                                        "Невалидное значение для `protocol_date_year` фильтра",
                                    ))
                                })?;

                            Ok(date)
                        } else {
                            Err(ProcessingError::GetItemList(String::from(
                                "Невалидное значение для `protocol_date_year` фильтра",
                            )))
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;

                let mut date_filters = Vec::with_capacity(dates.len());
                for date in dates {
                    let (lower_date, upper_date) = (
                        date,
                        AsezDate::try_from_yo(date.year(), 365)
                            .expect("Проверено на уровне `dates`"),
                    );
                    date_filters.push(
                        Filter::between(
                            EcProtocol::protocol_date,
                            lower_date,
                            upper_date,
                        )
                        .into(),
                    );
                }
                cleared_filter_tree.push(FilterTree::Or(date_filters));
            }
            "protocol_status_id" => {
                has_status_500_filter = filter.values.iter().any(|v| match v {
                    Value::Int(status) => {
                        *status == EcProtocolStatus::Deleted as i64
                    }
                    _ => false,
                }) || has_status_500_filter;

                cleared_filter_tree.push(filter.into());
            }
            field @ "plan_id" | field @ Plan::customer_id => {
                let values = filter
                    .values
                    .iter()
                    .map(|v| match v {
                        Value::Int(int) => Ok(*int),
                        _ => Err(ProcessingError::GetItemList(format!(
                            "Невалидное значение для `{}` фильтра",
                            field
                        ))),
                    })
                    .collect::<Result<AHashSet<_>>>()?;
                if field == "plan_id" {
                    extra_validations.plan_ids = Some(values);
                } else {
                    extra_validations.plan_customer_ids = Some(values);
                }
            }
            _ => cleared_filter_tree.push(filter.into()),
        }
    }

    select.field_list = new_column_list;
    select.filter_list = FilterTree::And(cleared_filter_tree);

    if !has_status_500_filter {
        select = select.eq(EcProtocol::is_removed, false);
    }
    select = select.eq(EcProtocol::protocol_type_id, dto.protocol_type_id as i16);

    Ok((select, extra_validations))
}
