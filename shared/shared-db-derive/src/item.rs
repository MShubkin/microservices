use crate::shared::*;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput};

// Имена атрибутов для разбора proc-macro.
pub(crate) const AGGR_INSERT: &str = "item_aggr_insert";
pub(crate) const ITEM_PKEY: &str = "item_field_pkey";
pub(crate) const ITEM_AUTOGEN: &str = "item_field_autogen";
pub(crate) const ITEM_AUTOGEN_ALWAYS: &str = "item_field_autogen_always";
const ITEM_ACTIVATE: &str = "item_field_activate_with";
pub(crate) const ITEM_TABLE: &str = "item_table";
const ITEM_ACTIVATE_ALL: &str = "item_activate_all_with";
const ITEM_MANUALLY_ACTIVATE_FIELDS: &str = "item_manually_activate_fields";
const ITEM_SKIP_FIELD_TOLERANCE: &str = "item_skip_field_tolerance";
pub(crate) const DB_FIELD_NAME: &str = "db_field_name";
const ITEM_FIELD_REQUIRED: &str = "item_field_require_from_row";

/// Основная точка входа макроса `#[derive(DbItem)]`.
///
/// Разбирает структуру, собирает информацию о полях и атрибутах,
/// генерирует:
/// 1. `impl FromRow` -- чтение строки из БД (с дефолтами для отсутствующих колонок).
/// 2. `impl DbItem` -- константы и методы bind для CRUD-операций.
/// 3. `impl sqlx::Type` + `impl sqlx::decode::Decode` -- для использования структуры
///    в качестве колонки в JOIN-запросах (PostgreSQL RECORD тип).
/// 4. `impl FieldTolerance` -- если не задан `item_skip_field_tolerance`.
/// 5. Константы `pub const field_name: &'static str = "col_name"` для каждого поля --
///    основа типобезопасных select-запросов через `Select::with_fields([T::field])`.
pub(crate) fn item_inner(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);

    let old_name = &inp.ident;

    // Макрос работает только со структурами.
    let input_struct = get_struct(&inp, "DbItem", old_name);
    let fields = get_named_fields(input_struct, "DbItem");

    if fields.is_empty() {
        panic!("`DbItem` does not deal with empty structures.");
    }
    // old_field_names -- имена полей в Rust. db_field_names -- имена колонок в БД
    // (могут отличаться при использовании #[db_field_name = "col_name"]).
    let (old_field_names, db_field_names) = get_struct_and_db_field_names(&fields);

    // Извлекаем первичные ключи и их индексы в общем массиве полей.
    let (pkey_idx, pkeys0): (Vec<_>, Vec<_>) =
        find_idx_fields_with(&fields, ITEM_PKEY)
            .map(|x| (x.0, x.1.clone()))
            .unzip();
    let (pkeys, db_pkeys) = get_struct_and_db_field_names(&pkeys0);

    if pkeys.is_empty() {
        panic!("`DbItem` requires at least one primary key.")
    }

    // Имя таблицы: из атрибута `#[item_table = "name"]` или snake_case имени структуры.
    let table_name = get_attr_ident(&inp.attrs, ITEM_TABLE).unwrap_or_else(|| {
        syn::Ident::new(
            &old_name.to_string().to_snake_case(),
            proc_macro2::Span::call_site(),
        )
    });

    let manual_activation = has_attr(&inp.attrs, ITEM_MANUALLY_ACTIVATE_FIELDS);
    let skip_field_tolerance = has_attr(&inp.attrs, ITEM_SKIP_FIELD_TOLERANCE);
    let other_field_activation = has_attr(&inp.attrs, ITEM_ACTIVATE_ALL)
        || has_attr(&inp.attrs, ITEM_ACTIVATE);

    // item_manually_activate_fields и авто-активация взаимоисключающие.
    if manual_activation && other_field_activation {
        panic!(
            "DbItem `{}`: `item_manually_activate_fields` is exclusive to (`item_field_activate_with` || `item_activate_all_with`)",
            old_name
        );
    }

    // INSERT_FIELDS: все поля кроме autogen и autogen_always.
    let insertable_fields0 =
        find_fields_without(&fields, &[ITEM_AUTOGEN, ITEM_AUTOGEN_ALWAYS])
            .cloned()
            .collect::<Vec<_>>();
    let (insertable_fields, db_insertable_fields) =
        get_struct_and_db_field_names(&insertable_fields0);

    // NO_UPDATE_INDICES: индексы полей с autogen_always.
    let autogen_always = |x: &syn::Field| has_attr(&x.attrs, ITEM_AUTOGEN_ALWAYS);
    let no_update_idx = fields
        .iter()
        .enumerate()
        .filter_map(|(i, x)| autogen_always(x).then_some(i));

    // UPDATE_FIELDS: все поля кроме autogen_always.
    let updatable_fields0 = fields
        .iter()
        .filter(|x| !autogen_always(x))
        .cloned()
        .collect::<Vec<_>>();

    let (_, db_updatable_fields) =
        get_struct_and_db_field_names(&updatable_fields0);

    let activated_fields = get_field_activation(&fields, &inp.attrs);

    let old_field_types = extract_field_types(&fields);
    let count = (0..old_field_names.len()).collect::<Vec<usize>>();
    let insertable_idx = insertable_fields.len();

    let insertable_fields_ref = &insertable_fields;

    // row_activator: список выражений `row.try_get("col_name")` для FromRow.
    // partial_select: true если структура может читаться из неполного SELECT.
    let (row_activator, partial_select) =
        row_try_get(&fields, &db_field_names, &inp.attrs);

    let types = get_unique_field_types(&fields);

    // Генерируем impl FromRow. Использует try_get с fallback на Default для полей
    // без #[item_field_require_from_row], что позволяет SELECT с подмножеством колонок.
    let mut stream = quote! {
        impl<'r, R> sqlx::FromRow<'r, R> for #old_name
        where R: sqlx::Row,
        &'r str: sqlx::ColumnIndex<R>,
        #(#types: sqlx::decode::Decode<'r, R::Database> + sqlx::types::Type<R::Database>,)*
        {
            fn from_row(row: &'r R) -> std::result::Result<Self, sqlx::Error> {
                Ok(Self {
                    #(
                        #old_field_names: #row_activator,
                    )*
                })
            }
        }
    };
    if partial_select {
        stream.extend(quote!(impl asez2_shared_db::db_item::db_item_core::DbItemPartialSelect for #old_name {}))
    }

    // Основной блок impl DbItem: константы и методы bind.
    let mut stream_item_a = quote! {
            const TABLE: &'static str = stringify!(#table_name);
            const PRIMARY_KEYS: &'static[&'static str] = &[
                #( stringify!(#db_pkeys), )*
            ];
            const PRIMARY_KEY_INDICES: &'static[usize] = &[
                #( #pkey_idx, )*
            ];
            const FIELDS: &'static [&'static str] = &[
                #( stringify!(#db_field_names), )*
            ];
            const INSERT_FIELDS: &'static [&'static str] = &[
                #( stringify!(#db_insertable_fields), )*
            ];
            const UPDATE_FIELDS: &'static [&'static str] = &[
                #( stringify!(#db_updatable_fields), )*
            ];
            const NO_UPDATE_INDICES: &'static[usize] = &[
                #( #no_update_idx, )*
            ];

            fn bind_pkeys<'a>(
                &'a self,
                query: sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments>,
            ) -> sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments> {
                query
                    #(.bind(&self.#pkeys) )*
            }

            fn bind_to_query<'a>(
                &'a self,
                query: sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments>,
            ) -> sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments> {
                query
                    #(.bind(&self.#old_field_names) )*
            }

            fn bind_to_query_insert<'a>(
                &'a self,
                query: sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments>,
            ) -> sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments> {
                query
                    #(.bind(&self.#insertable_fields_ref) )*
            }

            fn bind_limited_fields<'a>(
                &'a self,
                mut query: sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments>,
                mask: &asez2_shared_db::db_item::DbFieldMask<Self>,
            ) -> sqlx::query::Query<'a, sqlx::Postgres, <sqlx::Postgres as sqlx::database::HasArguments<'a>>::Arguments> {
                #(if mask[#count] {
                    query = query.bind(&self.#old_field_names);
                })*
                query
            }
    };

    if !manual_activation {
        // Авто-генерация activate_fields: для каждого поля, если значение равно
        // Default::default(), подставляется активирующее выражение.
        stream_item_a.extend(quote! {
            fn activate_fields(&mut self) {
                #(
                    if self.#old_field_names == <#old_field_types>::default() {
                        self.#old_field_names = #activated_fields;
                    }
                    )*
            }
        });
    } else {
        // Ручная активация: вызывается метод, который нужно написать вручную.
        stream_item_a.extend(quote! {
            fn activate_fields(&mut self) {
                self.activate_fields_manually();
            }
        });
    };

    // Генерируем UNNEST-версии insert_vec/update_vec если задан атрибут item_aggr_insert.
    // Более эффективно для больших объёмов данных за счёт одного DB-вызова,
    // но медленнее для малых из-за клонирования данных в промежуточные векторы.
    if inp.attrs.iter().any(|a| a.path().is_ident(AGGR_INSERT)) {
        stream_item_a.extend(quote!{
            /// Альтернативная реализация вставки через UNNEST -- один DB-вызов для всего среза.
            /// Для малых наборов может быть медленнее дефолтной из-за клонирования полей.
            async fn insert_vec_inner(
                items: &mut [Self],
                pool: &mut sqlx::Transaction<sqlx::Postgres>,
                returning: bool,
            ) -> asez2_shared_db::result::Result<asez2_shared_db::db_item::ReturningEither<Self>> {
                use sqlx::FromRow;
                use asez2_shared_db::db_item::ReturningEither;
                use futures::TryStreamExt;

                if items.is_empty() {
                    let r: ReturningEither<Self> = ReturningEither::new(returning);
                    return Ok(r);
                }
                let mut ret: ReturningEither<Self> = ReturningEither::new(returning);
                let fields = Self::insert_fields_string();
                let arrays: String = (0..#insertable_idx)
                    .map(|n| format!("${}", n + 1))
                    .collect::<Vec<_>>()
                    .as_slice()
                    .join(",");
                let query_string = format!(
                    "INSERT INTO {table_name}({fields}) SELECT * FROM UNNEST({arrays}){return_clause};",
                    table_name = Self::TABLE,
                    fields = fields,
                    arrays = arrays,
                    return_clause = if returning { " returning *" } else { "" },
                );
                // Ограничиваем чанк чтобы не перегрузить PostgreSQL RAM.
                for items in items.chunks_mut(asez2_shared_db::db_item::UPDATE_UNNEST_VALUES / #insertable_idx) {
                    #(
                        let mut #insertable_fields_ref = Vec::with_capacity(items.len());
                    )*
                    // Клонируем поля в отдельные векторы для UNNEST.
                    for item in items {
                        item.activate_fields();
                        #( #insertable_fields_ref.push(item.#insertable_fields_ref.to_owned()); )*
                    }
                    let mut query = sqlx::query(&query_string);
                    #(
                        query = query.bind(&#insertable_fields_ref[..]);
                    )*
                    match ret {
                        ReturningEither::RowsAffected(ref mut n) => {
                            *n += query.execute(&mut *pool).await?.rows_affected();
                        }
                        ReturningEither::Items(ref mut items) => {
                            let mut stream = query.try_map(|x| Self::from_row(&x)).fetch(&mut *pool);
                            while let Some(item) = stream.try_next().await? {
                                items.push(item);
                            }
                        }
                    }
                }
                Ok(ret)
            }
            /// Альтернативная реализация обновления через UNNEST.
            /// TODO: Корректно обработать обновления без PKey.
            async fn update_vec_inner(
                items: &[Self],
                update_fields: Option<&[&str]>,
                return_fields: Option<&[&str]>,
                pool: &mut sqlx::Transaction<sqlx::Postgres>,
            ) -> asez2_shared_db::result::Result<asez2_shared_db::db_item::ReturningEither<Self>> {
                use sqlx::FromRow;
                use asez2_shared_db::db_item::ReturningEither::{self, *};
                use asez2_shared_db::db_item::{
                    make_bind_mask, select_fields_helper, update_fields_helper
                };
                use asez2_shared_db::db_item::{field_counter};
                use futures::TryStreamExt;

                if items.is_empty() {
                    let r: ReturningEither<Self> = ReturningEither::new(return_fields.is_some());
                    return Ok(r);
                }
                let mut ret: ReturningEither<Self> = ReturningEither::new(return_fields.is_some());
                let t = 't';
                let upd = "upd";

                let bind_mask = update_fields
                    .as_ref()
                    .map(|x| make_bind_mask::<Self>(&x))
                    .unwrap_or_else(|| make_bind_mask::<Self>(Self::UPDATE_FIELDS));
                let selected_fields = update_fields_helper::<Self>(&bind_mask);
                let field_counts = field_counter(&selected_fields, 0);

                let return_clause = return_fields
                    .map(|x| {
                        let bind_mask = make_bind_mask::<Self>(x);
                        let fields = select_fields_helper::<Self>(&bind_mask)
                            .into_iter()
                            .map(|x| format!("{t}.{x}", t = t, x = x))
                            .collect::<Vec<_>>();
                        format!(" RETURNING {}", fields.join(","))
                    })
                    .unwrap_or_default();
                let field_mapping = selected_fields
                    .iter()
                    .map(|x| format!("{f}={upd}.{f}", upd = upd, f = x))
                    .collect::<Vec<_>>()
                    .join(",");
                let where_clause = Self::PRIMARY_KEYS
                    .iter()
                    .map(|k| format!("{t}.{k}={upd}.{k}", t = t, upd = upd, k = k))
                    .collect::<Vec<_>>()
                    .join(" AND ");

                let query_string = format!(
                    r#"UPDATE {table} AS {t} SET
                    {field_mappings}
                    FROM
                    (SELECT * FROM UNNEST{field_counts}) AS {upd}({fields})
                    WHERE {where_clause}{returning};"#,
                    table = Self::TABLE,
                    fields = selected_fields.join(","),
                    field_counts = field_counts,
                    field_mappings = field_mapping,
                    t = t,
                    upd = upd,
                    where_clause = where_clause,
                    returning = return_clause,
                );

                let count = update_fields.as_ref().map(|x| x.len()).unwrap_or(#insertable_idx);
                for items in items.chunks(asez2_shared_db::db_item::UPDATE_UNNEST_VALUES / count) {
                    #( let #old_field_names = match bind_mask[#count] {
                        false => vec![],
                        true => items.iter().map(|x| x.#old_field_names.to_owned()).collect(),
                    }; )*
                    let mut query = sqlx::query(&query_string);
                    #( if !#old_field_names.is_empty() {
                        query = query.bind(&#old_field_names[..]);
                    } )*
                    match ret {
                        ReturningEither::RowsAffected(ref mut n) => {
                            *n += query.execute(&mut *pool).await?.rows_affected();
                        }
                        ReturningEither::Items(ref mut items) => {
                            let mut stream = query.try_map(|x| Self::from_row(&x)).fetch(&mut *pool);
                            while let Some(item) = stream.try_next().await? {
                                items.push(item);
                            }
                        }
                    }
                }
                Ok(ret)
            }
        });
    }
    let item_stream_outer = quote! {
        #[async_trait::async_trait]
        impl asez2_shared_db::db_item::DbItem for #old_name {
            #stream_item_a
        }
    };
    stream.extend(item_stream_outer);

    // Реализация sqlx::Type и sqlx::decode::Decode позволяет использовать
    // структуру как RECORD-тип в JOIN (т.е. читать целую строку в поле).
    stream.extend(quote! {
        impl sqlx::Type<sqlx::Postgres> for #old_name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("RECORD")
            }
        }

        impl sqlx::postgres::PgHasArrayType for #old_name {
            fn array_type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("RECORD[]")
            }
        }

        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for #old_name
        where
            #(#old_field_types: 'r,)*
            #(#old_field_types: sqlx::Type<sqlx::Postgres>,)*
            #(#old_field_types: for<'a> sqlx::decode::Decode<'a, sqlx::Postgres>,)*
        {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
                #[allow(unused)]
                let mut decoder = sqlx::postgres::types::PgRecordDecoder::new(value)?;

                #(let #old_field_names: #old_field_types = decoder.try_decode()?;)*

                Ok(#old_name {
                    #(#old_field_names,)*
                })
            }
        }
    });
    if !skip_field_tolerance {
        stream.extend(quote! {
            impl asez2_shared_db::db_item::FieldTolerance for #old_name {}
        });
    }

    // Это ключевая часть: каждое поле получает константу с именем колонки.
    // `pub const field_name: &'static str = "col_name"` -- именно так
    // `Select::with_fields([Entity::field_name])` получает типобезопасные имена.
    stream.extend(quote! {
        #[allow(non_upper_case_globals)]
        impl #old_name {
        #(
            #[allow(dead_code)]
            pub const #old_field_names: &'static str = stringify!(#db_field_names);
        )*
        }
    });

    stream.into()
}

/// Строит вектор выражений активации полей перед INSERT.
///
/// Для каждого поля проверяет: есть ли `#[item_field_activate_with = "expr"]`,
/// затем глобальный `#[item_activate_all_with = "expr"]`, иначе -- `T::default()`.
fn get_field_activation(
    fields: &[syn::Field],
    global: &[Attribute],
) -> Vec<TokenStream> {
    let global_activator = find_attribute_as_expr(global, ITEM_ACTIVATE_ALL);
    fields
        .iter()
        .map(|x| {
            find_attribute_as_expr(&x.attrs, ITEM_ACTIVATE)
                .or_else(|| global_activator.clone())
                .unwrap_or_else(|| {
                    let ftype = &x.ty;
                    quote::quote!(<#ftype>::default())
                })
        })
        .collect::<Vec<_>>()
}

fn find_attribute_as_expr(attrs: &[Attribute], key: &str) -> Option<TokenStream> {
    get_attr_expr(attrs, key)
}

/// Строит выражения `row.try_get("col_name")` для каждого поля.
///
/// Два режима:
/// 1. Если поле имеет `#[item_field_require_from_row]` или задан глобальный
///    `item_field_require_from_row` -- отсутствие колонки в SELECT вернёт ошибку
///    с понятным сообщением. `partial_select = false`.
/// 2. Иначе -- отсутствие колонки заменяется на `T::default()`.
///    Это позволяет SELECT с подмножеством полей. `partial_select = true`.
fn row_try_get(
    fields: &[syn::Field],
    db_fields: &[syn::Ident],
    global: &[Attribute],
) -> (Vec<TokenStream>, bool) {
    let has_tag = |attrs: &[Attribute]| -> bool {
        attrs.iter().any(|x| x.path().is_ident(ITEM_FIELD_REQUIRED))
    };
    let has_global_require_field = has_tag(global);
    let mut partial_select = !has_global_require_field;
    let res = fields
        .iter()
        .zip(db_fields.iter())
        .map(|(x, db_field)| {
            let ident: String = db_field.to_string();
            let ident_ref: &str = ident.as_ref();
            let t = &x.ty;
            if has_global_require_field || has_tag(&x.attrs) {
                partial_select = false;
                quote!(
                    match row.try_get(#ident_ref) {
                        Ok(r) => r,
                        Err(sqlx::Error::ColumnNotFound(index)) =>
                           Err(sqlx::Error::ColumnDecode {
                               index,
                               source: format!("column is absent in `SELECT` statement due to `{}` attribute", #ITEM_FIELD_REQUIRED).into()
                           })?,
                        x => x?,
                    }
                )
            } else {
                quote!(
                    match row.try_get(#ident_ref) {
                        Ok(r) => r,
                        Err(sqlx::Error::ColumnNotFound(_)) => <#t>::default(),
                        x => x?,
                    }
                )
            }
        })
        .collect::<Vec<_>>();
    (res, partial_select)
}

/// Возвращает уникальные типы полей -- нужно для where-bounds в `impl FromRow`.
///
/// Дубликаты удаляются через `HashSet`, чтобы не генерировать повторные bounds
/// вида `i64: Decode + ..., i64: Decode + ...`.
fn get_unique_field_types(
    fields: &[syn::Field],
) -> impl Iterator<Item = syn::Type> {
    fields
        .iter()
        .map(|x| x.ty.to_owned())
        .collect::<std::collections::HashSet<syn::Type>>()
        .into_iter()
}

/// Возвращает два списка: Rust-имена полей и имена колонок в БД.
///
/// Если поле помечено `#[db_field_name = "col_name"]`, имя колонки берётся
/// из атрибута. Иначе -- имя поля совпадает с именем колонки.
fn get_struct_and_db_field_names(
    fields: &[syn::Field],
) -> (Vec<&proc_macro2::Ident>, Vec<proc_macro2::Ident>) {
    fields
        .iter()
        .map(|x| {
            let old_name = x.ident.as_ref().unwrap();
            let db_name = rename(&x.attrs, DB_FIELD_NAME, old_name);
            (old_name, db_name)
        })
        .unzip()
}
