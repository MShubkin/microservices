//! Инициализация логирования для сервисов, использующих CEF-формат.
//!
//! Вынесено в отдельный крейт, а не в `shared-essential`, по причине компиляции:
//! `shared-essential` большой и тянет много зависимостей, поэтому всё, что можно
//! скомпилировать параллельно с ним — лучше вынести. `trace-setup` зависит только
//! от `igg-tracing` и компилируется независимо.
//!
//! Сейчас используется в: `processing`, `plan-db`.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::span::Span;
use tracing_appender::non_blocking::WorkerGuard as AppenderGuard;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};

use igg_tracing::{CEFTracingLayer, ServiceDescription};

/// CEF-фильтр для операций доступа к данным — попадают в access-лог SIEM.
///
/// "read" и "download" объединены в "get", чтобы не плодить категории для
/// операций, которые в нашей системе всегда идут вместе.
pub const ACCESS_OPS: &[&str] = &["get", "insert", "update"];

/// CEF-фильтр для событий безопасности — будет использован когда сообщения
/// RabbitMQ начнут содержать информацию о входах/выходах пользователей.
pub const SECURITY_OPS: &[&str] = &["users"];
const VENDOR: &str = "gazprom.ru";
const SERVICE: &str = "srm";

/// Создаёт CEF-слой, пишущий в файл.
///
/// UTC+3 (Москва) захардкожен — SIEM-система ожидает локальное московское время
/// в метках CEF, и менять это через конфиг смысла нет пока система развёрнута
/// только в российском контуре.
///
/// Возвращает `WorkerGuard` — его нужно хранить живым до завершения процесса.
/// При дропе guard ждёт, пока буфер non-blocking writer не опустеет, чтобы
/// не потерять последние строки лога при shutdown.
pub fn new_cef<'a>(
    service: &ServiceDescription,
    path: impl AsRef<Path>,
    ops: &'a [&'a str],
) -> (CEFTracingLayer<'a>, AppenderGuard) {
    service.clone().into_guarded_cef_layer_file(
        path,
        ops,
        time::UtcOffset::from_hms(3, 0, 0).expect("Moscow exists."),
    )
}

/// Создаёт CEF-слой, пишущий в stdout. Используется в режиме `Cef` (LOGGER_MODE=1),
/// когда логи собирает docker/k8s из stdout вместо монтирования файловой системы.
pub fn new_cef_stdout<'a>(
    service: &ServiceDescription,
    ops: &'a [&'a str],
) -> (CEFTracingLayer<'a>, AppenderGuard) {
    service.clone().into_guarded_cef_layer_stdout(
        ops,
        time::UtcOffset::from_hms(3, 0, 0).expect("Moscow exists."),
    )
}

/// Режим логирования сервиса.
///
/// Читается из `LOGGER_MODE` через `TracingCfg::from_env()`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum TracingKind {
    /// Логирование отключено (LOGGER_MODE=0 или не задан). Используется в unit-тестах.
    None,
    /// Человекочитаемый вывод в stdout — для локальной разработки (LOGGER_MODE=2 | "normal").
    Normal,
    /// CEF в stdout — для контейнерных окружений, где логи забирает агент (LOGGER_MODE=1 | "cef").
    Cef,
    /// CEF в файл + JSON в stdout одновременно — для продакшна с SIEM-интеграцией (LOGGER_MODE=3 | "json_cef").
    JsonCef {
        path: PathBuf,
    },
}

impl Default for TracingKind {
    fn default() -> Self {
        Self::None
    }
}

impl TracingKind {
    /// Инициализирует глобальный tracing subscriber и возвращает корневой span.
    ///
    /// Корневой span нужен сервисам без HTTP-сервера (processing, plan-db) — в них нет
    /// `ServiceRootSpanBuilder`, который создаёт span на каждый запрос. Вместо этого
    /// один долгоживущий span охватывает весь процесс, и CEF-события привязываются к нему.
    ///
    /// Возвращённые `guards` нужно хранить до конца `main()`. Дроп guard-а запускает
    /// flush non-blocking writer-а — без этого последние строки лога могут пропасть.
    pub fn initiate_log(
        &self,
        _service_name: &str,
        host: &str,
        port: u16,
        access_ops: &'static [&'static str],
    ) -> TracingSetupResult {
        let own_ip = local_ip_address::local_ip()?.to_string();
        let root_span = tracing::span!(
            tracing::Level::DEBUG,
            "processing-root-span",
            "uri" = "root-span",
            "user_agent" = "Unknown",
            "user_code" = "Unknown",
            "source_ip" = &own_ip as &str,
            "source_port" = port,
            "request_id" = "none",
        );

        let service = ServiceDescription {
            vendor: VENDOR.to_owned(),
            name: SERVICE.to_owned(),
            // TODO: See if we need some script ot keep this up to date.
            version: "0.0.1".to_owned(),
            // TODO: Is this correct?
            host: host.parse()?,
        };
        let guards = match self {
            TracingKind::None => vec![],
            TracingKind::Normal => {
                igg_tracing::setup_dev_logger();
                vec![]
            }
            TracingKind::Cef => {
                let (cef_layer, cef_guard) = new_cef_stdout(&service, &[]);
                igg_tracing::setup_loggers!(cef_layer);
                vec![cef_guard]
            }
            TracingKind::JsonCef { path } => {
                let (cef_layer, cef_guard) = new_cef(&service, path, access_ops);
                igg_tracing::setup_loggers!(
                    cef_layer,
                    JsonStorageLayer,
                    BunyanFormattingLayer::new("json".to_string(), std::io::stdout)
                );
                vec![cef_guard]
            }
        };

        Ok((guards, root_span))
    }
}

/// Алиас для удобства — чтобы каждый сервис не тащил `tracing-appender` напрямую.
pub type TracingSetupResult = Result<(Vec<AppenderGuard>, Span), TsError>;

#[derive(thiserror::Error, Debug)]
pub enum TsError {
    #[error("Ip address parsing error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("Error getting own IP address: {0}")]
    LocalIp(#[from] local_ip_address::Error),
    #[error("Error setting tracing subscriber: {0}")]
    SetTracer(#[from] tracing::subscriber::SetGlobalDefaultError),
}
