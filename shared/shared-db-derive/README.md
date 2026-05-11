# shared-db-derive

Procedural macro crate that extends [`shared-db`](../shared-db/).

## Derive Macros

### `DbItem`

Maps a struct to a single database table. Generates insert, select, update, and delete logic via `sqlx`.

Required attributes:
- `#[item_table = "table_name"]` — table the struct maps to
- `#[item_field_pkey]` — marks the primary key field(s)

Optional field attributes:
- `#[db_field_name = "col"]` — override the column name
- `#[item_field_autogen]` — column managed by the DB (serial/autoincrement); excluded from inserts
- `#[item_field_autogen_always]` — `GENERATED ALWAYS AS` column; excluded from inserts and updates
- `#[item_field_activate_with = "fn"]` — transform this field before insert
- `#[item_field_require_from_row]` — field must be present when building from a DB row

Optional struct attributes:
- `#[item_aggr_insert]` — generate a bulk insert using `UNNEST` (efficient for large batches)
- `#[item_activate_all_with = "fn"]` — apply a transform to all fields before insert
- `#[item_manually_activate_fields]` — implement `activate_fields_manually` yourself
- `#[item_skip_field_tolerance]` — skip automatic `FieldTolerance` derivation

---

### `DbAdaptor`

Generates a "shadow" struct that mirrors a `DbItem` struct, useful for DTOs and inter-layer conversions.

Field attributes:
- `#[adaptor_type = "T"]` — override the field type; converted with `Into::into` by default
- `#[adaptor_into = "fn"]` — custom converter from adaptor field to original field
- `#[adaptor_from = "fn"]` — custom converter from original field to adaptor field
- `#[adaptor_try_from = "fn"]` — fallible custom converter
- `#[adaptor_field_duplicate = "new_name"]` — duplicate a field under an additional name
- `#[adaptor_rename = "name"]` — rename the adaptor struct or field

Struct attributes:
- `#[adaptor_derive(A, B)]` — add derives to the generated struct
- `#[adaptor_attributes(#[attr])]` — add attributes to the generated struct
- `#[adaptor_attribute_for_all(#[attr])]` — add attributes to all generated fields
- `#[adaptor_fields_with_values]` — implement `DbAdaptorFieldsWithValues`

---

### `DbItemExt`

Derives `DbItemExt`, a trait with helper functions for field-level inspection used in the processing service.

---

### `DbUpsert`

Derives `DbUpsert` for `INSERT … ON CONFLICT DO UPDATE`.

- `#[item_aggr_insert]` — use `UNNEST`-based bulk upsert

---

### `DbVersioned`

Generates a versioned copy of a struct with an additional `pricing_version: i16` primary key field. Creates `to_versioned()` and `to_active()` conversion helpers.

- `#[db_version_table = "table"]` — table for the versioned struct

---

### `DbEnum`

Derives database-compatible conversions for a `#[repr(i*)]` enum.

- `#[db_default]` on a variant — marks the default value
