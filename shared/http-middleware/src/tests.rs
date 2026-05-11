use crate::login::{do_not_check_cookie_flag, AUTH_WITHOUT_COOKIE};
use actix_web::body::MessageBody;
use actix_web::dev::Service;
use actix_web::http::Version;
use actix_web::test::TestRequest;

const COOKIE: &str = "id=sw8qoXaGT%2FJRA7I+FoS1WtSCxlaVxDUkgJ1rohBzUOfz08eGK2FaLzBE9ntvq7pP4Dh2%2Fylq5P4MfQydSrGGSsTFS8s+2CqSIJXyVB%2FSqKQF";
const COOKIE_SESSION_ID: &str = "5329de02-903f-11ef-85a0-566ff2f30017";
fn create_request() -> TestRequest {
    TestRequest::post()
        .uri("/v1/get/plans/?user_id=666")
        .version(Version::HTTP_11)
        .insert_header(("content-type", "application/json"))
        .insert_header(("host", "localhost:3000"))
        .insert_header(("connection", "keep-alive"))
        .insert_header(("content-length", "0"))
        .insert_header(("cookie", COOKIE))
        .insert_header(("accept", "*/*"))
}

#[actix_web::test]
async fn test_check_query() {
    let req = create_request().to_http_request();
    println!("{req:?}");
    let user_id = crate::login::check_query(&req).unwrap();
    assert_eq!(user_id, 666);
}

#[actix_web::test]
async fn test_check_session_token() {
    let session = crate::cookie_decoder::default_cookie_decoder();
    let req = create_request().to_request();

    let serv =
        actix_web::test::init_service(actix_web::App::new().wrap(session).service(
            actix_web::web::resource("/v1/get/plans/").to(|req| async move {
                crate::login::check_session_token(&req).unwrap().to_string()
            }),
        ))
        .await;

    let res = serv.call(req).await.unwrap().into_body().try_into_bytes().unwrap();
    let res = String::from_utf8_lossy(&res);
    assert_eq!(&res, COOKIE_SESSION_ID);
}

fn safely_run_cookie_flag(run: fn() -> bool) -> bool {
    std::env::remove_var(AUTH_WITHOUT_COOKIE);
    let r = run();
    std::env::remove_var(AUTH_WITHOUT_COOKIE);
    r
}

#[test]
fn test_check_cookie_flag() {
    let r = safely_run_cookie_flag(do_not_check_cookie_flag);
    assert!(!r);

    let r = safely_run_cookie_flag(|| {
        std::env::set_var(AUTH_WITHOUT_COOKIE, "true");
        do_not_check_cookie_flag()
    });
    assert!(r);

    let r = safely_run_cookie_flag(|| {
        std::env::set_var(AUTH_WITHOUT_COOKIE, "TRUE");
        do_not_check_cookie_flag()
    });
    assert!(r);

    let r = safely_run_cookie_flag(|| {
        std::env::set_var(AUTH_WITHOUT_COOKIE, "false");
        do_not_check_cookie_flag()
    });
    assert!(!r);

    let r = safely_run_cookie_flag(|| {
        std::env::set_var(AUTH_WITHOUT_COOKIE, "FALSE");
        do_not_check_cookie_flag()
    });
    assert!(!r);
}
