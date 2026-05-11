# shared-db

PostgreSQL interaction layer for ASEZ 2.0 services, built on top of `sqlx`.

## `PgDbOptions`

Connection pool configuration. Build from environment variables or a JSON config file:

```rust
let opts = PgDbOptions::from_env()?;
let pool = opts.get_pool().await?;
```

| Method | Description |
|--------|-------------|
| `from_env()` | Build from `POSTGRES_*` env variables |
| `from_env_with_suffix(n)` | Same, but appends `n` to the database name |
| `open(path)` | Build from a JSON config file |
| `get_pool()` | Acquire a connection pool |
| `get_silent_pool()` | Acquire a lazy pool with statement logging disabled |
| `get_create_pool(create)` | Acquire a pool, optionally creating the database |
| `get_create_pool_tests(create)` | Acquire a pool for tests (prefixes db name with crate name) |

## Traits

| Trait | Description |
|-------|-------------|
| `DbItem` | Select, insert, update, delete for a single table row |
| `DbAdaptor` | DTO/shadow-struct conversion to/from a `DbItem` |

Both traits are derived via macros in [`shared-db-derive`](../shared-db-derive/).

## Macros

| Macro | Description |
|-------|-------------|
| `uuid!(str)` | Parse a `&str` into a `uuid::Uuid`, panicking on failure |
| `asez_date!(str)` | Parse a `&str` into `AsezDate` |
| `asez_timestamp!(str)` | Parse a `&str` into `AsezTimestamp` |

## Re-exports

- `ahash` — fast hash map
- `sqlx` — the underlying database driver
- `paste` — macro helper for identifier manipulation
- `Value`, `IntWithOriginal` — typed wrappers for database values

## Test Setup

`shared_db::test_setup` provides helpers for integration tests that need a live PostgreSQL database. Set `TEST_CFG_PATH` to the path of a JSON config file pointing at an empty test database.
