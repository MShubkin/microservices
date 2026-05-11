use async_trait::async_trait;
use shared_essential::domain::budget_item::BudgetItem;
use sqlx::PgPool;

use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
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
pub struct BudgetItemDirectory(pub(crate) MasterDataCommonDirectory<BudgetItem>);

#[async_trait]
impl MasterDataDirectoryInterface<BudgetItem> for BudgetItemDirectory {
    async fn load(&mut self, pool: &PgPool) -> MasterDataResult<()> {
        self.0.load(pool).await
    }

    async fn get_by_ids(
        &self,
        ids: &[i32],
    ) -> MasterDataResult<(Messages, Vec<BudgetItem>)> {
        self.0.get_by_ids(ids).await
    }

    async fn get_by_id(&self, id: &i32) -> MasterDataResult<BudgetItem> {
        self.0.get_by_id(id).await
    }

    async fn search(
        &self,
        search_request: &SearchByUserInput,
    ) -> MasterDataResult<(Messages, Vec<BudgetItem>)> {
        self.0.search(search_request).await
    }

    async fn get_updates(
        &self,
        timestamp: AsezTimestamp,
    ) -> MasterDataResult<Vec<BudgetItem>> {
        self.0.get_updates(timestamp).await
    }

    async fn get_full_data(&self) -> MasterDataResult<Vec<BudgetItem>> {
        self.0.get_full_data().await
    }
}

#[async_trait]
impl HierarchyGenerator for BudgetItemDirectory {
    async fn generate(&self) -> MasterDataResult<(&'static str, Hierarchy)> {
        let items = self.get_full_data().await?;
        Ok(("budget_item", build_common_hierarchy(items).into()))
    }
}

impl HierarchyEntry for BudgetItem {
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

    fn get_from_date(&self) -> AsezDate {
        self.from_date
    }

    fn get_to_date(&self) -> AsezDate {
        self.to_date
    }
}
