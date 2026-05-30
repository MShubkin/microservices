use std::{borrow::Cow, path::Path, sync::Arc};

use asez2_shared_db::{ahash::AHashMap, result::SharedDbError, PgDbOptions};
use sqlx::{
    migrate::{Migration, MigrationSource, MigrationType, Migrator},
    PgPool,
};
use tokio::sync::{Mutex, OnceCell};

use crate::id_pool::FreeId;

/// Возвращает пул для БД с суффиксом `suffix` (например, `dbname_3`).
///
/// Пулы кешируются в `POOLS` — повторный вызов с тем же суффиксом отдаёт
/// тот же `Arc<PgPool>` без нового подключения. Мьютекс нужен, потому что
/// несколько тестов могут одновременно запросить пул для нового суффикса.
pub(crate) async fn get_test_pool(
    suffix: u16,
) -> Result<Arc<PgPool>, SharedDbError> {
    static POOLS: OnceCell<Mutex<AHashMap<u16, Arc<PgPool>>>> =
        OnceCell::const_new();
    let mut pools = POOLS
        .get_or_init(|| async { Mutex::new(AHashMap::new()) })
        .await
        .lock()
        .await;
    if let Some(pool) = pools.get(&suffix).cloned() {
        Ok(pool)
    } else {
        let opts = PgDbOptions::from_env_with_suffix(suffix)?;
        let pool = Arc::new(opts.get_create_pool_tests(true).await?);
        pools.insert(suffix, pool.clone());
        Ok(pool)
    }
}

/// Сбрасывает схему и применяет миграции заново.
///
/// `DROP SCHEMA CASCADE` убивает все таблицы, индексы, типы и функции за один запрос.
/// Это быстрее, чем перебирать объекты по одному. После — чистый `CREATE SCHEMA public`
/// и прогон миграций из `source`.
pub(crate) async fn migrate_db<'s, M>(
    pool: &PgPool,
    source: M,
) -> Result<(), SharedDbError>
where
    M: MigrationSource<'s>,
{
    sqlx::query("DROP SCHEMA public CASCADE").execute(pool).await?;
    sqlx::query("CREATE SCHEMA public").execute(pool).await?;

    let migrator = sqlx::migrate::Migrator::new(source).await?;
    migrator.run(pool).await?;
    Ok(())
}

async fn count_rows(table_name: &str, pool: &PgPool) -> Result<i64, SharedDbError> {
    let (count,) = sqlx::query_as(&format!("select count(*) from {table_name}"))
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub(crate) async fn get_empty_tables(
    pool: &PgPool,
) -> Result<Vec<String>, SharedDbError> {
    const SQL: &str = "
        select pc.relname, pc.reltuples
            from pg_class pc
            join pg_namespace
            pn on pn.oid = pc.relnamespace
        where pc.relkind = 'r'
            and pn.nspname = 'public'";
    let all_names: Vec<(String,)> = sqlx::query_as(SQL).fetch_all(pool).await?;

    let mut empty_tables = vec![];
    for (name,) in all_names {
        if count_rows(&name, pool).await? == 0 {
            empty_tables.push(name)
        }
    }

    Ok(empty_tables)
}

pub(crate) async fn truncate_tables<I, S>(
    tables: I,
    pool: &PgPool,
) -> Result<(), SharedDbError>
where
    I: IntoIterator<Item = S>,
    S: std::fmt::Display,
{
    use itertools::Itertools;
    sqlx::query(&format!(
        "truncate {} restart identity",
        tables.into_iter().join(",")
    ))
    .execute(pool)
    .await?;
    Ok(())
}

/// Применяет произвольный SQL-скрипт как временную миграцию, затем удаляет запись о ней.
///
/// Хак с `_sqlx_migrations`: sqlx требует уникальный версионный номер для каждой миграции.
/// Используем `99999999000000` — заведомо больше любого реального номера, чтобы не конфликтовать.
/// После применения сразу удаляем запись, чтобы тот же скрипт можно было применить ещё раз
/// в другом тесте.
pub(crate) async fn apply_sql<S: Into<Cow<'static, str>>>(
    pool: &PgPool,
    sql: S,
) -> Result<(), SharedDbError> {
    let version: i64 = 99999999000000;
    let migrations =
        vec![Migration::new(version, "".into(), MigrationType::Simple, sql.into())];
    let migrator = Migrator {
        migrations: migrations.into(),
        ignore_missing: true,
    };
    // TODO: the following two statements should run in a transaction...
    migrator.run(pool).await?;
    sqlx::query("delete from _sqlx_migrations where version = $1;")
        .bind(version)
        .execute(pool)
        .await?;
    Ok(())
}

/// Подготавливает БД для теста: при первом использовании применяет миграции,
/// при повторном — только TRUNCATE таблиц, которые были пустыми после миграций.
///
/// Логика `is_new()`: если `FreeId` взят из пула (уже использовался), значит
/// миграции уже применены. Нужно только очистить данные предыдущего теста.
/// Список "пустых после миграций таблиц" хранится отдельно, чтобы не TRUNCATE-ить
/// таблицы со справочниками, которые мигрировались с данными.
pub(crate) async fn prepare_for_test<'s, M>(
    migrations: Option<M>,
) -> Result<(Arc<PgPool>, FreeId), SharedDbError>
where
    M: MigrationSource<'s>,
{
    let id = FreeId::new();
    let pool = get_test_pool(id.id()).await?;

    if let Some(migrations) = migrations {
        if id.is_new() {
            migrate_db(&pool, migrations).await?;
            let empty_tables = get_empty_tables(&pool).await?;
            empty_tables::set(id.id(), empty_tables).await;
        } else {
            let tables = empty_tables::get(id.id()).await;
            truncate_tables(&tables, &pool).await?;
        }
    }

    Ok((pool, id))
}

pub(crate) async fn prepare_for_test_fixture_files<'s, M, P>(
    migrations: Option<M>,
    fixtures_dir: P,
    fixtures: &[&'static str],
) -> Result<(Arc<PgPool>, FreeId), SharedDbError>
where
    M: MigrationSource<'s>,
    P: AsRef<Path>,
{
    let id = FreeId::new();
    let pool = get_test_pool(id.id()).await?;

    if let Some(migrations) = migrations {
        if id.is_new() {
            migrate_db(&pool, migrations).await?;
            let empty_tables = get_empty_tables(&pool).await?;
            empty_tables::set(id.id(), empty_tables).await;
        } else {
            let tables = empty_tables::get(id.id()).await;
            truncate_tables(&tables, &pool).await?;
        }
    }

    let fixtures_dir = fixtures_dir.as_ref();
    for fixture in fixtures {
        let fixture =
            tokio::fs::read_to_string(&fixtures_dir.join(fixture)).await?;
        apply_sql(&pool, fixture).await?;
    }

    Ok((pool, id))
}

/// Реестр "изначально пустых" таблиц по суффиксу БД.
///
/// Хранит список таблиц, которые были пустыми сразу после миграций.
/// Только они TRUNCATE-ируются между тестами — таблицы со справочными данными
/// из миграций остаются нетронутыми.
mod empty_tables {
    use asez2_shared_db::ahash::AHashMap;
    use tokio::sync::{OnceCell, RwLock};

    static EMPTY_TABLES: OnceCell<RwLock<AHashMap<u16, Vec<String>>>> =
        OnceCell::const_new();

    async fn empty_tables() -> &'static RwLock<AHashMap<u16, Vec<String>>> {
        EMPTY_TABLES
            .get_or_init(|| async { RwLock::new(AHashMap::new()) })
            .await
    }

    pub(super) async fn set(id: u16, tables: Vec<String>) {
        empty_tables().await.write().await.insert(id, tables);
    }

    pub(super) async fn get(id: u16) -> Vec<String> {
        empty_tables().await.read().await.get(&id).cloned().unwrap_or_default()
    }
}
#[cfg(test)]
mod tests {
    use std::{path::Path, time::Instant};

    use crate::id_pool::FreeId;

    #[tokio::test]
    async fn migrate_db() -> anyhow::Result<()> {
        let id = FreeId::new();
        let pool = super::get_test_pool(id.id()).await?;

        let res =
            super::migrate_db(&pool, Path::new("../../view-storage/migrations"))
                .await;
        assert!(res.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn get_empty_tables() -> anyhow::Result<()> {
        let id = FreeId::new();
        let pool = super::get_test_pool(id.id()).await?;
        super::migrate_db(&pool, Path::new("../../view-storage/migrations"))
            .await?;

        sqlx::query("insert into users (id, login) values (666, 'guzdn')")
            .execute(&*pool)
            .await?;

        let empty_tables =
            super::get_empty_tables(&pool).await.expect("get_empty_tables");
        assert_eq!(empty_tables.len(), 10);
        assert!(!empty_tables.iter().any(|t| t == "users"));

        Ok(())
    }

    #[tokio::test]
    async fn apply_fixture() -> anyhow::Result<()> {
        const FIXTURE: &str = r#"
            drop table if exists apply_fixture_test_table;
            create table apply_fixture_test_table
            (
                id serial primary key,
                text varchar not null
            );

            insert into apply_fixture_test_table (text) values ('one'), ('two'), ('three');
        "#;

        let id = FreeId::new();
        let pool = super::get_test_pool(id.id()).await?;

        let res = super::apply_sql(&pool, FIXTURE).await;
        assert!(res.is_ok(), "{res:?}");

        let (count,): (i64,) =
            sqlx::query_as("select count(*) from apply_fixture_test_table;")
                .fetch_one(&*pool)
                .await?;
        assert_eq!(count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn apply_other_fixture() -> anyhow::Result<()> {
        const FIXTURE1: &str = r#""#;
        const FIXTURE2: &str = r#"
            drop table if exists apply_fixture_test_table;
            create table apply_fixture_test_table
            (
                id serial primary key,
                text varchar not null
            );

            insert into apply_fixture_test_table (text) values ('one'), ('two'), ('three');
        "#;

        let id = FreeId::new();
        let pool = super::get_test_pool(id.id()).await?;
        super::apply_sql(&pool, FIXTURE1).await?;

        let res = super::apply_sql(&pool, FIXTURE2).await;
        assert!(res.is_ok(), "{res:?}");

        let (count,): (i64,) =
            sqlx::query_as("select count(*) from apply_fixture_test_table;")
                .fetch_one(&*pool)
                .await?;
        assert_eq!(count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn apply_sql_duration() -> anyhow::Result<()> {
        let id = FreeId::new();
        let pool = super::get_test_pool(id.id()).await?;
        for file in [
            "single.sql",
            "multiple.sql",
            "empty.sql",
            "huge.sql",
            "no-trailing-semicolon.sql",
            "semicolon-in-string.sql",
        ] {
            let path = Path::new("tests/files").join(file);
            let sql = tokio::fs::read_to_string(path).await?;
            let start = Instant::now();
            let res = super::apply_sql(&pool, sql).await;
            let dur = Instant::now() - start;
            assert!(res.is_ok(), "{res:?}");
            println!("migrated {file:?} in {dur:?}");
        }
        Ok(())
    }
}
