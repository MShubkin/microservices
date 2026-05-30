use ahash::{AHashMap, AHashSet};
use asez2_shared_db::{
    db_item::{joined::JoinTo, Select},
    result::SharedDbError,
};
use sqlx::{Executor, Postgres};

use shared_essential::{
    application::records::Recorder,
    domain::{
        processing::agenda_item::{
            JoinedEcAgendaItemEcAgendaRelAgendaProtocolItem as JoinedAgendaItem,
            JoinedEcAgendaItemEcAgendaRelAgendaProtocolItemSelector as JoinedAgendaItemSelector,
        },
        EcAgenda, EcAgendaItem, PlanOrAmendment, PlanStatus, PricingUnitId,
    },
    presentation::dto::{
        processing::{ColorFullThreshold, ColorScheme},
        response_request::{Message, Messages},
    },
};
use uuid::Uuid;

use crate::common::{ProcessingError, Result};

/// Проверка на существование элементов Повестки СК по переданным
/// ППЗ/ДС, которые имеют is_removed=false и is_removed=false
///
/// Выбираются самые последние по created_at Повестки и его элементы
pub(crate) async fn examine_agenda_items<T, E>(
    plans: &[PlanOrAmendment],
    message_fn: T,
    messages: &mut Messages,
    db_conn: E,
) -> Result<Vec<JoinedAgendaItem>>
where
    T: Fn(&JoinedAgendaItem, &PlanOrAmendment) -> Option<Message>,
    E: for<'a> Executor<'a, Database = Postgres>,
{
    let joined_agenda_items =
        fetch_agenda_items(plans, Some(false), Some(false), db_conn).await?;

    let mut plan_checker =
        plans.iter().map(|x| (*x.uuid(), x)).collect::<AHashMap<_, _>>();

    for j in joined_agenda_items.iter() {
        // remove чтобы сформировать только одно сообщение с одним agenda_item
        if let Some(plan) = plan_checker.remove(&j.agenda_item.source_uuid) {
            if let Some(message) = message_fn(j, plan) {
                messages.add_prepared_message(message);
            }
        }
    }

    Ok(joined_agenda_items)
}

/// Если ППЗ/ДС включена в Повестку с наивысшей датой создания,
/// которая не удалена/is_removed = false, то по позиции Повестки (тоже не удалена/is_removed = false)
/// производятся изменения
pub(crate) async fn update_agenda_items<F>(
    agenda_items: Vec<JoinedAgendaItem>,
    mut modify_fn: F,
    fields: &[&'static str],
    messages: &mut Messages,
    recorder: &mut Recorder<'_>,
) -> Result<Vec<EcAgendaItem>>
where
    F: FnMut(JoinedAgendaItem) -> Option<EcAgendaItem>,
{
    let mut to_update_agenda_items = Vec::with_capacity(agenda_items.len());

    for j in agenda_items {
        if let Some(updated_agenda_item) = modify_fn(j) {
            to_update_agenda_items.push(updated_agenda_item);
        }
    }

    if to_update_agenda_items.is_empty() {
        return Ok(Vec::new());
    }

    Ok(recorder
        .process_update(to_update_agenda_items, fields, messages)
        .await?)
}
/// Выбираются самые последние по created_at Повестки элементы Повестки по переданным ППЗ/ДС
pub(crate) async fn fetch_agenda_items<'a, E>(
    plans: &[PlanOrAmendment],
    is_removed: Option<bool>,
    is_excluded: Option<bool>,
    db_conn: E,
) -> Result<Vec<JoinedAgendaItem>>
where
    E: Executor<'a, Database = Postgres>,
{
    let source_uuids = plans.iter().map(|p| (*p.uuid()).into());

    let mut agenda_item_select =
        Select::full_in::<_, EcAgendaItem>(EcAgendaItem::source_uuid, source_uuids);
    let mut agenda_select = Select::full::<EcAgenda>();

    if let Some(is_excluded) = is_excluded {
        agenda_item_select =
            agenda_item_select.eq(EcAgendaItem::is_excluded, is_excluded);
    }

    if let Some(is_removed) = is_removed {
        agenda_select = agenda_select.eq(EcAgenda::is_removed, is_removed);
        agenda_item_select =
            agenda_item_select.eq(EcAgendaItem::is_removed, is_removed);
    }

    let mut joined_agenda_items = JoinedAgendaItemSelector::new(agenda_item_select)
        .set_agenda(EcAgenda::join_default().selecting(agenda_select))
        .get(db_conn)
        .await?;

    // Нам нужна самая новая. Т.Е. цамое большое значение timestamp первым.
    // Что значит что сравниваем "на оборот"
    joined_agenda_items
        .sort_unstable_by(|a, b| b.agenda.created_at.cmp(&a.agenda.created_at));

    let mut unique_agenda_items = Vec::with_capacity(plans.len());
    let mut already_inserted = AHashSet::new();
    joined_agenda_items.into_iter().for_each(|i| {
        if already_inserted.insert(i.agenda_item.source_uuid) {
            unique_agenda_items.push(i);
        }
    });
    Ok(unique_agenda_items)
}

/// Возвращает threshold по `agenda_item_quantity_threshold` и `agenda_item_d647_quantity_threshold`
/// соответственно
///
/// При определении цветовой схемы используются только те элементы, которые имеют is_removed=false и is_excluded=false
pub(crate) fn calculate_quantity_thresholds(
    plans: &[PlanOrAmendment],
    agenda_items: &[EcAgendaItem],
    has_color: bool,
) -> Result<(ColorFullThreshold, ColorFullThreshold)> {
    let plan_status_map: AHashMap<Uuid, PlanStatus> =
        plans.iter().map(|p| (*p.uuid(), *p.status_id())).collect();

    let (mut quantity_threshold, mut d647_quantity_threshold) =
        (ColorFullThreshold::default(), ColorFullThreshold::default());
    let (mut in_person_items_count, mut d647_in_person_items_count) = (0, 0);

    for item in agenda_items.iter().filter(|i| !i.is_removed) {
        let status = plan_status_map.get(&item.source_uuid).ok_or(
            ProcessingError::DbBackendError(SharedDbError::Other(String::from("Нарушение консистентности базы данных: элемент Повестки не имеет ППЗ/ДС"))),
        )?;
        let is_in_person = *status == PlanStatus::EstimatedCommissionInPerson;

        quantity_threshold.value[0] +=
            if !item.is_registered_by_d647 { 1 } else { 0 };
        d647_quantity_threshold.value[0] +=
            if item.is_registered_by_d647 { 1 } else { 0 };

        if !item.is_excluded {
            quantity_threshold.value[1] +=
                if !item.is_registered_by_d647 { 1 } else { 0 };
            d647_quantity_threshold.value[1] +=
                if item.is_registered_by_d647 { 1 } else { 0 };

            in_person_items_count += if !item.is_registered_by_d647 && is_in_person
            {
                1
            } else {
                0
            };
            d647_in_person_items_count +=
                if item.is_registered_by_d647 && is_in_person {
                    1
                } else {
                    0
                };
        }
    }

    quantity_threshold.color_scheme_id = has_color
        .then(|| check_colour(quantity_threshold.value[1], in_person_items_count))
        .unwrap_or(ColorScheme::Undefined);
    d647_quantity_threshold.color_scheme_id = has_color
        .then(|| {
            check_colour(
                d647_quantity_threshold.value[1],
                d647_in_person_items_count,
            )
        })
        .unwrap_or(ColorScheme::Undefined);

    Ok((quantity_threshold, d647_quantity_threshold))
}

/// Если количество всех элементов равно 0, то color_scheme_id будет 0
/// Если во всех записях текущий статус ППЗ/ДС ≠ Сметная комиссия. Очная СК/251 то дополнительным параметром/color_sheme_id к кол-ву передавать значение 3 (Красный).
/// Если хоть в одной записи текущий статус ППЗ/ДС ≠ Сметная комиссия. Очная СК/251 то дополнительным параметром/color_sheme_id к кол-ву передавать значение 2 (Оранжевый).
/// Если во всех записях текущий статус ППЗ/ДС = Сметная комиссия. Очная СК/251 то дополнительным параметром/color_sheme_id к кол-ву передавать значение 1 (Зеленый).
fn check_colour(all: usize, in_person: usize) -> ColorScheme {
    match (all, in_person) {
        (0, _) => ColorScheme::Undefined,
        (_, 0) => ColorScheme::Red,
        (all, in_person) if all == in_person => ColorScheme::Green,
        _ => ColorScheme::Yellow,
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum AgendaPricingUnitCheck {
    /// У ППЗ/ДС разные департаменты
    DifferentDepartment,
    /// У ППЗ/ДС разные секции
    DifferentSections,
}

pub(crate) fn examine_pricing_unit<'a, I>(
    items: I,
) -> std::result::Result<(), AgendaPricingUnitCheck>
where
    I: Iterator<Item = &'a PlanOrAmendment>,
{
    let (mut d645, mut d646, mut d647, mut gpk) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut items_count = 0;
    for plan in items {
        items_count += 1;
        match plan.pricing_organization_unit_id() {
            PricingUnitId::D645 => d645.push(plan),
            PricingUnitId::D646 => d646.push(plan),
            PricingUnitId::D647 => d647.push(plan),
            PricingUnitId::Gpk => gpk.push(plan),
            PricingUnitId::Undefined => {}
        }
    }

    // Если у выбранных ППЗ/ДС значение в pricing_organization_unit_id различается и равно 1 и 2
    // ИЛИ различается и равно 1, 2 и 3, то формируем предупреждающее сообщение.
    if (!d646.is_empty() && !d647.is_empty() && !gpk.is_empty())
        || (d646.len() + d647.len() == items_count
            && !d646.is_empty()
            && !d647.is_empty())
    {
        return Err(AgendaPricingUnitCheck::DifferentDepartment);
    }

    let (s43, s46) = count_sections(&gpk);

    // Если у выбранных ППЗ/ДС значение в поле pricing_organization_unit_id
    // различается и равно 1 и 3, то для позиций у которых значение в поле
    // pricing_organization_unit_id = 3 выполняем проверку на значение поля
    // "Раздел плана"/section_id. Если значение поля "Раздел плана"/section_id =  4.3 и(или) 4.6,
    // то переходим к следующей проверке, иначе формируем предупреждающее сообщение.
    if !d646.is_empty()
        && !gpk.is_empty()
        && d647.is_empty()
        && gpk.len() != s43 + s46
    {
        return Err(AgendaPricingUnitCheck::DifferentDepartment);
    }

    // Если у выбранных ППЗ/ДС значение в поле pricing_organization_unit_id
    // различается и равно 2 и 3, то для позиций у которых значение
    // в поле pricing_organization_unit_id = 3 выполняем проверку
    // на значение поля "Раздел плана"/section_id. Если значение поля
    // "Раздел плана"/section_id ≠ 4.3 и(или) 4.6, то переходим
    // к следующей проверке, иначе формируем предупреждающее сообщение.
    if !d647.is_empty() && !gpk.is_empty() && d646.is_empty() && s43 + s46 > 0 {
        return Err(AgendaPricingUnitCheck::DifferentDepartment);
    }

    // Если у выбранных ППЗ/ДС одинаковое значение в поле pricing_organization_unit_id = 3,
    // то проверяем значение в поле "Раздел Плана"/section_id.
    if items_count == gpk.len() {
        // Если значения в поле "Раздел плана" различается и равно 4.3  (11) и 4.6 (14),
        // то переходим к следующей проверке, иначе формируем сообщение
        if s43 != 0 && s46 != 0 && gpk.len() != s43 + s46 {
            return Err(AgendaPricingUnitCheck::DifferentSections);
        }

        // Если значение в поле Раздел Плана иммет значения хотябы для одной из ППЗ/ДС
        // section_id = 11 или 14, а по остальным ППЗ/ДС значение section_id ≠ 11 или 14,
        // то переходим к следующей проверке, иначе формируем сообщение
        if (s43 != 0 || s46 != 0) && gpk.len() != s43 + s46 {
            return Err(AgendaPricingUnitCheck::DifferentSections);
        }
    }

    Ok(())
}

/// Подсчет количества ППЗ/ДС с секцией 4.3 и 4.6
///
/// в справочнике значения раздела плана:
/// - 4.3 = 11
/// - 4.6 = 14
fn count_sections(x: &[&PlanOrAmendment]) -> (usize, usize) {
    x.iter().fold((0, 0), |(s43, s46), p| match p.section_id() {
        11 => (s43 + 1, s46),
        14 => (s43, s46 + 1),
        _ => (s43, s46),
    })
}

#[cfg(test)]
mod pricing_unit_check_tests {
    use shared_essential::domain::{Plan, PlanOrAmendment, PricingUnitId};

    use crate::app_process::common::agenda::AgendaPricingUnitCheck;

    use super::examine_pricing_unit;

    /// Проверки на поле Департамент АЦ/pricing_organization_unit_id имеют бОльший приоритет.
    /// Если проверка не пройдена (значение в поле Департамент АЦ/pricing_organization_unit_id отличается хотя бы у
    /// двух позиций = 1 и 2, см. п. 3 итоговой таблицы), то необходимо выводить предупреждающие сообщение.
    #[test]
    fn different_departments() {
        // Выбраны 3 ППЗ/ДС значение в поле pricing_organization_unit_id = 1, 2, 3
        let item_list = vec![
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::D646,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::D647,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::Gpk,
                ..Default::default()
            }),
        ];

        let res = examine_pricing_unit(item_list.iter());
        assert_eq!(res, Err(AgendaPricingUnitCheck::DifferentDepartment));
    }

    /// Выбрано несколько ППЗ/ДС pricing_organization_unit_id = 3 с разными значениями в поле section_id
    #[test]
    fn different_sections_in_d647_and_gpk() {
        // Выбраны 3 ППЗ/ДС: одна позиция - значение в поле pricing_organization_unit_id = 2,
        // и две позиции со значением в поле pricing_organization_unit_id = 3 со значениями в поле section_id = 11 и 7
        let item_list = vec![
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::D647,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::Gpk,
                section_id: 11,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::Gpk,
                section_id: 7,
                ..Default::default()
            }),
        ];

        let res = examine_pricing_unit(item_list.iter());
        assert_eq!(res, Err(AgendaPricingUnitCheck::DifferentDepartment));
    }

    /// Значения в поле "Раздел Плана" различается и равно 4.3  (11) и 4.6 (14)
    #[test]
    fn only_valid_sections_in_gpk() {
        // Выбраны ППЗ/ДС в поле pricing_organization_unit_id = 3, значение в поле  section_id = 11 и 14.
        let item_list = vec![
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::Gpk,
                section_id: 11,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::Gpk,
                section_id: 14,
                ..Default::default()
            }),
        ];

        let res = examine_pricing_unit(item_list.iter());
        assert_eq!(res, Ok(()));
    }

    /// Значение в поле "Раздел Плана" имеет значения для одной из ППЗ/ДС  section_id = 11 или 14,
    /// а по остальным ППЗ/ДС значение  section_id ≠ 11 или 14
    #[test]
    fn only_one_valid_section_in_gpk() {
        // выбраны ППЗ/ДС в поле pricing_organization_unit_id = 3, значение в поле  section_id = 11 и 7
        let item_list = vec![
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::Gpk,
                section_id: 11,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                pricing_organization_unit_id: PricingUnitId::Gpk,
                section_id: 7,
                ..Default::default()
            }),
        ];

        let res = examine_pricing_unit(item_list.iter());
        assert_eq!(res, Err(AgendaPricingUnitCheck::DifferentSections));
    }
}
