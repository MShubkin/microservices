use actix_cors::Cors;
use actix_web::{
    web::{self, get, Json, ServiceConfig},
    App, HttpServer,
};

use shared_essential::infrastructure::server_config::ServerConfig;

use crate::common::result::*;

/// Максимальное время (в секундах), в течение которого этот запрос CORS может быть кэширован
const MAX_AGE_CORS_CACHE: usize = 3600;

/// # Описание
///
/// Создание HTTP сервера
/// # Возвращает
/// * [`Ok(())`] - Успешное создание HTTP сервера
/// * [`Err(Error)`] - Ошибка при создании HTTP сервера
pub(crate) async fn start_http_server(host: &str, port: u16) -> Result<()> {
    let url = format!("{}:{}", host, port);
    HttpServer::new(move || App::new().configure(setup_router).wrap(setup_cors()))
        .bind(url)?
        .run()
        .await
        .map_err(Into::into)
}

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

/// Настройка HTTP эндпоинтов сервиса
fn setup_router(conf: &mut ServiceConfig) {
    // Galko: Тут / на конце не нужен!
    let monitoring_scope = web::scope("/monitoring")
        .route("/config", get().to(config_handler))
        .route("/test", get().to(healthcheck_handler));

    conf.service(monitoring_scope);
}

/// Хендлер для проверки, жив ли сервис или нет
pub async fn healthcheck_handler() -> String {
    "Processing is alive".into()
}

/// Получение конфигурации сервера
pub async fn config_handler() -> Json<ServerConfig> {
    let server_cfg = ServerConfig::new();
    Json(server_cfg)
}
