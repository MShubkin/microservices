use crate::shared::*;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote};
use syn::{DeriveInput, Field, Path, PathArguments, Type, TypePath};

const ITEM_PKEY: &str = "item_field_pkey";

pub(crate) fn inner(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);

    // Get a new name for our structure.
    let old_name = &inp.ident;

    // Panic quickly if we are not dealing with a structure.
    // (For `sqlx` DB items enums are of dubious utility).
    let input_struct = get_struct(&inp, "DbItemExt", old_name);
    let fields = get_named_fields(input_struct, "DbItemExt");

    let field_counts = (0..fields.len()).collect::<Vec<_>>();
    let total_field_count = fields.len();

    // Get the primary keys.
    let pkeys = find_fields_with(&fields, ITEM_PKEY).cloned().collect::<Vec<_>>();

    let field_names = fields.iter().map(|x| &x.ident).collect::<Vec<_>>();
    let pkey_count = (0..pkeys.len()).collect::<Vec<usize>>();

    find_fields_with(&fields, ITEM_PKEY)
        .find(|x| {
            x.ident.as_ref().map(|x| x.to_string()) == Some("uuid".to_string())
        })
        .expect(
            "`uuid` field pkey of type Uuid must be present to derive DbItemExt.",
        );

    let s: Path = parse_quote!(self);
    let field_converters_a = convert_fields(&fields, s.clone());
    let field_converters_b = convert_fields(&fields, parse_quote!(b));
    let pkey_converters = convert_fields(&pkeys, s);

    quote! {
        impl asez2_shared_db::db_item::DbItemExt for #old_name {
            /// The uuid of the record (DbItem).
            /// It should be noted that for now this functionality will only
            /// work properly for items that use uuid as the primary key as the historian
            /// table uses `record_uuid` as the definitive primary key of the item
            fn record_uuid(&self) -> uuid::Uuid {
                self.uuid
            }
            // Get pkeys and their values.
            fn pkeys_with_values(&self) -> Vec<asez2_shared_db::db_item::Field> {
                vec![#(
                    asez2_shared_db::db_item::Field::new(Self::PRIMARY_KEYS[#pkey_count], #pkey_converters),
                )*]
            }

            fn fields_with_values(&self) -> Vec<asez2_shared_db::db_item::Field> {
                vec![#(
                    asez2_shared_db::db_item::Field::new(
                        Self::FIELDS[#field_counts],
                        #field_converters_a,
                    ),
                )*]
            }

            /// Should return all fields that are not equal.
            /// The function should be symmetrical.
            fn differing_fields(&self, b: &Self) -> Vec<asez2_shared_db::db_item::Field>  {
                let mut output = Vec::with_capacity(#total_field_count);

                #( if self.#field_names != b.#field_names {
                    output.push(asez2_shared_db::db_item::Field::new(
                        Self::FIELDS[#field_counts],
                        #field_converters_b,
                    ));
                } )*

                output.shrink_to_fit();
                output
            }
        }
    }
    .into()
}

/// This functionality allows us to deal with optional fields.
/// The `Fields` structure uses `Option<Value>` to represent the fact
/// that some arbitrary field may be NULL, however some fields are NOT NULL.
/// fields that can be nulled are usually represented by `Option<T>`, while
/// those that cannot are represented by `T`.
///
/// Thus this function exists to correctly convert both fields to `Option<Value>`
pub fn convert_fields(input: &[Field], outer: Path) -> Vec<syn::Expr> {
    input
        .iter()
        .map(|f| {
            let name = &f.ident;
            if identify_option(&f.ty) {
                parse_quote!(#outer.#name.as_ref().map(|x| x.to_owned()))
            } else {
                parse_quote!(Some(#outer.#name.to_owned()))
            }
        })
        .collect::<Vec<_>>()
}

/// This could be better.
fn identify_option(x: &Type) -> bool {
    let path = match x {
        Type::Path(TypePath { path, .. }) => &path.segments,
        _ => return false,
    };
    if path.is_empty() {
        return false;
    }
    let s = &path[path.len() - 1];

    matches!(s.arguments, PathArguments::AngleBracketed { .. })
        && &s.ident.to_string() == "Option"
}
