//! CEF (Common Event Format) — слой трейсинга для SIEM-систем.
//!
//! Каждое событие сериализуется в одну строку вида:
//! ```text
//! May 11 12:00:00 192.168.1.1 CEF:0|Vendor|Name|1.0|category|event_id|severity|key=val ...
//! ├─────────────┘ └─────────┘ └──┘ └────────────────────────────────┘ └──────┘ └───────┘
//!   timestamp      host ip    ver        7 заголовочных полей (pipe)    sev.   extension
//! ```
//!
//! Запись в файл/stdout происходит через `tracing_appender::NonBlocking` —
//! асинхронный writer, не блокирующий рабочий поток Tokio.
use time::{OffsetDateTime, UtcOffset};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Subscriber};
use tracing_appender::non_blocking::NonBlocking as AppenderNonBlocking;
use tracing_appender::non_blocking::WorkerGuard as AppenderGuard;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;
use std::net;
use std::path::Path;

use super::{event_severity, ReqBaseData, ServiceDescription};

/// Слой `tracing_subscriber`, записывающий события в формате CEF.
///
/// Конструируется через `ServiceDescription::into_guarded_cef_layer_file` или
/// `into_guarded_cef_layer_stdout`. Возвращаемый `WorkerGuard` **должен жить
/// до конца программы** — его drop сигнализирует фоновому потоку прекратить
/// запись, что ведёт к потере буферизованных логов.
///
/// Слой фильтрует события по полю `kind`: логируются только события, у которых
/// `kind` входит в срез `kinds`. Пустой `kinds` означает «логировать всё».
#[derive(Debug, Clone)]
pub struct CEFTracingLayer<'a> {
    /// Non-blocking writer — запись делегируется фоновому потоку, не блокируя async executor.
    writer: AppenderNonBlocking,
    host: net::IpAddr,
    srv_vendor: String,
    srv_name: String,
    srv_version: String,
    /// Whitelist категорий событий (поле `kind`). Пустой срез = логировать всё.
    kinds: &'a [&'a str],
    /// Часовой пояс для timestamp в CEF-строке. Дефолт — Москва (UTC+3).
    tz: UtcOffset,
}

/// Временный аккумулятор данных одного `tracing::Event` для CEF-записи.
///
/// Заполняется через `Visit::record_str` / `record_debug` при обходе полей события.
/// После заполнения `is_traceable` определяет, нужно ли писать в лог.
#[derive(Debug, Clone, Default)]
pub struct CEFVisitorEvent<'a> {
    /// Whitelist из родительского `CEFTracingLayer` — нужен для проверки `kind`.
    kinds: &'a [&'a str],
    /// Становится `true`, когда встречено поле `kind` со значением из `kinds`.
    /// Инициализируется `kinds.is_empty()`: пустой список = логировать всё.
    is_traceable: bool,
    /// CEF-поле `Name` (event_id) — заполняется из поля `id` или `message`.
    id: String,
    /// Дополнительные key=value пары для CEF Extension.
    ext: String,
    /// Код пользователя из поля `suser` — переопределяет `user_code` из span.
    user_code: Option<String>,
}

impl ServiceDescription {
    /// Создаёт CEF-слой с записью в файл.
    ///
    /// Файл открывается в режиме append (дозапись) и создаётся, если не существует.
    /// Это позволяет безопасно ротировать файлы внешними инструментами (logrotate).
    ///
    /// **Важно**: `WorkerGuard` необходимо сохранить в переменную (не `_`).
    /// Если guard будет дропнут сразу, фоновый поток записи завершится,
    /// и часть буферизованных логов может быть потеряна — в том числе при панике.
    pub fn into_guarded_cef_layer_file<'a>(
        self,
        path: impl AsRef<Path>,
        kinds: &'a [&'a str],
        tz: UtcOffset,
    ) -> (CEFTracingLayer<'a>, AppenderGuard) {
        let mut open_options = OpenOptions::new();
        let writer = open_options
            .append(true)
            .create(true)
            .open(path)
            .expect("failed to created CEF appender");
        let (writer, guard) = tracing_appender::non_blocking(writer);

        let layer = CEFTracingLayer {
            writer,
            host: self.host,
            srv_vendor: self.vendor,
            srv_name: self.name,
            srv_version: self.version,
            kinds,
            tz,
        };
        (layer, guard)
    }

    /// Создаёт CEF-слой с записью в stdout.
    ///
    /// Используется в контейнерных окружениях, где логи собираются со stdout.
    /// Те же требования к `WorkerGuard`, что и у `into_guarded_cef_layer_file`.
    pub fn into_guarded_cef_layer_stdout<'a>(
        self,
        kinds: &'a [&'a str],
        tz: UtcOffset,
    ) -> (CEFTracingLayer<'a>, AppenderGuard) {
        let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());

        let layer = CEFTracingLayer {
            writer,
            host: self.host,
            srv_vendor: self.vendor,
            srv_name: self.name,
            srv_version: self.version,
            kinds,
            tz,
        };
        (layer, guard)
    }
}

impl<'a> CEFVisitorEvent<'a> {
    pub(crate) fn new(kinds: &'a [&'a str]) -> Self {
        Self {
            kinds,
            // Если список kinds пуст — сразу помечаем событие как traceable,
            // иначе ждём поле `kind` со значением из whitelist.
            is_traceable: kinds.is_empty(),
            id: String::with_capacity(24),
            ext: String::with_capacity(255),
            user_code: None,
        }
    }
}

/// Обход полей одного `tracing::Event` для заполнения CEF-аккумулятора.
impl<'a> Visit for CEFVisitorEvent<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        match field.name() {
            // Поле `id` имеет приоритет над `message` для CEF Name-поля.
            // `message` используется как fallback, только если `id` ещё не задан.
            "message" if self.id.is_empty() => self.id = format!("{:?}", value),
            _ => self.ext.push_str(&format!("{}={:?}", field.name(), value)),
        }
    }

    fn record_str(&mut self, field: &Field, v: &str) {
        let fname = field.name();
        match fname {
            // Поле `kind` — фильтр: событие логируется только если его категория
            // входит в whitelist. Пустой kinds уже обработан в `new()`.
            "kind" if self.kinds.is_empty() || self.kinds.contains(&v) => {
                self.is_traceable = true
            }
            // `suser` — код пользователя прямо в событии (переопределяет span-данные).
            "suser" => self.user_code = Some(v.to_owned()),
            // `id` → CEF Name field (event identifier).
            "id" => self.id = v.to_owned(),
            _ => self.ext.push_str(&format!("{}={}", fname, v)),
        }
    }
}

impl<'a> CEFTracingLayer<'a> {
    /// Форматирует и записывает одну CEF-строку в non-blocking writer.
    ///
    /// Структура CEF-строки:
    /// ```text
    /// <timestamp> <host> CEF:0|<vendor>|<name>|<ver>|<cat>|<id>|<sev>|<extension>
    /// ```
    /// где extension = `suser=... src=... spt=... request=... requestClientApplication=... dpid=...`
    ///
    /// Ошибки записи игнорируются намеренно: нет безопасного способа обработать
    /// их в контексте `tracing::Layer` без риска рекурсии или паники.
    pub(crate) fn write_event(
        &self,
        event: &CEFVisitorEvent<'_>,
        data: &ReqBaseData,
        level: &Level,
    ) {
        let time_format = time::macros::format_description!(
            "[month repr:short] [day] [hour]:[minute]:[second]"
        );

        let timestamp = OffsetDateTime::now_utc()
            .to_offset(self.tz)
            .format(&time_format)
            .expect("Time should be valid.");

        // Приоритет user_code: поле `suser` события > поле из span > "unknown".
        let user_code = event.user_code.as_ref().map_or_else(
            || {
                if data.user_code.is_empty() {
                    "unknown"
                } else {
                    &data.user_code
                }
            },
            |suser| suser,
        );

        let ext = format!(
            "suser={user_code} src={src_ip} spt={src_port} request={uri} \
             requestClientApplication={user_agent} dpid={pid}",
            user_code = escape_cef_value(user_code, CEFContext::Extension),
            src_ip = escape_cef_value(&data.src_ip, CEFContext::Extension),
            src_port = data.src_port,
            uri = escape_cef_value(&data.uri, CEFContext::Extension),
            user_agent = escape_cef_value(&data.user_agent, CEFContext::Extension),
            pid = std::process::id(),
        );

        let mut writer = self.writer.make_writer();
        // I know of no good way to check the result unless we wish to crash
        // the thread.
        let _ = writeln!(writer,
            "{timestamp} {host} CEF:0|{srv_vendor}|{srv_name}|{srv_version}|{category}|{id}|{severity}|{ext}",
            timestamp = timestamp,
            host = self.host,
            srv_vendor = escape_cef_value(&self.srv_vendor, CEFContext::Header),
            srv_name = escape_cef_value(&self.srv_name, CEFContext::Header),
            srv_version = escape_cef_value(&self.srv_version, CEFContext::Header),
            category = escape_cef_value(&data.category, CEFContext::Header),
            id = escape_cef_value(&event.id, CEFContext::Header),
            severity = event_severity(level),
            ext = ext,
        );
        // NB. For non-blocking writer `flush` call is performed by the thread
        // that does actual writing.
    }
}

/// Реализация `Layer` позволяет CEFTracingLayer встроиться в цепочку
/// `tracing_subscriber` рядом с другими слоями (fmt, json, opentelemetry).
///
/// Требует `'static` lifetime, потому что `tracing_subscriber` хранит слои
/// в `Arc<dyn Layer>` — компилятор не может гарантировать, что `'a` живёт
/// достаточно долго. На практике `kinds` живёт в статической памяти программы.
impl<S> tracing_subscriber::Layer<S> for CEFTracingLayer<'static>
where
    S: Subscriber + for<'c> LookupSpan<'c>,
{
    /// Сохраняет `ReqBaseData` в extensions нового span-а, если у него есть поле `user_code`.
    ///
    /// Это позволяет дочерним событиям (вызовам `tracing::info!` внутри span-а)
    /// унаследовать контекст HTTP-запроса без явной передачи через аргументы.
    /// Если `ReqBaseData` уже существует в extensions (вложенные span-ы),
    /// данные дополняются, а не перезаписываются.
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if attrs.fields().iter().any(|f| f.name() == "user_code") {
            let span = ctx.span(id).expect("Id cannot be wrong somehow?");

            let mut exts = span.extensions_mut();
            if let Some(base_data) = exts.get_mut::<ReqBaseData>() {
                attrs.values().record(base_data);
            } else {
                let mut data = ReqBaseData::default();
                attrs.values().record(&mut data);
                exts.insert(data);
            };
        }
    }

    /// CEF-слой не использует момент входа в span — только события.
    fn on_enter(&self, _id: &Id, _ctx: Context<'_, S>) {}

    /// Обрабатывает `span.record(field, value)` — обновляет `ReqBaseData` в extensions
    /// при динамическом обновлении полей span-а (например, когда middleware
    /// дописывает `user_id` уже после создания span-а).
    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Id cannot be wrong somehow?");

        if let Some(base_data) = span.extensions_mut().get_mut::<ReqBaseData>() {
            values.record(base_data);
        };
    }

    /// Основная точка вывода: вызывается для каждого `tracing::info!/warn!/error!`.
    ///
    /// Алгоритм:
    /// 1. Собрать поля события в `CEFVisitorEvent`
    /// 2. Проверить `is_traceable` — если `kind` не в whitelist, выйти
    /// 3. Найти `ReqBaseData` в ближайшем родительском span-е (обход по scope)
    /// 4. Записать CEF-строку. Если span не найден — записать с пустым `ReqBaseData`
    ///    (событие без контекста запроса — например, логи при старте сервиса).
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut visitor_event = CEFVisitorEvent::new(self.kinds);

        // Сначала нужно обойти все поля, только потом проверять is_traceable —
        // поле `kind` может быть не первым в списке полей события.
        event.record(&mut visitor_event);
        if !visitor_event.is_traceable {
            return;
        }
        let level = event.metadata().level();

        if let Some(scope) = ctx.event_scope(event) {
            for span in scope {
                if let Some(data) = span.extensions().get::<ReqBaseData>() {
                    self.write_event(&visitor_event, data, level);
                    return;
                }
            }
        }
        // Событие вне span-а (например, инфраструктурное при старте) —
        // пишем с пустым контекстом, не теряя само сообщение.
        self.write_event(&visitor_event, &ReqBaseData::default(), level);
    }
}

/// Контекст экранирования: правила разные для заголовка и extension CEF-строки.
#[derive(Copy, Clone)]
enum CEFContext {
    /// В заголовке полями-разделителями являются `|` — их нужно экранировать.
    Header,
    /// В extension парами являются `key=value` — нужно экранировать `=`.
    Extension,
}

/// Экранирует специальные символы CEF согласно спецификации ArcSight.
///
/// Правила, общие для обоих контекстов:
/// - `\` → `\\`
/// - `\n` → `\n` (литерал)
/// - `\r` → `\r` (литерал)
///
/// Контекст-специфичные правила:
/// - Header: `|` → `\|` (разделитель полей заголовка)
/// - Extension: `=` → `\=` (разделитель ключ-значение)
fn escape_cef_value(value: &str, context: CEFContext) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '|' if matches!(context, CEFContext::Header) => escaped.push_str("\\|"),
            '=' if matches!(context, CEFContext::Extension) => {
                escaped.push_str("\\=")
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cef_tracing_layer_write() {
        // We need these parameters to wirte the file.
        let dir = tempfile::tempdir().expect("We can make a temporary directory");
        let file_name = "MyLog.log";
        let tz = time::UtcOffset::from_hms(3, 0, 0).expect("Moscow exists.");
        {
            let service_description = ServiceDescription::new(
                "MyVendor",
                "MyName",
                "0.0.1",
                "127.0.0.1".parse().expect("This is an IP address"),
            );
            let (cef_tracing_layer, _guard) = service_description
                .into_guarded_cef_layer_file(
                    dir.path().join(file_name),
                    ["kind"].as_slice(),
                    tz,
                );

            let event_data = ReqBaseData {
                category: "category".to_string(),
                uri: "https://inlinegroup.ru".to_string(),
                src_ip: "127.0.0.1".to_string(),
                src_port: 0,
                user_agent: "Agent Smith".to_string(),
                user_code: "075EK".to_string(),
                request_id: "some-request-uuid".to_string(),
                trace_id: "some-trace-uuid".to_string(),
            };

            let mut event = CEFVisitorEvent::new(&[]);
            event.id = "some-event_id".to_owned();

            cef_tracing_layer.write_event(&event, &event_data, &Level::TRACE);
        }
        let path = std::path::PathBuf::from(dir.path()).join(file_name);
        let log =
            std::fs::read_to_string(path).expect("We should be able to read log");

        // We need the regex to replace the date.
        let rex_date =
            regex::Regex::new("[\\w]{3} [\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2}")
                .expect("This is valid regex.");
        let rex_pid =
            regex::Regex::new("dpid=[\\d]+").expect("This is valid regex.");

        let log = rex_date.replace(&log, "[date]");
        let log = rex_pid.replace(&log, "[dpid]");

        let expected = "[date] 127.0.0.1 CEF:0|MyVendor|MyName|0.0.1|category|\
        some-event_id|1|suser=075EK src=127.0.0.1 spt=0 request=https://inlinegroup.ru \
        requestClientApplication=Agent Smith [dpid]\n";

        assert_eq!(&log, expected);
    }

    #[test]
    fn test_escape_cef_value() {
        // Header
        assert_eq!(
            escape_cef_value("field|value", CEFContext::Header),
            "field\\|value", // `|` должно экранироваться
        );
        assert_eq!(
            escape_cef_value("field\\value", CEFContext::Header),
            "field\\\\value", // `\` всегда экранируется
        );
        assert_eq!(
            escape_cef_value("field=value", CEFContext::Header),
            "field=value", // `=` в header НЕ экранируется
        );

        // Extension
        assert_eq!(
            escape_cef_value("field=value", CEFContext::Extension),
            "field\\=value", // `=` должно экранироваться
        );
        assert_eq!(
            escape_cef_value("field\nvalue", CEFContext::Extension),
            "field\\nvalue", // `\n` всегда экранируется
        );
        assert_eq!(
            escape_cef_value("field\rvalue", CEFContext::Extension),
            "field\\rvalue", // `\r` всегда экранируется
        );
        assert_eq!(
            escape_cef_value("field\\value", CEFContext::Extension),
            "field\\\\value", // `\` всегда экранируется
        );
        assert_eq!(
            escape_cef_value("field|value", CEFContext::Extension),
            "field|value", // `|` в extension НЕ экранируется
        );

        // Все символы экранированы правильно
        assert_eq!(
            escape_cef_value("field|=\\\n\rvalue", CEFContext::Extension),
            "field|\\=\\\\\\n\\rvalue",
        );
    }
}
