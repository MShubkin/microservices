//! Единая точка чтения конфигурации из переменных окружения.
//!
//! Все сервисы читают конфиг через этот крейт — так переменные не дублируются
//! и гарантировано совпадают между микросервисами одного деплоя.
//!
//! Паттерн: каждая структура конфига (`RabbitCfg`, `ServerCfg`, ...) имеет метод
//! `from_env()`, который читает нужные переменные и возвращает `Result<Self, EnvError>`.
//! Вспомогательные функции `var()` и `try_get()` дают понятные сообщения об ошибках
//! (в т.ч. имя переменной, которой не хватает) вместо паники.
mod time;
pub use crate::time::{TimeParseError, TimeWithOffset};

use lettre::Address;
use serde::{Deserialize, Deserializer};
use std::{env::VarError, str::FromStr};
use thiserror::Error;
use trace_setup::TracingKind;

use url::Url;

pub const LOGGER_MODE: &str = "LOGGER_MODE";
pub const LOGGER_DIR: &str = "LOGGER_DIR";
pub const LOGGER_FILE: &str = "LOGGER_FILE";

pub const RABBITMQ_HOST: &str = "RABBITMQ_HOST";
pub const RABBITMQ_VHOST: &str = "RABBITMQ_VHOST";
pub const RABBITMQ_PORT: &str = "RABBITMQ_PORT";
pub const RABBITMQ_USERNAME: &str = "RABBITMQ_USERNAME";
pub const RABBITMQ_PASSWORD: &str = "RABBITMQ_PASSWORD";
pub const RABBITMQ_RETRIES: &str = "RABBITMQ_RETRIES";
pub const RABBITMQ_INTERVAL_MS: &str = "RABBITMQ_INTERVAL_MS";

pub const PLANDB_SRV_INNER_ADDR: &str = "PLANDB_SRV_INNER_ADDR";
pub const PLANDB_SRV_INNER_PORT: &str = "PLANDB_SRV_INNER_PORT";

pub const SRV_HOST: &str = "SRV_HOST";
pub const SRV_PORT: &str = "SRV_PORT";
pub const SRV_PAYLOAD_LIMIT: &str = "SRV_PAYLOAD_LIMIT";
pub const SRV_JSON_LIMIT: &str = "SRV_JSON_LIMIT";

pub const SRV_HOST_DEFAULT_VALUE: &str = "0.0.0.0";
pub const SRV_PORT_DEFAULT_VALUE: &str = "3000";
pub const SRV_PAYLOAD_LIMIT_DEFAULT_VALUE: usize = 256 * 1024; // 256KiB
pub const SRV_JSON_LIMIT_DEFAULT_VALUE: usize = 2 * 1024 * 1024; // 2MiB

pub const SRV_WORKERS: &str = "SRV_WORKERS";
pub const SRV_MAX_CONN_PER_WORKER: &str = "SRV_MAX_CONN_PER_WORKER";
pub const SRV_BLOCKING_THREADS_PER_WORKER: &str = "SRV_BLOCKING_THREADS_PER_WORKER";
pub const SRV_THREAD_COUNT: &str = "SRV_THREAD_COUNT";

pub const POSTGRES_HOST: &str = "POSTGRES_HOST";
pub const POSTGRES_VHOST: &str = "POSTGRES_VHOST";
pub const POSTGRES_PORT: &str = "POSTGRES_PORT";
pub const POSTGRES_DB: &str = "POSTGRES_DB";
pub const POSTGRES_USER: &str = "POSTGRES_USER";
pub const POSTGRES_PASSWORD: &str = "POSTGRES_PASSWORD";
pub const POSTGRES_MIN_CONNECTIONS: &str = "POSTGRES_MIN_CONNECTIONS";
pub const POSTGRES_MAX_CONNECTIONS: &str = "POSTGRES_MAX_CONNECTIONS";
pub const POSTGRES_TIMEOUT_S: &str = "POSTGRES_TIMEOUT_S";
pub const POSTGRES_CONNECTION_REFRESH_S: &str = "POSTGRES_CONNECTION_REFRESH_S";

pub const EMAIL_LOGIN: &str = "EMAIL_LOGIN";
pub const EMAIL_SENDER: &str = "EMAIL_SENDER";
pub const EMAIL_PASSWORD: &str = "EMAIL_PASSWORD";
pub const EMAIL_SMTP_SERVER: &str = "EMAIL_SMTP_SERVER";
pub const EMAIL_SMTP_PORT: &str = "EMAIL_SMTP_PORT";
pub const DEFAULT_EMAIL_SMTP_PORT: u16 = 25;

pub const MASTER_DATA_BASE_URL: &str = "MASTER_DATA_BASE_URL";

pub const MONOLITH_BASE_URL: &str = "MONOLITH_BASE_URL";
pub const MONOLITH_TECH_USER_ID: &str = "MONOLITH_TECH_USER_ID";

/// Читает переменную `$var` и парсит в тип `$tpe`, или использует `$default`.
///
/// Отличие от `try_get()`: принимает имя переменной как идентификатор (константу),
/// а не строку — это избавляет от дублирования строк там, где уже есть константа.
#[macro_export]
macro_rules! try_get {
    ($var:ident, $default:expr, $tpe:ty) => {
        $crate::var($var)
            .map(|x| x.trim().to_string())
            .unwrap_or_else(|_| $default.to_string())
            .parse::<$tpe>()
    };
}

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("Error getting env from IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error parsing value: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Environment variable error: {0}")]
    Var(#[from] std::env::VarError),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Ошибка парсинга почты: {0}")]
    EmailParse(#[from] lettre::address::AddressError),
    #[error("Ошибка парсинга времени: {0}")]
    TimeParse(#[from] TimeParseError),
    #[error("Crucial environmental variable \"{0}\" is not set.")]
    MissingVar(String),
    #[error("Crucial environmental variable \"{0}\" is malformed: \"{1:?}\".")]
    InvalidVar(String, std::ffi::OsString),
    #[error("Some error getting environmental variables: {0}")]
    Other(String),
}

impl From<String> for EnvError {
    fn from(e: String) -> Self {
        Self::Other(e)
    }
}

/// Получение переменной среды.
pub fn var(s: &str) -> Result<String, EnvError> {
    std::env::var(s).map_err(|e| match e {
        VarError::NotPresent => EnvError::MissingVar(s.to_string()),
        VarError::NotUnicode(x) => EnvError::InvalidVar(s.to_string(), x),
    })
}

/// Получение переменной среды или `None`.
pub fn var_maybe(s: &str) -> Result<Option<String>, EnvError> {
    match std::env::var(s) {
        Ok(x) => Ok(Some(x)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(x)) => Err(EnvError::InvalidVar(s.to_string(), x)),
    }
}

/// Получение значения из переменной среды `var`, или значение `default`, если
/// переменная не определена.
pub fn try_get<T>(var: &str, default: T) -> Result<T, EnvError>
where
    T: FromStr,
    EnvError: From<T::Err>,
{
    Ok(var_maybe(var)?
        .map(|x| x.trim().parse())
        .transpose()?
        .unwrap_or(default))
}

/// Получение опционоального значения из переменной среды `var`.
pub fn try_get_maybe<T>(var: &str) -> Result<Option<T>, EnvError>
where
    T: FromStr,
    EnvError: From<T::Err>,
{
    Ok(var_maybe(var)?.map(|x| x.trim().parse()).transpose()?)
}

/// This is a complete configuration for microservices.
pub struct EnvCfg {
    pub rabbit: RabbitCfg,
    pub server: ServerCfg,
    pub postgres: PostgresCfg,
    pub email: MailerCfg,
    pub logger: TracingCfg,
}

/// Конфигурация подключения к RabbitMQ.
///
/// `Deserialize` реализован для интеграционных тестов, которые загружают конфиг
/// из TOML/JSON-фикстур вместо реальных переменных окружения.
///
/// `retries`/`retry_interval_ms` — параметры повторного подключения при старте.
/// Дефолт 10 попыток × 100ms = 1 секунда, что достаточно для Docker-compose
/// где RabbitMQ стартует чуть позже сервиса.
// `Debug` реализован вручную (см. ниже), чтобы пароль не утекал в логи.
#[derive(Clone, Deserialize, PartialEq)]
pub struct RabbitCfg {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pw: String,
    pub vhost: String,
    pub retries: u32,
    pub retry_interval_ms: u64,
}

impl std::fmt::Debug for RabbitCfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RabbitCfg")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("pw", &"<redacted>")
            .field("vhost", &self.vhost)
            .field("retries", &self.retries)
            .field("retry_interval_ms", &self.retry_interval_ms)
            .finish()
    }
}

impl RabbitCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        let retries = try_get!(RABBITMQ_RETRIES, "10", u32)?;
        let retry_interval_ms = try_get!(RABBITMQ_INTERVAL_MS, "100", u64)?;

        Ok(RabbitCfg {
            host: var(RABBITMQ_HOST)?,
            port: var(RABBITMQ_PORT)?.parse::<u16>()?,
            user: var(RABBITMQ_USERNAME)?,
            pw: var(RABBITMQ_PASSWORD)?,
            vhost: var(RABBITMQ_VHOST)?,
            retries,
            retry_interval_ms,
        })
    }
}

/// Конфигурация HTTP-сервера.
///
/// `max_conn_per_worker` ограничен до 1000 вместо actix-дефолта 25000 — потому что
/// при 25000 соединениях на worker первый же DDoS или медленный клиент выедает весь
/// файловый дескриптор процесса. 1000 — более консервативное ограничение для API-сервиса.
///
/// `blocking_threads_per_worker` = 2 — почти все операции async, blocking-потоки нужны
/// только для редкого sync-IO (чтение файлов, некоторые FFI-вызовы). Больше двух —
/// это уже пустые потоки, которые съедают память стека.
///
/// `Deserialize` — для тестовых фикстур.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ServerCfg {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_conn_per_worker: usize,
    pub blocking_threads_per_worker: usize,
    pub server_thread_count: u8,
}

impl ServerCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        let max_conn_per_worker = try_get!(SRV_MAX_CONN_PER_WORKER, "1000", usize)?;
        let blocking_threads_per_worker =
            try_get!(SRV_BLOCKING_THREADS_PER_WORKER, "2", usize)?;
        let server_thread_count = try_get!(SRV_THREAD_COUNT, "4", u8)?;

        let port = try_get!(SRV_PORT, SRV_PORT_DEFAULT_VALUE, u16)?;
        let host =
            var(SRV_HOST).unwrap_or_else(|_| SRV_HOST_DEFAULT_VALUE.to_owned());

        Ok(ServerCfg {
            host,
            port,
            workers: try_get!(SRV_WORKERS, "2", usize)?,
            max_conn_per_worker,
            blocking_threads_per_worker,
            server_thread_count,
        })
    }
}

/// Конфигурация пула соединений PostgreSQL.
///
/// `conn_refresh_interval_s` = 3600 — SQLx не умеет сам обнаруживать разрывы TCP-соединений
/// (например, при перезапуске PG или сетевых таймаутах файрвола). Раз в час соединения
/// переоткрываются превентивно, чтобы не получить `connection reset` в продакшне.
///
/// `min_connections` = 1 — держим одно прогретое соединение даже в простое.
/// `max_connections` = 4 — каждый worker Actix держит свой пул, итого worker×4 соединений
/// на сервис. Больше не нужно: у нас не read-heavy нагрузка, а короткие transactional запросы.
///
/// `Deserialize` — для тестовых фикстур.
// `Debug` — вручную, чтобы скрыть пароль.
#[derive(Clone, Deserialize, PartialEq)]
pub struct PostgresCfg {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pw: String,
    pub db_name: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub conn_timeout_s: u32,
    pub conn_refresh_interval_s: u64,
}

impl std::fmt::Debug for PostgresCfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresCfg")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("pw", &"<redacted>")
            .field("db_name", &self.db_name)
            .field("min_connections", &self.min_connections)
            .field("max_connections", &self.max_connections)
            .field("conn_timeout_s", &self.conn_timeout_s)
            .field("conn_refresh_interval_s", &self.conn_refresh_interval_s)
            .finish()
    }
}

impl PostgresCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            host: var(POSTGRES_VHOST)?,
            port: var(POSTGRES_PORT)?.parse::<u16>()?,
            user: var(POSTGRES_USER)?,
            pw: var(POSTGRES_PASSWORD)?,
            db_name: var(POSTGRES_DB)?,
            min_connections: try_get!(POSTGRES_MIN_CONNECTIONS, "1", u32)?,
            max_connections: try_get!(POSTGRES_MAX_CONNECTIONS, "4", u32)?,
            conn_timeout_s: try_get!(POSTGRES_TIMEOUT_S, "10", u32)?,
            conn_refresh_interval_s: try_get!(
                POSTGRES_CONNECTION_REFRESH_S,
                "3600",
                u64
            )?,
        })
    }

    pub fn get_connection_string(&self) -> String {
        // отдельные переменные вместо connection string — исторически, для обратной совместимости
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.pw, self.host, self.port, self.db_name
        )
    }
}

// `Debug` — вручную, чтобы скрыть пароль SMTP.
#[derive(Clone, PartialEq)]
pub struct MailerCfg {
    pub login: String,
    pub pw: String,
    pub sender: Address,
    pub stmp_server: String,
    pub port: u16,
}

impl std::fmt::Debug for MailerCfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailerCfg")
            .field("login", &self.login)
            .field("pw", &"<redacted>")
            .field("sender", &self.sender)
            .field("stmp_server", &self.stmp_server)
            .field("port", &self.port)
            .finish()
    }
}

impl MailerCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            login: var(EMAIL_LOGIN)?,
            pw: var(EMAIL_PASSWORD)?,
            sender: var(EMAIL_SENDER)?.parse()?,
            stmp_server: var(EMAIL_SMTP_SERVER)?,
            port: try_get!(EMAIL_SMTP_PORT, DEFAULT_EMAIL_SMTP_PORT, u16)?,
        })
    }
}

/// Конфигурация режима логирования.
///
/// `Deserialize` — для тестовых фикстур.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TracingCfg {
    pub tracing_kind: trace_setup::TracingKind,
}

impl TracingCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        let mode =
            var(LOGGER_MODE).unwrap_or_else(|_| "0".to_string()).to_lowercase();
        // Числовые коды оставлены для обратной совместимости с ansible-скриптами,
        // которые передают "1"/"2"/"3". Текстовые алиасы — для читаемости в docker-compose.
        let tracing_kind = match mode.as_ref() {
            "2" | "normal" => TracingKind::Normal,
            "1" | "cef" => TracingKind::Cef,
            "3" | "json_cef" => TracingKind::JsonCef {
                path: var(LOGGER_FILE)?.into(),
            },
            _ => TracingKind::None,
        };

        Ok(Self { tracing_kind })
    }
}

/// Describes the address of the plan server
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PlanAddress {
    pub host: String,
    pub port: u16,
}

impl PlanAddress {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            host: var(PLANDB_SRV_INNER_ADDR)?,
            // Это адрес внутри контура, не внешний адрес
            port: var(PLANDB_SRV_INNER_PORT)
                .unwrap_or_else(|_| SRV_PORT_DEFAULT_VALUE.to_owned())
                .parse()?,
        })
    }
}

/// Describes the address of planning(monolith) rest service
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PlanningRestCfg {
    pub url: String,
}

impl PlanningRestCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            url: var(MONOLITH_BASE_URL)?,
        })
    }
}

/// GPI Monolith base ulr configuration
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MonolithCfg {
    #[serde(deserialize_with = "deserialize_url")]
    pub url: Url,
    pub tech_user_id: i32,
}

impl MonolithCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            url: {
                let maybe_url = var(MONOLITH_BASE_URL)?;
                Url::parse(&maybe_url)?
            },
            tech_user_id: var(MONOLITH_TECH_USER_ID)?.parse()?,
        })
    }
}

/// Master-Data-Service base ulr configuration
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MDSCfg {
    #[serde(deserialize_with = "deserialize_url")]
    pub url: Url,
}

impl MDSCfg {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            url: {
                let maybe_url = var(MASTER_DATA_BASE_URL)?;
                Url::parse(&maybe_url)?
            },
        })
    }
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let maybe_url: &str = serde::de::Deserialize::deserialize(deserializer)?;
    Url::parse(maybe_url).map_err(serde::de::Error::custom)
}

/// Ограничение размера тела запроса (bytes/string payload).
///
/// Отдельный лимит от `JsonConfig` — потому что multipart/form-data и
/// бинарные тела требуют других пределов, чем JSON-API.
/// `None` означает actix-дефолт (256KiB для payload, 32KiB для JSON).
#[derive(Clone, Debug)]
pub struct PayloadConfig {
    limit: Option<usize>,
}

impl PayloadConfig {
    pub fn from_env() -> Result<Self, EnvError> {
        let limit = try_get_maybe(SRV_PAYLOAD_LIMIT)?;
        Ok(PayloadConfig { limit })
    }

    pub fn for_actix_web(&self) -> actix_web::web::PayloadConfig {
        let x = actix_web::web::PayloadConfig::default();
        if let Some(limit) = self.limit {
            x.limit(limit)
        } else {
            x
        }
    }
}

/// Ограничение размера JSON-тела запроса.
///
/// Отдельный лимит нужен, чтобы разрешить загрузку больших файлов через `PayloadConfig`
/// и при этом не пускать гигантские JSON-запросы, которые никогда не должны быть большими.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JsonConfig {
    limit: Option<usize>,
}

impl JsonConfig {
    pub fn new(limit: usize) -> Self {
        JsonConfig { limit: Some(limit) }
    }

    pub fn from_env() -> Result<Self, EnvError> {
        let limit = try_get_maybe(SRV_JSON_LIMIT)?;
        Ok(JsonConfig { limit })
    }

    pub fn for_actix_web(&self) -> actix_web::web::JsonConfig {
        let x = actix_web::web::JsonConfig::default();
        if let Some(limit) = self.limit {
            x.limit(limit)
        } else {
            x
        }
    }
}
