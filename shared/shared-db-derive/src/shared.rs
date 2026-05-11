use proc_macro2::{Ident, Span, TokenStream};
use syn::*;
use syn::{Attribute, Meta};

/// This function dumps all attributes with the exception of `#[adaptor_derive(Stuff)]`,
/// which it renames to `#[derive(Stuff)]`
pub(super) fn retain_attributes(
    inp_attrs: &[Attribute],
    attr_name: &str,
) -> Vec<Attribute> {
    inp_attrs
        .iter()
        .cloned()
        .find_map(|mut x| {
            let mut m = match &x.meta {
                Meta::List(v) if x.path().is_ident(attr_name) => v.to_owned(),
                _ => return None,
            };
            m.path = parse_quote_spanned!(Span::call_site() => derive);
            x.meta = Meta::List(m);
            Some(vec![x])
        })
        .unwrap_or_default()
}

/// Finds the attribute with the `default_kind` name and offers it as the new
/// `Type` for the field. If not, it will use the default type (the original)
/// type of the attribute usually.
pub(super) fn retype(
    inp_attrs: &[Attribute],
    rename_kind: &str,
    default_type: Type,
) -> Type {
    find_name_value_attr(inp_attrs, rename_kind)
        .and_then(|x| {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(ref x),
                ..
            }) = x.value
            {
                let x: Path =
                    x.parse().expect("Could not parse converted function.");
                Some(parse_quote_spanned!(Span::call_site() => std::option::Option<#x>))
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            parse_quote_spanned!(Span::call_site() => std::option::Option<#default_type>)
        })
}

/// Essentially retrieves the string literal from a `key = "value"` kind
/// of attribute and extracts it as an identity.
pub(super) fn get_attr_ident(
    inp_attrs: &[Attribute],
    attr_ident: &str,
) -> Option<Ident> {
    find_name_value_attr(inp_attrs, attr_ident).and_then(|x| {
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(ref x),
            ..
        }) = x.value
        {
            Some(Ident::new(&x.value(), Span::call_site()))
        } else {
            None
        }
    })
}

/// Essentially retrieves the string literal from a `key = "value"` kind
/// of attribute and extracts it as an expression.
pub(super) fn get_attr_expr(
    inp_attrs: &[Attribute],
    attr_ident: &str,
) -> Option<TokenStream> {
    find_name_value_attr(inp_attrs, attr_ident).and_then(|x| {
        let Expr::Lit(ExprLit {
            lit: Lit::Str(ref x),
            ..
        }) = x.value
        else {
            return None;
        };
        Some(
            x.parse::<syn::Expr>()
                .map_or_else(|e| e.to_compile_error(), |e| quote::quote!(#e)),
        )
    })
}

/// Renames an entity based on the attributes.
pub(super) fn rename(
    inp_attrs: &[Attribute],
    rename_kind: &str,
    default_name: &Ident,
) -> Ident {
    get_attr_ident(inp_attrs, rename_kind).unwrap_or(default_name.to_owned())
}

pub(super) fn find_name_value_attr(
    inp_attrs: &[Attribute],
    name: &str,
) -> Option<MetaNameValue> {
    inp_attrs.iter().find_map(|x| match &x.meta {
        Meta::NameValue(v) if x.path().is_ident(name) => Some(v.to_owned()),
        _ => None,
    })
}

pub(super) fn has_attr(inp_attrs: &[Attribute], name: &str) -> bool {
    inp_attrs.iter().any(|x| x.path().is_ident(name))
}

pub(super) fn extract_field_types(fields: &[Field]) -> Vec<Type> {
    fields.iter().map(|x| x.ty.to_owned()).collect::<Vec<_>>()
}

/// This is a simple convenience function.
pub(super) fn find_fields_without<'a>(
    fields: &'a [Field],
    exclude_attrs: &'a [&'a str],
) -> impl Iterator<Item = &'a Field> {
    fields.iter().filter(|x| {
        !x.attrs
            .iter()
            .any(|x| exclude_attrs.iter().any(|ex_attr| x.path().is_ident(ex_attr)))
    })
}

/// This is a simple convenience function.
pub(super) fn find_fields_with<'a>(
    fields: &'a [Field],
    attr: &'a str,
) -> impl Iterator<Item = &'a Field> {
    find_idx_fields_with(fields, attr).map(|(_, x)| x)
}

/// This is a simple convenience function.
pub(super) fn find_idx_fields_with<'a>(
    fields: &'a [Field],
    attr: &'a str,
) -> impl Iterator<Item = (usize, &'a Field)> {
    fields
        .iter()
        .enumerate()
        .filter(|(_, x)| x.attrs.iter().any(|x| x.path().is_ident(attr)))
}

pub(super) fn get_struct<'a>(
    inp: &'a DeriveInput,
    module: &str,
    name: &Ident,
) -> &'a syn::DataStruct {
    match inp.data {
        Data::Struct(ref s) => s,
        Data::Union(_) => panic!(
            "{module} can only be derived for structures \"{name}\" is a union.",
            name = name,
            module = module,
        ),
        Data::Enum(_) => panic!(
            "{module} can only be derived for structures \"{name}\" is an enum.",
            name = name,
            module = module,
        ),
    }
}

pub(super) fn get_named_fields(
    inp: &syn::DataStruct,
    module: &str,
) -> Vec<syn::Field> {
    match &inp.fields {
        Fields::Named(FieldsNamed { named: x, .. }) => x.to_owned(),
        _ => panic!(
            "`{module}` does not deal with unnamed fields or unit structs.",
            module = module
        ),
        // NB: We can also process other kinds of fields, but this would only lead
        // to problems since we are inserting *named* fields into *named* database
        // table columns.
        // Fields::Unnamed(FieldsUnnamed { unnamed: x, .. }) => x.to_owned(),
        // Fields::Unit => Punctuated::new(),
    }
    .into_iter()
    .collect::<Vec<_>>()
}

/// Получение всех вариантов енама с атрибутом
pub(super) fn find_variants_with<'a>(
    variants: impl IntoIterator<Item = &'a Variant>,
    attr: &'a str,
) -> impl Iterator<Item = &'a Variant> {
    variants
        .into_iter()
        .filter(|variant| variant.attrs.iter().any(|x| x.path().is_ident(attr)))
}
