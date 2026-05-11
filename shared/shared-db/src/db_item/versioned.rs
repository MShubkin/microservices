//! Module for versioning DbItems.
use super::*;
pub use shared_db_derive::DbVersioned;

use ahash::AHashMap;
use sqlx::FromRow;

#[async_trait::async_trait]
/// ТОДО: Пока что логика insert и update почти такие же, так как надо
/// чтобы код срабатывал для новых версии, даже если insert.
pub trait DbVersioned: DbItem {
    type Versioned: DbItem;

    fn to_versioned(&self, pricing_version: i16) -> Self::Versioned;

    fn to_active(v: &Self::Versioned) -> Self;

    fn id(&self) -> i64;

    /// Создать версии без обновления и других.
    #[tracing::instrument(skip_all)]
    async fn insert_version_vec_returning(
        items: &[Self],
        pool: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<Vec<Self::Versioned>> {
        insert_vec_inner(items, pool).await
    }
}

/// Когда вставляем версии, разрешено вставлять несколько записей с тем же самым id.
/// При этом версии у них должны отличаться (эта функция должна это гарантировать).
/// С этой функцией надо осторожно, так как может вызывать "deadlock".
async fn insert_vec_inner<T: DbVersioned>(
    items: &[T],
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<Vec<T::Versioned>> {
    let v_table = T::Versioned::TABLE;
    // Если не запирать таким образом, то будет беда. отпирать можно ТОЛЬКО когда вся транзакция
    // закончит, иначе опять будет читать старые записи.
    let lock_query = format!("LOCK TABLE {v_table} IN ACCESS EXCLUSIVE MODE");
    sqlx::query(&lock_query).execute(&mut *tx).await?;

    let ids = items.iter().map(|x| x.id()).collect::<Vec<_>>();
    let query_string = format!(
        "SELECT id,max(pricing_version) FROM {v_table} WHERE id=ANY($1) GROUP BY id",
    );
    tracing::info!(kind = "infra", "Input ids: {:?}", ids);

    let mut ver = sqlx::query(&query_string)
        .bind(ids)
        .try_map(|r| <(i64, i16)>::from_row(&r))
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .collect::<AHashMap<i64, i16>>();

    tracing::info!(kind = "infra", "Output (id, version_asez2)s: {:?}", ver);

    let mut new_versions = items
        .iter()
        .map(|x| {
            let version = ver.entry(x.id()).and_modify(|x| *x += 1).or_insert(1);
            x.to_versioned(*version)
        })
        .collect::<Vec<_>>();
    T::Versioned::insert_vec_returning(&mut new_versions, &mut *tx).await
}
