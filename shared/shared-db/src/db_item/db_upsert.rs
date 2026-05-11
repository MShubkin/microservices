//! This module contains `DbUpsert` which allows one to upsert in one step.
use super::*;
use crate::result::Result;

use futures::TryStreamExt;

#[async_trait::async_trait]
pub trait DbUpsert: DbItem {
    /// This is an additional function for updating or inserting that
    /// is not needed for most entities, so to avoid unnecessary codegen,
    /// it is kept here.
    async fn upsert_returning(
        items: &mut [Self],
        update_fields: Option<&[&str]>,
        tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<Vec<Self>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let insert_fields = Self::insert_fields_string();
        // It is preferable to only update selected fields, but all data in these fields
        // must be valid.

        let update_fields = update_fields.map(Self::apply_tolerance_to_fields);

        let pkeys = Self::PRIMARY_KEYS.join(",");

        // It is preferable to only update selected fields, but all data in these fields
        // must be valid.
        let bind_mask = update_fields
            .as_ref()
            .map(|x| make_bind_mask::<Self>(x))
            .unwrap_or_else(|| make_bind_mask::<Self>(Self::UPDATE_FIELDS));
        // Define selected fields. We use the bind mask because it has some inbuilt
        // guarantees.
        let update_fields = update_fields_helper::<Self>(&bind_mask);
        let conflict_string = update_fields
            .iter()
            .map(|f| format!("{f}=excluded.{f}"))
            .collect::<Vec<_>>()
            .join(",");

        let mut query_response = Vec::with_capacity(items.len());
        for items in items.chunks_mut(MAX_BINDINGS / Self::INSERT_FIELDS.len()) {
            let query_string =
                gen_upsert_query(items, &insert_fields, &pkeys, &conflict_string);
            let mut q = sqlx::query(&query_string);
            for item in items {
                item.activate_fields();
                q = item.bind_to_query_insert(q);
            }
            let mut stream = q.try_map(|x| Self::from_row(&x)).fetch(&mut *tx);
            while let Some(item) = stream.try_next().await? {
                query_response.push(item);
            }
        }

        Ok(query_response)
    }
}

/// Not inlined for testing.. Should generate.
///
/// "INSERT INTO my_table(field_1,field_2) values($1,$2),($3,$4)
///    ON CONFLICT (pkey) DO UPDATE SET field_1=excluded.field_1,field_2=excluded.field_2
///    RETURNING pkey,field_1,field_2;"
pub(super) fn gen_upsert_query<F: DbItem>(
    items: &[F],
    insert_fields: &str,
    pkeys: &str,
    conflict_string: &str,
) -> String {
    let mut query_string = format!(
        "INSERT INTO {table}({fields}) values{field_count}",
        table = F::TABLE,
        fields = insert_fields,
        field_count = F::insert_field_counter(0),
    );
    for i in 1..items.len() {
        query_string.push(',');
        query_string.push_str(&F::insert_field_counter(i));
    }
    query_string.push_str(" ON CONFLICT (");
    query_string.push_str(pkeys);
    query_string.push_str(") DO UPDATE SET ");

    query_string.push_str(conflict_string);

    query_string.push_str(" RETURNING ");
    query_string.push_str(&F::FIELDS.join(","));
    query_string.push(';');
    query_string
}
