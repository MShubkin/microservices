use std::sync::Arc;

use crate::app_process::external::common::MasterDataCfg;
use crate::app_process::external::enrichment::enrich_data_records;
use crate::common::{ProcessingError, Result};
use export_table::process_export_table_request;
use rabbit_services::specialized_departments::SpecializedDepartmentsService;
use shared_essential::presentation::dto::{
    general::DataRecords,
    processing::ExportReq,
    response_request::{ApiResponse, Status},
};
use sqlx::PgPool;

mod export_table;

pub(crate) async fn export_data(
    request: ExportReq,
    db_pool: Arc<PgPool>,
    sd_service: SpecializedDepartmentsService,
) -> Result<ApiResponse<DataRecords, ()>> {
    tracing::trace!(
        kind = "get",
        "Получен запрос на экспорт данных: {req:?}\n",
        req = request,
    );

    let (data, messages) =
        process_export_table_request(&request, db_pool, sd_service).await?;

    let enriched_data = enrich_data_records(
        data,
        MasterDataCfg::default(),
        request.user_id,
        &request.token,
    )
    .await
    .map_err(ProcessingError::from)?;

    Ok(ApiResponse {
        status: Status::Ok,
        data: enriched_data,
        messages,
        objects: vec![],
    })
}
