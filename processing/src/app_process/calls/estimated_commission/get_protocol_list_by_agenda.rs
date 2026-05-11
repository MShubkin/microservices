use std::sync::Arc;

use asez2_shared_db::db_item::{AdaptorableIter, Select};
use shared_essential::{
    domain::{
        EcAgenda, EcProtocol,
        JoinedEcAgendaRelAgendaProtocolEcProtocolSelector as AgendaWithProtocolsSelector,
    },
    presentation::dto::{
        processing::{
            GetProtocolListByAgendaReq, GetProtocolListByAgendaResponse,
            GetProtocolListByAgendaResponseData,
        },
        response_request::ApiResponse,
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

const GET_PROTOCOL_LIST_BY_AGENDA: &str =
    "/rest/estimated_commission/v1/get/protocol_list_by_agenda";

const PROTOCOL_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    "protocol_id",
    "protocol_status_id",
    EcProtocol::protocol_date,
    EcProtocol::registration_number,
    EcProtocol::pricing_organization_unit_id,
];

pub(crate) async fn get_protocol_list_by_agenda(
    request: GetProtocolListByAgendaReq,
    db_pool: Arc<PgPool>,
) -> Result<GetProtocolListByAgendaResponse> {
    tracing::info!(
        kind = "get",
        "Получен запрос на получение списка Протоколов по Повестке СК ({get}): {req:?}\n",
        req = request,
        get = GET_PROTOCOL_LIST_BY_AGENDA
    );

    let agenda_select =
        Select::with_fields([EcAgenda::uuid, EcAgenda::meeting_date])
            .eq(EcAgenda::uuid, request.uuid);
    let joined_agenda = AgendaWithProtocolsSelector::new(agenda_select)
        .get(db_pool.as_ref())
        .await?
        .pop()
        .ok_or_else(|| {
            ProcessingError::GetProtocolListByAgenda(format!(
                "Повестка СК № {} не найдена",
                request.id
            ))
        })?;

    let agenda = joined_agenda.agenda;
    let protocol_list = joined_agenda
        .protocols
        .into_iter()
        .adaptors_with_fields(PROTOCOL_FIELDS)
        .collect();

    let response = GetProtocolListByAgendaResponseData {
        id: agenda.id,
        commission_date: agenda.meeting_date,
        item_list: protocol_list,
    };

    Ok(ApiResponse::default().with_data(response))
}
