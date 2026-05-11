use asez2_shared_db::{db_item::AsezTimestamp, DbItem};
use async_trait::async_trait;
use shared_essential::presentation::dto::{
    master_data::{error::MasterDataResult, request::SearchByUserInput},
    response_request::Messages,
};
use sqlx::PgPool;

/// Интерфейс записи словаря NSI.
pub trait MasterDataRecord {
    /// Идентификатор записи
    fn id(&self) -> i32;

    /// Дата изменения записи справочника
    fn changed_at(&self) -> AsezTimestamp;

    /// Поиск по значениям полей справочника
    fn search_record(&self, search: &str) -> bool;
}

/// Интерфейс словаря NSI.
#[async_trait]
#[allow(dead_code)]
pub trait MasterDataDirectoryInterface<T: MasterDataRecord + Sync + Send + DbItem> {
    /// Загрузка словаря из БД.
    async fn load(&mut self, pool: &PgPool) -> MasterDataResult<()>;

    /// Поиск по id.
    async fn get_by_ids(&self, ids: &[i32])
        -> MasterDataResult<(Messages, Vec<T>)>;

    /// Выборка элемента по id.
    async fn get_by_id(&self, id: &i32) -> MasterDataResult<T>;

    /// Строковый поиск.
    async fn search(
        &self,
        search_request: &SearchByUserInput,
    ) -> MasterDataResult<(Messages, Vec<T>)>;

    async fn get_updates(
        &self,
        timestamp: AsezTimestamp,
    ) -> MasterDataResult<Vec<T>>;

    async fn get_full_data(&self) -> MasterDataResult<Vec<T>>;
}
