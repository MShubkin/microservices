use asez2_shared_db::db_item::Select;
use shared_essential::{
    domain::routes::{RouteFull, RouteHeader},
    presentation::dto::{
        master_data::{
            error::MasterDataResult,
            request::RouteFindReqItem,
            response::{FoundRouteData, FoundRoutes},
        },
        response_request::{ApiResponse, Messages},
    },
};
use sqlx::PgPool;

use crate::{
    application::routes::EvalContext,
    presentation::dto::{RouteFindReqBody, RouteFindResData},
};
use crate::{
    application::routes::{fetch_routes, route_predicate},
    presentation::dto::RouteFindItem,
};

pub async fn route_find(
    dto: RouteFindReqBody,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteFindResData, ()>> {
    tracing::info!("Поиск подходящего маршрута");

    let RouteFindReqBody { type_id, item_list } = dto;

    let mut messages = Messages::default();
    let select = Select::default()
        .eq(RouteHeader::is_active, true)
        .eq(RouteHeader::type_id, type_id);
    let routes = fetch_routes(select, pool).await?;

    let ctx = EvalContext::create().await?;
    let route_data: RouteFindResData = item_list
        .into_iter()
        .map(|item| find_routes(item, &routes, ctx.clone(), &mut messages))
        .collect();

    Ok((route_data, messages).into())
}

fn find_routes(
    item: RouteFindReqItem<RouteFindItem>,
    routes: &[RouteFull],
    ctx: EvalContext,
    messages: &mut Messages,
) -> FoundRoutes {
    let RouteFindReqItem { id, item } = item;

    let mut predicate = route_predicate(&item, ctx, messages);

    let item_list =
        routes.iter().filter(|x| predicate(x)).map(to_result_item).collect();
    FoundRoutes { id, item_list }
}

fn to_result_item(route: &RouteFull) -> FoundRouteData {
    let RouteFull { route, data, .. } = route;
    FoundRouteData {
        route_id: route.id,
        data: data.data.0.clone(),
    }
}
