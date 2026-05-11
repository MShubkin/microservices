//! This module contains the CEF format relevant code.
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

/// This structure stores the non blocking writer and the variables which will
/// likely remain constant across writes.
#[derive(Debug, Clone)]
pub struct CEFTracingLayer<'a> {
    /// This writer should allow the writing of regularly "rolling"/"rotating"
    /// log files without blocking the thread.
    writer: AppenderNonBlocking,
    host: net::IpAddr,
    srv_vendor: String,
    srv_name: String,
    srv_version: String,
    /// Is there a better way of storing these?
    kinds: &'a [&'a str],
    /// This represents the timezone GMT +/- tz (hours).
    /// We assume that we're tracing in Moscow/Leningrad, but the country has 11
    /// timezones.
    tz: UtcOffset,
}

#[derive(Debug, Clone, Default)]
pub struct CEFVisitorEvent<'a> {
    kinds: &'a [&'a str],
    is_traceable: bool,
    id: String,
    ext: String,
    user_code: Option<String>,
}

impl ServiceDescription {
    /// NB: Quoting the documentation:
    ///
    /// Note that the WorkerGuard returned by non_blocking must be assigned to
    /// a binding that is not _, as _ will result in the WorkerGuard being
    /// dropped immediately. Unintentional drops of WorkerGuard remove the
    /// guarantee that logs will be flushed during a program’s termination,
    /// in a panic or otherwise.
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

    /// NB: Quoting the documentation:
    ///
    /// Note that the WorkerGuard returned by non_blocking must be assigned to
    /// a binding that is not _, as _ will result in the WorkerGuard being
    /// dropped immediately. Unintentional drops of WorkerGuard remove the
    /// guarantee that logs will be flushed during a program’s termination,
    /// in a panic or otherwise.
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
            is_traceable: kinds.is_empty(),
            id: String::with_capacity(24),
            ext: String::with_capacity(255),
            user_code: None,
        }
    }
}

impl<'a> Visit for CEFVisitorEvent<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        match field.name() {
            // `id` extension has higher priority than `message`
            "message" if self.id.is_empty() => self.id = format!("{:?}", value),
            _ => self.ext.push_str(&format!("{}={:?}", field.name(), value)),
        }
    }

    // Overloading the `record_str` method.
    fn record_str(&mut self, field: &Field, v: &str) {
        let fname = field.name();
        match fname {
            // for siem logging, we log everything, so use empty kinds list
            "kind" if self.kinds.is_empty() || self.kinds.contains(&v) => {
                self.is_traceable = true
            }
            "suser" => self.user_code = Some(v.to_owned()),
            "id" => self.id = v.to_owned(),
            _ => self.ext.push_str(&format!("{}={}", fname, v)),
        }
    }
}

impl<'a> CEFTracingLayer<'a> {
    /// This is the only CEF specific format part of the code. We can quite
    /// easily write in other formats by adding a format switch to `CEFTracingLayer`
    /// (and maybe rename it to `IGGTracingLayer`) and several `write_event_as_X`
    /// functions.
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

impl<S> tracing_subscriber::Layer<S> for CEFTracingLayer<'static>
where
    S: Subscriber + for<'c> LookupSpan<'c>,
{
    /// We check for fields named "user_code", and if at least one exists we
    /// extend the extensions.
    /// TODO: Ask Maxim about why we are doing this.
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if attrs.fields().iter().any(|f| f.name() == "user_code") {
            // TODO: Should this be done in a way that does not crash the tracer?
            let span = ctx.span(id).expect("Id cannot be wrong somehow?");

            let mut exts = span.extensions_mut();
            // The borrowed base data is what is updated and stored in extensions.
            if let Some(base_data) = exts.get_mut::<ReqBaseData>() {
                attrs.values().record(base_data);
            } else {
                let mut data = ReqBaseData::default();
                attrs.values().record(&mut data);
                exts.insert(data);
            };
        }
    }

    /// Specifically do nothing!
    fn on_enter(&self, _id: &Id, _ctx: Context<'_, S>) {}

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        // TODO: Should this be done in a way that does not crash the tracer?
        let span = ctx.span(id).expect("Id cannot be wrong somehow?");

        if let Some(base_data) = span.extensions_mut().get_mut::<ReqBaseData>() {
            values.record(base_data);
        };
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut visitor_event = CEFVisitorEvent::new(self.kinds);

        event.record(&mut visitor_event);
        // NB: We must observe/record whether the event is traceable before we can
        // decide whether we need to return or not.
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
        // If we have not returned after writing an event, we write a blank.
        // NB: If we want to add format flexibility, we can get away by adding
        // a switch here and some more `write_event_as_X` functions.
        self.write_event(&visitor_event, &ReqBaseData::default(), level);
    }
}

#[derive(Copy, Clone)]
enum CEFContext {
    Header,
    Extension,
}

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
