use std::sync::Arc;

use crate::common::{ProcessingError, Result};
use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbAdaptor;
use shared_essential::domain::{EcAgenda, EcAgendaRep};
use shared_essential::presentation::dto::processing::{
    GetAgendaListByDateReq, GetAgendaListByDateResponse,
};
use shared_essential::presentation::dto::response_request::{
    ApiResponse, PaginatedData, Status,
};
use sqlx::PgPool;

const GET_AGENDA_LIST_BY_DATE: &str =
    "/rest/estimated_commission/v1/get/agenda_list_by_date/";

const RETURN_AGENDA_FIELDS: &[&str] =
    &[EcAgenda::uuid, EcAgenda::id, EcAgenda::pricing_organization_unit_id];

// Процессинг получения элементов Повестки СК по дате
pub(crate) async fn get_agenda_list_by_date(
    dto_request: GetAgendaListByDateReq,
    db_pool: Arc<PgPool>,
) -> Result<GetAgendaListByDateResponse> {
    tracing::info!(
        kind = "get",
        "Процессинг получения элементов Повестки СК по дате ({get}): {req:?}\n",
        req = dto_request,
        get = GET_AGENDA_LIST_BY_DATE
    );

    let GetAgendaListByDateReq { date, date_type } = dto_request;
    check_date_type(&date_type)?;

    let agenda_list_select: Select = Select::with_fields(RETURN_AGENDA_FIELDS)
        .eq(date_type.as_ref(), date)
        .eq(EcAgenda::is_removed, false);

    let mapped_list = EcAgendaRep::select(&agenda_list_select, &*db_pool).await?;

    Ok(ApiResponse {
        data: PaginatedData::new(&agenda_list_select, mapped_list),
        status: Status::Ok,
        ..Default::default()
    })
}

/// Доступные поля у сущности [`EcAgenda`] с типом дата: meeting_date
fn check_date_type(date_type: &String) -> Result<()> {
    match date_type.as_ref() {
        "meeting_date" => Ok(()),
        _ => {
            let msg = format!(
                "Невалидное значения поля для \"date_type\": \"{}\"",
                date_type
            );
            Err(ProcessingError::GetAgendaListByDate(msg))
        }
    }
}
