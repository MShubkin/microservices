//! Setup for more complex database tests.
use asez2_shared_db::db_item::Select;
use sqlx::migrate::Migrator;
use sqlx::PgPool;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use asez2_shared_db::{DbItem, PgDbOptions};

use crate::Plan;

pub trait TestSetupError:
    Error
    + From<std::io::Error>
    + From<sqlx::Error>
    + From<tokio::task::JoinError>
    + Send
    + 'static
{
}

#[derive(Debug, PartialEq)]
pub struct XError(String);

impl Error for XError {}
impl TestSetupError for XError {}

impl std::fmt::Display for XError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
macro_rules! xerror {
    ($t:ty) => {
        impl From<$t> for XError {
            fn from(x: $t) -> Self {
                Self(x.to_string())
            }
        }
    };
}
xerror!(sqlx::Error);
xerror!(std::io::Error);
xerror!(tokio::task::JoinError);
xerror!(sqlx::migrate::MigrateError);

/// Drop all existing processing tables.
pub async fn drop_tables<E>(
    pool: Arc<PgPool>,
    tables: &'static [&'static str],
) -> Result<(), E>
where
    E: TestSetupError,
{
    for table in tables {
        let q = format!("TRUNCATE TABLE IF EXiSTS {table};", table = table);
        sqlx::query(&q).execute(&*pool).await?;
    }
    Ok(())
}

pub async fn insert_null_data<E: TestSetupError>(_: Arc<PgPool>) -> Result<(), E> {
    Ok::<_, E>(())
}

pub async fn run_migs_for_file<E>(
    pool: Arc<PgPool>,
    mig_path: PathBuf,
) -> Result<(), E>
where
    E: TestSetupError,
{
    let commands = std::fs::read_to_string(&mig_path)?;

    'migration: for command in commands.split_inclusive(';') {
        // Some of the suggested commands in our migrations will not execute.
        // Usually after these start we can ignore the rest.
        // To save time we finish as soon as the comment migrations start.
        if command.contains("COMMENT ON") {
            break 'migration;
        } else if let Err(e) = sqlx::query(command).execute(&*pool).await {
            println!("{:?}\n{}\n {}", mig_path, command, e);
            return Err(e.into());
        }
    }
    Ok::<_, E>(())
}

/// Накатывание всех миграций параллельно без сохранения их порядка
pub async fn run_migrations_for_dir<E>(
    pool: Arc<PgPool>,
    mig_path: PathBuf,
) -> Result<(), E>
where
    E: TestSetupError,
{
    let read_dir = mig_path.read_dir().unwrap_or_else(|err| {
        panic!("Не удается прочитать директорию {mig_path:?}: {err}")
    });
    let mut handles = Vec::new();

    for e in read_dir {
        let e = e?;
        // Someone did something really stupid in the migrations, so now we have to compensate.
        if !e.metadata()?.is_file() {
            continue;
        }
        // Ignore non-sql files
        if !e.path().extension().map_or(false, |ext| ext == "sql") {
            continue;
        }
        // Sending to task speeds up the process, since we have a lot of commands
        // to send potentially.
        let pool = pool.clone();
        let handle = tokio::task::spawn(async move {
            let path = e.path();
            run_migs_for_file::<E>(pool, path).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

/// Накатывание всех миграций синхронно с сохранением порядка по их имени
pub async fn run_migrations_for_dir_sync<E>(
    pool: Arc<PgPool>,
    mig_path: PathBuf,
) -> Result<(), E>
where
    E: TestSetupError,
{
    let mut entries = mig_path
        .read_dir()
        .unwrap_or_else(|_| panic!("Директория {} не найдена", mig_path.display()))
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    entries.sort_by_key(|a| a.path());

    for e in entries {
        // Someone did something really stupid in the migrations, so now we have to compensate.
        if !e.metadata()?.is_file() {
            continue;
        }
        // Sending to task speeds up the process, since we have a lot of commands
        // to send potentially.
        let pool = pool.clone();
        let path = e.path();
        run_migs_for_file::<E>(pool, path).await?;
    }

    Ok(())
}

pub async fn run_migrations<E: TestSetupError>(
    pool: Arc<PgPool>,
    tables: &'static [&'static str],
) -> Result<(), E> {
    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    run_migrations_global(pool, root_dir, tables).await
}

pub async fn run_migrations_global<E: TestSetupError>(
    pool: Arc<PgPool>,
    root_dir: String,
    tables: &'static [&'static str],
) -> Result<(), E> {
    let mig_path = std::path::PathBuf::from(&root_dir).join("migrations");

    drop_tables::<E>(pool.clone(), tables).await?;

    let m = Migrator::new(mig_path).await.expect("Ошибка при накатывании миграций");
    m.run(&*pool).await.expect("Ошибка при накатывании миграций");

    println!("Main migrations successful.");
    let plans = Plan::select(&Select::default(), &*pool).await.unwrap();
    println!("{:?}", plans);
    Ok(())
}

/// This is an alternative to DBTests, where things are dropped manually.
/// This is because we need PgPool and multiple tables for these functions, while for
/// `asez2_shared_db::test_setup::run_db_test` we use Transaction.
async fn run_db_test_inner<'a, E, F, FutFn>(
    pool: Arc<PgPool>,
    run: FutFn,
) -> tokio::task::JoinHandle<Result<(), E>>
where
    E: TestSetupError,
    F: futures::Future<Output = ()> + 'a + Send,
    FutFn: FnOnce(Arc<PgPool>) -> F + Send + 'static,
{
    tokio::task::spawn(async move {
        run(Arc::clone(&pool)).await;
        Ok::<_, E>(())
    })
}

pub async fn run_db_test_full<E, F, FutFn, F2, FutFn2, F3, FutFn3>(
    extra_migrations: FutFn2,
    run_migrations: FutFn3,
    root_dir: String,
    drop_tables: &'static [&'static str],
    run: FutFn,
) where
    E: TestSetupError,
    F: futures::Future<Output = ()> + Send,
    F2: futures::Future<Output = Result<(), E>> + Send,
    F3: futures::Future<Output = Result<(), E>> + Send,
    FutFn: FnOnce(Arc<PgPool>) -> F + Send + 'static,
    FutFn2: FnOnce(Arc<PgPool>) -> F2 + Send + 'static,
    FutFn3:
        FnOnce(Arc<PgPool>, String, &'static [&'static str]) -> F3 + Send + 'static,
{
    let free_id = testing::id_pool::FreeId::new();
    let opt = PgDbOptions::from_env_with_suffix(free_id.id())
        .expect("Could not get pg options from .env.test");

    let pool = opt.get_create_pool(true).await.expect("Could not get pool.");
    let pool = Arc::new(pool);

    println!("Начало миграций в БД {} ...", opt.db_name());
    run_migrations(Arc::clone(&pool), root_dir, drop_tables).await.unwrap();
    extra_migrations(Arc::clone(&pool)).await.unwrap();
    println!("Все миграции успешно выполнены");

    println!("Начало теста...");
    let res = run_db_test_inner::<E, _, _>(Arc::clone(&pool), run).await.await;

    // This should solve certain errors when many tests are run.
    pool.close().await;
    drop(pool);

    match res {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(r) => {
            r.unwrap();
        }
    }
}

#[cfg(test)]
pub(crate) async fn run_db_test<F, FutFn>(run: FutFn)
where
    F: futures::Future<Output = ()> + Send,
    FutFn: FnOnce(Arc<PgPool>) -> F + Send + 'static,
{
    use crate::*;

    const PROCESSING_TABLES: &[&str] = &[
        ContractAmendment::TABLE,
        ContractAmendmentItem::TABLE,
        EcAgenda::TABLE,
        EcAgendaItem::TABLE,
        EsCommissionResult::TABLE,
        EcProtocol::TABLE,
        EcProtocolItem::TABLE,
        FieldChange::TABLE,
        Plan::TABLE,
        PlanItem::TABLE,
        PlanLegacy::TABLE,
        PlanItemLegacy::TABLE,
        RelAgendaProtocol::TABLE,
        RelAgendaProtocolItem::TABLE,
    ];

    testing::BaseMigPath::MigrationsHome
        .run_test_with_migrations_with_root(
            "../../processing",
            "",
            &[],
            PROCESSING_TABLES,
            run,
        )
        .await
}
