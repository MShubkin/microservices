use std::collections::HashSet;
use std::sync::Arc;

use asez2_shared_db::db_item::Select;
use sqlx::PgPool;
use tokio::sync::RwLock;

use asez2_shared_db::DbItem;
use shared_essential::domain::master_data::routes::RouteUsers;
use shared_essential::presentation::dto::master_data::error::MasterDataResult;

#[derive(Default, Debug)]
pub(crate) struct RouteUsersDirectory {
    data: RwLock<Vec<Arc<RouteUsers>>>,
}

impl RouteUsersDirectory {
    pub(crate) async fn load(&self, pool: &PgPool) -> MasterDataResult<()> {
        let records =
            RouteUsers::select(&Select::with_fields::<&str>(&[]), pool).await?;
        self.init_data(records).await?;
        Ok(())
    }

    pub(crate) async fn get_by_route_ids(
        &self,
        route_ids: &HashSet<i64>,
    ) -> MasterDataResult<Vec<Arc<RouteUsers>>> {
        let lock = self.data.read().await;
        let result_vec = lock
            .iter()
            .filter_map(|value| {
                if route_ids.contains(&value.route_id) {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(result_vec)
    }

    async fn init_data(
        &self,
        route_users_vec: Vec<RouteUsers>,
    ) -> MasterDataResult<()> {
        let mut lock = self.data.write().await;
        lock.reserve(route_users_vec.len());
        route_users_vec.into_iter().for_each(|value| {
            lock.push(Arc::new(value));
        });
        Ok(())
    }
}
