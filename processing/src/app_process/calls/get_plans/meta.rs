//
// Данный модуль содержит функции, которые нужны для вставки поля _meta в выдаче ППЗ/ДС.
//

use std::sync::Arc;

use crate::app_process::get_plans_with_last_agenda_items;
use shared_essential::{
    domain::{
        CommissionKind, Plan, PlanOrAmendment, Section, SectionKind,
        TypeOfPurchaseId,
    },
    presentation::dto::{
        general::Metadata,
        processing::price_analysis::{
            GetPlansWithLastAgendaItemsReq, GetPlansWithLastAgendaItemsRes,
        },
        response_request::{MessageKind, Messages},
    },
};
use sqlx::PgPool;

use crate::common::Result;

use super::GetPlansItem;

const PURCHASING_TYPE_ID: &str = Plan::purchasing_type_id;
const COMMISSION_KIND_ID: &str = Plan::commission_kind_id;
const COMMISSION_DATE: &str = Plan::commission_date;

// Кажется, что нет смысла дополнительно запрашивать второй раз планы для этих полей, проще их подмешать
// в основной селект и если они не нужны в ответе, то не выводить их на фронт
#[derive(Debug, PartialEq)]
pub(crate) enum TransientFields {
    PurchasingTypeId,
    CommissionKindId,
}

pub(crate) fn insert_additional_fields(
    section: &Section,
    field_list: &mut Vec<String>,
) -> Option<Vec<TransientFields>> {
    // Дополнительные поля нужны только в секции АЦ
    if section.kind() != SectionKind::PriceAnalysis {
        // Текущая секция не АЦ
        return None;
    }
    let mut transient_fields = vec![];
    let additional_fields = [
        (PURCHASING_TYPE_ID.to_string(), TransientFields::PurchasingTypeId),
        (COMMISSION_KIND_ID.to_string(), TransientFields::CommissionKindId),
    ];

    for (id, field) in additional_fields.into_iter() {
        if !field_list.contains(&id) {
            field_list.push(id);
            transient_fields.push(field);
        }
    }
    Some(transient_fields)
}

// Запрашиваем только в случае если у нас найдена неконкурентная закупка
pub(crate) async fn request_non_competitive_plans_with_last_agenda<'a, I>(
    section: &Section,
    plans: I,
    db_pool: Arc<PgPool>,
) -> Result<Option<GetPlansWithLastAgendaItemsRes>>
where
    I: IntoIterator<Item = &'a PlanOrAmendment>,
{
    // Дополнительные поля нужны только в секции АЦ
    if section.kind() != SectionKind::PriceAnalysis {
        // Текущая секция не АЦ
        return Ok(None);
    }

    let mut non_competitive_plan_uuids = Vec::new();
    let plan_uuids = plans
        .into_iter()
        .map(|p| {
            let purchasing_type_id = *p.purchasing_type_id();
            let uuid = *p.uuid();

            if purchasing_type_id == TypeOfPurchaseId::NotCompetitive as i16 {
                non_competitive_plan_uuids.push(Some(uuid));
            }

            uuid
        })
        .collect::<Vec<_>>();

    // Получаем данные только если есть неконкурентная закупка
    let last_agenda_items = if !non_competitive_plan_uuids.is_empty() {
        let last_agenda_items = get_plans_with_last_agenda_items(
            GetPlansWithLastAgendaItemsReq {
                plans_uuid: plan_uuids,
            },
            db_pool,
        )
        .await?;
        Some(last_agenda_items)
    } else {
        None
    };

    Ok(last_agenda_items.map(|x| x.data))
}

pub(crate) async fn fill_meta_field(
    section: &Section,
    plans: &mut [GetPlansItem],
    agenda_items: Option<GetPlansWithLastAgendaItemsRes>,
    transient_fields: Option<Vec<TransientFields>>,
) -> Result<Messages> {
    let mut messages = Messages::default();
    // Дополнительные поля нужны только в секции АЦ
    if section.kind() != SectionKind::PriceAnalysis {
        // Текущая секция не АЦ
        return Ok(messages);
    }
    let disabled_field_list =
        vec![COMMISSION_KIND_ID.to_string(), COMMISSION_DATE.to_string()];

    for item in plans.iter_mut() {
        let is_amendment = item.plan.item.is_amendment();
        match item.plan.item.purchasing_type_id() {
            x if *x == TypeOfPurchaseId::Competitive as i16 && !is_amendment => {
                // Конкурентная закупка
                item._meta = Some(Metadata {
                    disabled_field_list: disabled_field_list.clone(),
                })
            }
            x if (*x == TypeOfPurchaseId::NotCompetitive as i16)
                || is_amendment =>
            {
                // Неконкурентная закупка
                if let Some(agenda_items) = agenda_items.as_ref() {
                    fill_meta_for_non_competitive_purchase(
                        item,
                        agenda_items,
                        &disabled_field_list,
                        &mut messages,
                    );
                }
            }
            _ => {}
        }

        // Если commission_kind_id  ≠ 1/Очная СК, то по данной записи ППЗ/ДС передается структура _meta - disabled_field_list с полем commission_date для последующей блокировки поля на FE.
        if *item.plan.item.commission_kind_id() != CommissionKind::InPerson {
            // Это поле должно быть пустым, даже если в БД есть какая-то дата, но комиссия не является очной
            *item.plan.item.commission_date_mut() = None;

            if item._meta.is_none() {
                item._meta = Some(Metadata {
                    disabled_field_list: vec![COMMISSION_DATE.to_string()],
                });
            }
        }

        // Если эти поля не нужны в ответе, то и их надо убрать
        if let Some(transient_fields) = &transient_fields {
            if transient_fields.contains(&TransientFields::CommissionKindId) {
                *item.plan.item.commission_kind_id_mut() = Default::default();
            }
        }
    }

    Ok(messages)
}

fn fill_meta_for_non_competitive_purchase(
    plan_item: &mut GetPlansItem,
    agenda_items: &GetPlansWithLastAgendaItemsRes,
    disabled_field_list: &[String],
    messages: &mut Messages,
) {
    let plan_uuid = plan_item.plan.item.uuid();

    if agenda_items.last_agenda_item_hashmap.get(plan_uuid).is_some() {
        plan_item._meta = Some(Metadata {
            disabled_field_list: disabled_field_list.to_owned(),
        });
    } else {
        messages.add_message(
            MessageKind::Warning,
            format!(
                "ППЗ/ДС с ID {} не включена в актуальную Повестку.",
                plan_item.plan.item.id()
            ),
        );
    }
}
