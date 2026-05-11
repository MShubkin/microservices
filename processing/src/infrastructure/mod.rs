//! This module contains the infrastructure including the configuration and rabbit setup.
//! TODO: Standardize configuration to perhaps use .env for rabbit setup.
//! This, however, comes at a slightly later stage.
use crate::common::{RabbitConfig, Result};

use env_setup::MonolithCfg;
use serde::Deserialize;
use trace_setup::{TracingKind, TracingSetupResult};

pub(crate) mod rabbit;
pub(crate) use rabbit::QueueSpec;

mod http;
pub(crate) use http::start_http_server;
use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;

/// We need to describe both the broker address and
/// the address of the plan-db server.
/// For good measure we also add a DB config.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProcessingConfig {
    #[allow(dead_code)]
    pub(crate) host: String,
    #[allow(dead_code)]
    pub(crate) port: u16,
    pub(crate) rabbit_cfg: RabbitConfig,
    pub(crate) db_config: asez2_shared_db::PgDbOptions,
    #[serde(default)]
    pub(crate) tracing: TracingKind,
    /// This determines how many worker threads the server
    /// runtime spawns on startup. May be useful.
    pub(crate) server_thread_count: u8,
    pub(crate) monolith_cfg: MonolithCfg,
}

impl ProcessingConfig {
    #[cfg(feature = "integration_tests")]
    #[allow(unused)]
    pub(crate) fn from_file(path: &str) -> Result<Self> {
        let file = std::fs::read_to_string(path)?;
        serde_json::from_str::<Self>(&file).map_err(Into::into)
    }

    pub(crate) fn from_env() -> Result<Self> {
        use env_setup::*;

        let port = try_get!(SRV_PORT, SRV_PORT_DEFAULT_VALUE, u16)
            .map_err(EnvError::from)?;
        let host =
            var(SRV_HOST).unwrap_or_else(|_| SRV_HOST_DEFAULT_VALUE.to_owned());

        Ok(ProcessingConfig {
            host,
            port,
            rabbit_cfg: RabbitCfg::from_env()?.into(),
            db_config: PostgresCfg::from_env()?.into(),
            tracing: TracingCfg::from_env()?.tracing_kind,
            server_thread_count: env_setup::try_get!(SRV_THREAD_COUNT, "4", u8)
                .expect("4"),
            monolith_cfg: MonolithCfg::from_env()?,
        })
    }

    #[tracing::instrument(skip_all)]
    pub(crate) async fn launch(self, queues: Vec<QueueSpec>) -> Result<()> {
        let db_pool = self.db_config.get_pool().await.map_err(|e| {
            tracing::error!(
                kind = "infra",
                "Processing: Error connecting DB: {}",
                e
            );
            e
        })?;

        let monolith_driver =
            MonolithHttpDriver::basic_driver(self.monolith_cfg.url.clone())?;
        let monolith_service = MonolithService::new(monolith_driver);

        self.rabbit_cfg.dig_forever(queues, db_pool, monolith_service).await
    }

    pub(crate) fn initiate_log(&self) -> TracingSetupResult {
        self.tracing
            .initiate_log(
                "srm",
                &self.host,
                self.port,
                &[
                    "broker",
                    "infra",
                    "legacy_request",
                    "request",
                    "get",
                    "insert",
                    "update",
                    "siem",
                ],
            )
            .map_err(Into::into)
    }
}
