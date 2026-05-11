//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
use std::sync::Arc;

use itertools::Itertools;
use sqlx::{FromRow, PgPool};

use shared_essential::{
    domain::{processing::price_analysis_user::UserType, Section},
    presentation::dto::{processing::price_analysis::*, response_request::*},
};

use crate::common::Result;

const SECTIONS_GET_COUNT: &str = "/v1/sections/get/count";
const SECTION_SUM_CASES: &[(&str, &str, Section)] = &[
    ("CASE WHEN status_id IN (221, 341, 351, 371) THEN 1 ELSE 0 END", "assign_expert", Section::PriceAnalysisAssignExpert),
    ("CASE WHEN status_id IN (222, 342, 352, 372) AND is_check_documentation = TRUE THEN 1 ELSE 0 END", "determine_price", Section::PriceAnalysisDeterminePrice),
    ("CASE WHEN status_id IN (222, 342, 352, 372) AND is_check_documentation = FALSE THEN 1 ELSE 0 END", "primary_expert_control", Section::PriceAnalysisPrimaryExpertControl),
    ("CASE WHEN status_id IN (223, 343, 353, 373) THEN 1 ELSE 0 END", "approve_price", Section::PriceAnalysisApprovePrice),
    ("CASE WHEN status_id IN (356) THEN 1 ELSE 0 END", "lotting_mtr", Section::PriceAnalysisLottingMTP)
];

#[derive(Default, FromRow)]
struct FetchedSectionSums {
    #[sqlx(default)]
    assign_expert: Option<i64>,
    #[sqlx(default)]
    determine_price: Option<i64>,
    #[sqlx(default)]
    primary_expert_control: Option<i64>,
    #[sqlx(default)]
    approve_price: Option<i64>,
    #[sqlx(default)]
    lotting_mtr: Option<i64>,
}

/// Функция по передаче полных планов с документами.
/// НБ: Пока что тут не нужны, секции (так как везде None).
#[tracing::instrument(skip_all)]
pub(crate) async fn pa_get_sections_count(
    req: GetSectionsCountRequest,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetSectionsCountResponse, ()>> {
    tracing::info!(
        kind = "get",
        "Processing: Got request to send to plans on ({get}): {req:?}\n",
        req = req,
        get = SECTIONS_GET_COUNT
    );

    let GetSectionsCountRequest {
        departments,
        user_id,
        user_type,
        section_list,
    } = req;

    let expert_filter = match user_type {
        UserType::Expert => "AND pricing_expert_id = $2",
        _ => "",
    };

    let requested_sums = section_list
        .into_iter()
        .filter_map(|requested_section| {
            SECTION_SUM_CASES
                .iter()
                .find(|(_, _, section)| requested_section == *section)
        })
        .map(|(sum_case, response_name, _)| {
            format!("COALESCE(SUM({}), 0) as {}", sum_case, response_name)
        })
        .join(",");
    let query = format!("
    SELECT
        {sum_cases}
    FROM (
        SELECT status_id, is_check_documentation
        FROM plan
        WHERE is_actual = true AND pricing_organization_unit_id = ANY($1) {expert_filter}
        UNION ALL
        SELECT status_id, is_check_documentation
        FROM contract_amendment
        WHERE is_actual = true AND pricing_organization_unit_id = ANY($1) {expert_filter}
    )", sum_cases = requested_sums, expert_filter = expert_filter);

    let departments = departments.into_iter().map(Into::into).collect::<Vec<i16>>();
    let data: FetchedSectionSums = match user_type {
        UserType::Expert => {
            sqlx::query_as(&query)
                .bind(&departments[..])
                .bind(user_id)
                .fetch_one(db_pool.as_ref())
                .await?
        }
        _ => {
            sqlx::query_as(&query)
                .bind(&departments[..])
                .fetch_one(db_pool.as_ref())
                .await?
        }
    };

    Ok((data, vec![]).into())
}

impl From<FetchedSectionSums> for GetSectionsCountResponse {
    fn from(val: FetchedSectionSums) -> Self {
        GetSectionsCountResponse {
            approve_price: val.approve_price,
            assign_expert: val.assign_expert,
            determine_price: val.determine_price,
            lotting_mtr: val.lotting_mtr,
            primary_expert_control: val.primary_expert_control,
        }
    }
}
