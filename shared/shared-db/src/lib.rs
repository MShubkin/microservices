use crate::result::Result;

pub use ahash;
use serde::Deserialize;
use sqlx::{
    migrate::MigrateDatabase,
    postgres::{PgPool, PgPoolOptions},
    Postgres,
};
use std::path::Path;

pub mod db_item;
pub mod result;
pub mod value;

pub use db_item::{DbAdaptor, DbItem};
/// Реэкспорт нужен макросам из shared-db-derive -- они генерируют код,
/// ссылающийся на `asez2_shared_db::paste`, поэтому крейт должен быть виден.
pub use paste;
pub use sqlx;
pub use value::{IntWithOriginal, Value};

/// Настройки подключения к PostgreSQL.
///
/// Читается из JSON-конфига или переменных окружения через [`env_setup::PostgresCfg`].
/// Хранит все параметры пула -- лимиты соединений, таймауты -- чтобы не таскать их
/// по всему коду россыпью.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PgDbOptions {
    host: String,
    port: u16,
    db_name: String,
    user: String,
    pw: String,
    min_connections: u32,
    max_connections: u32,
    /// Таймаут установки соединения в секундах.
    timeout_s: u64,
    /// Максимальный возраст соединения перед переподключением (сек).
    connection_refresh_s: u64,
}

impl From<env_setup::PostgresCfg> for PgDbOptions {
    fn from(o: env_setup::PostgresCfg) -> Self {
        Self {
            host: o.host,
            port: o.port,
            db_name: o.db_name,
            user: o.user,
            pw: o.pw,
            min_connections: o.min_connections,
            max_connections: o.max_connections,
            timeout_s: o.conn_timeout_s as u64,
            connection_refresh_s: o.conn_refresh_interval_s,
        }
    }
}

impl PgDbOptions {
    /// Читает JSON-файл с настройками по пути `f`.
    pub fn open<P: AsRef<Path>>(f: P) -> Result<Self> {
        let db_options = std::fs::read_to_string(f)?;

        serde_json::from_str::<Self>(&db_options).map_err(Into::into)
    }

    /// Читает настройки из переменных окружения через `env_setup`.
    pub fn from_env() -> Result<Self> {
        Ok(env_setup::PostgresCfg::from_env()?.into())
    }

    /// Читает настройки из окружения и добавляет числовой суффикс к имени базы.
    ///
    /// Нужно для параллельных интеграционных тестов: каждый тест получает
    /// свою изолированную БД вида `mydb42`, чтобы они не мешали друг другу.
    pub fn from_env_with_suffix(suffix: u16) -> Result<Self> {
        let mut cfg = PgDbOptions::from(env_setup::PostgresCfg::from_env()?);
        cfg.db_name += &suffix.to_string();
        Ok(cfg)
    }

    /// Возвращает пул соединений к уже существующей БД.
    pub async fn get_pool(&self) -> Result<PgPool> {
        self.get_create_pool(false).await
    }

    /// Возвращает "тихий" пул без логирования запросов.
    ///
    /// Полезен там, где логи sqlx засоряют вывод или нагружают систему
    /// при высокой частоте соединений (например, health-check).
    /// Создаётся ленивое подключение (`connect_lazy`), так что соединение
    /// устанавливается только при первом реальном запросе.
    pub fn get_silent_pool(&self) -> PgPool {
        use sqlx::postgres::PgConnectOptions;
        use sqlx::ConnectOptions;

        let pool_options = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(self.max_connections)
            .connect_timeout(core::time::Duration::new(self.timeout_s, 0))
            .max_lifetime(core::time::Duration::new(self.connection_refresh_s, 0));

        let options = PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user)
            .password(&self.pw)
            .database(&self.db_name)
            .disable_statement_logging()
            .to_owned();
        pool_options.connect_lazy_with(options)
    }

    /// Внутренняя реализация: строит URL подключения и открывает пул.
    /// Если `create == true` и БД не существует -- создаёт её через `CREATE DATABASE`.
    async fn get_create_pool_inner(
        &self,
        db_name: &str,
        create: bool,
    ) -> Result<PgPool> {
        let url = format!(
            "postgres://{usr}:{pw}@{host}:{port}/{db}",
            usr = self.user,
            pw = self.pw,
            host = self.host,
            port = self.port,
            db = db_name,
        );

        if create && !Postgres::database_exists(&url).await? {
            Postgres::create_database(&url).await?;
        }

        PgPoolOptions::new()
            .min_connections(self.min_connections)
            .max_connections(self.max_connections)
            .connect_timeout(core::time::Duration::new(self.timeout_s, 0))
            .max_lifetime(core::time::Duration::new(self.connection_refresh_s, 0))
            .connect(&url)
            .await
            .map_err(Into::into)
    }

    /// Возвращает пул; при `create == true` создаёт БД если её ещё нет.
    pub async fn get_create_pool(&self, create: bool) -> Result<PgPool> {
        self.get_create_pool_inner(&self.db_name, create).await
    }

    /// То же, что [`get_create_pool`], но подключается к тестовой БД
    /// (`CARGO_PKG_NAME_<db_name>`).
    ///
    /// Позволяет тестам не трогать продуктовую базу.
    pub async fn get_create_pool_tests(&self, create: bool) -> Result<PgPool> {
        self.get_create_pool_inner(&self.test_db_name(), create).await
    }

    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// Возвращает имя тестовой БД в формате `{CARGO_PKG_NAME}_{db_name}`.
    ///
    /// Если переменная `CARGO_PKG_NAME` не установлена -- подставляет `"unknown"`.
    pub fn test_db_name(&self) -> String {
        let name = std::env::var("CARGO_PKG_NAME")
            .unwrap_or_else(|_| "unknown".to_string());
        format!("{name}_{}", self.db_name)
    }
}

/// Парсит строку в [`uuid::Uuid`] с паникой при ошибке.
///
/// Используется там, где UUID задан жёсткой константой в коде и невалидная
/// строка -- это баг, а не рантайм-ошибка.
#[macro_export]
macro_rules! uuid {
    ($uuid_str:expr) => {
        $uuid_str.parse::<uuid::Uuid>().expect("unparsable uuid")
    };
}

/// Парсит строку в [`AsezDate`] с паникой при ошибке.
#[macro_export]
macro_rules! asez_date {
    ($date_str:expr) => {
        $crate::db_item::date_time::AsezDate::try_from($date_str)
            .expect("date should be parsable")
    };
}

/// Парсит строку в [`AsezTimestamp`] с паникой при ошибке.
#[macro_export]
macro_rules! asez_timestamp {
    ($date_str:expr) => {
        $crate::db_item::date_time::AsezTimestamp::try_from($date_str)
            .expect("date should be parsable")
    };
}

/// Инфраструктура для интеграционных тестов, работающих с БД.
///
/// Все тесты, обращающиеся к базе данных, обязаны:
/// - задать путь к конфигу через переменную окружения `TEST_CFG_PATH`;
/// - работать с пустой базой, настроенной по этому конфигу.
pub mod test_setup;
