//! Тут содержится код для таблицы processing_sections.
//! Он определяет как должен работать каждый слой секций.
use std::ops::RangeInclusive;
use std::str::FromStr;

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgRow};
use sqlx::{prelude::Type, types::Json as SqlJ};
use sqlx::{FromRow, PgPool, Row};

use asez2_shared_db::{
    db_item::{joined::JoinTo, selection::*, Filter, FilterTree, Select},
    DbItem, Value,
};
use shared_essential::presentation::dto::general::{ColumnFilter, Filters};

use shared_essential::domain::{
    tables::processing::plan, ContractAmendmentRep, EcAgenda, EcAgendaItem,
    EcProtocol, EcProtocolItem, Plan, PlanOrAmendment, PlanRep, PlanStatus,
    ProtocolType, Section, AMENDMENT_ID_RANGE, PLAN_ID_RANGE,
};

use plan::section_selection::{
    JoinedContractAmendmentEcAgendaItemEcAgendaEcProtocolItemEcProtocolRelAgendaProtocolItemSelector as ContractAmendmentAPSelector,
    JoinedPlanEcAgendaItemEcAgendaEcProtocolItemEcProtocolRelAgendaProtocolItemSelector as PlanAPSelector,
};

use crate::common::{ProcessingError, Result};

use super::{mapping::SectionMap, SectionDataItem};

/// Разрешенные Таблицы.

// pub(crate) SectionTables

/// Represents a processing section. Using this descriptor
/// We can describe a lot of variable sections that follow a number of serial checks.
/// 1.) Filter by various parameters of Plan.
/// 2.) Sort agendas.
/// 3.) Sort of protocol items + optionality of protocol items.
/// 4.) Check for removed protocol item
/// 5.) Check for optionality of agenda items.
/// 6.) Filter if outside of a given set of status and lacks special fields.
///
/// TODO: This will likely need to be updated and moved to tables.
#[derive(Debug, Default, Clone, DbItem)]
#[item_table = "processing_section"]
pub(crate) struct ProcessingSection {
    #[item_field_pkey = "section_id"]
    pub(crate) section_id: Section,
    pub(crate) base_plan_filters: Vec<SqlJ<Filters>>,
    pub(crate) extra_plan_status_filters: Option<Vec<PlanStatus>>,
    pub(crate) other_filters: Option<Vec<SqlJ<ExtraFilter>>>,
    pub(crate) user_filter_column: Option<String>,
    pub(crate) year_offset: Option<i16>,
    pub(crate) has_agenda_item_filter: bool,
    pub(crate) has_protocol_item_filter: bool,
    pub(crate) protocol_type: Option<ProtocolType>,
    pub(crate) agenda_dependency_on_protocol: bool,
    pub(crate) user_priority_filter_fields: Option<Vec<String>>,
    pub(crate) extra_fields: Option<Vec<ExtraField>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct ExtraFilter {
    entity: EntityType,
    filters: Vec<Filters>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[sqlx(type_name = "extra_field")]
pub(crate) struct ExtraField {
    entity: EntityType,
    fields: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "entity_type", rename_all = "snake_case")]
pub(crate) enum EntityType {
    Plan,
    ContractAmendment,
    Protocol,
    ProtocolItem,
    Agenda,
    AgendaItem,
}

#[derive(Clone, Debug)]
pub(crate) struct PartedByEntity<T> {
    data: Vec<(EntityType, Vec<T>)>,
}

#[derive(Clone, Debug)]
pub(crate) struct JoinedPlanSelect {
    pub(crate) plan_select: Select,
    pub(crate) amendment_select: Select,
    pub(crate) orderings: Vec<(EntityType, FieldSortOrder)>,
    pub(crate) extra_select: Option<JoinedExtraSelect>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct JoinedExtraSelect {
    pub(crate) extra_filters: PartedByEntity<Filter>,
    pub(crate) extra_fields: PartedByEntity<String>,
}

impl JoinedPlanSelect {
    pub(crate) fn from_select(
        mut select: Select,
        section: &ProcessingSection,
    ) -> Self {
        // Тут фильтрация планов по ID, так что могут быть несколько версий по uuid.
        // надо брать по признаку is_actual==true.
        let actual_filter = Filter::eq(Plan::is_actual, true).into();
        select.filter_list = select.filter_list.and(actual_filter);

        let mut plan_select = select.filtered_copy_for::<PlanRep>();
        PlanRep::enrich_select_by_section(
            &mut plan_select,
            section.section_id.kind(),
        );

        let mut amendment_select =
            select.filtered_copy_for::<ContractAmendmentRep>();
        ContractAmendmentRep::enrich_select_by_section(
            &mut amendment_select,
            section.section_id.kind(),
        );

        // если пришел фильтр по object_type подсовываем фильтр по айди для простоты
        let mut binding = select.filter_list.slice();
        let object_type_filter =
            binding.iter_mut().find(|x| x.field == "object_type");
        if let Some(otf) = object_type_filter {
            if let Some(Value::String(x)) = otf.values.first() {
                let range: Option<RangeInclusive<i64>> = match x.as_str() {
                    "plan" => Some(PLAN_ID_RANGE),
                    "contract_amendment" => Some(AMENDMENT_ID_RANGE),
                    _ => None,
                };
                if let Some(rng) = range {
                    let new_otf = Filter::between("id", rng.start(), rng.end());
                    plan_select.filter_list.push_filter(new_otf.clone());
                    amendment_select.filter_list.push_filter(new_otf);
                }
            };
        }

        if section.plan_filters_only() {
            return JoinedPlanSelect {
                plan_select,
                amendment_select,
                orderings: select
                    .order_list
                    .into_iter()
                    .map(|ordering| (EntityType::Plan, ordering))
                    .collect(),
                extra_select: None,
            };
        }

        let special_field_checker = section
            .extra_fields
            .as_ref()
            .expect("Секция подразумевает наличие выборку по сторонним сущностям")
            .iter()
            .flat_map(|f| f.fields.iter().map(|field| (field.as_str(), f.entity)))
            .collect::<AHashMap<_, _>>();

        let orderings = select
            .order_list
            .into_iter()
            .map(|ordering| {
                let ty = special_field_checker
                    .get(ordering.field.as_str())
                    .unwrap_or(&EntityType::Plan);

                (*ty, ordering)
            })
            .collect();
        let extra_filters =
            PartedByEntity::part_by(select.filter_list.into_filters(), |filter| {
                special_field_checker.get(filter.field.as_str()).copied()
            });
        let extra_fields = PartedByEntity::part_by(select.field_list, |field| {
            special_field_checker.get(field.as_str()).copied()
        });

        JoinedPlanSelect {
            plan_select,
            amendment_select,
            orderings,
            extra_select: Some(JoinedExtraSelect {
                extra_fields,
                extra_filters,
            }),
        }
    }

    pub(crate) fn add_expand_filter<I, V>(
        self,
        field: &str,
        selection_kind: SelectionKind,
        values: I,
    ) -> Self
    where
        I: IntoIterator<Item = V> + Clone,
        V: Into<Value>,
    {
        self.add_plan_expand_filter(field, selection_kind, values.clone())
            .add_amendment_expand_filter(field, selection_kind, values)
    }

    pub(crate) fn add_plan_expand_filter<I, V>(
        mut self,
        field: &str,
        selection_kind: SelectionKind,
        values: I,
    ) -> Self
    where
        I: IntoIterator<Item = V> + Clone,
        V: Into<Value>,
    {
        self.plan_select =
            self.plan_select.add_expand_filter(field, selection_kind, values);
        self
    }

    pub(crate) fn add_amendment_expand_filter<I, V>(
        mut self,
        field: &str,
        selection_kind: SelectionKind,
        values: I,
    ) -> Self
    where
        I: IntoIterator<Item = V> + Clone,
        V: Into<Value>,
    {
        self.amendment_select =
            self.amendment_select.add_expand_filter(field, selection_kind, values);
        self
    }
}

impl ProcessingSection {
    /// If this is true, we do not perform extra steps.
    pub(crate) fn plan_filters_only(&self) -> bool {
        !self.has_agenda_item_filter && !self.has_protocol_item_filter
    }

    pub(crate) async fn fetch_one(
        section: Section,
        db_conn: &PgPool,
    ) -> Result<ProcessingSection> {
        let section_select = Select::full::<ProcessingSection>()
            .eq(ProcessingSection::section_id, section)
            .take_first();

        // Всегда только одна подходящая секция
        ProcessingSection::select(&section_select, db_conn)
            .await?
            .pop()
            .ok_or_else(|| {
                ProcessingError::Section(format!(
                    "Секция `{}` не найдена.",
                    section
                ))
            })
    }

    pub(crate) fn add_other_filters(
        &self,
        mut select: Select,
        entity_kind: EntityType,
    ) -> Select {
        if let Some(other_filters) = &self.other_filters {
            for other_filter in
                other_filters.iter().filter(|f| f.entity == entity_kind)
            {
                let initial_filter_list = select.filter_list;

                let other = other_filter.filters.iter().flat_map(|f| {
                    f.values.iter().map(|column_filter| {
                        Filter::with_values(
                            &f.field,
                            column_filter.selection_kind,
                            column_filter
                                .values
                                .iter()
                                .cloned()
                                .filter_map(|v| Value::try_from(v).ok()),
                        )
                    })
                });
                let other_as_tree = FilterTree::and_from_list(other);

                select.filter_list = initial_filter_list.and(other_as_tree);
            }
        }

        select
    }

    /// This alters the initial select based on plan filters and plan user id.
    pub(crate) fn add_base_filters(
        &self,
        mut select: JoinedPlanSelect,
        user_id: i32,
    ) -> JoinedPlanSelect {
        let filters = self
            .base_plan_filters
            .iter()
            .flat_map(|x| x.values.iter().map(|v| (&x.field, x.is_key, v)));

        for (field, _, c) in filters {
            let ColumnFilter {
                selection_kind,
                values,
            } = c;
            select = select.add_expand_filter(
                field,
                *selection_kind,
                values.iter().cloned().filter_map(|v| Value::try_from(v).ok()),
            );
        }
        if let Some(ref user_filter) = self.user_filter_column {
            select = select.add_expand_filter(
                user_filter,
                SelectionKind::Equals,
                [user_id],
            );
        }

        select
    }

    pub(crate) fn add_base_and_other_filters(
        &self,
        select: JoinedPlanSelect,
        user_id: i32,
    ) -> JoinedPlanSelect {
        let mut with_base = self.add_base_filters(select, user_id);
        with_base.plan_select =
            self.add_other_filters(with_base.plan_select, EntityType::Plan);
        with_base.amendment_select = self.add_other_filters(
            with_base.amendment_select,
            EntityType::ContractAmendment,
        );

        with_base
    }

    /// Накладывание фильтров по agenda_item с protocol_item
    ///
    /// Возвращает массив uuid ППЗ и массив ДС которые подходят под фильтры по agenda_item и protocol_item
    pub(crate) async fn select_related_entities(
        &self,
        cleaned_select: &JoinedPlanSelect,
        pool: &PgPool,
    ) -> Result<Vec<SectionDataItem>> {
        // We must always make distinct on source_uuid and sort by it,
        // else we may get a very strange mix.
        let mut agenda_item_select =
            Select::with_fields([EcAgendaItem::source_uuid])
                .eq(EcAgendaItem::is_removed, false)
                .eq(EcAgendaItem::is_excluded, false)
                .add_replace_order_asc(EcAgendaItem::source_uuid)
                .add_replace_order_desc(EcAgendaItem::created_at)
                .distinct_on(&[EcAgendaItem::source_uuid]);
        let mut agenda_select =
            Select::full::<EcAgenda>().eq(EcAgenda::is_removed, false);

        let mut protocol_item_select =
            Select::with_fields([EcProtocolItem::source_uuid])
                .eq(EcProtocolItem::is_removed, false)
                .eq(EcProtocolItem::is_excluded, false)
                .add_replace_order_asc(EcProtocolItem::source_uuid)
                .add_replace_order_desc(EcProtocolItem::created_at)
                .distinct_on(&[EcProtocolItem::source_uuid]);
        let mut protocol_select =
            Select::full::<EcProtocol>().eq(EcProtocol::is_removed, false);

        if let Some(extra_select) = &cleaned_select.extra_select {
            let extend_filter_tree =
                |tree: FilterTree, ty: EntityType| -> FilterTree {
                    if let Some(filters) = extra_select.extra_filters.get(ty) {
                        tree.and(FilterTree::and_from_list(filters.to_vec()))
                    } else {
                        tree
                    }
                };

            agenda_select.filter_list =
                extend_filter_tree(agenda_select.filter_list, EntityType::Agenda);
            agenda_item_select.filter_list = extend_filter_tree(
                agenda_item_select.filter_list,
                EntityType::AgendaItem,
            );
            protocol_select.filter_list = extend_filter_tree(
                protocol_select.filter_list,
                EntityType::Protocol,
            );
            protocol_item_select.filter_list = extend_filter_tree(
                protocol_item_select.filter_list,
                EntityType::ProtocolItem,
            );
        }

        let amendments = ContractAmendmentAPSelector::new(
            cleaned_select.amendment_select.clone(),
        )
        .set_agenda_item(
            EcAgendaItem::join_default().selecting(agenda_item_select.clone()),
        )
        .set_agenda(EcAgenda::join_default().selecting(agenda_select.clone()))
        .set_protocol_item(
            EcProtocolItem::join_default().selecting(protocol_item_select.clone()),
        )
        .set_protocol(EcProtocol::join_default().selecting(protocol_select.clone()))
        .get(pool)
        .await?;

        let plans = PlanAPSelector::new(cleaned_select.plan_select.clone())
            .set_agenda_item(
                EcAgendaItem::join_default().selecting(agenda_item_select),
            )
            .set_agenda(EcAgenda::join_default().selecting(agenda_select))
            .set_protocol_item(
                EcProtocolItem::join_default().selecting(protocol_item_select),
            )
            .set_protocol(EcProtocol::join_default().selecting(protocol_select))
            .get(pool)
            .await?;

        let plans_with_items = amendments
            .into_iter()
            .map(|a| {
                (
                    PlanOrAmendment::from(a.amendment),
                    a.agenda,
                    a.agenda_item,
                    a.protocol,
                    a.protocol_item,
                    a.agenda_protocol_item_rel,
                )
            })
            .chain(plans.into_iter().map(|p| {
                (
                    PlanOrAmendment::from(p.plan),
                    p.agenda,
                    p.agenda_item,
                    p.protocol,
                    p.protocol_item,
                    p.agenda_protocol_item_rel,
                )
            }))
            .map(|(plan, agenda, agenda_item, protocol, protocol_item, rels)| {
                let protocol_info = (protocol.is_some() && protocol_item.is_some())
                    .then(|| (protocol.unwrap(), protocol_item.unwrap()));
                let agenda_info = (agenda.is_some() && agenda_item.is_some())
                    .then(|| (agenda.unwrap(), agenda_item.unwrap(), rels));

                SectionDataItem::complex(plan, agenda_info, protocol_info)
            })
            .collect();

        Ok(plans_with_items)
    }
}

impl<T> PartedByEntity<T> {
    pub fn part_by<F>(items: Vec<T>, retrieve_type_fn: F) -> Self
    where
        F: Fn(&T) -> Option<EntityType>,
    {
        let mut result: Vec<(EntityType, Vec<T>)> = Vec::new();

        for item in items {
            if let Some(ty) = retrieve_type_fn(&item) {
                if let Some((_, chunk)) =
                    result.iter_mut().find(|(chunk_ty, _)| *chunk_ty == ty)
                {
                    chunk.push(item);
                } else {
                    result.push((ty, vec![item]));
                }
            }
        }

        Self { data: result }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, ty: EntityType) -> Option<&[T]> {
        self.data
            .iter()
            .find(|(dty, _)| *dty == ty)
            .map(|(_, data)| data.as_slice())
    }
}

impl<T> Default for PartedByEntity<T> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl FromRow<'_, PgRow> for EntityType {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let ty: String = row.try_get("entity_type")?;

        Self::from_str(&ty).map_err(sqlx::Error::ColumnNotFound)
    }
}

impl FromStr for EntityType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ty = match s {
            "plan" => Self::Plan,
            "contract_amendment" => Self::ContractAmendment,
            "agenda" => Self::Agenda,
            "agenda_item" => Self::AgendaItem,
            "protocol" => Self::Protocol,
            "protocol_item" => Self::ProtocolItem,
            _ => return Err(format!("Невалидное значение для EntityType: {}", s)),
        };

        Ok(ty)
    }
}

impl PgHasArrayType for EntityType {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_entity_type")
    }
}

impl PgHasArrayType for ExtraField {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_extra_fields")
    }
}
