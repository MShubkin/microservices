use actix_cors::Cors;
use actix_web::middleware::{Logger, NormalizePath, TrailingSlash};
use actix_web::web::Data;
use actix_web::{
    web::{self, Json},
    App, HttpServer,
};
use igg_tracing::ServiceRootSpanBuilder;
use shared_essential::presentation::dto::Source;
use std::sync::Arc;
use tracing::info;
use tracing_actix_web::TracingLogger;

use http_middleware::AsezSessionWatcher;
use shared_essential::infrastructure::server_config::ServerConfig;
use shared_essential::presentation::dto::master_data::error::MasterDataResult;

use crate::infrastructure::GlobalConfig;
use crate::presentation::handlers::http::*;

pub async fn start_http_server(config: Arc<GlobalConfig>) -> MasterDataResult<()> {
    let host = config.env.url.0.clone();
    let port = config.env.url.1;
    let url = format!("{}:{}", host, port);
    info!(
        kind = "infra",
        "Master Data Service started at {}:{}",
        host.clone(),
        port
    );
    HttpServer::new(move || {
        App::new()
            .app_data(Data::from(config.broker_adapter.clone()))
            .app_data(Source::MasterData)
            .app_data(web::Data::from(config.clone()))
            .wrap(setup_cors())
            .wrap(NormalizePath::new(TrailingSlash::Always))
            .wrap(http_middleware::default_cookie_decoder())
            .wrap(Logger::default())
            .configure(config_url_handlers)
    })
    .bind(url.clone())?
    .run()
    .await
    .unwrap();

    info!(kind = "infra", "Master Data Service is stopped");

    Ok(())
}

/// Настройка http-маршрутов
pub fn config_url_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(index_root)))
        .service(
            web::scope("/v1")
                .wrap(AsezSessionWatcher)
                .wrap(TracingLogger::<ServiceRootSpanBuilder>::new())
                // POST
                .route("/test/routes/", web::post().to(test_routes))
                .route(
                    "/create/favorite_item/",
                    web::post().to(create_favorite_item),
                )
                .service(
                    web::scope("/organizational_user_assignment")
                        .route(
                            "/search/",
                            web::post().to(search_organizational_user_assignment),
                        )
                        .route(
                            "/search_by_id/",
                            web::post()
                                .to(search_organizational_user_assignment_by_id),
                        ),
                )
                .service(
                    web::scope("/organizational_structure")
                        .route(
                            "/search_by_id/",
                            web::post().to(organization_structure_search_by_id),
                        )
                        .route(
                            "/search/",
                            web::post().to(organizational_structure_search),
                        ),
                )
                .service(
                    web::scope("/plan_reason_cancel")
                        .route(
                            "/get/item_list/",
                            web::post().to(plan_reasons_cancel_get_item_list),
                        )
                        .route(
                            "/get/detail/",
                            web::post().to(plan_reasons_cancel_get_detail),
                        )
                        .route(
                            "/search/",
                            web::post().to(plan_reasons_cancel_search),
                        )
                        .route(
                            "/search_by_id/",
                            web::post().to(plan_reasons_cancel_search_by_id),
                        )
                        .route(
                            "/create/item/",
                            web::post().to(plan_reasons_cancel_create),
                        )
                        .route(
                            "/update/item/",
                            web::post().to(plan_reasons_cancel_update),
                        )
                        .route(
                            "/delete/item/",
                            web::post().to(plan_reasons_cancel_delete),
                        )
                        .route(
                            "/restore/item/",
                            web::post().to(plan_reasons_cancel_restore),
                        )
                        .route(
                            "/export/",
                            web::post().to(export_plan_reasons_cancel),
                        ),
                )
                .route("/{directory}/search_by_id/", web::post().to(search_by_ids))
                .route("/{directory}/search/", web::post().to(search))
                // GET
                .route(
                    "/master_data/get_updates/{timestamp}/",
                    web::get().to(get_updates),
                )
                .service(
                    web::scope("/get")
                        .route("/favorite_list/", web::get().to(get_favorite_list))
                        .route(
                            "/hierarchy/{dictionary}/",
                            web::get().to(get_hierarchy),
                        ),
                )
                .route("/{directory}/{id}/", web::get().to(find_by_id))
                .route("", web::get().to(index))
                // DELETE
                .route(
                    "/delete/favorite_item/",
                    web::delete().to(delete_favorite_item),
                ),
        )
        .service(
            web::scope("/monitoring")
                .route("/config", web::get().to(config_handler))
                .route("/test", web::get().to(healthcheck_handler))
                .route("/config/", web::get().to(config_handler))
                .route("/test/", web::get().to(healthcheck_handler)),
        );
}

/// Хендлер для проверки, жив ли сервис или нет
pub async fn healthcheck_handler() -> String {
    "Master-Data is alive".into()
}

/// Получение конфигурации сервера
pub async fn config_handler() -> Json<ServerConfig> {
    let server_cfg = ServerConfig::new();
    Json(server_cfg)
}

const MAX_AGE_CORS_CACHE: usize = 3600;

/// Настройка CORS политики сервиса
pub fn setup_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allow_any_header()
        .allow_any_method()
        .supports_credentials()
        // кэши
        // https://blog.gelin.ru/2018/12/cors.html
        .disable_vary_header()
        .max_age(MAX_AGE_CORS_CACHE)
}
