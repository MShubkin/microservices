use std::convert::Infallible;

use actix_http::{Request, StatusCode};
use actix_web::{
    dev::{Service, ServiceResponse},
    web::{Json, ServiceConfig},
    App,
};
use env_setup::JsonConfig;

async fn spawn_app(
    limit: Option<usize>,
) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
    let json_config = limit.map_or_else(JsonConfig::default, JsonConfig::new);
    actix_web::test::init_service(
        App::new().app_data(json_config.for_actix_web()).configure(setup_router),
    )
    .await
}

fn setup_router(conf: &mut ServiceConfig) {
    use actix_web::web::*;
    let f = resource("/test/").route(post().to(test_handler));
    conf.service(f);
}

async fn test_handler(_: Json<serde_json::Value>) -> Result<String, Infallible> {
    Ok("".to_string())
}

fn test_req(size: usize) -> Request {
    let mut data = vec![b' '; size];
    data[..2].copy_from_slice(b"{}");
    actix_web::test::TestRequest::post()
        .uri("/test/")
        .set_payload(data)
        .insert_header(("content-type", "application/json"))
        .to_request()
}

#[actix_web::test]
async fn increased() {
    const SIZE: usize = 4 * 1024 * 1024;
    let app = spawn_app(Some(SIZE)).await;
    let response = actix_web::test::call_service(&app, test_req(SIZE)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response = actix_web::test::call_service(&app, test_req(SIZE + 1)).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[actix_web::test]
async fn default() {
    const SIZE: usize = 2 * 1024 * 1024;
    let app = spawn_app(None).await;
    let response = actix_web::test::call_service(&app, test_req(SIZE)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response = actix_web::test::call_service(&app, test_req(SIZE + 1)).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
