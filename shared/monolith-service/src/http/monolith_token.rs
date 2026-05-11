use actix_web::FromRequest;

use super::MONOLITH_TOKEN_COOKIE;

/// Контейнер для получения токена монолита из запроса к сервису.
#[derive(Debug, Clone)]
pub struct MonolithToken {
    token: String,
}

impl MonolithToken {
    pub fn into_inner(self) -> String {
        self.token
    }
}

impl FromRequest for MonolithToken {
    type Error = Box<dyn std::error::Error>;

    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_http::Payload,
    ) -> Self::Future {
        let token = req
            .cookie(MONOLITH_TOKEN_COOKIE)
            .map(|token| token.value().to_owned())
            .unwrap_or_default();
        std::future::ready(Ok(MonolithToken { token }))
    }
}
