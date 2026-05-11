use async_trait::async_trait;
use shared_essential::domain::category::Category;
use sqlx::PgPool;

use asez2_shared_db::db_item::AsezTimestamp;
use shared_essential::presentation::dto::{
    master_data::{error::MasterDataResult, request::SearchByUserInput},
    response_request::Messages,
};

use crate::application::hierarchical_values::{
    Hierarchy, HierarchyEntry, HierarchyGenerator,
};
use crate::domain::MasterDataDirectoryInterface;

use super::{build_common_hierarchy, MasterDataCommonDirectory};

#[derive(Debug, Default)]
pub struct CategoryDirectory(pub(crate) MasterDataCommonDirectory<Category>);

#[async_trait]
impl MasterDataDirectoryInterface<Category> for CategoryDirectory {
    async fn load(&mut self, pool: &PgPool) -> MasterDataResult<()> {
        self.0.load(pool).await
    }

    async fn get_by_ids(
        &self,
        ids: &[i32],
    ) -> MasterDataResult<(Messages, Vec<Category>)> {
        self.0.get_by_ids(ids).await
    }

    async fn get_by_id(&self, id: &i32) -> MasterDataResult<Category> {
        self.0.get_by_id(id).await
    }

    async fn search(
        &self,
        search_request: &SearchByUserInput,
    ) -> MasterDataResult<(Messages, Vec<Category>)> {
        self.0.search(search_request).await
    }

    async fn get_updates(
        &self,
        timestamp: AsezTimestamp,
    ) -> MasterDataResult<Vec<Category>> {
        self.0.get_updates(timestamp).await
    }

    async fn get_full_data(&self) -> MasterDataResult<Vec<Category>> {
        self.0.get_full_data().await
    }
}

#[async_trait]
impl HierarchyGenerator for CategoryDirectory {
    async fn generate(&self) -> MasterDataResult<(&'static str, Hierarchy)> {
        let items = self.get_full_data().await?;
        Ok(("category", build_common_hierarchy(items).into()))
    }
}

impl HierarchyEntry for Category {
    type Id = i16;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn parent_id(&self) -> Self::Id {
        self.parent_id
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn is_removed(&self) -> bool {
        self.is_removed
    }

    fn get_from_date(&self) -> asez2_shared_db::db_item::AsezDate {
        self.from_date
    }

    fn get_to_date(&self) -> asez2_shared_db::db_item::AsezDate {
        self.to_date
    }
}
