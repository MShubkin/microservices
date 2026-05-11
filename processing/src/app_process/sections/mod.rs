//! This submodule deals with sections. Here we substitute some enum variants for
//! fixed values for filters.
//! TODO: Find a good way of not inlining the rules. Certainly it is easy to store the
//! Plan filters, but what of the weird and wonderful conditions from other tables?
pub(crate) mod mapping;
pub(crate) mod table;

use ahash::AHashSet;
use itertools::Itertools;
use sqlx::PgPool;

use asez2_shared_db::db_item::selection::FieldSortOrder;

use shared_essential::{
    domain::{
        EcAgenda, EcAgendaItem, EcProtocol, EcProtocolItem, Plan, PlanOrAmendment,
        ProtocolType, RelAgendaProtocolItem, ResultId, Section,
    },
    presentation::dto::processing::{
        GetExpertPlansCountData, PlansCountRequest, PlansRequest,
    },
};

use crate::common::Result;
use table::{JoinedPlanSelect, ProcessingSection};

use self::table::{EntityType, PartedByEntity};

const UUID_FIELD: &str = "uuid";

#[derive(Debug)]
pub(crate) struct SectionData {
    pub(crate) select_info: SectionSelectInfo,
    pub(crate) data: Vec<SectionDataItem>,
}

#[derive(Debug)]
pub struct SectionDataItem {
    pub(crate) plan: PlanOrAmendment,
    pub(crate) agenda_info:
        Option<(EcAgenda, EcAgendaItem, Vec<RelAgendaProtocolItem>)>,
    pub(crate) protocol_info: Option<(EcProtocol, EcProtocolItem)>,
}

#[derive(Debug)]
pub struct SectionSelectInfo {
    pub(crate) plan_request_fields: Vec<String>,
    pub(crate) amendment_request_fields: Vec<String>,
    pub(crate) extra_fields: Option<PartedByEntity<String>>,
    pub(crate) orderings: Vec<(EntityType, FieldSortOrder)>,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn process_count_sections(
    req: PlansCountRequest,
    db_conn: &PgPool,
) -> Result<GetExpertPlansCountData> {
    let PlansCountRequest {
        select,
        pricing_expert_ids,
        user_id,
        section,
    } = req;
    // Не для всех секций может быть определено поведение, поэтому при ненаходе
    // берем пустые настройки секции, которые соответствуют Section::None
    let desired_section = ProcessingSection::fetch_one(section, db_conn).await?;

    // Создание селектов для Plan и ContractAmendment, так как выборка по ним
    // может различаться
    let mut select = select.in_any(Plan::pricing_expert_id, &pricing_expert_ids);
    if !select.field_list.iter().any(|f| f == Plan::pricing_expert_id) {
        select.field_list.push(Plan::pricing_expert_id.to_string());
    }
    let mut joined_select = JoinedPlanSelect::from_select(select, &desired_section);
    joined_select =
        desired_section.add_base_and_other_filters(joined_select, user_id);

    let plans_amendments = PlanOrAmendment::select_dual(
        &joined_select.plan_select,
        &joined_select.amendment_select,
        db_conn,
    )
    .await?;

    let counts =
        plans_amendments.iter().filter_map(|h| *h.pricing_expert_id()).counts();

    Ok(pricing_expert_ids
        .into_iter()
        .map(|expert| (expert, counts.get(&expert).cloned().unwrap_or(0)))
        .collect())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn process_sections(
    req: PlansRequest,
    db_conn: &PgPool,
) -> Result<SectionData> {
    let PlansRequest {
        section,
        mut select,
        user_id,
    } = req;
    tracing::info!("{:#?}", select);

    // Не для всех секций может быть определено поведение, поэтому при ненаходе
    // берем пустые настройки секции, которые соответствуют Section::None
    let desired_section = ProcessingSection::fetch_one(section, db_conn).await?;
    // Нам обязательно требуется добавить `uuid` к выборке
    select.field_list.push(UUID_FIELD.to_string());

    // Создание селектов для Plan и ContractAmendment, так как выборка по ним
    // может различаться
    let mut joined_select = JoinedPlanSelect::from_select(select, &desired_section);
    joined_select =
        desired_section.add_base_and_other_filters(joined_select, user_id);

    // Если наши секции примитивны по выборке, то есть без лишних джойнов,
    // то можно сразу выходить из функции
    if desired_section.plan_filters_only() {
        let trivial_contracts = PlanOrAmendment::select_dual(
            &joined_select.plan_select,
            &joined_select.amendment_select,
            db_conn,
        )
        .await?
        .into_iter()
        .map(SectionDataItem::trivial)
        .collect();

        // field_list можно использовать от любого селекта, так как он никак не изменяется
        let res = SectionData {
            data: trivial_contracts,
            select_info: SectionSelectInfo {
                plan_request_fields: joined_select.plan_select.field_list,
                amendment_request_fields: joined_select.amendment_select.field_list,
                // Селект был тривиальным, поэтому все ордеринги уже были в нем учтены
                orderings: joined_select.orderings,
                extra_fields: None,
            },
        };

        return Ok(res);
    }

    // Делаем сразу выборку со смежными для ППЗ/ДС сущностями и разбиваем на два массива для дальнейшей работы
    let (contracts_with_items, contracts_without_items): (Vec<_>, Vec<_>) =
        desired_section
            .select_related_entities(&joined_select, db_conn)
            .await?
            .into_iter()
            .map(|mut i| {
                // Здесь нам надо выбрать только те данные по Протоколам и Повесткам,
                // которые требует сама секция, поэтому если нет фильтра по сущности,
                // то мы ее убираем
                if !i.is_trivial() {
                    if !desired_section.has_agenda_item_filter {
                        i.agenda_info = None;
                    }

                    if !desired_section.has_protocol_item_filter {
                        i.protocol_info = None;
                    }
                }

                filter_related_items(i, &desired_section)
            })
            .partition(|i| !i.is_trivial());

    // Надо отфильтровать по экстра статус фильтрам ППЗ/ДС у которых нет Протоколов и Повесток,
    // но делать это только в том случае если пользователь не передал фильтр на status_id
    let filtered_contracts_without_items = match desired_section
        .extra_plan_status_filters
    {
        Some(extra_status_filters) => {
            let p_filters = joined_select.plan_select.filter_list.slice();
            let a_filters = joined_select.amendment_select.filter_list.slice();

            let user_filter_fields = p_filters
                .iter()
                .chain(a_filters.iter())
                .map(|f| f.field.to_owned())
                .collect::<AHashSet<_>>();

            // Если пользователь передал фильтр на статус, то не фильтруем
            if user_filter_fields.contains(Plan::status_id) {
                contracts_without_items
            } else {
                contracts_without_items
                    .into_iter()
                    .filter(|i| {
                        extra_status_filters.iter().any(|s| s == i.plan.status_id())
                    })
                    .collect()
            }
        }
        None => contracts_without_items,
    };

    // Если нет специальных полей, то это значит что нам нужны планы без смежных для них сущностей,
    // но с экстра статус фильтров
    if joined_select
        .extra_select
        .as_ref()
        .map(|s| s.extra_fields.is_empty())
        .expect("Проверено выше в plan_filters_only")
        // В закупках ЕИ мы должны возвращать ППЗ/ДС и с Повестками/Протоколами, независимо от
        // того запросил ли пользователь поля по ним или нет.
        && section != Section::EstimatedCommissionProcurements
    {
        let res = SectionData {
            data: filtered_contracts_without_items,
            select_info: SectionSelectInfo {
                plan_request_fields: joined_select.plan_select.field_list,
                amendment_request_fields: joined_select.amendment_select.field_list,
                orderings: joined_select.orderings,
                extra_fields: None,
            },
        };

        return Ok(res);
    }

    let complex_contracts = filtered_contracts_without_items
        .into_iter()
        .chain(contracts_with_items)
        .collect::<Vec<_>>();

    let JoinedPlanSelect {
        plan_select,
        amendment_select,
        orderings,
        extra_select,
    } = joined_select;
    let extra_select = extra_select.expect("Проверено выше");

    let res = SectionData {
        data: complex_contracts,
        select_info: SectionSelectInfo {
            plan_request_fields: plan_select.field_list,
            amendment_request_fields: amendment_select.field_list,
            orderings,
            extra_fields: Some(extra_select.extra_fields),
        },
    };

    Ok(res)
}

/// Здесь вынесены доп фильтры по Протоколам и Повесткам, которые просто
/// не вынести на уровень выборки
/// Не возвращаем поля по протоколу если:
/// - Протокол не соответствует типу указному в секции.
/// - Если результат 3 (не согласован), но только в тех секциях в которых это правило действует.
///
/// Не возвращаются поля по повестки если:
/// - В частном случае у Section::EstimatedCommissionProcurements, если у найденого Протокола тип Заочный
/// - Если для повестки существует протокол (есть записи в таблице agenda_protocol_relation/item_relation_agenda_protocol).
fn filter_related_items(
    mut item: SectionDataItem,
    section: &ProcessingSection,
) -> SectionDataItem {
    match (&item.protocol_info, section.protocol_type) {
        (Some((protocol, _)), Some(ty)) if protocol.protocol_type_id != ty => {
            item.protocol_info = None;
        }
        _ => {}
    };

    if section.agenda_dependency_on_protocol {
        if let Some((protocol, protocol_item)) = &item.protocol_info {
            let protocol_type = protocol.protocol_type_id;
            let result_id = protocol_item.result_id;

            match (result_id, section.section_id) {
                (_, Section::EstimatedCommissionProcurements)
                    if protocol_type == ProtocolType::CorrespondenceMeeting =>
                {
                    item.agenda_info = None;
                }
                (ResultId::NotAgreed, _) => {
                    item.protocol_info = None;

                    if let Some((_, _, agenda_protocol_item_rels)) =
                        &item.agenda_info
                    {
                        if !agenda_protocol_item_rels.is_empty() {
                            item.agenda_info = None;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    item
}

impl SectionData {
    pub(crate) fn pure_plans<T>(self) -> T
    where
        T: FromIterator<PlanOrAmendment>,
    {
        self.data.into_iter().map(|p| p.plan).collect()
    }
}

impl SectionDataItem {
    fn trivial(plan: PlanOrAmendment) -> Self {
        SectionDataItem {
            plan,
            agenda_info: None,
            protocol_info: None,
        }
    }

    fn complex(
        plan: PlanOrAmendment,
        agenda_info: Option<(EcAgenda, EcAgendaItem, Vec<RelAgendaProtocolItem>)>,
        protocol_info: Option<(EcProtocol, EcProtocolItem)>,
    ) -> Self {
        SectionDataItem {
            plan,
            agenda_info,
            protocol_info,
        }
    }

    fn is_trivial(&self) -> bool {
        self.agenda_info.is_none() && self.protocol_info.is_none()
    }
}
