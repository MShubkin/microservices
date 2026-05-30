use crate::shared::*;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::*;
use syn::{parse_macro_input, DeriveInput};

// Строковые константы атрибутов — используются и в `has_attr`, и в
// `find_name_value_attr`, поэтому выносим сюда, чтобы опечатка не
// привела к тихому молчанию макроса.
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

/// Направление конвертации для [`make_conversion`].
///
/// `F` (forwards) — из адаптора в DbItem (`into_item`).
/// `R` (reverse) — из DbItem в адаптор (`from_item_masked`).
#[derive(Debug, Clone, Copy)]
pub(crate) enum ConversionKind {
    F,
    R,
}

/// Точка входа `#[derive(DbAdaptor)]`.
///
/// Генерирует теневую структуру-адаптор `{Name}Rep` (или с именем из
/// `adaptor_rename`) и реализует для неё трейт `DbAdaptor`.
///
/// Ключевые шаги:
/// 1. Переименовываем структуру (`adaptor_rename`) и обрабатываем поля
///    (`process_fields`): каждое поле превращается в `Option<T>`.
/// 2. Собираем поля-дубликаты (`adaptor_field_duplicate`) — это поля,
///    которые присутствуют в адапторе под двумя именами сразу, но в
///    DbItem хранятся один раз. Нужно для того, чтобы фронтенд мог
///    читать одно и то же значение через разные ключи (например,
///    `id` и `item_id` одновременно).
/// 3. Генерируем прямые (`F`) и обратные (`R`) конвертации — они
///    учитывают `adaptor_from`, `adaptor_try_from`, `adaptor_into`.
/// 4. Опционально генерируем `DbAdaptorFieldsWithValues` для экспорта
///    всех полей в виде `Vec<Field>` (нужно для Excel/CSV).
pub(crate) fn adaptor_inner(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);
    let vis = &inp.vis;

    // Имя адаптора: по умолчанию `{OldName}Rep`, переопределяется через
    // `#[adaptor_rename = "CustomName"]` на структуре.
    let old_name = &inp.ident;
    let default_name = Ident::new(&format!("{}Rep", old_name), Span::call_site());
    let new_name = rename(&inp.attrs, "adaptor_rename", &default_name);

    // Ранняя проверка — `DbAdaptor` применим только к структурам,
    // для enum он бессмысленен (нет именованных полей для частичного обновления).
    let input_struct = get_struct(&inp, "DbAdaptor", old_name);
    let fields = get_named_fields(input_struct, "DbAdaptor");

    if fields.is_empty() {
        panic!("`DbItem` does not deal with empty structures.");
    }
    // Индексы полей нужны для генерации `mask[#field_counts]` в методах
    // `bind_mask`, `zero_fields`, `unset_fields`, `from_item_masked`.
    let field_counts = (0..fields.len()).collect::<Vec<usize>>();

    // Оригинальные имена полей DbItem используются в `into_item` и
    // `into_item_merged` для присваивания `Self::DbItem { field: ... }`.
    let old_field_names =
        fields.iter().map(|x| x.ident.as_ref().unwrap()).collect::<Vec<_>>();

    // Преобразуем поля: переименовываем и оборачиваем в `Option<T>`.
    let adaptor_fields = process_fields(&fields);
    // Дублирующие поля — отдельный список, потому что они не входят в
    // `DbItem::FIELDS` и не учитываются в `bind_mask`/`field_counts`.
    let (duplicate, dup_field_indices) = duplicate_fields(&fields);

    let new_field_names = get_ident(&adaptor_fields);
    let new_duplicate_fname = get_ident(&duplicate);

    // Объединённый список всех полей адаптора (основные + дубликаты)
    // нужен для `DbAdaptorFieldsWithValues::FIELDS`.
    let combined_field_names: Vec<_> = new_field_names
        .iter()
        .chain(new_duplicate_fname.iter())
        .cloned()
        .collect();

    let dup_field_counter = 0..new_duplicate_fname.len();

    // `forwards` — выражения для `into_item`: Adaptor.field → DbItem.field.
    // `backwards` — выражения для `from_item_masked`: DbItem.field → Adaptor.field.
    // `backwards_dup` — то же, но для дублирующих полей (с `.clone()` чтобы
    // не потреблять значение раньше времени).
    let (forwards, _) =
        field_conversion(&fields, &adaptor_fields, &[], ConversionKind::F);
    let (backwards, backwards_dup) = field_conversion(
        &fields,
        &adaptor_fields,
        &dup_field_indices,
        ConversionKind::R,
    );

    // Атрибуты-наследники: derive-трейты, внешние атрибуты структуры и
    // атрибуты, применяемые ко всем полям.
    let derives = retain_attributes(&inp.attrs, ADAPTOR_DERIVE);
    let outer_attributes = adaptor_attributes(&inp.attrs, ADAPTOR_ATTRIBUTES);
    let extra_inherits = adaptor_attributes(&inp.attrs, ADAPTOR_ATTRIBUTE_FOR_ALL);

    // Атрибуты конкретных полей (например, `#[serde(rename = "...")]`).
    let inherited_field_attrs =
        fields.iter().map(|x| inherit_attributes(x, &extra_inherits));
    let inherited_duplicate_attrs =
        duplicate.iter().map(|x| inherit_attributes(x, &extra_inherits));

    // Объединённый список полей для `DbAdaptorFieldsWithValues`.
    let combined_field: Vec<_> =
        adaptor_fields.iter().chain(duplicate.iter()).cloned().collect();
    let combined_field_counts = (0..combined_field.len()).collect::<Vec<usize>>();

    let s: Path = parse_quote!(self);
    let field_converters_a = crate::item_ext::convert_fields(&combined_field, s);

    let mut stream = quote! {
        # (#derives)*
        # (#outer_attributes)*
        // `#[serde(default)]` на структуре — отсутствующее поле в JSON становится
        // `None` (Default для Option), а не ошибкой десериализации. Именно это
        // обеспечивает семантику "поле не передано = не трогать".
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

            // DUP_FIELDS содержит только имена дублирующих полей — тех,
            // которых нет в DbItem::FIELDS. Разделение нужно чтобы
            // DbAdaptorFieldMask мог обрабатывать оба набора независимо.
            const DUP_FIELDS: &'static [&'static str] = &[#(stringify!(#new_duplicate_fname),)*];

            fn into_item(self) -> asez2_shared_db::result::Result<Self::DbItem> {
                // `a` — соглашение: все сгенерированные выражения конвертации
                // обращаются к `a.field_name`, поэтому переименовываем `self`.
                let a = self;
                Ok(Self::DbItem {
                    #(
                        #old_field_names: #forwards,
                    )*
                })
            }

            /// Конвертирует DbItem в адаптор с учётом маски полей.
            ///
            /// Поля вне маски устанавливаются в `None` — это значит "не передавать
            /// фронтенду". Используется при ответе сервера: если клиент запросил
            /// только часть полей, остальные должны быть опущены в JSON.
            ///
            /// NB: tolerance (псевдонимы полей) здесь не применяется — маска
            /// уже построена по каноническим именам.
            fn from_item_masked(
                item: Self::DbItem,
                mask: &asez2_shared_db::db_item::DbAdaptorFieldMask<Self>,
            ) -> Self {
                use asez2_shared_db::db_item::FieldTolerance;

                let field_mask = mask.item_field_mask();
                let dup_mask = mask.dup_field_mask();

                let a = item;
                let output = #new_name {
                    // Дублирующие поля идут первыми — они читают `a.field.clone()`,
                    // поэтому важно, что они не потребляют значение до основного поля.
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

            /// Строит маску по тому, какие поля адаптора не `None`.
            ///
            /// Используется при частичном UPDATE: только поля с `Some(v)` попадают
            /// в SET-часть запроса. Дублирующие поля не участвуют — они не
            /// соответствуют колонкам в таблице.
            fn bind_mask(&self) -> asez2_shared_db::db_item::DbFieldMask<Self::DbItem>
            {
                let mut mask = asez2_shared_db::db_item::DbFieldMask::<Self::DbItem>::none();
                #(
                    mask[#field_counts] = self.#new_field_names.is_some();
                )*
                mask
            }

            /// Строгий вариант `bind_mask` для массового UPDATE: требует, чтобы
            /// одно и то же множество полей было `Some` во всех элементах среза.
            ///
            /// Это нужно для UNNEST-запроса: все строки должны обновлять одинаковый
            /// набор колонок, иначе запрос нельзя построить в виде одного UPDATE.
            fn create_strict_bind_mask(items: &[Self]) -> asez2_shared_db::result::Result<asez2_shared_db::db_item::DbFieldMask<Self::DbItem>> {
                let mut mask = asez2_shared_db::db_item::DbFieldMask::<Self::DbItem>::none();
                #(
                    // Свёртка: если хотя бы один элемент имеет поле Some, а другой None
                    // — это ошибка несогласованности батча.
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

            /// Применяет адаптор к существующему DbItem: перезаписывает только те поля,
            /// где значение `Some`. Используется для PATCH-семантики: берём текущую
            /// запись из БД и накладываем поверх неё изменения от фронтенда.
            fn into_item_merged(self, mut item: Self::DbItem) -> asez2_shared_db::result::Result<Self::DbItem> {
                let a = self;
                #(
                    if a.#new_field_names.is_some() {
                        item.#old_field_names = #forwards;
                    }
                )*
                Ok(item)
            }

            /// Переводит поля из маски в `Some(Default::default())`.
            ///
            /// Используется когда нужно явно "обнулить" значения в адапторе:
            /// поле становится `Some(default)`, то есть UPDATE напишет дефолт в БД.
            fn zero_fields(mut self, fields: &asez2_shared_db::db_item::DbFieldMask<Self::DbItem>) -> Self {
                #(
                    if fields[#field_counts] {
                        self.#new_field_names = Some(Default::default());
                    }
                )*
                self
            }

            /// Переводит поля из маски в `None`.
            ///
            /// Используется чтобы исключить поля из UPDATE: поле становится `None`,
            /// и `bind_mask` его проигнорирует.
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
    // `adaptor_fields_with_values` генерируется только по явному запросу,
    // потому что реализация требует `Value::from` для каждого типа поля,
    // что не всегда выполнимо (например, для вложенных структур без `Into<Value>`).
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

/// Строит вектор выражений конвертации поля для всего списка полей.
///
/// Возвращает два вектора:
/// - основные конвертации (один к одному с `input_fields`)
/// - конвертации для дублирующих полей по `duplicate_indices`
///
/// Дублирующие поля читают то же поле `a.X`, но с `.clone()` — иначе
/// значение будет потреблено при первом использовании.
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

/// Строит одно выражение конвертации для пары полей `(field_a, field_b)`.
///
/// Для направления `F` (Adaptor → DbItem):
/// - `adaptor_from = "fn"` → `a.field.map(|x| fn(x)).unwrap_or_default()`
/// - `adaptor_try_from = "fn"` → то же, но с `?` (fallible)
/// - по умолчанию → `a.field.map(|x| Into::into(x)).unwrap_or_default()`
///
/// Для направления `R` (DbItem → Adaptor):
/// - `adaptor_into = "fn"` → `Some(fn(a.field))`
/// - по умолчанию → `Some(Into::into(a.field))`
///
/// `clone = true` используется для дублирующих полей, чтобы не потребить
/// значение при первом обращении.
// Конвертация не рекурсивная: если тип требует сложного преобразования,
// его нужно задать через `adaptor_from`/`adaptor_into` явно.
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
    // Дублирующие поля клонируются, чтобы не потребить `a.field`
    // до того, как основное поле выполнит свою конвертацию.
    let field = match clone {
        false => quote! { a.#fname },
        true => quote! { a.#fname.clone() },
    };

    match kind {
        // `a` — соглашение: везде в сгенерированном коде входная структура
        // переименована в `a` (через `let a = self` или `let a = item`).
        ADAPTOR_FROM => parse_quote!{ #field.map(|x| #convertor(x)).unwrap_or_default() },
        ADAPTOR_INTO => parse_quote!{ Some(#convertor(#field)) },
        ADAPTOR_TRY_FROM => parse_quote!{ #field.map(|x| #convertor(x)).transpose()?.unwrap_or_default() },
        x => panic!(
            "Incorrect argument \"{}\" in derive in `make_conversion`, please contact the developer.",
            x,
        ),
    }
}

/// Применяет `process_field` ко всем полям структуры.
pub(super) fn process_fields(input_fields: &[Field]) -> Vec<Field> {
    input_fields
        .iter()
        .cloned()
        .map(|x| process_field(x, ADAPTOR_RENAME))
        .collect::<Vec<_>>()
}

/// Выделяет дублирующие поля из списка полей.
///
/// Возвращает пару `(поля, индексы)`:
/// - поля — обработанные Field с именем из `adaptor_field_duplicate`
/// - индексы — позиции в исходном массиве, нужны для выбора правильного
///   выражения конвертации в `field_conversion`
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

/// Обрабатывает одно поле для включения в адаптор:
///
/// 1. Переименовывает: `adaptor_rename` или `adaptor_field_duplicate` → новый ident.
/// 2. Оборачивает тип в `Option<T>` или `Option<NewType>` (если задан `adaptor_type`).
/// 3. Стирает атрибуты proc-macro — в итоговом коде они не нужны,
///    остаются только derive-трейты для полей через `adaptor_derive`.
fn process_field(mut x: Field, rename_kind: &str) -> Field {
    let old_ident = x.ident.expect("Adaptor is only derived for named fields.");
    x.ident = Some(rename(&x.attrs, rename_kind, &old_ident));
    x.ty = retype(&x.attrs, ADAPTOR_TYPE, x.ty.clone());
    // Стираем атрибуты в последнюю очередь: retype и rename читают их выше.
    x.attrs = retain_attributes(&x.attrs, ADAPTOR_DERIVE);
    x
}

/// Разворачивает атрибуты вида `#[adaptor_attributes(#[serde(rename_all = "camelCase")])]`
/// в плоский список `[#[serde(rename_all = "camelCase")]]`.
///
/// `kind` определяет имя оборачивающего атрибута:
/// - `adaptor_attributes` — атрибуты только для структуры адаптора
/// - `adaptor_attribute_for_all` — атрибуты для всех полей адаптора
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

/// Собирает финальный список атрибутов для поля адаптора.
///
/// Порядок приоритетов: атрибуты самого поля (`adaptor_attributes` на поле)
/// имеют приоритет над атрибутами структуры (`adaptor_attribute_for_all`).
/// Это нужно чтобы поле могло переопределить, например, `#[serde(skip)]`
/// даже если на структуре стоит `#[serde(rename_all = "camelCase")]`.
///
/// После этого добавляются два обязательных serde-атрибута для Option-полей:
/// - `#[serde(with = "DbAdaptorOption")]` — кастомный (де)сериализатор,
///   который правильно обрабатывает вложенный `Option` (null vs. отсутствие).
/// - `#[serde(skip_serializing_if = "Option::is_none")]` — поле не сериализуется,
///   если оно `None`, то есть не передаётся в JSON-ответе вообще.
fn inherit_attributes(f: &Field, extra_inherits: &[Attribute]) -> Vec<Attribute> {
    let mut attrs = adaptor_attributes(&f.attrs, ADAPTOR_ATTRIBUTES);
    // Атрибуты с тем же путём не дублируем: поле имеет приоритет.
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
