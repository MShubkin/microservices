//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
use std::sync::Arc;

use itertools::Itertools;
use shared_essential::domain::{EcAgenda, PlanOrAmendmentRep};

use asez2_shared_db::db_item::{from_item_with_fields, joined::JoinTo, Select};
#[rustfmt::skip]
use shared_essential::{
    domain::{
        ContractAmendment, EcAgendaItem,
        EcProtocolItem,  Plan, Section, EcProtocol, PlanOrAmendment,

        JoinedEcAgendaEcAgendaItemPlanContractAmendment as JoinedAgendaItemList,
        JoinedEcAgendaEcAgendaItemPlanContractAmendmentSelector as JoinedAgendaItemListSelect,

        JoinedEcProtocolEcProtocolItemPlanContractAmendment as JoinedProtocolItemList,
        JoinedEcProtocolEcProtocolItemPlanContractAmendmentSelector as JoinedProtocolItemListSelect,
    },
    presentation::dto::{processing::*, response_request::*},
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

const GET_ITEM_LIST: &str = "/v1/get/item_list/";
const RETURN_PLAN_FIELDS: &[&str] = &[
    Plan::commission_date,
    Plan::contract_subject,
    Plan::currency_id,
    Plan::customer_id,
    "plan_id", //поле дубликат
    Plan::pricing_expert_id,
    Plan::pricing_resume,
    Plan::section_id,
    Plan::supplier_id,
    Plan::status_id,
    Plan::uuid,
];

const IN_PERSON_PREPARATION_FIELDS: &[&str] = &[
    EcAgendaItem::is_excluded,
    EcAgendaItem::reviewed_at,
    EcAgendaItem::sum_excluded_vat,
    EcAgendaItem::number,
];
const SUMMING_UP_IN_PERSON_FIELDS: &[&str] = &[
    EcProtocolItem::is_excluded,
    EcProtocolItem::commission_sum_excluded_vat,
    EcProtocolItem::pricing_sum_excluded_vat,
    EcProtocolItem::sum_excluded_vat,
    EcProtocolItem::result_id,
    EcProtocolItem::number,
];
const SUMMING_UP_CORRESPONDENCE_FIELDS: &[&str] = &[
    EcProtocolItem::is_excluded,
    EcProtocolItem::commission_sum_excluded_vat,
    EcProtocolItem::pricing_sum_excluded_vat,
    EcProtocolItem::sum_excluded_vat,
    EcProtocolItem::number,
];

/// Процессинг получения элементов Повестки СК
pub(crate) async fn get_item_list(
    dto: GetItemListReq,
    db_pool: Arc<PgPool>,
) -> Result<GetItemListResponse> {
    tracing::info!(
        kind = "get",
        "Процессинг: обработка запроса на получение списка элементов Повестки СК ({get}): {req:?}\n",
        req = dto,
        get = GET_ITEM_LIST
    );

    let GetItemListReq { section_id, .. } = dto;

    let data = match section_id {
        Section::EstimatedCommissionInPersonPreparation => {
            handle_in_person_preparation_section(dto, &db_pool).await?
        }
        Section::EstimatedCommissionSummingUpInPerson => {
            handle_summing_up_in_person_section(dto, &db_pool).await?
        }
        Section::EstimatedCommissionSummingUpCorrespondence => {
            handle_summing_up_correspondence_section(dto, &db_pool).await?
        }
        _ => {
            let msg =
                format!("Секция {} недоступна для этого действия", section_id);
            return Err(ProcessingError::GetItemList(msg));
        }
    };

    Ok(ApiResponse {
        data,
        status: Status::Ok,
        ..Default::default()
    })
}

async fn handle_in_person_preparation_section(
    dto: GetItemListReq,
    db_pool: &PgPool,
) -> Result<GetItemListResponseData> {
    let GetItemListReq {
        id,
        is_registered_by_d647,
        ..
    } = dto;

    let joined_agenda =
        fetch_joined_agenda(id, is_registered_by_d647, db_pool).await?;

    let JoinedAgendaItemList {
        agenda,
        items,
        plans,
        amendments,
    } = joined_agenda;

    let mut plan_lookup = PlanOrAmendment::collect_map_by_uuid(plans, amendments);

    let plan_from_item =
        PlanOrAmendmentRep::from_item_with_fields(RETURN_PLAN_FIELDS);
    let agenda_from_item = from_item_with_fields(IN_PERSON_PREPARATION_FIELDS);
    let item_list = items
        .into_iter()
        // TODO: убрать при фиксе селекта
        .unique_by(|item| item.uuid)
        .filter_map(|agenda_item| {
            // Должно быть что то одно, так как джойнилось все по agenda_item.source_uuid
            let plan = plan_lookup.remove(&agenda_item.source_uuid)?;

            let agenda_item = agenda_from_item(agenda_item);
            let plan = plan_from_item(plan);

            Some(GetItemListItem {
                plan,
                agenda_item: Some(agenda_item),
                protocol_item: None,
            })
        })
        .sorted_unstable_by(|a, b| {
            a.agenda_item
                .as_ref()
                .expect("Выше явно задаем")
                .number
                .cmp(&b.agenda_item.as_ref().expect("Выше явно задаем").number)
        })
        .collect::<Vec<GetItemListItem>>();

    Ok(GetItemListResponseData {
        id,
        meeting_date: Some(agenda.meeting_date),
        protocol_date: None,
        item_list,
    })
}

async fn handle_summing_up_in_person_section(
    dto: GetItemListReq,
    db_pool: &PgPool,
) -> Result<GetItemListResponseData> {
    handle_protocol_section(
        dto.id,
        dto.is_registered_by_d647,
        SUMMING_UP_IN_PERSON_FIELDS,
        db_pool,
    )
    .await
}

async fn handle_summing_up_correspondence_section(
    dto: GetItemListReq,
    db_pool: &PgPool,
) -> Result<GetItemListResponseData> {
    // Для EstimatedCommissionSummingUpCorrespondence секции не передается is_registered_by_d647 признак
    handle_protocol_section(dto.id, None, SUMMING_UP_CORRESPONDENCE_FIELDS, db_pool)
        .await
}

async fn handle_protocol_section(
    protocol_id: i64,
    is_registered_by_d647: Option<bool>,
    protocol_item_fields: &[&str],
    db_pool: &PgPool,
) -> Result<GetItemListResponseData> {
    let joined_protocol =
        fetch_joined_protocol(protocol_id, is_registered_by_d647, db_pool).await?;

    let JoinedProtocolItemList {
        protocol,
        items,
        plans,
        amendments,
    } = joined_protocol;

    let mut plan_lookup = PlanOrAmendment::collect_map_by_uuid(plans, amendments);

    let plan_from_item =
        PlanOrAmendmentRep::from_item_with_fields(RETURN_PLAN_FIELDS);
    let protocol_from_item = from_item_with_fields(protocol_item_fields);
    let item_list =
        items
            .into_iter()
            // TODO: убрать при фиксе селекта
            .unique_by(|item| item.uuid)
            .filter_map(|protocol_item| {
                // Должно быть что то одно, так как джойнилось все по protocol_item.source_uuid
                let plan = plan_lookup.remove(&protocol_item.source_uuid)?;

                let plan = plan_from_item(plan);
                let protocol_item = protocol_from_item(protocol_item);

                Some(GetItemListItem {
                    plan,
                    agenda_item: None,
                    protocol_item: Some(protocol_item),
                })
            })
            .sorted_by(|a, b| {
                a.protocol_item.as_ref().expect("Выше явно задаем").number.cmp(
                    &b.protocol_item.as_ref().expect("Выше явно задаем").number,
                )
            })
            .collect::<Vec<GetItemListItem>>();

    Ok(GetItemListResponseData {
        id: protocol_id,
        meeting_date: None,
        protocol_date: Some(protocol.protocol_date),
        item_list,
    })
}

async fn fetch_joined_agenda(
    agenda_id: i64,
    is_registered_by_d647: Option<bool>,
    db_pool: &PgPool,
) -> Result<JoinedAgendaItemList> {
    let agenda_select = Select::with_fields([EcAgenda::id, EcAgenda::meeting_date])
        .eq(EcAgenda::id, agenda_id)
        .eq(EcAgenda::is_removed, false)
        .take_first();
    let mut agenda_item_select =
        Select::full::<EcAgendaItem>().eq(EcAgendaItem::is_removed, false);
    let plan_select = Select::full::<Plan>();
    let amendment_select = Select::full::<ContractAmendment>();

    if let Some(is_registered_by_d647) = is_registered_by_d647 {
        agenda_item_select = agenda_item_select
            .eq(EcAgendaItem::is_registered_by_d647, is_registered_by_d647);
    }

    let join_select = JoinedAgendaItemListSelect::new(agenda_select)
        .set_items(EcAgendaItem::join_default().selecting(agenda_item_select))
        .set_plans(Plan::join_default().selecting(plan_select))
        .set_amendments(
            ContractAmendment::join_default().selecting(amendment_select),
        )
        .distinct();

    let mut joined_agendas = join_select.get(db_pool).await?;

    if joined_agendas.is_empty() {
        let msg =
            format!("Повестка СК c идентификатором {} не была найдена", agenda_id);
        return Err(ProcessingError::GetItemList(msg));
    }

    Ok(joined_agendas.remove(0))
}

async fn fetch_joined_protocol(
    protocol_id: i64,
    is_registered_by_d647: Option<bool>,
    db_pool: &PgPool,
) -> Result<JoinedProtocolItemList> {
    let protocol_select =
        Select::with_fields([EcProtocol::id, EcProtocol::protocol_date])
            .eq(EcProtocol::id, protocol_id)
            .eq(EcProtocol::is_removed, false)
            .take_first();
    let mut protocol_item_select =
        Select::full::<EcProtocolItem>().eq(EcProtocolItem::is_removed, false);
    let plan_select = Select::full::<Plan>();
    let amendment_select = Select::full::<ContractAmendment>();

    if let Some(is_registered_by_d647) = is_registered_by_d647 {
        protocol_item_select = protocol_item_select
            .eq(EcProtocolItem::is_registered_by_d647, is_registered_by_d647);
    }

    let join_select = JoinedProtocolItemListSelect::new(protocol_select)
        .set_items(EcProtocolItem::join_default().selecting(protocol_item_select))
        .set_plans(Plan::join_default().selecting(plan_select))
        .set_amendments(
            ContractAmendment::join_default().selecting(amendment_select),
        )
        .distinct();

    let mut joined_protocols = join_select.get(db_pool).await?;

    if joined_protocols.is_empty() {
        let msg =
            format!("Протокол СК c идентификатором {} не был найден", protocol_id);
        return Err(ProcessingError::GetItemList(msg));
    }

    Ok(joined_protocols.remove(0))
}
