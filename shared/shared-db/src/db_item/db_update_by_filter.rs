//! Тут лежит код на удаление.
use super::*;
use crate::db_item::selection::FilterTree;
use crate::result::SharedDbError;

const NO_FILTERS: &str = "No filters supplied for update by filters";

/// A trait for updating items with the same values for fields given set of filters.
/// For the sake of tight control of types, we do use DbItem initially, however,
/// it can be constructed directly. For example:
///
/// ```ignore
/// let update_values = MyItem {
///     customer_name: "Bob".to_string(),
///     order_type: 55,
///     ..MyItem::default()
/// };
/// let filters = Filter::eq(MyItem::year, 2065);
/// let update_fields = [MyItem::customer_name, MyItem::order];
///
/// // This is "UPDATE my_item SET (customer_name, order_type)=('Bob',55) WHERE year=2065"
/// update_values.update_by_filter(&update_fields, &filters, &pool).await?;
/// ```
///
/// The interface would be more comfortable if update values and update fields were
/// the same variable, but it is possible that we only wish to update a subset of the
/// `update_values` depending on some circumstances, in which case this may be rational.
#[async_trait::async_trait]
pub trait DbUpdateByFilter: DbItem {
    async fn update_by_filter<'a, Ex>(
        &self,
        update_fields: &[&str],
        filters: &FilterTree,
        conn: Ex,
    ) -> Result<u64>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        match update_by_filter_inner(self, update_fields, filters, None, conn)
            .await?
        {
            ReturningEither::RowsAffected(x) => Ok(x),
            _ => panic!("`update_by_filter` only returns rows."),
        }
    }

    async fn update_by_filter_returning<'a, Ex>(
        &self,
        update_fields: &[&str],
        filters: &FilterTree,
        returning_fields: Option<&[&str]>,
        conn: Ex,
    ) -> Result<Vec<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        let returning_fields = match returning_fields {
            Some(f) => Some(f),
            None => Some(Self::FIELDS),
        };
        match update_by_filter_inner(
            self,
            update_fields,
            filters,
            returning_fields,
            conn,
        )
        .await?
        {
            ReturningEither::Items(x) => Ok(x),
            _ => panic!("`update_by_filter_returning` only returns items."),
        }
    }
}

async fn update_by_filter_inner<'a, Ex, T: DbItem>(
    item: &T,
    update_fields: &[&str],
    filters: &FilterTree,
    return_fields: Option<&[&str]>,
    conn: Ex,
) -> Result<ReturningEither<T>>
where
    Ex: Executor<'a, Database = Postgres>,
{
    if filters.is_empty() {
        return Err(SharedDbError::Other(NO_FILTERS.to_string()));
    }
    // The first half of the function is the same as a normal update.
    let tolerated_fields = T::apply_tolerance_to_fields(update_fields);
    let bind_mask =
        DbFieldMask::<T>::with_fields(&tolerated_fields).without_autogen();
    // Define selected fields. We use the bind mask because it has some inbuilt
    // guarantees. NB: We do not use the classic `update_fields_helper`, since we
    // do not update by pkey.
    let selected_fields = select_fields_helper::<T>(&bind_mask);
    let field_count = field_counter(&selected_fields, 0);

    let fields = selected_fields.join(",");

    let fields_to_set = if selected_fields.len() == 1 {
        format!("{}={}", fields, field_count)
    } else {
        format!("({})=({})", fields, field_count)
    };

    // Build initial part "UPDATE my_table SET (field_a, field_b)=(a, b) WHERE"
    let mut q = format!(
        "UPDATE {table} SET {fields_to_set} WHERE",
        table = T::TABLE,
        fields_to_set = fields_to_set,
    );
    // Add content of the WHERE clause
    filters.build_sql(&mut q, selected_fields.len() + 1)?;

    // ADD " returning field_c,field_d" if we have return fields.
    if let Some(return_fields) = return_fields {
        let return_fields = T::apply_tolerance_to_fields(return_fields);
        let return_clause = {
            let bind_mask = make_bind_mask::<T>(&return_fields);
            let fields = select_fields_helper::<T>(&bind_mask);
            format!(" RETURNING {}", fields.join(","))
        };
        q.push_str(&return_clause);
    }

    // Bind variables to update query
    let mut query = sqlx::query(&q);
    query = item.bind_limited_fields(query, &bind_mask);
    query = filters.bind_vars_to_query(query);

    if return_fields.is_none() {
        let ret = query.execute(conn).await?.rows_affected();
        Ok(ReturningEither::RowsAffected(ret))
    } else {
        let ret = query.try_map(|r| T::from_row(&r)).fetch_all(conn).await?;
        Ok(ReturningEither::Items(ret))
    }
}
