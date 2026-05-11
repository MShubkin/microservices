use std::sync::Arc;

use asez2_shared_db::{
    db_item::{AsezTimestamp, Select, SelectionKind},
    DbItem,
};
use shared_essential::{
    domain::processing::price_analysis_user::PriceAnalysisUser,
    presentation::dto::{
        processing::price_analysis::{
            GetPriceAnalysisUsersReq, GetPriceAnalysisUsersResponseData,
        },
        response_request::ApiResponse,
    },
};
use sqlx::PgPool;

use crate::common::Result;

pub(crate) async fn get_price_analysis_user(
    req: GetPriceAnalysisUsersReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetPriceAnalysisUsersResponseData, ()>> {
    let GetPriceAnalysisUsersReq {
        unit_ids,
        user_ids,
        user_types,
    } = req;

    let now = AsezTimestamp::now();
    let select = Select::full::<PriceAnalysisUser>()
        .in_any_maybe(PriceAnalysisUser::user_id, user_ids)
        .in_any_maybe(PriceAnalysisUser::pricing_organization_unit_id, unit_ids)
        .in_any_maybe(PriceAnalysisUser::type_user_id, user_types)
        .eq(PriceAnalysisUser::is_removed, false)
        .add_expand_filter(
            PriceAnalysisUser::start_date,
            SelectionKind::Less,
            [now],
        )
        .add_expand_filter(
            PriceAnalysisUser::end_date,
            SelectionKind::Greater,
            [now],
        );

    let users = PriceAnalysisUser::select(&select, db_pool.as_ref()).await?;

    Ok(ApiResponse::default().with_data(users))
}
