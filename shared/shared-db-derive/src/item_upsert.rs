use crate::item::AGGR_INSERT;
use crate::shared::*;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

const UPSERT: &str = "DbUpsert";

pub(crate) fn inner(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);

    // Get a new name for our structure.
    let entity = &inp.ident;
    // Do we use aggregate upserts with UNNEST?
    let use_aggr = has_attr(&inp.attrs, AGGR_INSERT);

    // We allow ordinary code to handle this normally (this is an optimisation)
    if !use_aggr {
        return quote! {
            #[async_trait::async_trait]
            impl asez2_shared_db::db_item::DbUpsert for #entity {}
        }
        .into();
    }

    // If we are creating an aggregate "upserter", we conтinue.

    // Panic quickly if we are not dealing with a structure.
    // (For `sqlx` DB items enums are of dubious utility).
    let input_struct = get_struct(&inp, UPSERT, entity);
    let fields = get_named_fields(input_struct, UPSERT);

    let field_counts = (0..fields.len()).collect::<Vec<_>>();

    let field_names = fields.iter().map(|x| &x.ident).collect::<Vec<_>>();

    quote! {
        #[async_trait::async_trait]
        impl asez2_shared_db::db_item::DbUpsert for #entity {
            /// Should generate:
            ///
            /// "INSERT INTO upsert_test(id,field_1,field_2) SELECT * FROM UNNEST($1, $2, $3)
            ///    ON CONFLICT (id) DO UPDATE SET field_1=excluded.field_1,field_2=excluded.field_2
            ///    RETURNING pkey,field_1,field_2;"
            async fn upsert_returning(
                items: &mut [Self],
                update_fields: Option<&[&str]>,
                tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
            ) -> asez2_shared_db::result::Result<Vec<Self>> {
                use sqlx::FromRow;
                use asez2_shared_db::db_item::{make_bind_mask, update_fields_helper};
                use futures::TryStreamExt;

                if items.is_empty() {
                    return Ok(vec![]);
                }

                // It is preferable to only update selected fields, but all data in these fields
                // must be valid.
                let update_mask = make_bind_mask::<Self>(Self::UPDATE_FIELDS);
                let insert_mask = make_bind_mask::<Self>(Self::INSERT_FIELDS);
                let bind_mask = update_fields
                    .as_ref()
                    .map(|x| make_bind_mask::<Self>(&x))
                    .unwrap_or_else(|| update_mask.clone());
                // Define selected fields. We use the bind mask because it has some inbuilt
                // guarantees.
                let conflict_string = update_fields_helper::<Self>(&bind_mask)
                    .iter()
                    .map(|f| format!("{f}=excluded.{f}"))
                    .collect::<Vec<_>>()
                    .join(",");

                let query_string = format!(
                    "INSERT INTO {table_name}({fields})\nSELECT * FROM UNNEST{arrays}\nON CONFLICT({pkeys}) DO UPDATE SET {conflict}\nRETURNING {return_fields};",
                    table_name = Self::TABLE,
                    fields = Self::insert_fields_string(),
                    arrays = Self::insert_field_counter(0),
                    pkeys = Self::PRIMARY_KEYS.join(","),
                    conflict = conflict_string,
                    return_fields = Self::FIELDS.join(","),
                );

                // We do not want to bind more than a million values at a time, since this
                // can cause postgres to run out of memory and crash if dealing with json.
                let mut query_response = Vec::with_capacity(items.len());
                let insertable_count = Self::INSERT_FIELDS.len();
                for items in items.chunks_mut(asez2_shared_db::db_item::UPDATE_UNNEST_VALUES / insertable_count) {
                    #(
                        let mut #field_names = Vec::with_capacity(items.len());
                    )*
                    let mut q = sqlx::query(&query_string);
                    for item in items {
                        item.activate_fields();
                        #( #field_names.push(item.#field_names.to_owned()); )*
                    }
                    // Bind inserts
                    #(  if insert_mask[#field_counts] {
                            q = q.bind(&#field_names[..]);
                        }
                    )*

                    let mut stream = q.try_map(|x| Self::from_row(&x)).fetch(&mut *tx);
                    while let Some(item) = stream.try_next().await? {
                        query_response.push(item);
                    }
                }
                Ok(query_response)
            }

        }
    }
    .into()
}
