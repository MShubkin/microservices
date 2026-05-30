use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::{Key, SameSite};

/// Имя переменной окружения с ключом подписи сессионной куки.
pub const SESSION_KEY_ENV: &str = "SESSION_KEY";

/// Минимальная длина ключа в байтах. `actix_web::cookie::Key::from` требует ≥ 64 байт
/// для HMAC-SHA-256 (RFC 2104).
const MIN_KEY_LEN: usize = 64;

/// Ошибка инициализации ключа сессии. Возвращается из [`try_default_cookie_decoder`].
#[derive(Debug, thiserror::Error)]
pub enum CookieDecoderError {
    #[error("переменная окружения `{0}` не установлена — нужен ключ подписи сессий")]
    MissingKey(&'static str),
    #[error("ключ сессии короче минимально допустимых {min} байт (получено {got})")]
    KeyTooShort { min: usize, got: usize },
    #[error("ключ сессии содержит только нули — небезопасно")]
    KeyAllZeros,
}

/// Декодировщик сессионной куки с продакшн-настройками.
///
/// Ключ HMAC берётся из `SESSION_KEY` (минимум 64 байта). Должен быть одинаковым
/// на всех узлах кластера и храниться как secret (vault / k8s-secret).
///
/// `cookie_secure(true)`, `SameSite::Strict`, `http_only(true)` — стандартный
/// набор защиты от XSS и MITM. Для отладочной среды без HTTPS / с кросс-доменом
/// используйте [`default_cookie_decoder_with_options`].
///
/// # Panics
///
/// Паникует при старте, если `SESSION_KEY` отсутствует, слишком короткий или
/// нулевой. Это намеренно: микросервис не должен запускаться с небезопасным
/// ключом подписи сессий. Для проверки без паники см. [`try_default_cookie_decoder`].
pub fn default_cookie_decoder() -> SessionMiddleware<CookieSessionStore> {
    try_default_cookie_decoder()
        .expect("не удалось инициализировать ключ подписи сессий")
}

/// Fallible вариант [`default_cookie_decoder`] — возвращает `Result` вместо паники.
pub fn try_default_cookie_decoder()
    -> Result<SessionMiddleware<CookieSessionStore>, CookieDecoderError>
{
    let key = load_session_key()?;
    Ok(SessionMiddleware::builder(CookieSessionStore::default(), key)
        .cookie_http_only(true)
        .cookie_secure(true)
        .cookie_same_site(SameSite::Strict)
        .build())
}

/// Сборка с явным контролем флагов — для среды без HTTPS или кросс-домена.
/// В проде должна вызываться только через `default_cookie_decoder`.
pub fn default_cookie_decoder_with_options(
    http_only: bool,
    secure: bool,
    same_site: SameSite,
) -> Result<SessionMiddleware<CookieSessionStore>, CookieDecoderError> {
    let key = load_session_key()?;
    Ok(SessionMiddleware::builder(CookieSessionStore::default(), key)
        .cookie_http_only(http_only)
        .cookie_secure(secure)
        .cookie_same_site(same_site)
        .build())
}

fn load_session_key() -> Result<Key, CookieDecoderError> {
    let raw = std::env::var(SESSION_KEY_ENV)
        .map_err(|_| CookieDecoderError::MissingKey(SESSION_KEY_ENV))?;
    let bytes = raw.as_bytes();
    if bytes.len() < MIN_KEY_LEN {
        return Err(CookieDecoderError::KeyTooShort {
            min: MIN_KEY_LEN,
            got: bytes.len(),
        });
    }
    if bytes.iter().all(|b| *b == 0) {
        return Err(CookieDecoderError::KeyAllZeros);
    }
    Ok(Key::from(bytes))
}
