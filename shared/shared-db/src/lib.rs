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
/// These re-exports improve the work of macros.
pub use paste;
pub use sqlx;
pub use value::{IntWithOriginal, Value};

/// This is a builder that allows a config file to be used when launching the
/// service and to thus set basic parameters of the DB.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PgDbOptions {
    host: String,
    port: u16,
    db_name: String,
    user: String,
    pw: String,
    min_connections: u32,
    max_connections: u32,
    timeout_s: u64,
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
    /// New instance from filepath.
    pub fn open<P: AsRef<Path>>(f: P) -> Result<Self> {
        let db_options = std::fs::read_to_string(f)?;

        serde_json::from_str::<Self>(&db_options).map_err(Into::into)
    }

    /// Creates options from environment variables.
    pub fn from_env() -> Result<Self> {
        Ok(env_setup::PostgresCfg::from_env()?.into())
    }

    /// Creates options from environment, appending suffix to the db name.
    pub fn from_env_with_suffix(suffix: u16) -> Result<Self> {
        let mut cfg = PgDbOptions::from(env_setup::PostgresCfg::from_env()?);
        cfg.db_name += &suffix.to_string();
        Ok(cfg)
    }

    /// Returns Postgress connection pool.
    pub async fn get_pool(&self) -> Result<PgPool> {
        self.get_create_pool(false).await
    }

    /// Returns Postgress connection pool which does not log connections.
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

    /// Returns a Postgress connection pool, creating a new DB if `create` is
    /// true and the specified DB does not exist.
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

    /// Returns a Postgress connection pool, creating a new DB if `create` is
    /// true and the specified DB does not exist.
    pub async fn get_create_pool(&self, create: bool) -> Result<PgPool> {
        self.get_create_pool_inner(&self.db_name, create).await
    }

    /// Returns a Postgress connection pool, creating a new DB if `create` is
    /// true and the specified DB does not exist.
    /// Функция для тестов которая переименует базу.
    pub async fn get_create_pool_tests(&self, create: bool) -> Result<PgPool> {
        self.get_create_pool_inner(&self.test_db_name(), create).await
    }

    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    pub fn test_db_name(&self) -> String {
        let name = std::env::var("CARGO_PKG_NAME")
            .unwrap_or_else(|_| "unknown".to_string());
        format!("{name}_{}", self.db_name)
    }
}

/// Uuid from &str
#[macro_export]
macro_rules! uuid {
    ($uuid_str:expr) => {
        $uuid_str.parse::<uuid::Uuid>().expect("unparsable uuid")
    };
}

/// AsezDate from &str
#[macro_export]
macro_rules! asez_date {
    ($date_str:expr) => {
        $crate::db_item::date_time::AsezDate::try_from($date_str)
            .expect("date should be parsable")
    };
}

/// AsezDate from &str
#[macro_export]
macro_rules! asez_timestamp {
    ($date_str:expr) => {
        $crate::db_item::date_time::AsezTimestamp::try_from($date_str)
            .expect("date should be parsable")
    };
}

/// This test module contains the necessary setup steps to connect to a test database
/// and generate the initial load.
///
/// All the tests that use a DB _MUST_ provide a path ("TEST_CFG_PATH") to a config file
/// and a correctly configured empty database.
pub mod test_setup;
