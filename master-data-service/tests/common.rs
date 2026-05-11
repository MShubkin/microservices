use std::sync::Arc;

use actix_http::Request;
use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::{
    dev::{Service, ServiceResponse},
    test,
    web::Data,
    App,
};
use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;
use once_cell::sync::OnceCell;
use shared_essential::presentation::dto::Source;
use tracing::Span;
use tracing_appender::non_blocking::WorkerGuard;

use master_data_service::application::master_data::load_master_data;
use master_data_service::infrastructure::web::config_url_handlers;
use master_data_service::infrastructure::{
    setup_rabbit_adapter, Env, GlobalConfig,
};
use trace_setup::TracingKind;

/// Используется для единичной инициализации логгера
static LOGGER_GUARD: OnceCell<(Vec<WorkerGuard>, Span)> = OnceCell::new();

pub async fn setup_test(
) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
    let env = Env::setup().unwrap();

    init_logger(&env.logger);

    let rabbit_adapter = Arc::new(
        setup_rabbit_adapter(&env.rabbit_config)
            .await
            .expect("cannot setup rabbit adapter"),
    );
    let db_pool = env
        .postgres_cfg
        .get_pool()
        .await
        .expect("cannot setup postgres adapter");
    let monolith_service = MonolithService::new(
        MonolithHttpDriver::basic_driver(env.monolith_cfg.url.clone())
            .expect("HTTP драйвер монолита"),
    );
    let global_config = Arc::new(GlobalConfig::new(
        env,
        rabbit_adapter.clone(),
        Arc::new(db_pool),
        monolith_service,
    ));

    /*
    sqlx::query("INSERT INTO sessions (user_id, token, time_expiry, time_live)
        VALUES
        ('199', '11110000-0000-1111-2222-000000000000', '2030-12-31 00:00:00.157', '2023-01-31 00:00:00.157')")
        .execute(&db_pool)
        .await
        .unwrap();
    */

    load_master_data(global_config.clone()).await.unwrap();

    test::init_service(
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Always))
            .configure(config_url_handlers)
            .wrap(http_middleware::default_cookie_decoder())
            .app_data(Data::from(rabbit_adapter))
            .app_data(Source::MasterData)
            .app_data(Data::new(global_config)),
    )
    .await
}

/// Единичная инициализация логгера
fn init_logger(tracing_kind: &TracingKind) {
    LOGGER_GUARD.get_or_init(|| {
        tracing_kind
            .initiate_log(
                "srm",
                "0.0.0.0",
                3100,
                &["master-data", "infra", "broker"],
            )
            .expect("Не удалось зарегистрировать логгер")
    });
}
