use std::{collections::HashMap, sync::Arc};

use crate::common::Result;
use shared_essential::presentation::dto::{
    processing::price_analysis::{
        GetPlansWithLastAgendaItemsReq, GetPlansWithLastAgendaItemsRes,
    },
    response_request::{ApiResponse, Messages},
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Запрос на получение повестки по плану
#[tracing::instrument(skip_all)]
pub(crate) async fn get_plans_with_last_agenda_items(
    req: GetPlansWithLastAgendaItemsReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetPlansWithLastAgendaItemsRes, ()>> {
    let db_conn = db_pool.as_ref();

    tracing::info!(kind = "get", "Получение повесток по плану: {req:?}", req = req,);

    let GetPlansWithLastAgendaItemsReq { plans_uuid } = req;
    let response_data = GetPlansWithLastAgendaItemsRes {
        last_agenda_item_hashmap: get_plans_with_last_agenda_items_query(
            &plans_uuid,
            db_conn,
        )
        .await?,
    };
    Ok(ApiResponse::from((response_data, Messages::default())))
}

/// Получение последних не удаленных и не исключенных повесток по UUID ППЗ/ДС
async fn get_plans_with_last_agenda_items_query(
    plans_uuids: &[Uuid],
    db_conn: &PgPool,
) -> Result<HashMap<Uuid, Uuid>> {
    // TODO: Когда наша ORM будет готова к таким запросам, то следует переписать и не оставлять голый SQL запрос

    let query = "
        WITH combined_plans AS (
            SELECT uuid, id FROM plan WHERE uuid = ANY ($1)
            UNION ALL
            SELECT uuid, id FROM contract_amendment WHERE uuid = ANY ($1)
        ),
        latest_agendas_items AS (
            SELECT
                agenda_item.uuid AS ai_uuid,
                agenda_item.source_uuid AS ai_source_uuid,
                ROW_NUMBER() OVER (
                    PARTITION BY agenda_item.source_uuid
                    ORDER BY agenda.created_at DESC, agenda_item.created_at DESC
                ) AS ai_row_num
            FROM
                agenda
            INNER JOIN
                agenda_item ON agenda_item.agenda_uuid = agenda.uuid
            INNER JOIN
                combined_plans cp ON agenda_item.source_uuid = cp.uuid
            WHERE
                agenda.is_removed = false
                AND agenda_item.is_removed = false
                AND agenda_item.is_excluded = false
        )
        SELECT
            combined_plans.uuid AS plan_uuid,
            latest_agendas_items.ai_uuid
        FROM
            combined_plans
        LEFT JOIN
            latest_agendas_items ON latest_agendas_items.ai_source_uuid = combined_plans.uuid 
            AND latest_agendas_items.ai_row_num = 1
        WHERE
            latest_agendas_items.ai_row_num = 1;";

    let rows = sqlx::query(query).bind(plans_uuids).fetch_all(db_conn).await?;

    let result: HashMap<Uuid, Uuid> = rows
        .into_iter()
        .filter_map(|row| {
            let plan_uuid: Uuid = row.try_get("plan_uuid").ok()?;
            let latest_agenda_uuid: Option<Uuid> = row.try_get("ai_uuid").ok();

            latest_agenda_uuid.map(|agenda_uuid| (plan_uuid, agenda_uuid))
        })
        .collect();

    Ok(result)
}
