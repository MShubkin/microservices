use std::{collections::HashSet, sync::Arc};

use asez2_shared_db::db_item::{from_item_with_fields, joined::JoinTo, Select};
use shared_essential::{
    domain::{
        ContractAmendment, EcAgenda, EcAgendaItem,
        JoinedEcAgendaEcAgendaItemPlanContractAmendmentRelAgendaProtocolItem as JoinedAgenda,
        JoinedEcAgendaEcAgendaItemPlanContractAmendmentRelAgendaProtocolItemSelector as JoinedAgendaSelector,
        Plan, PlanOrAmendment, PlanOrAmendmentRep, RelAgendaProtocolItem,
    },
    presentation::dto::{
        general::Metadata,
        processing::{
            GetAgendaItemsForProtocolCreateItem,
            GetAgendaItemsForProtocolCreateReq,
            GetAgendaItemsForProtocolCreateResponseData,
        },
        response_request::{ApiResponse, Messages, Status},
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

const RESPONSE_PLAN_FIELDS: &[&str] = &[
    "plan_id",
    Plan::customer_id,
    Plan::supplier_id,
    Plan::section_id,
    Plan::status_id,
];
const RESPONSE_AGENDA_ITEM_FIELDS: &[&str] = &[
    EcAgendaItem::uuid,
    EcAgendaItem::sum_excluded_vat,
    EcAgendaItem::reviewed_at,
];

pub(crate) async fn get_agenda_items_for_protocol_create(
    dto: GetAgendaItemsForProtocolCreateReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetAgendaItemsForProtocolCreateResponseData, ()>> {
    let GetAgendaItemsForProtocolCreateReq { agenda_id, .. } = dto;

    let agenda = fetch_agenda(agenda_id, &db_pool).await?;
    let validateable_struct = construct_validateable_data(agenda)?;

    let meeting_date = validateable_struct.agenda.meeting_date;

    let plan_from_item =
        PlanOrAmendmentRep::from_item_with_fields(RESPONSE_PLAN_FIELDS);
    let agenda_from_item = from_item_with_fields(RESPONSE_AGENDA_ITEM_FIELDS);
    let item_list = validateable_struct
        .items
        .into_iter()
        .map(|item| {
            let meta = item.has_protocol.then(|| Metadata {
                disabled_field_list: vec![String::from(
                    "is_can_be_included_in_protocol",
                )],
            });
            // Потому что значит "включение в Протокол"
            let is_can_be_included_in_protocol = !item.has_protocol;

            let plan = plan_from_item(item.plan);
            let agenda_item = agenda_from_item(item.agenda_item);

            GetAgendaItemsForProtocolCreateItem {
                is_can_be_included_in_protocol,
                agenda_item,
                plan,
                _meta: meta,
            }
        })
        .collect();

    let response = GetAgendaItemsForProtocolCreateResponseData {
        agenda_id: validateable_struct.agenda.id,
        meeting_date,
        uuid: validateable_struct.agenda.uuid,
        item_list,
    };

    Ok(ApiResponse {
        status: Status::Ok,
        data: response,
        messages: Messages::default(),
        objects: vec![],
    })
}

struct ValidateableData {
    agenda: EcAgenda,
    items: Vec<ValidateableDataItem>,
}

struct ValidateableDataItem {
    agenda_item: EcAgendaItem,
    plan: PlanOrAmendment,
    has_protocol: bool,
}

fn construct_validateable_data(
    joined_agenda: JoinedAgenda,
) -> Result<ValidateableData> {
    let JoinedAgenda {
        agenda,
        agenda_items,
        plans,
        amendments,
        item_relation_agenda_protocol,
    } = joined_agenda;

    let mut plans = PlanOrAmendment::collect_map_by_uuid(plans, amendments);

    let rels_agenda_item_checker = item_relation_agenda_protocol
        .into_iter()
        .map(|item| item.agenda_item_uuid)
        .collect::<HashSet<_>>();

    let items = agenda_items
        .into_iter()
        .map(|agenda_item: EcAgendaItem| {
            let plan = plans.remove(&agenda_item.source_uuid).ok_or_else(|| {
                let msg = format!(
                    "Нарушение консистентности базы данных. По элементу Повестки СК с идентификатором {} нет ППЗ/ДС",
                    &agenda_item.uuid
                );
                ProcessingError::GetItemList(msg)
            })?;

            let has_protocol = rels_agenda_item_checker.contains(&agenda_item.uuid);

            Ok(ValidateableDataItem {
                agenda_item,
                plan,
                has_protocol,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ValidateableData { agenda, items })
}

async fn fetch_agenda(agenda_id: i64, db_pool: &PgPool) -> Result<JoinedAgenda> {
    let agenda_select =
        Select::with_fields([EcAgenda::uuid, EcAgenda::id, EcAgenda::meeting_date])
            .eq(EcAgenda::is_removed, false)
            .eq(EcAgenda::id, agenda_id)
            .take_first();
    let agenda_item_select = Select::with_fields([
        EcAgendaItem::uuid,
        EcAgendaItem::sum_excluded_vat,
        EcAgendaItem::reviewed_at,
    ])
    .eq(EcAgendaItem::is_removed, false)
    .eq(EcAgendaItem::is_excluded, false)
    .eq(EcAgendaItem::is_registered_by_d647, false);

    let plan_select = Select::full::<Plan>();
    let amendment_select = Select::full::<ContractAmendment>();
    let rel_agenda_item_select = Select::full::<RelAgendaProtocolItem>();

    let joined_select = JoinedAgendaSelector::new(agenda_select)
        .set_agenda_items(
            EcAgendaItem::join_default().selecting(agenda_item_select),
        )
        .set_plans(Plan::join_default().selecting(plan_select))
        .set_amendments(
            ContractAmendment::join_default().selecting(amendment_select),
        )
        .set_item_relation_agenda_protocol(
            RelAgendaProtocolItem::join_default().selecting(rel_agenda_item_select),
        );

    let mut joined_agendas = joined_select.get(db_pool).await?;

    if joined_agendas.is_empty() {
        let msg =
            format!("Повестка СК c идентификатором {} не была найдена", agenda_id);
        return Err(ProcessingError::GetItemList(msg));
    }

    Ok(joined_agendas.remove(0))
}
