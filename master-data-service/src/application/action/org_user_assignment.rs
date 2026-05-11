use futures::{stream::FuturesOrdered, TryStreamExt};
use shared_essential::presentation::dto::{
    master_data::{
        error::MasterDataResult,
        request::{SearchByDepartment, SearchByDepartmentReq, SearchByIdReq},
        response::{OrgUserAssignmentResItem, OrgUserAssignmentSearchResponse},
    },
    response_request::ApiResponse,
};
use sqlx::PgPool;

use crate::application::master_data::org_user_assignment_search::*;

pub async fn search_by_id(
    dto: SearchByIdReq,
    _pool: &PgPool,
) -> MasterDataResult<ApiResponse<OrgUserAssignmentSearchResponse, ()>> {
    let res = organizational_user_assignment_by_id(dto.iter()).await?;
    Ok(ApiResponse::default().with_data(res))
}

pub async fn search_by_department(
    SearchByDepartmentReq(items): SearchByDepartmentReq,
    _pool: &PgPool,
) -> MasterDataResult<ApiResponse<OrgUserAssignmentSearchResponse, ()>> {
    let res = items
        .into_iter()
        .map(
            |SearchByDepartment {
                 department,
                 division,
             }| async move {
                if let Some(division) = division {
                    let precise = organizational_user_assignment(
                        0,
                        "",
                        Some(division),
                        0,
                        usize::MAX,
                    )
                    .await?;

                    if !precise.value.is_empty() {
                        return Ok(precise.value);
                    }
                }

                organizational_user_assignment(
                    0,
                    "",
                    Some(department),
                    0,
                    usize::MAX,
                )
                .await
                .map(|r| r.value)
            },
        )
        .collect::<FuturesOrdered<_>>();

    let res: Vec<Vec<OrgUserAssignmentResItem>> = res.try_collect().await?;

    Ok(ApiResponse::default().with_data(OrgUserAssignmentSearchResponse {
        value: res.into_iter().flatten().collect(),
    }))
}
