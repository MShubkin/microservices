use shared_essential::domain::routes::RouteActivationType;
use sqlx::PgPool;
use std::collections::HashSet;

use shared_essential::domain::master_data::routes::Route;
use shared_essential::presentation::dto::master_data::error::MasterDataResult;

use crate::application::master_data::routes::route_crit::RouteCritDirectory;
use crate::application::master_data::routes::route_header::RouteHeaderDirectory;
use crate::application::master_data::routes::route_users::RouteUsersDirectory;

mod route_crit;
mod route_header;
mod route_users;

/// Маршруты согласования
///TODO Убрать маршруты согласования из кеша НСИ
#[derive(Default, Debug)]
pub struct RoutesDirectory {
    /// Заголовки маршрутов согласования
    pub(crate) route_header: RouteHeaderDirectory,
    /// Критерии маршрутов согласования
    pub(crate) route_crit: RouteCritDirectory,
    /// Пользователи и периоды действия их записей для маршрутов согласования
    pub(crate) route_users: RouteUsersDirectory,
}

impl RoutesDirectory {
    pub(crate) async fn load(&self, pool: &PgPool) -> MasterDataResult<()> {
        self.route_header.load(pool).await?;
        self.route_crit.load(pool).await?;
        self.route_users.load(pool).await?;
        Ok(())
    }

    pub(crate) async fn get_routes(
        &self,
        route_type: RouteActivationType,
    ) -> MasterDataResult<Vec<Route>> {
        let headers = self.route_header.get_by_type(route_type).await?;
        let route_ids: HashSet<i64> =
            headers.iter().map(|value| value.id).collect();
        let route_crit_vec = self.route_crit.get_by_route_ids(&route_ids).await?;
        let route_users_vec = self.route_users.get_by_route_ids(&route_ids).await?;
        let result = headers
            .into_iter()
            .map(|value| {
                let header = &*value;
                Route {
                    route_header: header.clone(),
                    route_crit: route_crit_vec
                        .iter()
                        .filter(|crit| crit.route_id == header.id)
                        .map(|value| (**value).clone())
                        .collect(),
                    route_users: route_users_vec
                        .iter()
                        .filter(|user_route| user_route.route_id == header.id)
                        .map(|value| (**value).clone())
                        .collect(),
                }
            })
            .collect();
        Ok(result)
    }
}
