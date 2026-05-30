mod adaptor;
mod enumeration;
mod item;
mod item_ext;
mod item_upsert;
mod shared;
mod versioned;

/// Генерирует теневую структуру-адаптор (`{Name}Rep`) для передачи данных между
/// фронтендом и БД через частичные обновления.
///
/// Все поля адаптора имеют тип `Option<T>`: `None` означает "поле не затронуто",
/// `Some(v)` -- обновить до `v`. Это позволяет фронтенду передавать только
/// изменённые поля без знания полного состояния записи.
///
/// Атрибуты для полей:
/// - `adaptor_type = "NewType"` -- заменяет тип поля в адапторе.
/// - `adaptor_into = "fn"` -- кастомная конвертация из адаптора в DbItem (поле -> поле).
/// - `adaptor_from = "fn"` -- кастомная конвертация из DbItem в адаптор (поле -> поле).
/// - `adaptor_try_from = "fn"` -- то же, но с `?` (fallible).
/// - `adaptor_field_duplicate = "new_name"` -- создаёт дополнительное поле как
///   копию текущего под другим именем. Дублирующие поля не участвуют в DB-операциях.
/// - `adaptor_rename = "new_name"` -- переименовывает поле или структуру в адапторе.
///
/// Атрибуты для структуры:
/// - `adaptor_derive(A, B, C)` -- дополнительные derive-трейты для адаптора.
/// - `adaptor_attributes(#[attr])` -- дополнительные атрибуты самой структуры адаптора.
/// - `adaptor_attribute_for_all(#[attr])` -- атрибут, применяемый ко всем полям.
/// - `adaptor_fields_with_values` -- реализует `DbAdaptorFieldsWithValues` для экспорта.
#[proc_macro_derive(
    DbAdaptor,
    attributes(
        adaptor_type,
        adaptor_into,
        adaptor_from,
        // adaptor_try_into,
        adaptor_try_from,
        adaptor_field_duplicate,
        adaptor_rename,
        adaptor_derive,
        adaptor_attributes,
        adaptor_attribute_for_all,
        adaptor_fields_with_values,
    )
)]
pub fn db_adaptor(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    adaptor::adaptor_inner(inp)
}

/// Генерирует реализацию трейта [`DbItem`] для структуры, привязанной к таблице PostgreSQL.
///
/// Макрос по полям структуры генерирует:
/// - Константы `TABLE`, `FIELDS`, `PRIMARY_KEYS`, `INSERT_FIELDS`, `UPDATE_FIELDS`,
///   `PRIMARY_KEY_INDICES`, `NO_UPDATE_INDICES`.
/// - Методы `bind_pkeys`, `bind_to_query`, `bind_to_query_insert`, `bind_limited_fields`,
///   `activate_fields`.
/// - `impl FromRow` для чтения строк результата запроса.
/// - `impl sqlx::Type` и `impl sqlx::decode::Decode` для использования в JOIN.
/// - Константы-поля `pub const field_name: &'static str` для типобезопасного
///   указания полей в `Select::with_fields([Entity::field_name])`.
///
/// Обязательные атрибуты:
/// - `#[item_table = "table_name"]` над структурой -- имя таблицы в БД.
///   Если не указан, используется snake_case имени структуры.
/// - `#[item_field_pkey]` над полем -- первичный ключ (можно несколько).
///
/// Дополнительные атрибуты:
/// - `#[db_field_name = "col_name"]` -- имя колонки в БД отличается от имени поля.
/// - `#[item_field_autogen]` -- поле генерируется БД (SERIAL/AUTOINCREMENT),
///   не вставляется при INSERT, но доступно для UPDATE.
/// - `#[item_field_autogen_always]` -- поле вычисляется БД (`GENERATED ALWAYS AS`),
///   не вставляется и не обновляется никогда.
/// - `#[item_aggr_insert]` -- генерирует UNNEST-версию `insert_vec`/`update_vec`,
///   эффективную для массовых вставок больших объёмов данных.
/// - `#[item_field_activate_with = "expr"]` -- выражение для инициализации поля
///   перед INSERT если оно равно дефолту (вместо `Default::default()`).
/// - `#[item_activate_all_with = "expr"]` -- то же, но применяется ко всем полям.
/// - `#[item_manually_activate_fields]` -- вместо авто-активации вызывается
///   `activate_fields_manually()`, которую нужно написать вручную.
/// - `#[item_field_require_from_row]` -- поле обязательно в `SELECT`, иначе ошибка
///   декодирования. Отключает `DbItemPartialSelect` для этой структуры.
/// - `#[item_skip_field_tolerance]` -- пропускает авто-генерацию `FieldTolerance`;
///   трейт нужно реализовать вручную с заполненным `TOLERATED`.
#[proc_macro_derive(
    DbItem,
    attributes(
        db_field_name,
        item_activate_all_with,
        item_aggr_insert,
        item_field_activate_with,
        item_field_autogen,
        item_field_autogen_always,
        item_field_pkey,
        item_manually_activate_fields,
        item_skip_field_tolerance,
        item_table,
        item_field_require_from_row,
    )
)]
pub fn db_item(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    item::item_inner(inp)
}

/// Генерирует реализацию трейта `DbItemExt`, предоставляющего вспомогательные
/// функции для работы с полями структуры в агностичном относительно типа виде.
///
/// Используется в сервисном слое для автоматизации проверок и обработки полей.
#[proc_macro_derive(DbItemExt)]
pub fn db_item_ext(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    item_ext::inner(inp)
}

/// Генерирует реализацию трейта `DbUpsert` для операции `INSERT ... ON CONFLICT DO UPDATE`.
///
/// Атрибут:
/// - `#[item_aggr_upsert]` -- использует UNNEST для массового upsert.
///   Более производительно в теории, но не работает для полей-массивов.
#[proc_macro_derive(DbUpsert, attributes(item_aggr_insert))]
pub fn db_item_upsert(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    item_upsert::inner(inp)
}

/// Генерирует структуру версионной сущности и конвертации между ней и оригиналом.
///
/// Создаёт `{Name}Version` -- копию структуры с дополнительным полем `pricing_version: i16`.
/// Первичный ключ версионной структуры: `id` + `pricing_version`.
///
/// Генерируются методы:
/// ```ignore
/// impl Original {
///     fn to_versioned(&self, pricing_version: i16) -> OriginalVersion;
/// }
/// impl OriginalVersion {
///     fn to_active(&self) -> Original;
/// }
/// ```
///
/// Версионная структура может содержать поля, которых нет в оригинале.
/// Оригинал не должен содержать полей, которых нет в версионной структуре.
///
/// Атрибут:
/// - `#[db_version_table = "table_name"]` -- имя версионной таблицы.
#[proc_macro_derive(DbVersioned, attributes(versioned, db_version_table))]
pub fn versioned(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    versioned::inner(inp)
}

/// Генерирует вспомогательные реализации для enum, представляющего
/// перечисление возможных значений колонки в БД.
///
/// Для указания варианта по умолчанию используется `#[db_default]` на нужном варианте.
/// Обязателен `#[repr(i16)]` (или другой числовой repr) для конвертации в число.
///
/// Генерируемые имплементации для `Enum`:
/// 1. [`Default`] со значением варианта, помеченного `#[db_default]`
/// 2. `From<Enum>` for `repr` (конвертация в число)
/// 3. `From<repr>` for `Enum` (конвертация из числа; неизвестные значения -> default)
/// 4. `From<Enum>` for `asez2_shared_db::Value`
/// 5. `EnumDiscriminant` -- таблица соответствия вариантов их числовым значениям
/// 6. `PgHasArrayType` для поддержки PostgreSQL-массивов
#[proc_macro_derive(DbEnum, attributes(db_default))]
pub fn db_enumeration(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enumeration::db_enumeration_inner(inp)
}
