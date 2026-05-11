use sqlx::migrate::Migrator;
use sqlx::postgres::PgRow;
use sqlx::{Executor, PgPool, Row};
use std::path::{Path, PathBuf};

const TABLE_EXISTS_SQL: &str = "SELECT EXISTS( \
   SELECT FROM information_schema.tables \
   WHERE table_catalog = $1 AND table_name = $2 \
)";

/// Переменная окружения, управляющая "накатом" скрипта `BASELINE_SQL`
pub const APPLY_BASELINE: &str = "APPLY_BASELINE";
/// Переменная окружения, управляющая "накатом" миграций
pub const APPLY_MIGRATIONS: &str = "APPLY_MIGRATIONS";

const BASELINE_SQL: &str = "baseline.sql";

const SQLX_MIGRATION_METADATA_TABLE: &str = "_sqlx_migrations";

const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";

/// "Накатываем" миграции на базу. База уже должна существовать (т.е. выполнено `CREATE DATABASE ...`).
/// Скрипты "наката" берутся из директории миграции (обычно `<project_root>/migrations`) и должны иметь суффикс `.up.sql`
/// Поддиректории игнорируются.
/// В результате миграции будет создана (или обновлена) таблица `_sqlx_migrations`
pub async fn run_migration(
    pool: &PgPool,
    migrations_directory: PathBuf,
) -> Result<(), sqlx::Error> {
    let migrator = Migrator::new(migrations_directory).await?;
    migrator.run(pool).await?;
    Ok(())
}

/// "Откатываем" миграции до заданной версии.
/// Скрипты "наката" берутся из директории миграции (обычно `<project_root>/migrations`) и должны иметь суффикс `.down.sql`
/// Нужную версию можно узнать из таблицы `_sqlx_migrations`
pub async fn undo_migrations(
    pool: &PgPool,
    migrations_directory: PathBuf,
    target_version: i64,
) -> Result<(), sqlx::Error> {
    let migrator = Migrator::new(migrations_directory).await?;
    migrator.undo(pool, target_version).await?;
    Ok(())
}

/// Выполняем проверку и создание, в случае необходимости, предварительно заполненной таблицы
/// миграции `_sqlx_migrations` с данными.
/// Проверяем существование таблицы `_sqlx_migrations` и наличие скрипта `baseline.sql`
/// в директории миграции (обычно `<project_root>/migrations`)
pub async fn process_baseline(
    pool: &PgPool,
    migrations_directory: &Path,
    schema_name: &str,
) -> Result<(), sqlx::Error> {
    let baseline = migrations_directory.join(BASELINE_SQL);
    if baseline.exists()
        && !check_table_exists(pool, schema_name, SQLX_MIGRATION_METADATA_TABLE)
            .await?
    {
        run_baseline(pool, baseline).await?;
    }
    Ok(())
}

/// Накатываем `baseline` скрипт - предварительно созданная таблица `_sqlx_migrations` с данными
/// Скрипт `baseline.sql` ищется в "корне" директории миграции (обычно `<project_root>/migrations`)
async fn run_baseline(pool: &PgPool, baseline: PathBuf) -> Result<(), sqlx::Error> {
    let sql_script = std::fs::read_to_string(baseline)?;
    pool.execute(sql_script.as_str()).await?;
    Ok(())
}

/// Проверка существования таблицы в базе.
pub async fn check_table_exists(
    pool: &PgPool,
    schema_name: &str,
    table_name: &str,
) -> Result<bool, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query(TABLE_EXISTS_SQL)
        .bind(schema_name)
        .bind(table_name)
        .try_map(|row: PgRow| row.try_get::<bool, _>(0))
        .fetch_one(&mut conn)
        .await
}

/// Конструирует путь "по-умолчанию" к папке с миграциями (`<project_root>/migrations`)
pub fn default_migration_path() -> PathBuf {
    let project_root_path = get_app_dir();
    Path::new(&project_root_path).join("migrations")
}

/// Конструирует путь "по-умолчанию" к папке с `UP` миграциями (`<project_root>/migrations/up`)
pub fn default_migration_up_path() -> PathBuf {
    default_migration_path().join("up")
}

/// Конструирует путь "по-умолчанию" к папке с `DOWN` миграциями (`<project_root>/migrations/down`)
pub fn default_migration_down_path() -> PathBuf {
    default_migration_path().join("down")
}

/// Получаем текущую директорию приложения так, чтобы это было удобно при разработке и в автономном окружении.
/// Если запускаем в окружении разработки - текущей считается директория (под-)проекта,
/// если запускаем в "боевом" ("тестовом") окружении - берем текущую директорию.
fn get_app_dir() -> String {
    std::env::var(CARGO_MANIFEST_DIR)
        .or_else(|_| std::env::current_dir().map(|dir| dir.display().to_string()))
        .unwrap()
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use asez2_shared_db::PgDbOptions;
    use env_setup::PostgresCfg;
    use sqlx::migrate::MigrateDatabase;
    use sqlx::migrate::Migrator;
    use sqlx::{PgPool, Postgres};

    /// Проверка миграции. Запускается на "чистом" инстансе постгреса.
    /// Базу можно не создавать, она будет создана "в процессе" теста.
    /// Изначально создавалось для модуля `НСИ` (`master-data-service`).
    /// Чтобы ничего не сломать тест игнорируется.
    #[actix_web::test]
    #[ignore]
    pub async fn test_migration() {
        let postgres_cfg = PostgresCfg::from_env().unwrap();
        let postgres: PgDbOptions = postgres_cfg.clone().into();
        dbg!(&postgres);

        let project_root_path = get_app_dir();
        let migrations_path = std::path::Path::new(&project_root_path)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("master-data-service")
            .join("migrations");
        let up_migrations = migrations_path.join("up");

        dbg!(&migrations_path);
        assert!(migrations_path.exists());

        let migrator = Migrator::new(up_migrations).await.unwrap();

        let pool: PgPool = postgres.get_create_pool(true).await.unwrap();
        assert!(Postgres::database_exists(&postgres_cfg.get_connection_string())
            .await
            .unwrap());

        migrator.run(&pool).await.unwrap();

        assert!(check_table_exists(
            &pool,
            &postgres_cfg.db_name,
            "status_sample_conclusion"
        )
        .await
        .unwrap());

        migrator.undo(&pool, 0).await.unwrap();

        assert!(!check_table_exists(
            &pool,
            &postgres_cfg.db_name,
            "status_sample_conclusion"
        )
        .await
        .unwrap());
    }
}
