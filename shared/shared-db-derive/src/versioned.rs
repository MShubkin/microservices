use crate::shared::*;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, parse_quote};
use syn::{Attribute, DeriveInput, Field};

const VERSIONED: &str = "versioned";
const DB_VERSION_TABLE: &str = "db_version_table";

pub(crate) fn inner(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);

    // Get a new name for our structure.
    let old_name = &inp.ident;
    let default_name =
        Ident::new(&format!("{}Version", old_name), Span::call_site());
    let new_name = rename(&inp.attrs, VERSIONED, &default_name);

    // Panic quickly if we are not dealing with a structure.
    // (For `sqlx` DB items enums are of dubious utility).
    let input_struct = get_struct(&inp, VERSIONED, old_name);
    let fields = get_named_fields(input_struct, "DbVersioned");

    let table_name = get_attr_ident(&inp.attrs, DB_VERSION_TABLE)
        .expect("`db_version_table` MUST be specified for `derive(DbVersioned)`.");

    let old_field_names =
        fields.iter().map(|x| x.ident.as_ref().unwrap()).collect::<Vec<_>>();

    // This is the big panic that stops us before we go too far.
    if fields.is_empty() {
        panic!("`DbVersioned` does not deal with empty structures.");
    }

    // We mostly keep the old attributes: DbVersion structures should generally not
    // be distinguishable from the parent structure.
    // However, we do wish to remove the `item_table` attribute as the versioned
    // struct uses a different table.
    let outer_attributes = filter_attributes(&inp.attrs);
    // We remove any `item_field_pkey` attributes, and if we find an `id` field add it there.
    let modified_fields = modify_fields(&fields);

    let table_name = table_name.to_string();
    let version_table: Attribute = parse_quote!(#[item_table = #table_name]);

    quote! {
        // derive attributes are not inherited. For simplicity we only derive
        // the bare minimum.
        #[derive(
            Debug,
            Default,
            Clone,
            PartialEq,
            DbItem,
            DbAdaptor)]
        #[adaptor_derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
        #( #outer_attributes )*
        #version_table
        pub struct #new_name {
            #[item_field_pkey]
            pub pricing_version: i16,
            #(
                #modified_fields,
            )*
        }

        #[async_trait::async_trait]
        impl asez2_shared_db::db_item::DbVersioned for #old_name {
            type Versioned = #new_name;

            #[allow(clippy::needless_update)]
            fn to_versioned(&self, pricing_version: i16) -> #new_name {
                #new_name {
                    pricing_version,
                    #( #old_field_names: self.#old_field_names.to_owned(), )*
                    ..Default::default()
                }
            }

            fn to_active(v: &Self::Versioned) -> #old_name {
                #old_name {
                    #( #old_field_names: v.#old_field_names.to_owned(), )*
                }
            }

            fn id(&self) -> i64 {
                self.id as i64
            }
        }
    }
    .into()
}

fn filter_attributes(old_attrs: &[Attribute]) -> Vec<Attribute> {
    old_attrs
        .iter()
        .filter(|x| {
            // Keep attributes from db_item. Likewise, keep adaptor attributes,
            // associated with field names and types, but not with derives.
            (x.path().is_ident("sqlx")
                || x.path().is_ident("db_field_name")
                || x.path()
                    .get_ident()
                    .unwrap()
                    .to_string()
                    .starts_with("adaptor_")
                || x.path().get_ident().unwrap().to_string().starts_with("item_"))
                && !x.path().is_ident(crate::item::ITEM_TABLE)
                && !x.path().is_ident(crate::adaptor::ADAPTOR_DERIVE)
                && !x.path().is_ident(crate::adaptor::ADAPTOR_ATTRIBUTES)
                && !x.path().is_ident(crate::adaptor::ADAPTOR_ATTRIBUTE_FOR_ALL)
        })
        .cloned()
        .collect::<Vec<_>>()
}

fn modify_fields(fields: &[Field]) -> Vec<Field> {
    fields
        .iter()
        .cloned()
        .map(|mut x| {
            // Remove pkey from all fields.
            x.attrs = x
                .attrs
                .iter()
                .filter(|x| !x.path().is_ident(crate::item::ITEM_PKEY))
                .cloned()
                .collect::<Vec<_>>();
            // Re-add pkey to id field (in this case we specifically need id.)
            let is_id =
                x.ident.as_ref().map(|x| &x.to_string() == "id").unwrap_or(false);
            if is_id {
                x.attrs.push(parse_quote!(#[item_field_pkey]));
            }
            x
        })
        .collect::<Vec<_>>()
}
