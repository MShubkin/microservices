use std::sync::Arc;

use ahash::AHashSet;

use itertools::Itertools;
use sqlx::PgPool;

use asez2_shared_db::{
    db_item::{
        from_item_with_fields,
        joined::JoinTo,
        selection::{Filter, FilterTree},
        AsezDate, FieldTolerance, Select,
    },
    DbAdaptor, Value,
};
use shared_essential::{
    domain::{
        ContractAmendment, EcAgenda, EcAgendaItem, EcAgendaRep, EcAgendaStatus,
        EcProtocol,
        JoinedEcAgendaEcAgendaItemPlanContractAmendmentRelAgendaProtocolEcProtocol as JoinedAgenda,
        JoinedEcAgendaEcAgendaItemPlanContractAmendmentRelAgendaProtocolEcProtocolSelector as JoinedAgendaSelect,
        Plan, PlanOrAmendment, RelAgendaProtocol,
    },
    presentation::dto::{
        processing::{GetAgendaListItem, GetAgendaListReq, GetAgendaListResponse},
        response_request::{ApiResponse, PaginatedData, Status},
    },
};

use crate::{
    app_process::common,
    common::{ProcessingError, Result},
};

const GET_AGENDA_LIST: &str = "/rest/estimated_commission/v1/get/agenda_list";

#[derive(Debug)]
struct ExtraRequestParts {
    /// Нужны ли данные по agenda_item с is_registered_by_d647=true
    d647_item_threshold: bool,
    /// Нужны ли данные по agenda_item с is_registered_by_d647=false
    item_threshold: bool,
    /// Нужно ли количество протоколов, которые связаны по
    /// agenda_protocol_relation - agenda_uuid = agenda - uuid
    protocol_quantity: bool,
    /// Доп фильтр по supplier_id для plan
    plan_supplier_id_filter: AHashSet<i32>,
    /// Доп фильтр по customer_id для plan
    plan_customer_id_filter: AHashSet<i32>,
    /// Доп фильтр по id для plan
    plan_id_filter: AHashSet<i64>,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_agenda_list(
    request: GetAgendaListReq,
    db_pool: Arc<PgPool>,
) -> Result<GetAgendaListResponse> {
    tracing::info!(
        kind = "get",
        "Получен запрос на получение списка Повесток СК ({get}): {req:?}\n",
        req = request,
        get = GET_AGENDA_LIST
    );
    let agenda_select = request.select.clone();

    let response_items = get_agenda_list_inner(request, &db_pool).await?;

    Ok(ApiResponse {
        data: PaginatedData::new(&agenda_select, response_items),
        status: Status::Ok,
        ..Default::default()
    })
}

pub(crate) async fn get_agenda_list_inner(
    request: GetAgendaListReq,
    db_pool: &sqlx::PgPool,
) -> Result<Vec<GetAgendaListItem>> {
    let (actual_select, extra) = normalize_select(request)?;

    if extra.has_extra() {
        get_agenda_with_extra_info(actual_select.clone(), extra, db_pool).await
    } else {
        get_agenda_simple(actual_select.clone(), db_pool).await
    }
}

async fn get_agenda_simple(
    select: Select,
    db_pool: &PgPool,
) -> Result<Vec<GetAgendaListItem>> {
    let agendas: Vec<_> = EcAgendaRep::select(&select, db_pool).await?;

    if agendas.is_empty() {
        return Ok(Vec::new());
    }

    agendas
        .into_iter()
        .map(|item| {
            Ok(GetAgendaListItem {
                agenda: item,
                agenda_item_quantity_threshold: None,
                agenda_item_d647_quantity_threshold: None,
                protocol_quantity: None,
            })
        })
        .collect::<Result<Vec<_>>>()
}

/// Эта выборка применяется, если пользователь запросил хотя бы одно
/// специальное поле, которое никак не относитс к [`EcAgenda`]
async fn get_agenda_with_extra_info(
    agenda_select: Select,
    extra: ExtraRequestParts,
    db_pool: &PgPool,
) -> Result<Vec<GetAgendaListItem>> {
    let agenda_fields = agenda_select.field_list.clone();

    let agendas = fetch_joined_agendas(agenda_select, db_pool).await?;
    if agendas.is_empty() {
        return Ok(Vec::new());
    }

    let mut result_items = Vec::with_capacity(agendas.len());
    let from_agenda = from_item_with_fields(&agenda_fields);

    for agenda in agendas {
        if extra.has_plan_filters() {
            if extra.has_plan_customer_id_filter()
                && !agenda.plans.iter().any(|p| {
                    extra.plan_customer_id_filter.contains(p.customer_id())
                })
            {
                continue;
            }

            if extra.has_plan_supplier_id_filter()
                && !agenda.plans.iter().any(|p| {
                    extra.plan_supplier_id_filter.contains(p.supplier_id())
                })
            {
                continue;
            }

            if extra.has_plan_id_filter()
                && !agenda
                    .plans
                    .iter()
                    .any(|p| extra.plan_id_filter.contains(p.id()))
            {
                continue;
            }
        }

        let (agenda_item_quantity_threshold, agenda_item_d647_quantity_threshold) =
            if extra.has_agenda_item_extra() {
                let has_color = !matches!(
                    agenda.agenda.status_id,
                    EcAgendaStatus::ProtocolFormed | EcAgendaStatus::Deleted
                );

                let (quantity_threshold, d647_quantity_threshold) =
                    common::agenda::calculate_quantity_thresholds(
                        &agenda.plans,
                        &agenda.agenda_items,
                        has_color,
                    )?;

                (
                    extra.item_threshold.then_some(quantity_threshold),
                    extra.d647_item_threshold.then_some(d647_quantity_threshold),
                )
            } else {
                (None, None)
            };

        let protocol_quantity =
            extra.protocol_quantity.then_some(agenda.protocol_quantity);

        let agenda = from_agenda(agenda.agenda);
        let result_item = GetAgendaListItem {
            agenda,
            agenda_item_d647_quantity_threshold,
            agenda_item_quantity_threshold,
            protocol_quantity,
        };
        result_items.push(result_item);
    }

    Ok(result_items)
}

struct ValidatableAgenda {
    agenda: EcAgenda,
    agenda_items: Vec<EcAgendaItem>,
    plans: Vec<PlanOrAmendment>,
    protocol_quantity: usize,
}

async fn fetch_joined_agendas(
    mut agenda_select: Select,
    db_pool: &PgPool,
) -> Result<Vec<ValidatableAgenda>> {
    let agenda_orderings = std::mem::take(&mut agenda_select.order_list);

    let agenda_item_select = Select::with_fields([EcAgendaItem::uuid])
        .eq(EcAgendaItem::is_removed, false);
    let plan_select =
        Select::with_fields([Plan::id, Plan::customer_id, Plan::supplier_id]);
    let protocol_rel_select = Select::full::<RelAgendaProtocol>();
    let protocol_select = Select::with_fields([
        EcProtocol::id,
        EcProtocol::registration_number,
        EcProtocol::protocol_date,
        EcProtocol::status_id,
    ]);

    let mut joined_select = JoinedAgendaSelect::new(agenda_select)
        .set_plans(Plan::join_default().selecting(plan_select.clone()))
        .set_amendments(
            ContractAmendment::join_default().selecting(plan_select.clone()),
        )
        .set_agenda_items(
            EcAgendaItem::join_default().selecting(agenda_item_select),
        )
        .set_protocol_rel(
            RelAgendaProtocol::join_default().selecting(protocol_rel_select),
        )
        .set_protocols(EcProtocol::join_default().selecting(protocol_select));
    for ordering in agenda_orderings {
        joined_select = joined_select
            .add_order(EcAgenda::actual_fieldname(&ordering.field), ordering.order);
    }

    let joined_agendas = joined_select.get(db_pool).await?;

    // TODO: селект возвращает почему то дубликаты даже при distinct
    let joined_agendas = joined_agendas
        .into_iter()
        .map(|j| {
            let JoinedAgenda {
                agenda,
                agenda_items,
                plans,
                amendments,
                protocols,
                ..
            } = j;

            let agenda_items =
                agenda_items.into_iter().unique_by(|i| i.uuid).collect();
            let plans = amendments
                .into_iter()
                .map(PlanOrAmendment::from)
                .chain(plans.into_iter().map(PlanOrAmendment::from))
                .unique_by(|i| *i.uuid())
                .collect();
            let protocol_quantity =
                protocols.into_iter().unique_by(|i| i.uuid).count();

            ValidatableAgenda {
                agenda,
                agenda_items,
                plans,
                protocol_quantity,
            }
        })
        .collect();

    Ok(joined_agendas)
}

fn normalize_select(req: GetAgendaListReq) -> Result<(Select, ExtraRequestParts)> {
    let GetAgendaListReq {
        section_id: _,
        mut select,
    } = req;

    let mut extra_fields = ExtraRequestParts {
        d647_item_threshold: false,
        item_threshold: false,
        protocol_quantity: false,
        plan_supplier_id_filter: AHashSet::new(),
        plan_customer_id_filter: AHashSet::new(),
        plan_id_filter: AHashSet::new(),
    };

    let mut cleared_field_list = Vec::with_capacity(select.field_list.len());
    for field in select.field_list {
        match field.as_str() {
            "agenda_item_quantity_threshold" => extra_fields.item_threshold = true,
            "agenda_item_d647_quantity_threshold" => {
                extra_fields.d647_item_threshold = true
            }
            "protocol_quantity" => extra_fields.protocol_quantity = true,
            _ => cleared_field_list.push(field),
        }
    }

    let mut cleared_filter_tree: Vec<FilterTree> = Vec::new();
    let mut has_deleted_status_filter = false;

    for filter in select.filter_list.into_filters() {
        match filter.field.as_str() {
            "agenda_status_id" | EcAgenda::status_id => {
                has_deleted_status_filter = filter.values.iter().any(|v| {
                    if let Value::Int(s) = v {
                        *s == EcAgendaStatus::Deleted as i16 as i64
                    } else {
                        false
                    }
                }) || has_deleted_status_filter;

                cleared_filter_tree.push(filter.into())
            }
            "meeting_date_year" => {
                let dates = filter.values
                    .iter()
                    .map(|v| {
                        if let Value::Int(year) = v {
                            let year: i32 = (*year).try_into().map_err(|_| {
                                ProcessingError::GetAgendaList(String::from(
                                    "Невалидное значения для `meeting_date_year` фильтра",
                                ))
                            })?;

                            // Является проверкой для всех остальных
                            let date =
                                AsezDate::try_from_yo(year, 1).map_err(|_| {
                                    ProcessingError::GetAgendaList(String::from(
                                        "Невалидное значения для `meeting_date_year` фильтра",
                                    ))
                                })?;

                            Ok(date)
                        } else {
                            Err(ProcessingError::GetAgendaList(String::from(
                                "Невалидное значения для `meeting_date_year` фильтра",
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
                            EcAgenda::meeting_date,
                            lower_date,
                            upper_date,
                        )
                        .into(),
                    );
                }
                cleared_filter_tree.push(FilterTree::Or(date_filters));
            }
            "plan_id" => {
                extra_fields.plan_id_filter = filter_vals_to_ints(filter)?;
            }
            Plan::customer_id => {
                extra_fields.plan_customer_id_filter = filter_vals_to_ints(filter)?;
            }
            Plan::supplier_id => {
                extra_fields.plan_supplier_id_filter = filter_vals_to_ints(filter)?;
            }
            _ => cleared_filter_tree.push(filter.into()),
        }
    }

    select.field_list = cleared_field_list;
    select.filter_list = FilterTree::And(cleared_filter_tree);

    if !has_deleted_status_filter {
        select.filter_list =
            select.filter_list.and(Filter::eq(EcAgenda::is_removed, false).into());
    }

    Ok((select, extra_fields))
}

fn filter_vals_to_ints<T>(filter: Filter) -> Result<AHashSet<T>>
where
    T: TryFrom<i64> + Eq + std::hash::Hash,
{
    if filter.values.is_empty() {
        return Ok(AHashSet::new());
    }

    let vals = &filter.values;
    let field_name = &filter.field;

    vals.iter()
        .map(|v| {
            if let Value::Int(int) = v {
                let int_val: T = (*int).try_into().map_err(|_| {
                    ProcessingError::GetAgendaList(format!(
                        "Невалидное значения для `{}` фильтра", field_name
                    ))
                })?;
                Ok(int_val)
            } else {
                Err(ProcessingError::GetAgendaList(format!("Невалидное значения для `{}` фильтра. Ожидается числовое значение", field_name)))
            }
        })
        .collect()
}

impl ExtraRequestParts {
    fn has_extra(&self) -> bool {
        self.has_agenda_item_extra()
            || self.has_plan_filters()
            || self.has_protocol_extra()
    }

    fn has_protocol_extra(&self) -> bool {
        self.protocol_quantity
    }

    fn has_agenda_item_extra(&self) -> bool {
        self.d647_item_threshold || self.item_threshold
    }

    fn has_plan_filters(&self) -> bool {
        self.has_plan_id_filter()
            || self.has_plan_customer_id_filter()
            || self.has_plan_supplier_id_filter()
    }

    fn has_plan_customer_id_filter(&self) -> bool {
        !self.plan_customer_id_filter.is_empty()
    }

    fn has_plan_supplier_id_filter(&self) -> bool {
        !self.plan_supplier_id_filter.is_empty()
    }

    fn has_plan_id_filter(&self) -> bool {
        !self.plan_id_filter.is_empty()
    }
}
