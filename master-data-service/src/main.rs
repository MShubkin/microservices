use std::sync::Arc;

use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;
use tracing::info;

use crate::{
    application::master_data::load_master_data,
    infrastructure::{
        rabbit::{declare_queues, start_rabbit_listener},
        setup_rabbit_adapter,
        web::start_http_server,
        Env, GlobalConfig,
    },
};

mod application;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::setup()?;

    let _logger_guard = env.logger.initiate_log(
        "srm",
        &env.url.0,
        env.url.1,
        &["infra", "broker", "master_data", "siem"],
    )?;

    info!(kind = "infra", "Setup postgres pool");
    let db_pool = Arc::new(env.setup_postgres_pool().await?);

    info!(kind = "infra", "Setup rabbit adapter");
    let rabbit_adapter = Arc::new(setup_rabbit_adapter(&env.rabbit_config).await?);
    declare_queues(&rabbit_adapter).await?;

    info!(kind = "infra", "Setup monolith service");
    let monolith_service = MonolithService::new(
        MonolithHttpDriver::basic_driver(env.monolith_cfg.url.clone())
            .expect("HTTP драйвер монолита"),
    );

    let global_config =
        Arc::new(GlobalConfig::new(env, rabbit_adapter, db_pool, monolith_service));

    info!(kind = "infra", "Load master data im memory");
    load_master_data(global_config.clone()).await?;

    info!(kind = "infra", "Register consumer");
    start_rabbit_listener(global_config.clone()).await?;

    info!(kind = "infra", "Start Http Server....");
    start_http_server(global_config.clone()).await?;

    Ok(())
}
