//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
mod calculated_fields;
mod meta;

use asez2_shared_db::db_item::{
    from_item_with_fields, selection::FieldSortKind, Select,
};
use rabbit_services::specialized_departments::SpecializedDepartmentsService;
use sqlx::PgPool;
use std::{cmp::Ordering, sync::Arc};

use shared_essential::{
    common::maps::map_2,
    domain::{
        EcAgenda, EcAgendaItem, EcProtocol, EcProtocolItem, Plan, PlanOrAmendment,
        PlanOrAmendmentRep, Section,
    },
    presentation::dto::{
        general::Metadata,
        processing::{
            calculated_fields::*, GetPlansCalculatedItem, GetPlansResponse,
            PlansRequest,
        },
        response_request::{ApiResponse, Messages, PaginatedData},
    },
};

use crate::app_process::sections::*;
use crate::common::Result;

use calculated_fields::add_calculated_fields;
use meta::{
    fill_meta_field, insert_additional_fields,
    request_non_competitive_plans_with_last_agenda,
};

use self::{mapping::SectionMap, table::EntityType};

const GET_PLANS: &str = "/v1/get_plans";

/// Промежуточное представление данных при вычислениях
#[derive(Debug)]
pub struct GetPlansItem {
    pub plan: Calculated<PlanOrAmendment>,
    pub agenda: Option<EcAgenda>,
    pub protocol: Option<EcProtocol>,
    pub agenda_item: Option<EcAgendaItem>,
    pub protocol_item: Option<Calculated<EcProtocolItem>>,
    pub _meta: Option<Metadata>,
}

/// This is the actual function.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_plans(
    mut req: PlansRequest,
    db_pool: Arc<PgPool>,
    spec_deps: SpecializedDepartmentsService,
) -> Result<GetPlansResponse> {
    let mut messages = Messages::default();
    let db_conn = db_pool.as_ref();

    tracing::info!(
        kind = "get",
        "Получение ППЗ/ДС ({get}): {req:?}",
        req = req,
        get = GET_PLANS
    );

    let select_copy = req.select.clone();
    req.select.clear_pagination();
    let has_calculated_fields = has_calculated_field(&req.select.field_list);

    // Необходимо для расчетных полей
    if has_calculated_fields {
        req.select.field_list.push(Plan::status_id.to_string());
        req.select.field_list.push(Plan::pricing_started_at.to_string());
        req.select.field_list.push(Plan::check_documentation_date.to_string());
    }
    // Необходимо для meta
    let transient_fields =
        insert_additional_fields(&req.section, &mut req.select.field_list);

    let section_data = process_sections(req.clone(), db_conn).await?;

    let mut items = section_data
        .data
        .into_iter()
        .map(Into::into)
        .collect::<Vec<GetPlansItem>>();

    // Задействован механизм расчётных полей.
    if has_calculated_fields {
        add_calculated_fields(
            &mut items,
            db_conn,
            &spec_deps,
            &req.select,
            &req.section,
        )
        .await?
    };

    // Заполняем поле meta
    let agenda_items = request_non_competitive_plans_with_last_agenda(
        &req.section,
        items.iter().map(|i| &i.plan.item),
        db_pool.clone(),
    )
    .await?;

    let additional_messages =
        fill_meta_field(&req.section, &mut items, agenda_items, transient_fields)
            .await?;
    messages.add_messages(additional_messages);

    finalise_response(
        items,
        section_data.select_info,
        &select_copy,
        messages,
        req.section,
    )
}

fn finalise_response(
    data: Vec<GetPlansItem>,
    select_info: SectionSelectInfo,
    select: &Select,
    messages: Messages,
    section: Section,
) -> Result<GetPlansResponse> {
    let from_item = PlanOrAmendmentRep::from_item_with_fields_split(
        &select_info.plan_request_fields,
        &select_info.amendment_request_fields,
    );

    macro_rules! from_with_extra {
        ($kind:expr) => {
            select_info
                .extra_fields
                .as_ref()
                .and_then(|f| f.get($kind))
                .map(|f| from_item_with_fields(f))
        };
    }

    let from_agenda = from_with_extra!(EntityType::Agenda);
    let from_proto = from_with_extra!(EntityType::Protocol);
    let from_agenda_item = from_with_extra!(EntityType::AgendaItem);
    let from_proto_item = from_with_extra!(EntityType::ProtocolItem);

    let mut data = data
        .into_iter()
        .map(|i| {
            let GetPlansItem {
                plan,
                agenda,
                agenda_item,
                protocol,
                protocol_item,
                _meta,
            } = i;

            let plan: Calculated<PlanOrAmendmentRep> = {
                let plan_item = from_item(plan.item);

                let fields = if plan_item.is_amendment() {
                    &select_info.amendment_request_fields
                } else {
                    &select_info.plan_request_fields
                };
                Calculated {
                    item: plan_item.apply_section_mappings(section.kind(), fields),
                    calculated: plan.calculated,
                }
            };

            if select_info.extra_fields.is_none() {
                return Ok(GetPlansCalculatedItem {
                    plan,
                    _meta,
                    ..Default::default()
                });
            }

            let protocol =
                map_2(protocol, from_proto.as_ref(), |protocol, from| {
                    from(protocol)
                });
            let agenda =
                map_2(agenda, from_agenda.as_ref(), |agenda, from| from(agenda));
            let protocol_item = map_2(
                protocol_item,
                from_proto_item.as_ref(),
                Calculated::map_item,
            );
            let agenda_item = map_2(
                agenda_item,
                from_agenda_item.as_ref(),
                |agenda_item, from| from(agenda_item),
            );

            Ok(GetPlansCalculatedItem {
                agenda,
                protocol,
                agenda_item,
                protocol_item,
                plan,
                _meta,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    sort(&select_info, &mut data);
    let data = PaginatedData::new(select, data);

    Ok(ApiResponse::from((data, messages)))
}

fn sort(select_info: &SectionSelectInfo, data: &mut [GetPlansCalculatedItem]) {
    macro_rules! sort_branch {
        ($a: expr, $b: expr, $ordering: ident) => {{
            match $ordering.order {
                FieldSortKind::Asc => {
                    $a.field(&$ordering.field).cmp(&$b.field(&$ordering.field))
                }
                FieldSortKind::Desc => {
                    $b.field(&$ordering.field).cmp(&$a.field(&$ordering.field))
                }
            }
        }};
        ($ty: ty, $a: expr, $b: expr, $ordering: ident) => {{
            match (&$a, &$b) {
                (None, None) => Ordering::Equal,
                (Some(_), None) => match $ordering.order {
                    FieldSortKind::Asc => Ordering::Greater,
                    FieldSortKind::Desc => Ordering::Less,
                },
                (None, Some(_)) => match $ordering.order {
                    FieldSortKind::Asc => Ordering::Less,
                    FieldSortKind::Desc => Ordering::Greater,
                },
                (Some(item1), Some(item2)) => match $ordering.order {
                    FieldSortKind::Asc => item1
                        .field(&$ordering.field)
                        .cmp(&item2.field(&$ordering.field)),
                    FieldSortKind::Desc => item2
                        .field(&$ordering.field)
                        .cmp(&item1.field(&$ordering.field)),
                },
            }
        }};
    }
    // Сортировка ведется именно в обратном порядке, так высшие уровни сортировки идут первыми
    for (ty, ordering) in select_info.orderings.iter().rev() {
        data.sort_by(|a, b| {
            // По факту здесь всегда будет использовано только валидное имя поля,
            // так как в process_sections уже есть проверки на это
            match ty {
                EntityType::Plan | EntityType::ContractAmendment => {
                    match (
                        a.plan.calculated.field(&ordering.field),
                        b.plan.calculated.field(&ordering.field),
                    ) {
                        (
                            Some(CalculatedPartField::OptionAsezTimestamp(a_ts)),
                            Some(CalculatedPartField::OptionAsezTimestamp(b_ts)),
                        ) => match ordering.order {
                            FieldSortKind::Asc => a_ts.cmp(b_ts),
                            FieldSortKind::Desc => b_ts.cmp(a_ts),
                        },
                        (
                            Some(CalculatedPartField::OptionColorThreshold(a_thr)),
                            Some(CalculatedPartField::OptionColorThreshold(b_thr)),
                        ) => {
                            let a_val = a_thr.as_ref().map(|t| t.value);
                            let b_val = b_thr.as_ref().map(|t| t.value);
                            match ordering.order {
                                FieldSortKind::Asc => a_val.cmp(&b_val),
                                FieldSortKind::Desc => b_val.cmp(&a_val),
                            }
                        }
                        _ => sort_branch!(a.plan.item, b.plan.item, ordering),
                    }
                }
                EntityType::Protocol => {
                    sort_branch!(
                        EcProtocol,
                        a.protocol.as_ref(),
                        b.protocol.as_ref(),
                        ordering
                    )
                }
                EntityType::Agenda => {
                    sort_branch!(
                        EcAgenda,
                        a.agenda.as_ref(),
                        b.agenda.as_ref(),
                        ordering
                    )
                }
                EntityType::AgendaItem => {
                    sort_branch!(
                        EcAgendaItem,
                        a.agenda_item.as_ref(),
                        b.agenda_item.as_ref(),
                        ordering
                    )
                }
                EntityType::ProtocolItem => {
                    sort_branch!(
                        EcProtocolItem,
                        a.protocol_item.as_ref().map(|p| &p.item),
                        b.protocol_item.as_ref().map(|p| &p.item),
                        ordering
                    )
                }
            }
        })
    }
}

impl From<SectionDataItem> for GetPlansItem {
    fn from(value: SectionDataItem) -> Self {
        let (agenda, agenda_item) = value
            .agenda_info
            .map(|(agenda, agenda_item, _)| (Some(agenda), Some(agenda_item)))
            .unwrap_or_default();
        let (protocol, protocol_item) = value
            .protocol_info
            .map(|(protocol, protocol_item)| (Some(protocol), Some(protocol_item)))
            .unwrap_or_default();

        GetPlansItem {
            plan: Calculated::new(value.plan),
            agenda,
            protocol,
            agenda_item,
            protocol_item: protocol_item.map(Calculated::new),
            _meta: None,
        }
    }
}
