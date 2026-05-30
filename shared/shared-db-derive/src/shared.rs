use proc_macro2::{Ident, Span, TokenStream};
use syn::*;
use syn::{Attribute, Meta};

/// Ищет атрибут с именем `attr_name` в формате `#[adaptor_derive(X, Y)]` и
/// переименовывает его путь в `derive`, чтобы получился `#[derive(X, Y)]`.
///
/// Если атрибут не найден -- возвращает пустой список. Нужно для того, чтобы
/// адаптор мог унаследовать дополнительные derive-трейты от исходной структуры.
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

/// Возвращает тип поля для адаптора.
///
/// Если поле помечено `#[adaptor_type = "NewType"]`, возвращает
/// `Option<NewType>`. Иначе возвращает `Option<OriginalType>`.
/// `Option` нужен потому что все поля адаптора опциональны -- `None` означает
/// "поле не затронуто при обновлении".
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

/// Извлекает строковой литерал из атрибута вида `#[attr_name = "value"]`
/// и возвращает его как [`Ident`].
///
/// Используется для получения имён функций и типов из атрибутов proc-macro.
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

/// Извлекает строковой литерал из атрибута вида `#[attr_name = "expr"]`
/// и парсит его как выражение Rust.
///
/// Используется для `item_field_activate_with` и аналогичных атрибутов,
/// где значение -- произвольное выражение типа `chrono::Utc::now()`.
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

/// Возвращает имя сущности (struct/field) с учётом атрибута переименования.
///
/// Если атрибут `rename_kind` задан -- используется его значение как `Ident`.
/// Иначе возвращается `default_name`.
pub(super) fn rename(
    inp_attrs: &[Attribute],
    rename_kind: &str,
    default_name: &Ident,
) -> Ident {
    get_attr_ident(inp_attrs, rename_kind).unwrap_or(default_name.to_owned())
}

/// Находит атрибут вида `#[name = value]` и возвращает пару `name = value`.
pub(super) fn find_name_value_attr(
    inp_attrs: &[Attribute],
    name: &str,
) -> Option<MetaNameValue> {
    inp_attrs.iter().find_map(|x| match &x.meta {
        Meta::NameValue(v) if x.path().is_ident(name) => Some(v.to_owned()),
        _ => None,
    })
}

/// Проверяет наличие атрибута-маркера (`#[name]`) среди атрибутов.
pub(super) fn has_attr(inp_attrs: &[Attribute], name: &str) -> bool {
    inp_attrs.iter().any(|x| x.path().is_ident(name))
}

/// Извлекает типы всех полей структуры.
pub(super) fn extract_field_types(fields: &[Field]) -> Vec<Type> {
    fields.iter().map(|x| x.ty.to_owned()).collect::<Vec<_>>()
}

/// Фильтрует поля: возвращает только те, у которых нет ни одного из
/// перечисленных атрибутов. Используется для исключения autogen-полей
/// из списка INSERT_FIELDS.
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

/// Возвращает поля с указанным атрибутом (без индекса).
pub(super) fn find_fields_with<'a>(
    fields: &'a [Field],
    attr: &'a str,
) -> impl Iterator<Item = &'a Field> {
    find_idx_fields_with(fields, attr).map(|(_, x)| x)
}

/// Возвращает пары `(индекс, поле)` для полей с указанным атрибутом.
///
/// Индекс нужен для генерации `PRIMARY_KEY_INDICES` и `NO_UPDATE_INDICES`.
pub(super) fn find_idx_fields_with<'a>(
    fields: &'a [Field],
    attr: &'a str,
) -> impl Iterator<Item = (usize, &'a Field)> {
    fields
        .iter()
        .enumerate()
        .filter(|(_, x)| x.attrs.iter().any(|x| x.path().is_ident(attr)))
}

/// Получает данные структуры из `DeriveInput` с паникой при попытке применить
/// макрос к enum или union.
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

/// Получает именованные поля структуры.
///
/// Паникует для структур с безымянными полями или unit-структур, так как
/// они не могут отображаться на именованные колонки БД.
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
    }
    .into_iter()
    .collect::<Vec<_>>()
}

/// Возвращает варианты enum с указанным атрибутом.
pub(super) fn find_variants_with<'a>(
    variants: impl IntoIterator<Item = &'a Variant>,
    attr: &'a str,
) -> impl Iterator<Item = &'a Variant> {
    variants
        .into_iter()
        .filter(|variant| variant.attrs.iter().any(|x| x.path().is_ident(attr)))
}
