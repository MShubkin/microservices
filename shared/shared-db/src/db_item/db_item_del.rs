//! Тут лежит код на удаление.
use super::*;
use crate::db_item::selection::FilterTree;
use crate::result::SharedDbError;

const NO_FILTERS: &str = "No filters supplied for deletion";

/// A trait for deleting DbItems. Since this is not a trait to take lightly,
/// since most objects in ASEZ never get deleted, it is not going to get a derive.
/// Designed for "update_protocol" and deleting relations between protocols and agendas.
#[async_trait::async_trait]
pub trait DbItemDel: DbItem {
    async fn delete_returning<'a, Ex>(
        filters: &FilterTree,
        conn: Ex,
    ) -> Result<Vec<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        if filters.is_empty() {
            return Err(SharedDbError::Other(NO_FILTERS.to_string()));
        }
        let mut q = format!("DELETE FROM {} WHERE", Self::TABLE);

        filters.build_sql(&mut q, 1)?;

        let return_fields = Self::FIELDS
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");

        q.push_str(" returning ");
        q.push_str(&return_fields);

        // Bind variables to delete query.
        let query = sqlx::query(&q);

        filters
            .bind_vars_to_query(query)
            .try_map(|x| Self::from_row(&x))
            .fetch_all(conn)
            .await
            .map_err(Into::into)
    }
}
