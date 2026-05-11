use asez2_shared_db::db_item::Select;
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use asez2_shared_db::DbItem;
use shared_essential::{
    domain::{master_data::routes::RouteCrit, routes::RouteCritName},
    presentation::dto::master_data::error::MasterDataResult,
};

#[derive(Default, Debug)]
pub(crate) struct RouteCritDirectory {
    data: RwLock<Vec<Arc<RouteCrit>>>,
}

impl RouteCritDirectory {
    pub(crate) async fn load(&self, pool: &PgPool) -> MasterDataResult<()> {
        //TODO получить через обычный sqlx route_crit и уже после этого добавиться значение из route_crit_name
        let route_crit_name =
            RouteCritName::select(&Select::with_fields::<&str>(&[]), pool).await?;
        let records =
            RouteCrit::select(&Select::with_fields::<&str>(&[]), pool).await?;
        self.init_data(records, route_crit_name).await?;
        Ok(())
    }

    pub(crate) async fn get_by_route_ids(
        &self,
        route_ids: &HashSet<i64>,
    ) -> MasterDataResult<Vec<Arc<RouteCrit>>> {
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
        route_crit_vec: Vec<RouteCrit>,
        _route_crit_name_vec: Vec<RouteCritName>,
    ) -> MasterDataResult<()> {
        let mut lock = self.data.write().await;
        lock.reserve(route_crit_vec.len());
        route_crit_vec.into_iter().for_each(|value| {
            lock.push(Arc::new(value));
        });
        Ok(())
    }
}
