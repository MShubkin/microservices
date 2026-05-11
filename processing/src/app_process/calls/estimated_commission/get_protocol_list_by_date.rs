use std::sync::Arc;

use asez2_shared_db::{db_item::Select, DbAdaptor};
use shared_essential::{
    domain::{EcProtocol, EcProtocolRep, ProtocolType},
    presentation::dto::{
        processing::{GetProtocolListByDateReq, GetProtocolListByDateResponse},
        response_request::{ApiResponse, PaginatedData, Status},
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

const GET_PROTOCOL_LIST_BY_DATE: &str =
    "/rest/estimated_commission/v1/get/protocol_list_by_date";

const PROTOCOL_FIELDS: &[&str] = &[
    "protocol_id",
    EcProtocol::uuid,
    EcProtocol::pricing_organization_unit_id,
];

pub(crate) async fn get_protocol_list_by_date(
    dto: GetProtocolListByDateReq,
    db_pool: Arc<PgPool>,
) -> Result<GetProtocolListByDateResponse> {
    tracing::info!(
        kind = "get",
        "Процессинг: Получение списка Протоколов СК по дате ({get}): {req:?}\n",
        req = dto,
        get = GET_PROTOCOL_LIST_BY_DATE
    );

    let GetProtocolListByDateReq {
        date,
        date_type,
        protocol_type_id,
    } = dto;
    check_date_type(&date_type)?;
    check_protocol_type_id(protocol_type_id)?;

    let protocol_list_select: Select = Select::with_fields(PROTOCOL_FIELDS)
        .eq(date_type.as_ref(), date)
        .eq(EcProtocol::protocol_type_id, protocol_type_id)
        .eq("is_removed", false);

    let mapped_list =
        EcProtocolRep::select(&protocol_list_select, &*db_pool).await?;

    Ok(ApiResponse {
        data: PaginatedData::new(&protocol_list_select, mapped_list),
        status: Status::Ok,
        ..Default::default()
    })
}

#[derive(thiserror::Error, Debug)]
pub enum GetProtocolListByDateError {
    #[error(r#"Невалидное значения поля для "date_type": "{0}""#)]
    InvalidDate(String),
    #[error(r#"Невалидное значения поля для "protocol_type_id": "{0}""#)]
    InvalidType(ProtocolType),
}

impl From<GetProtocolListByDateError> for ProcessingError {
    fn from(value: GetProtocolListByDateError) -> Self {
        ProcessingError::GetProtocolListByDate(value)
    }
}

/// Доступные значения поля date_type у запроса Протокола по дате.
fn check_date_type(date_type: &String) -> Result<()> {
    match date_type.as_ref() {
        "protocol_date" => Ok(()),
        _ => Err(GetProtocolListByDateError::InvalidDate(date_type.clone()))?,
    }
}

/// Доступные значения поля protocol_type_id
fn check_protocol_type_id(protocol_type_id: ProtocolType) -> Result<()> {
    if matches!(protocol_type_id, ProtocolType::Undefined) {
        return Err(GetProtocolListByDateError::InvalidType(protocol_type_id))?;
    }
    Ok(())
}
