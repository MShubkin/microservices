use asez2_shared_db::db_item::{from_item_with_fields, AdaptorableIter};
use itertools::{Either, Itertools};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use asez2_shared_db::db_item::{joined::JoinTo, Select};
use asez2_shared_db::DbAdaptor;
use shared_essential::presentation::dto::processing::MergedAgendaItem;
use shared_essential::{
    domain::*,
    presentation::dto::{
        processing::{GetAgendaDetailsReq, GetAgendaDetailsRes},
        response_request::*,
    },
};

use crate::common::{ProcessingError, Result};

use sqlx::PgPool;
use uuid::Uuid;

const GET_AGENDA_DETAILS: &str = "/rest/estimated_commission/v1/get/agenda_details";

const AGENDA_FIELDS: &[&str] = &[
    EcAgenda::uuid,
    EcAgenda::id,
    EcAgenda::status_id,
    EcAgenda::pricing_organization_unit_id,
    EcAgenda::meeting_date,
    EcAgenda::created_at,
    EcAgenda::changed_at,
    EcAgenda::created_by,
    EcAgenda::changed_by,
];

const AGENDA_ITEM_FIELDS: &[&str] = &[
    EcAgendaItem::uuid,
    EcAgendaItem::source_uuid,
    EcAgendaItem::is_excluded,
    EcAgendaItem::reviewed_at,
    EcAgendaItem::pricing_sum_excluded_vat,
    EcAgendaItem::sum_excluded_vat,
];

const PLAN_FIELDS: &[&str] = &[
    Plan::contract_subject,
    Plan::currency_id,
    Plan::customer_id,
    Plan::number_customer,
    "plan_id", // is a renamed field.
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::section_id,
    Plan::status_id,
    Plan::supplier_id,
];

const PARTNER_FIELDS: &[&str] = &[
    EcPartner::uuid,
    EcPartner::user_id,
    EcPartner::e_mail,
    EcPartner::is_checked_in,
    EcPartner::role_id,
];

#[tracing::instrument(skip_all)]
pub(crate) async fn get_agenda_details(
    request: GetAgendaDetailsReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetAgendaDetailsRes, ()>> {
    tracing::info!(
        kind = "get",
        "Получение подробностей Повестки СК ({get}): {req:?}\n",
        req = request,
        get = GET_AGENDA_DETAILS
    );

    let GetAgendaDetailsReq { id } = request;

    let mut data = get_data(id, &db_pool).await?;
    let mut messages = Messages::default();

    if let Some(datum) = data.pop() {
        let datum = convert(datum)?;
        Ok((datum, messages).into())
    } else {
        let msg = format!("Повеска № {} не найдена.", id);
        messages.add_message(MessageKind::Error, msg);
        Ok(messages.into())
    }
}

async fn get_data(id: i64, pool: &PgPool) -> Result<Vec<AgendaDetails>> {
    let select = Select::full::<EcAgenda>().eq("id", id);

    let plan_select = Select::full::<Plan>().distinct_on(&["uuid"]);
    let amendment_select =
        Select::full::<ContractAmendment>().distinct_on(&["uuid"]);

    let agenda_items_select = Select::full::<EcAgendaItem>()
        .eq("is_removed", false)
        .distinct_on(&["uuid"]);
    let partner_select = Select::full::<EcPartner>()
        .eq("is_removed", false)
        .distinct_on(&["uuid"]);
    let attachment_select = Select::full::<Attachment>()
        .eq(Attachment::is_removed, false)
        .distinct_on(&["uuid"]);
    let status_histories_select =
        Select::full::<StatusHistory>().distinct_on(&["uuid"]);

    AgendaDetailsSelector::new(select)
        .set_agenda_items(
            EcAgendaItem::join_default().selecting(agenda_items_select),
        )
        .set_amendments(
            ContractAmendment::join_default().selecting(amendment_select),
        )
        .set_plans(Plan::join_default().selecting(plan_select))
        .set_partner_list(EcPartner::join_default().selecting(partner_select))
        .set_attachment_list(
            Attachment::join_default().selecting(attachment_select),
        )
        .set_status_histories(
            StatusHistory::join_default().selecting(status_histories_select),
        )
        .distinct()
        .get(pool)
        .await
        .map_err(Into::into)
}

/// TODO: как то DISTINCT своё дело не делает.
fn convert(inp: AgendaDetails) -> Result<GetAgendaDetailsRes> {
    let mut plans = inp.plans;
    let mut amendments = inp.amendments;
    let mut agenda_items = inp.agenda_items;
    let mut partner_list = inp.partner_list;
    let mut attachment_list = inp.attachment_list;
    let mut status_histories = inp.status_histories;

    plans.sort_unstable_by(|a, b| a.uuid.cmp(&b.uuid));
    plans.dedup_by(|a, b| a.uuid == b.uuid);

    amendments.sort_unstable_by(|a, b| a.uuid.cmp(&b.uuid));
    amendments.dedup_by(|a, b| a.uuid == b.uuid);

    agenda_items.sort_unstable_by(|a, b| a.uuid.cmp(&b.uuid));
    agenda_items.dedup_by(|a, b| a.uuid == b.uuid);

    attachment_list.sort_unstable_by(|a, b| a.uuid.cmp(&b.uuid));
    attachment_list.dedup_by(|a, b| a.uuid == b.uuid);

    status_histories.sort_unstable_by(|a, b| a.uuid.cmp(&b.uuid));
    status_histories.dedup_by(|a, b| a.uuid == b.uuid);

    let attachment_list = attachment_list.into_iter().adaptors().collect();
    let status_histories = status_histories.into_iter().adaptors().collect();

    let from_item = PlanOrAmendmentRep::from_item_with_fields(PLAN_FIELDS);
    let plan_checker = PlanOrAmendment::into_iter(plans, amendments)
        .map(|x| {
            let uuid = *x.uuid();
            let plan = from_item(x);
            (uuid, plan)
        })
        .collect::<HashMap<Uuid, PlanOrAmendmentRep>>();

    let from_agenda = from_item_with_fields(AGENDA_ITEM_FIELDS);
    let merged_agenda_items = agenda_items
        .into_iter()
        .sorted_by(|i1, i2| i1.number.cmp(&i2.number))
        .map(|x| {
            let is_registered_by_d647 = x.is_registered_by_d647;
            let plan = plan_checker.get(&x.source_uuid).cloned()
                .ok_or(ProcessingError::GetAgendaDetails(format!("Нарушение консистентности базы данных. Элемент Повестки СК {} не имеет смежного ППЗ/ДС", x.uuid)))?;
            let agenda_item = from_agenda(x);

            let item = MergedAgendaItem {
                agenda_item,
                plan
            };
            Ok((is_registered_by_d647, item))
        }).collect::<Result<Vec<_>>>()?;

    let (agenda_item_list, agenda_item_d647_list) =
        merged_agenda_items.into_iter().partition_map(
            |(is_registered_by_d647, item)| match is_registered_by_d647 {
                false => Either::Left(item),
                true => Either::Right(item),
            },
        );

    // Сортируем партнёров по роли и пользователю.
    partner_list.sort_unstable_by(|a, b| match a.role_id.cmp(&b.role_id) {
        Ordering::Equal => a.user_id.cmp(&b.user_id),
        x => x,
    });
    let partner_list = partner_list
        .into_iter()
        .unique_by(|p| p.uuid)
        .adaptors_with_fields(PARTNER_FIELDS)
        .collect();

    Ok(GetAgendaDetailsRes {
        agenda: EcAgendaRep::from_item::<&str>(inp.agenda, Some(AGENDA_FIELDS)),
        agenda_item_d647_list,
        agenda_item_list,
        partner_list,
        attachment_list,
        status_histories,
    })
}
