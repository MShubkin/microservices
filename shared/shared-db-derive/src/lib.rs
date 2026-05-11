mod adaptor;
mod enumeration;
mod item;
mod item_ext;
mod item_upsert;
mod shared;
mod versioned;

/// Allow a structure "shadow" structure  used for DTO/other logic to be made
/// from a different declared structure. The structure will have all the fields
/// of the original, and their types will, by default be the same. The whole purpose
/// of this macro is to be able to create a "shadow" structure that will be very
/// similar to the original, so the fewer attributes are used the closer we are
/// to success!
///
/// However it can have different types, different field names and it can derive
/// different traits and use their attributes.
///  This structure may have different field types to the original. This is not
/// obligatory, but it is useful.
/// - `adaptor_type`: as #[adaptor_type = "x"] If the type is different from the original
///    it will be converted by default with `Into::into`.
/// - `adaptor_into`: as #[adaptor_into = "function"] If types on item and adaptor
///   fields differ, this can declare a custom converter function. By default `Into::into` is
///   used. Adaptor field -> Original field.
/// - `adaptor_from`: as #[adaptor_from = "function"] If types on item and adaptor
///   fields differ, this can declare a custom converter function. By default `Into::into` is
///   used. Original field -> adaptor field.
/// - `adaptor_try_into`: as #[adaptor_try_into = "function"] declares a fallible function
///   that is compatible with `?` for converting between field types.
/// - `adaptor_try_from`: as #[adaptor_try_from = "function"]
/// - `adaptor_field_duplicate`: as #[adaptor_field_duplicate = "new_name"] will create a
///   second field that duplicates the first field. The content of the DbItem field will
///   also be copied into this field. However, these fields never play any role in database
///   operations.
/// - `adaptor_rename`: as #[adaptor_rename = "new_name"] Allows either the adaptor structure
///   or its fields to be renamed. The fields have the same name as their parent
///   fields by default, the new struct simply has the name of the old structure
///   with `Rep` appended to the end of it.
/// - `adaptor_derive`: as #[adaptor_derive(A, B, C)] Allows the adaptor to derive
///   additional traits that may be useful for it. Thus this becomes `#[derive(A, B, C)]`.
/// - `adaptor_attributes`: as #[adaptor_attributes(#[serde(default)])], allows the
///   adaptor to inherit attributes from other traits it derives. In this example, this
///   becomes `#[serde(default)]`
/// - `adaptor_attribute_for_all`: as `adaptor_attributes`, but passes the attributes to
///   ALL fields.
/// - `adaptor_fields_with_values`: implements DbAdaptorFieldsWithValues trait for adaptor entity
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

/// Allow a structure to be inserted, selected and updated in a database.
/// This derive macro has two necessary attributes:
/// - `item_table`, used as #[item_table = "table_name"] above the struct name.
///   a structure can only be tied to one table in this manner. The derivation fails
///   if this is not declared.
/// - `item_field_pkey`, used as #[item_field_pkey] above the field that is to be
///   the primary key. A structure can have multiple keys. This exists to be able to
///   facilitate both of the following patterns:
///
///   ```sql
///   CONSTRAINT some_table_pkey PRIMARY KEY (id_field)
///   ```
///
///   ```sql
///   CONSTRAINT some_table_pkey PRIMARY KEY (id_field, other_id_field)
///   ```
///
///   The derivation fails if this is `item_field_pkey` is not declared.
///
/// It also has four additional attributes:
/// - `db_field_name`, used as #[db_field_name = "new_name"] allows the DB field
///   name to be independent of the DbItem field name.
/// - `item_field_autogen`, used as #[item_field_autogen] above the field.
///   Fields marked thus represent serial and autoincrementing types whose
///   creation is always handled by the database. They are never inserted
///   into a table and can only updated or selected. If a field is not tagged
///   with `item_field_autogen`, it is inserted into the DB on an insert call.
/// - `item_field_autogen_always`, used as #[item_field_autogen_always] above the field.
///   Fields marked thus represent types which cannot be inserted or updated (eg GENERATED ALWAYS AS).
///   They are never inserted or updated and can only be selected. If a field is not tagged
///   with `item_field_autogen_always`, it can be updated by an update call. The primary key should
///   not be marked with this attribute, as it may make it impossible to update the item.
/// - `item_aggr_insert` compiles an alternative insert function that inserts all items
///   in a vectorised form. This is more efficient for large inserts as only one DB call
///   is made if there ar more than 65535 binds (between 1000-1000 items), but it is
///   slower for smaller inserts as all data is cloned into new vectors. Thus, if you
///   have a small structure which you expect will be inserted truly en masse, this
///   option should be used.
/// - `item_field_activate_with` isa bit unusual. Sometimes the type being
///   inserted into a database might be invalid, eg `Option::None` into a
///   `NOT NULL` column, or we may wish to initialize a date with `now()` instead of
///   the default value, or we otherwise require to modify a default value prior to inserting.
///   This attribute tells the compiler how to activate such fields. In a well
///   configured DB it should not be needed. By default, fields are not modified prior to
///   insertion.
/// - `item_manually_activate_fields`: Вместо `item_field_activate_with` и
///   `item_manually_activate_fields` можно написать свою `activate_fields_manually`
///    для всей структуры сразу. Это полезно если есть много очень специфичных правил.
/// - `item_activate_all_with` действует как `item_field_activate_with` но пишется
///   над структурой и действует для всех полей.
/// - `item_field_required`: При генерации `FromRow` в исходных данных должно
///   присутствовать данное поле (или все поля).
/// - item_skip_field_tolerance: Usually FieldTolerance is automatically derived and
///   does nothing. However, it may be desirable to manually derive the trait in order
///   to tolerate some renamed fields from the frontend. In that case this attribute
///   is applied and the trait MUST be derived manually.
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

/// Derives the `DbItemExt` trait, which is contains helper functions for working
/// with fields of a structure while agnostic to what the structure itself is.
///
/// It is useful for automated implementation of various checks and processes inside
/// the processing service.
#[proc_macro_derive(DbItemExt)]
pub fn db_item_ext(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    item_ext::inner(inp)
}

/// Derives the `DbUpsert` trait, which allows the "INSERT .. ON CONFLICT DO UPDATE"
/// statement.
///
/// У этого дерайва есть один аттрибут.
/// - `item_aggr_upsert`: Создаёт как UNNEST, что теоретический более производительно,
///   но не работает для записи у которых есть поля массивы.
#[proc_macro_derive(DbUpsert, attributes(item_aggr_insert))]
pub fn db_item_upsert(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    item_upsert::inner(inp)
}

/// Derives the `DbVersioned` Trait, which creates a struct that is identical, except it also
/// contains a field with the name of `pricing_version`. The primary key then becomes
/// `id` + `pricing_version`. "Version" is suffixed to the name of the original structure.
///
/// The new structure only has the bare minimum derive attributes to function as
/// a DbItem.
///
/// Additionally the macro generates a helper function to convert between
/// itself and the new _Version structure. The derive macro creates the function and reverse_function:
/// ```ignore
/// impl Original {
///     fn to_versioned(&self, pricing_version: i16) -> VersionedItem;
/// }
/// impl Versioned {
///     fn to_active(&self) -> Original;
/// }
/// ```
/// The versioned may contain fields that the original does not, however the original
/// may not contain fields that versioned does not. This is done in keeping with our
/// development process.
///
/// Attributes:
/// - `db_version_table`: Indicates which table is used for the version structure.
#[proc_macro_derive(DbVersioned, attributes(versioned, db_version_table))]
pub fn versioned(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    versioned::inner(inp)
}

/// Дерайв макрос для енама, который описывает перечень возможных значений колонки.
///
/// Для указания варианта енама по дефолту в базе данных, надо указать атрибутом #\[db_default\]
/// на вариант енама. Также важно добавить #\[repr(...)\], чтобы значение можно было преобразовывать свободно
/// в число
///
/// Генерируемые имплементации для `Enum`:
/// 1. [`Default`] со значением варианта, помеченного #\[db_default\]
/// 2. [`From<Enum>`] for `repr`
/// 3. [`From<repr>`] for `Enum`
/// 4. [`From<Enum>`] for [`asez2_shared_db::Value`]
///
#[proc_macro_derive(DbEnum, attributes(db_default))]
pub fn db_enumeration(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enumeration::db_enumeration_inner(inp)
}
