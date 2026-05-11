use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Meta, Variant};

use crate::shared::{find_variants_with, retain_attributes};

pub(crate) fn db_enumeration_inner(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);

    let enumeration = match inp.data {
        Data::Enum(e) => e,
        _ => panic!("DbEnum может быть использован только для енамов"),
    };

    let enum_ident = inp.ident;
    let repr =
        retrieve_repr(&inp.attrs).expect("Требуется задать #[repr(...)] для енама");
    let variants = enumeration.variants;

    let default_variant_ident = find_variants_with(&variants, "db_default").next().expect(
        "Требуется пометить атрибутом #[db_default] вариант енама по дефолту в базе данных",
    ).ident.clone();
    let match_other_variants = variants
        .iter()
        .filter(|variant| variant.ident != default_variant_ident)
        .map(|variant| {
            let ident = variant.ident.clone();
            let (_, val) = variant.discriminant.clone().unwrap_or_else(|| {
                panic!("Для {} требуется указать дискриминант", ident)
            });
            quote! { #val => Self::#ident }
        });

    let mut variant_info = Vec::new();
    for Variant {
        ident,
        discriminant,
        ..
    } in &variants
    {
        if let Some((_, Expr::Lit(expr_lit))) = discriminant {
            if let syn::Lit::Int(lit_int) = &expr_lit.lit {
                let disc_value = lit_int
                    .base10_parse::<i16>()
                    .expect("Expected integer discriminant");
                // Store variant and its discriminant value
                variant_info.push(quote! {
                    (#enum_ident::#ident, #disc_value)
                });
            } else {
                panic!("Unsupported discriminant type");
            }
        }
    }

    let tokens = quote! {
        impl Default for #enum_ident {
            fn default() -> Self {
                Self::#default_variant_ident
            }
        }

        impl From<#enum_ident> for asez2_shared_db::Value {
            fn from(variant: #enum_ident) -> Self {
                asez2_shared_db::Value::Int(variant as i64)
            }
        }

        impl From<#enum_ident> for #repr {
            fn from(val: #enum_ident) -> Self {
                val as #repr
            }
        }

        impl From<#repr> for #enum_ident {
            fn from(num: #repr) -> Self {
                match num {
                    #(#match_other_variants,)*
                    _ => Self::#default_variant_ident,
                }
            }
        }
       impl asez2_shared_db::db_item::EnumDiscriminant for #enum_ident {
            const DISCRIMINANTS: &'static [(Self, #repr)] = &[ #(#variant_info),* ];
        }
        // It's an i16!
        impl sqlx::postgres::PgHasArrayType for #enum_ident {
            fn array_type_info() -> sqlx::postgres::PgTypeInfo {
                i16::array_type_info()
            }
        }
    };

    tokens.into()
}

/// Получение репрезентации енама как числа
fn retrieve_repr(attrs: &[Attribute]) -> Option<Ident> {
    retain_attributes(attrs, "repr").into_iter().next().map(|attr| {
        if let Meta::List(meta_list) = &attr.meta {
            meta_list
                .tokens
                .clone()
                .into_iter()
                .next()
                .map(|token| Ident::new(&token.to_string(), Span::call_site()))
                .unwrap()
        } else {
            panic!("Ожидался тип в #[repr(...)]")
        }
    })
}
