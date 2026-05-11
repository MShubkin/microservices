use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;

pub fn default_cookie_decoder() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 256]))
        .cookie_http_only(false)
        .cookie_secure(false)
        .cookie_same_site(actix_web::cookie::SameSite::None)
        .build()
}
