use crate::shared::*;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::*;
use syn::{parse_macro_input, DeriveInput};

const ADAPTOR_RENAME: &str = "adaptor_rename";
const ADAPTOR_TYPE: &str = "adaptor_type";
const ADAPTOR_FROM: &str = "adaptor_from";
const ADAPTOR_TRY_FROM: &str = "adaptor_try_from";
const ADAPTOR_INTO: &str = "adaptor_into";
//const ADAPTOR_TRY_INTO: &str = "adaptor_try_into";
pub(crate) const ADAPTOR_DERIVE: &str = "adaptor_derive";
pub(crate) const ADAPTOR_ATTRIBUTES: &str = "adaptor_attributes";
pub(crate) const ADAPTOR_ATTRIBUTE_FOR_ALL: &str = "adaptor_attribute_for_all";
const ADAPTOR_FIELD_DUPLICATE: &str = "adaptor_field_duplicate";
const ADAPTOR_FIELDS_WITH_VALUES: &str = "adaptor_fields_with_values";

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConversionKind {
    F,
    R,
}

/// This function derives adaptor in its totality.
pub(crate) fn adaptor_inner(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);
    let vis = &inp.vis;

    // Get a new name for our structure.
    let old_name = &inp.ident;
    let default_name = Ident::new(&format!("{}Rep", old_name), Span::call_site());
    let new_name = rename(&inp.attrs, "adaptor_rename", &default_name);

    // Panic quickly if we are not dealing with a structure.
    // (For `sqlx` DB items enums are of dubious utility).
    let input_struct = get_struct(&inp, "DbAdaptor", old_name);
    let fields = get_named_fields(input_struct, "DbAdaptor");

    // This is the big panic that stops us before we go too far.
    if fields.is_empty() {
        panic!("`DbItem` does not deal with empty structures.");
    }
    let field_counts = (0..fields.len()).collect::<Vec<usize>>();

    let old_field_names =
        fields.iter().map(|x| x.ident.as_ref().unwrap()).collect::<Vec<_>>();

    let adaptor_fields = process_fields(&fields);
    // Some fields are duplicated.
    let (duplicate, dup_field_indices) = duplicate_fields(&fields);

    let new_field_names = get_ident(&adaptor_fields);
    let new_duplicate_fname = get_ident(&duplicate);

    let combined_field_names: Vec<_> = new_field_names
        .iter()
        .chain(new_duplicate_fname.iter())
        .cloned()
        .collect();

    let dup_field_counter = 0..new_duplicate_fname.len();

    // Here we create the forwards and backwards conversions for the fields.
    let (forwards, _) =
        field_conversion(&fields, &adaptor_fields, &[], ConversionKind::F);
    let (backwards, backwards_dup) = field_conversion(
        &fields,
        &adaptor_fields,
        &dup_field_indices,
        ConversionKind::R,
    );

    // This refers to the #[derive(X, Y, Z)]
    let derives = retain_attributes(&inp.attrs, ADAPTOR_DERIVE);
    // This refers to eg #[serde(X, Y, Z)]
    let outer_attributes = adaptor_attributes(&inp.attrs, ADAPTOR_ATTRIBUTES);
    // This refers to eg #[serde(X, Y, Z)], and applies to all fields.
    let extra_inherits = adaptor_attributes(&inp.attrs, ADAPTOR_ATTRIBUTE_FOR_ALL);

    // This refers to eg #[serde(X, Y, Z)] of individual fields.
    let inherited_field_attrs =
        fields.iter().map(|x| inherit_attributes(x, &extra_inherits));
    let inherited_duplicate_attrs =
        duplicate.iter().map(|x| inherit_attributes(x, &extra_inherits));

    let combined_field: Vec<_> =
        adaptor_fields.iter().chain(duplicate.iter()).cloned().collect();
    let combined_field_counts = (0..combined_field.len()).collect::<Vec<usize>>();

    let s: Path = parse_quote!(self);
    let field_converters_a = crate::item_ext::convert_fields(&combined_field, s);

    let mut stream = quote! {
        # (#derives)*
        # (#outer_attributes)*
        #[serde(default)]
        #vis struct #new_name {
            #(
                # (#inherited_field_attrs)*
                #adaptor_fields,
            )*
            #(
                # (#inherited_duplicate_attrs)*
                #duplicate,
            )*
        }

        impl asez2_shared_db::db_item::DbAdaptor for #new_name {
            type DbItem = #old_name;

            const DUP_FIELDS: &'static [&'static str] = &[#(stringify!(#new_duplicate_fname),)*];

            fn into_item(self) -> asez2_shared_db::result::Result<Self::DbItem> {
                let a = self;
                Ok(Self::DbItem {
                    #(
                        #old_field_names: #forwards,
                    )*
                })
            }

            /// An optional list of fields to send is supplied. This allows us to deactivate
            /// certain fields before conversion and send only the remaining fields.
            /// NB: We do not apply tolerance internally here.
            fn from_item_masked(
                item: Self::DbItem,
                mask: &asez2_shared_db::db_item::DbAdaptorFieldMask<Self>,
            ) -> Self {
                use asez2_shared_db::db_item::FieldTolerance;

                let field_mask = mask.item_field_mask();
                let dup_mask = mask.dup_field_mask();

                let a = item;
                let output = #new_name {
                    #(
                        #new_duplicate_fname: match dup_mask[#dup_field_counter] {
                            true => #backwards_dup,
                            false => None,
                        },
                    )*
                    #(
                        #new_field_names: match field_mask[#field_counts] {
                            true => #backwards,
                            false => None,
                        },
                    )*
                };
                output
            }

            fn bind_mask(&self) -> asez2_shared_db::db_item::DbFieldMask<Self::DbItem>
            {
                let mut mask = asez2_shared_db::db_item::DbFieldMask::<Self::DbItem>::none();
                // If the value is not the default value, we can update it.
                // With the current implementation, default means we got a None
                // from the Json deserialization.
                #(
                    mask[#field_counts] = self.#new_field_names.is_some();
                )*
                mask
            }

            fn create_strict_bind_mask(items: &[Self]) -> asez2_shared_db::result::Result<asez2_shared_db::db_item::DbFieldMask<Self::DbItem>> {
                let mut mask = asez2_shared_db::db_item::DbFieldMask::<Self::DbItem>::none();
                // If the value is not the default value, we can update it.
                // With the current implementation, default means we got a None
                // from the Json deserialization.
                #(
                    let mask_val = items.iter().fold(Ok(false), |acc, x| {
                        match (acc, x.#new_field_names.is_some()) {
                            (Ok(true), false) => Err(asez2_shared_db::result::SharedDbError::Other(
                                format!("`{}` is not present in all items.", stringify!(#new_field_names))
                            )),
                            (Ok(false), x) => Ok(x),
                            (x, _) => x,
                        }
                    });
                    mask[#field_counts] = mask_val?;
                )*
                Ok(mask)
            }

            fn into_item_merged(self, mut item: Self::DbItem) -> asez2_shared_db::result::Result<Self::DbItem> {
                let a = self;
                #(
                    if a.#new_field_names.is_some() {
                        item.#old_field_names = #forwards;
                    }
                )*
                Ok(item)
            }

            fn zero_fields(mut self, fields: &asez2_shared_db::db_item::DbFieldMask<Self::DbItem>) -> Self {
                #(
                    if fields[#field_counts] {
                        self.#new_field_names = Some(Default::default());
                    }
                )*
                self

            }

            fn unset_fields(mut self, fields: &asez2_shared_db::db_item::DbFieldMask<Self::DbItem>) -> Self {
                #(
                    if fields[#field_counts] {
                        self.#new_field_names = None;
                    }
                )*
                self

            }
        }
    };
    if has_attr(&inp.attrs, ADAPTOR_FIELDS_WITH_VALUES) {
        stream.extend(quote! {
            impl asez2_shared_db::db_item::DbAdaptorFieldsWithValues for #new_name {
                const FIELDS: &'static [&'static str] = &[#(stringify!(#combined_field_names),)*];
                    fn fields_with_values(&self) -> Vec<asez2_shared_db::db_item::Field> {
                    vec![#(
                        asez2_shared_db::db_item::Field::new(
                            Self::FIELDS[#combined_field_counts],
                            #field_converters_a,
                        ),
                    )*]
                }
        }
        });
    }
    stream.into()
}

/// This function creates a list of function calls `function(variable)` which then
/// become part of a conversion for a field, ie `field_name: function(variable),`
/// where the `variable` is usually the same field of the original structure.
pub(super) fn field_conversion(
    input_fields: &[Field],
    adaptor_fields: &[Field],
    duplicate_indices: &[usize],
    dir: ConversionKind,
) -> (Vec<syn::Expr>, Vec<syn::Expr>) {
    if input_fields.len() != adaptor_fields.len() {
        panic!(
            "Error occurred when making fields: Fields are not of the same length"
        );
    }
    let convert = input_fields
        .iter()
        .zip(adaptor_fields.iter())
        .map(|(f, r)| make_conversion(f, r, dir, false))
        .collect::<Vec<_>>();

    let convert_duplicate = duplicate_indices
        .iter()
        .map(|i| make_conversion(&input_fields[*i], &adaptor_fields[*i], dir, true))
        .collect::<Vec<_>>();
    (convert, convert_duplicate)
}

// This does not recursively convert fields. Thus all fields must be internally convertible, else the
// conversion must be handled specifically by the stated function.
pub(super) fn make_conversion(
    field_a: &Field,
    field_b: &Field,
    dir: ConversionKind,
    clone: bool,
) -> syn::Expr {
    let (convertor, kind, f): (Path, &_, &_) = match dir {
        ConversionKind::F =>
        //(ADAPTOR_FROM, ADAPTOR_TRY_FROM, field_b),
        {
            find_name_value_attr(&field_a.attrs, ADAPTOR_FROM)
                .map(|x| (x, ADAPTOR_FROM))
                .or_else(|| {
                    find_name_value_attr(&field_a.attrs, ADAPTOR_TRY_FROM)
                        .map(|x| (x, ADAPTOR_TRY_FROM))
                })
                .and_then(|(x, kind)| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(ref x),
                        ..
                    }) = x.value
                    {
                        let x: Path =
                            x.parse().expect("Could not parse converted function.");
                        Some((parse_quote! { #x }, kind, field_b))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    (parse_quote!(Into::into), ADAPTOR_FROM, field_b)
                })
        }
        ConversionKind::R =>
        //(ADAPTOR_INTO, field_a),
        {
            find_name_value_attr(&field_a.attrs, ADAPTOR_INTO)
                .and_then(|x| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(ref x),
                        ..
                    }) = x.value
                    {
                        let x: Path =
                            x.parse().expect("Could not parse converted function.");
                        Some((parse_quote! { #x }, ADAPTOR_INTO, field_a))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    (parse_quote!(Into::into), ADAPTOR_INTO, field_a)
                })
        }
    };
    let fname = &f.ident;
    // We must clone duplicated fields since otherwise some of them, eg string fields
    // will be consumed during the first conversion.
    let field = match clone {
        false => quote! { a.#fname },
        true => quote! { a.#fname.clone() },
    };

    match kind {
        // NB: We use "a" as the variable name.
        ADAPTOR_FROM => parse_quote!{ #field.map(|x| #convertor(x)).unwrap_or_default() },
        ADAPTOR_INTO => parse_quote!{ Some(#convertor(#field)) },
        ADAPTOR_TRY_FROM => parse_quote!{ #field.map(|x| #convertor(x)).transpose()?.unwrap_or_default() },
        x => panic!(
            "Incorrect argument \"{}\" in derive in `make_conversion`, please contact the developer.",
            x,
        ),
    }
}

pub(super) fn process_fields(input_fields: &[Field]) -> Vec<Field> {
    input_fields
        .iter()
        .cloned()
        // Here the field is renamed and
        .map(|x| process_field(x, ADAPTOR_RENAME))
        .collect::<Vec<_>>()
}

fn duplicate_fields(input_fields: &[Field]) -> (Vec<Field>, Vec<usize>) {
    let pre_fields = input_fields
        .iter()
        .enumerate()
        .filter(|(_, x)| has_attr(&x.attrs, ADAPTOR_FIELD_DUPLICATE));

    let indices = pre_fields.clone().map(|x| x.0).collect::<Vec<usize>>();

    let new = pre_fields
        .map(|(_, x)| process_field(x.to_owned(), ADAPTOR_FIELD_DUPLICATE))
        .collect::<Vec<_>>();

    (new, indices)
}

/// This function exists to process fields. It does so in several stages.
///
/// 1. Rename the field.
/// 2. Retype the field.
/// 3. Strip all attributes but `adaptor_derive` attributes.
fn process_field(mut x: Field, rename_kind: &str) -> Field {
    let old_ident = x.ident.expect("Adaptor is only derived for named fields.");
    x.ident = Some(rename(&x.attrs, rename_kind, &old_ident));
    x.ty = retype(&x.attrs, ADAPTOR_TYPE, x.ty.clone());
    // Stripping attributes is done last.
    x.attrs = retain_attributes(&x.attrs, ADAPTOR_DERIVE);
    x
}

/// This function takes an attribute wrapped in:
/// `#[adaptor_attribute(my_attr(..))]`
/// and unwraps it into:
/// `kind` should be "adaptor_attributes" or "adaptor_attribute_for_all"
/// `#[my_attr(..)]`
pub(super) fn adaptor_attributes(
    inp_attrs: &[Attribute],
    kind: &str,
) -> Vec<Attribute> {
    inp_attrs
        .iter()
        .flat_map(|x| {
            if !x.path().is_ident(kind) {
                return vec![];
            };
            x.parse_args_with(Attribute::parse_outer)
                .expect("Expected inner attributes in `adaptor_attributes`")
        })
        .collect::<Vec<_>>()
}

fn inherit_attributes(f: &Field, extra_inherits: &[Attribute]) -> Vec<Attribute> {
    let mut attrs = adaptor_attributes(&f.attrs, ADAPTOR_ATTRIBUTES);
    // This prevents appending attributes with the same path to a field
    // (which would cause an error).
    // It also ensures precedence of field attributes over struct attributes.
    for x in extra_inherits.iter() {
        if !attrs.iter().any(|own_attr| own_attr.path() == x.path()) {
            attrs.push(x.to_owned());
        }
    }
    attrs.push(
        parse_quote!(#[serde(with = "asez2_shared_db::db_item::DbAdaptorOption")]),
    );
    attrs.push(parse_quote!(#[serde(skip_serializing_if = "Option::is_none")]));
    attrs
}

fn get_ident(fields: &[Field]) -> Vec<&Ident> {
    fields.iter().map(|x| x.ident.as_ref().unwrap()).collect::<Vec<_>>()
}
