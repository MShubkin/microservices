use asez2_shared_db::{result::SharedDbError, PgDbOptions};
use env_setup::PostgresCfg;
use sqlx::PgPool;

/// Creates a Postgres DB connection basing on the confiruration provided by `config`.
pub async fn setup_postgres_connection(
    config: &PostgresCfg,
) -> Result<PgPool, SharedDbError> {
    PgDbOptions::from(config.clone()).get_pool().await
}
