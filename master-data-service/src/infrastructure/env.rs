use sqlx::PgPool;

use asez2_shared_db::{result::SharedDbError, PgDbOptions};
use env_setup::{
    try_get, var, EnvError, MonolithCfg, PostgresCfg, RabbitCfg, TracingCfg,
    SRV_HOST, SRV_HOST_DEFAULT_VALUE, SRV_PORT, SRV_PORT_DEFAULT_VALUE,
};
use trace_setup::TracingKind;

#[derive(Clone)]
pub struct Env {
    pub url: (String, u16),
    pub rabbit_config: RabbitCfg,
    pub postgres_cfg: PgDbOptions,
    pub monolith_cfg: MonolithCfg,
    pub logger: TracingKind,
}

impl Env {
    pub fn setup() -> Result<Self, EnvError> {
        let port = try_get!(SRV_PORT, SRV_PORT_DEFAULT_VALUE, u16)?;
        let host =
            var(SRV_HOST).unwrap_or_else(|_| SRV_HOST_DEFAULT_VALUE.to_owned());

        let rabbit_config = RabbitCfg::from_env()?;
        let postgres_cfg = PostgresCfg::from_env()?.into();
        let monolith_cfg = MonolithCfg::from_env()?;

        let logger = TracingCfg::from_env()?.tracing_kind;

        Ok(Self {
            url: (host, port),
            rabbit_config,
            postgres_cfg,
            monolith_cfg,
            logger,
        })
    }

    pub async fn setup_postgres_pool(&self) -> Result<PgPool, SharedDbError> {
        self.postgres_cfg.get_create_pool(true).await
    }
}
