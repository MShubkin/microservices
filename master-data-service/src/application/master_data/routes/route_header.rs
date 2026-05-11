use std::sync::Arc;

use asez2_shared_db::db_item::Select;
use sqlx::PgPool;
use tokio::sync::RwLock;

use asez2_shared_db::DbItem;
use shared_essential::domain::master_data::routes::{
    RouteActivationType, RouteHeader,
};
use shared_essential::presentation::dto::master_data::error::MasterDataResult;

#[derive(Default, Debug)]
pub(crate) struct RouteHeaderDirectory {
    data: RwLock<Vec<Arc<RouteHeader>>>,
}

impl RouteHeaderDirectory {
    pub(crate) async fn load(&self, pool: &PgPool) -> MasterDataResult<()> {
        let records =
            RouteHeader::select(&Select::with_fields::<&str>(&[]), pool).await?;
        self.init_data(records).await?;
        Ok(())
    }

    pub(crate) async fn get_by_type(
        &self,
        route_type: RouteActivationType,
    ) -> MasterDataResult<Vec<Arc<RouteHeader>>> {
        let lock = self.data.read().await;
        let result_vec = lock
            .iter()
            .filter_map(|value| match route_type {
                RouteActivationType::Plan => {
                    if value.is_plan {
                        Some(value.clone())
                    } else {
                        None
                    }
                }
                RouteActivationType::ContractAmendment => {
                    if value.is_contract_amendment {
                        Some(value.clone())
                    } else {
                        None
                    }
                }
                //TODO поля "rpur_flag" больше не существует, добавить другую проверку
                RouteActivationType::PriceAnalysis => Some(value.clone()),
                RouteActivationType::Undefined => None,
            })
            .collect();
        Ok(result_vec)
    }

    async fn init_data(
        &self,
        route_header_vec: Vec<RouteHeader>,
    ) -> MasterDataResult<()> {
        let mut lock = self.data.write().await;
        lock.reserve(route_header_vec.len());
        route_header_vec.into_iter().for_each(|value| {
            lock.push(Arc::new(value));
        });
        Ok(())
    }

    /// Подгрузить типы маршрутов
    #[allow(unused)]
    fn upload_route_type() {
        todo!()
    }
}
