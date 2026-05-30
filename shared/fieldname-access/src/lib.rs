use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Expr, ExprLit,
    Fields, FieldsNamed, Generics, Lit, Meta, Type, TypeGenerics, Visibility,
    WhereClause,
};

/// Derive-макрос для доступа к полям структуры по строковому имени.
///
/// Зачем нужен: в некоторых частях системы поля структуры нужно обходить динамически
/// (например, для сериализации в кастомный формат или для построения SQL-запросов
/// из набора изменённых полей). Rust не предоставляет reflection, поэтому макрос
/// генерирует его вручную через enum и match.
///
/// Генерирует:
/// - enum `{Struct}Field<'field>` — варианты с `&'field T` для каждого типа поля;
/// - enum `{Struct}FieldMut<'field>` — то же, но с `&'field mut T`;
/// - `const FIELDS: [&'static str; N]` — имена полей в порядке объявления;
/// - `fn field(&self, name: &str) -> Option<{Struct}Field<'_>>` — доступ по имени;
/// - `fn field_mut(&mut self, name: &str) -> Option<{Struct}FieldMut<'_>>`;
/// - `fn field_iter(&self) -> impl Iterator<Item = (&'static str, ...)>` — обход полей.
///
/// Несколько полей одного типа дадут один вариант enum-а (`String(&'field String)`) —
/// это намеренно: enum не разрастается, а disambiguация идёт по имени в match arms.
///
/// Также он генерирует `const FIELDS: [&'static str; FIELDS_COUNT]` с полями структуры и
/// `field_iter` метод, который позволяет итерироваться по полям структуры в том порядке,
/// в котором они были декларированы
///
/// # Атрибуты контейнера
///
/// * `#fieldname_enum(name = "NewName")` - Наименование енама с возможными значениями структуры
/// ```rust
/// use fieldname_access::FieldnameAccess;
///
/// #[derive(FieldnameAccess, Default)]
/// #[fieldname_enum(name = "NewName")]
/// struct NamedFieldname {
///    name: String,
///    age: i64,
/// }
/// let mut instance = NamedFieldname::default();
/// match instance.field("name").unwrap() {
///     NewName::String(val) => {}
///     NewName::I64(val) => {},
///     NewName::None => {},
/// }
/// match instance.field_mut("name").unwrap() {
///     NewNameMut::String(val) => {}
///     NewNameMut::I64(val) => {},
///     NewNameMut::None => {},
/// }
/// ```
/// * `#fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])` - Дерайв макросы для енамов.
/// `derive` для енама с ссылками, `derive_mut` для енама с мут ссылками. Нужно, если например
/// `&mut` клонировать
/// ```rust
/// use fieldname_access::FieldnameAccess;
///
/// #[derive(FieldnameAccess)]
/// #[fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])]
/// struct NamedFieldname {
///    name: String,
///    age: i64,
/// }
/// ```
/// * `#fieldname_enum(derive_all = [Debug])` - Дерайв макросы как для енама
/// с обычными ссылками, так и для енама с мут ссылками
/// ```rust
/// use fieldname_access::FieldnameAccess;
///
/// #[derive(FieldnameAccess)]
/// #[fieldname_enum(derive_all = [Debug])]
/// struct NamedFieldname {
///    name: String,
///    age: i64,
/// }
/// ```
///
/// # Атрибуты полей
/// * `#fieldname = "FieldnameVariant"` - Наименование варианта енама. Может быть полезно,
/// когда надо заматчить определенное поле
/// ```rust
/// use fieldname_access::FieldnameAccess;
///
/// #[derive(FieldnameAccess, Default)]
/// struct NamedFieldname {
///    name: String,
///    #[fieldname = "MyAge"]
///    age: i64,
///    age_of_dog: i64
/// }
/// let mut instance = NamedFieldname::default();
/// match instance.field("name").unwrap() {
///     NamedFieldnameField::String(val) => {}
///     NamedFieldnameField::MyAge(val) => {}
///     NamedFieldnameField::I64(val) => {}
///     NamedFieldnameField::None => {}
/// }
/// match instance.field_mut("name").unwrap() {
///     NamedFieldnameFieldMut::String(val) => {}
///     NamedFieldnameFieldMut::MyAge(val) => {}
///     NamedFieldnameFieldMut::I64(val) => {}
///     NamedFieldnameFieldMut::None => {}
/// }
/// ```
#[proc_macro_derive(FieldnameAccess, attributes(fieldname_enum, fieldname))]
pub fn fieldname_accessor(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);

    let structure = match inp.data {
        Data::Struct(ref s) => s,
        Data::Union(_) => {
            panic!("FieldnameAccess может быть использован только для структур")
        }
        Data::Enum(_) => {
            panic!("FieldnameAccess может быть использован только для структур")
        }
    };

    let struct_ident = inp.ident;
    let visibility = inp.vis;
    let field_lifetime: syn::GenericParam = parse_quote!('field);
    let generics = inp.generics;
    let (impl_generics, ty_generics, where_clauses) = generics.split_for_impl();
    let mut enum_generics = generics.clone();
    enum_generics.params.push(field_lifetime.clone());

    let fields = match &structure.fields {
        Fields::Named(FieldsNamed { named: x, .. }) => x.to_owned(),
        Fields::Unnamed(_) | Fields::Unit => {
            panic!("Безымянные поля не поддерживаются")
        }
    };

    let field_map = fields
        .into_iter()
        .map(|field| {
            let field_type = field.ty;
            let field_name =
                field.ident.expect("Безымянные поля не поддерживаются");
            let variant_ident = if let Some(name) = retrieve_fieldname(&field.attrs)
            {
                name
            } else {
                let type_str = generate_variant_name(&field_type);
                Ident::new(&type_str, Span::call_site())
            };

            (field_name, field_type, variant_ident)
        })
        .collect::<Vec<_>>();
    let field_list = field_map.iter().map(|(name, _, _)| name.to_string());
    let field_count = field_map.len();

    let value_enum_ident = retrieve_enum_name(&inp.attrs).unwrap_or_else(|| {
        Ident::new(&format!("{}Field", struct_ident), Span::call_site())
    });
    let (derive, derive_mut) =
        if let Some(derives) = retrieve_derives(&inp.attrs, "derive_all") {
            (Some(derives.clone()), Some(derives))
        } else {
            let derive = retrieve_derives(&inp.attrs, "derive");
            let derive_mut = retrieve_derives(&inp.attrs, "derive_mut");
            (derive, derive_mut)
        };

    // Енам с ссылками на возможный тип поля структуры
    let value_variants = generate_enum_variants(&field_map, false);

    // Енам с мут ссылками на возможный тип поля структуры
    let value_enum_ident_mut =
        Ident::new(&format!("{}Mut", value_enum_ident), Span::call_site());
    let value_variants_mut = generate_enum_variants(&field_map, true);

    let match_arms = generate_match_arms(&field_map, &value_enum_ident, false);
    let match_arms_mut =
        generate_match_arms(&field_map, &value_enum_ident_mut, true);

    let iter_impl = generate_iter_impl(
        &visibility,
        &value_enum_ident,
        &struct_ident,
        &ty_generics,
        &where_clauses,
        &enum_generics,
        &field_lifetime,
    );

    let tokens = quote! {
        /// Енам, представляющий ссылки на поля структуры
        #derive
        #visibility enum #value_enum_ident #enum_generics #where_clauses {
            #(#value_variants,)*
            None,
        }

        /// Енам, представляющий мут ссылки на поля структуры
        #derive_mut
        #visibility enum #value_enum_ident_mut #enum_generics #where_clauses {
            #(#value_variants_mut,)*
            None,
        }

        #iter_impl

        impl #impl_generics #struct_ident #ty_generics #where_clauses {
            /// List with all struct fields
            const FIELDS: [&'static str; #field_count] = [#(#field_list),*];

            /// Метод для получения ссылки на поле структуры по его имени
            #visibility fn field<#field_lifetime>(&#field_lifetime self, fieldname: &str) -> Option<#value_enum_ident #enum_generics> {
                match fieldname {
                    #(#match_arms,)*
                    _ => None
                }
            }

            /// Метод для получения мут ссылки на поле структуры по его имени
            #visibility fn field_mut<#field_lifetime>(&#field_lifetime mut self, fieldname: &str) -> Option<#value_enum_ident_mut #enum_generics> {
                match fieldname {
                    #(#match_arms_mut,)*
                    _ => None
                }
            }
        }
    };

    tokens.into()
}

fn generate_iter_impl(
    vis: &Visibility,
    value_enum_ident: &Ident,
    struct_ident: &Ident,
    struct_generics: &TypeGenerics,
    where_clauses: &Option<&WhereClause>,
    enum_generics: &Generics,
    enum_lt: &syn::GenericParam,
) -> proc_macro2::TokenStream {
    let iter_ident =
        Ident::new(&format!("{}FieldIter", value_enum_ident), Span::call_site());

    let struct_generic_turbofish = struct_generics.as_turbofish();
    let struct_ident_turbofish = quote! { #struct_ident #struct_generic_turbofish };

    quote! {
        #vis struct #iter_ident #enum_generics #where_clauses {
            idx: usize,
            inner: &#enum_lt #struct_ident #struct_generics
        }

        impl #struct_generics #struct_ident #struct_generics #where_clauses {
            pub fn field_iter<#enum_lt>(&#enum_lt self) -> #iter_ident #enum_generics {
                #iter_ident {
                    idx: 0,
                    inner: self
                }
            }
        }

        impl #enum_generics Iterator for #iter_ident #enum_generics #where_clauses {
            type Item = (&'static str, #value_enum_ident #enum_generics);

            fn next(&mut self) -> Option<Self::Item> {
                (self.idx != #struct_ident_turbofish::FIELDS.len()).then(|| {
                    let field_name = #struct_ident_turbofish::FIELDS[self.idx];
                    self.idx += 1;
                    (field_name, self.inner.field(field_name).unwrap())
                })
            }
        }
    }
}

/// Строит имя варианта enum-а из типа поля.
/// `String` → `"String"`, `Option<Uuid>` → `"OptionUuid"`, `i64` → `"I64"`.
/// Примитивы поднимаются в CamelCase, сложные типы очищаются от угловых скобок.
fn generate_variant_name(ty: &syn::Type) -> String {
    let type_str = ty.to_token_stream().to_string();
    shorten_type(type_str)
}

fn shorten_type(type_str: String) -> String {
    let mut short_type =
        type_str.chars().skip_while(|c| !c.is_uppercase()).peekable();

    if short_type.peek().is_some() {
        // Комплексный тип, который содержит дженерики, лайфтаймы и т.д.
        let mut complex_type_str = String::new();

        while let Some(c) = short_type.next() {
            if c.is_ascii_alphanumeric() {
                complex_type_str.push(c);
            }

            if c == '<' {
                complex_type_str += &shorten_type(short_type.collect());
                break;
            }
        }

        complex_type_str
    } else {
        // Значит тип скорее всего является примитивным и его надо
        // приподнять и также очистить
        let cleaned_str = type_str
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();

        cleaned_str[0..1].to_uppercase() + &cleaned_str[1..]
    }
}

/// Генерирует варианты enum-а — по одному на уникальный тип поля.
/// `unique_by(variant_ident)` — поля одного типа дают один вариант,
/// иначе enum разрастался бы при десятках строковых полей.
fn generate_enum_variants(
    field_map: &[(Ident, Type, Ident)],
    is_mut: bool,
) -> Vec<proc_macro2::TokenStream> {
    field_map
        .iter()
        .unique_by(|(_, _, variant_ident)| variant_ident)
        .map(|(_, field_type, variant_ident)| {
            if is_mut {
                quote! {
                    #variant_ident(&'field mut #field_type)
                }
            } else {
                quote! {
                    #variant_ident(&'field #field_type)
                }
            }
        })
        .collect()
}

/// Генерирует ветки `match fieldname { "field_name" => Some(Enum::Variant(&self.field)) }`.
/// Здесь нет `unique_by` — каждое поле получает свою ветку, даже если тип совпадает.
fn generate_match_arms(
    field_map: &[(Ident, syn::Type, Ident)],
    value_enum_ident: &Ident,
    is_mut: bool,
) -> Vec<proc_macro2::TokenStream> {
    field_map
        .iter()
        .map(|(field_name, _, variant_ident)| {
            let field_name_str = field_name.to_string();
            if is_mut {
                quote! {
                    #field_name_str => Some(#value_enum_ident::#variant_ident(&mut self.#field_name))
                }
            } else {
                quote! {
                    #field_name_str => Some(#value_enum_ident::#variant_ident(&self.#field_name))
                }
            }
        })
        .collect()
}

/// Получение имени из атрибута
fn retrieve_enum_name(attrs: &[Attribute]) -> Option<Ident> {
    if let Some(TokenTree::Literal(lit)) = get_fieldname_enum_val(attrs, "name") {
        let lit = lit.to_string();
        // TODO: как избавиться от `"\"`
        Some(Ident::new(&lit[1..lit.len() - 1], Span::call_site()))
    } else {
        None
    }
}

/// Получение дерайв макросов для енама
fn retrieve_derives(
    attrs: &[Attribute],
    derive_group: &str,
) -> Option<proc_macro2::TokenStream> {
    if let Some(TokenTree::Group(group)) =
        get_fieldname_enum_val(attrs, derive_group)
    {
        let token_stream = group.stream();
        Some(quote!(#[derive(#token_stream)]))
    } else {
        None
    }
}

/// Получение имени из атрибута
fn retrieve_fieldname(attrs: &[Attribute]) -> Option<Ident> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(meta_name_value) => {
            let fieldname_enum_attr = meta_name_value.path.segments.first()?;
            if fieldname_enum_attr.ident != "fieldname" {
                return None;
            }
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(ref lit_str),
                ..
            }) = meta_name_value.value
            {
                Some(Ident::new(&lit_str.value(), Span::call_site()))
            } else {
                None
            }
        }
        _ => None,
    })
}

fn get_fieldname_enum_val(
    attrs: &[Attribute],
    attr_name: &str,
) -> Option<TokenTree> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::List(meta_list) => {
            let name_attr = meta_list.path.segments.first()?;
            if name_attr.ident != "fieldname_enum" {
                return None;
            }
            meta_list
                .tokens
                .clone()
                .into_iter()
                .skip_while(|token| token.to_string() != attr_name)
                .nth(2)
        }
        _ => None,
    })
}
