//! This is a micro-library that contains shared tracing setup that is used by
//! multiple services.
//!
//! The reason that it is separate is that shared-essential is becoming quite large
//! which is an unfavorable factor for compile time. This library has very little in
//! common with shared-essential, which means that it can be compiled prior or in
//! parallel which will improve project build time.
//!
//! It is also much easier to find the functionality here.
//!
//! Currently the services that use this setup are:
//! 1. processing
//! 2. plan-db

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::span::Span;
use tracing_appender::non_blocking::WorkerGuard as AppenderGuard;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};

use igg_tracing::{CEFTracingLayer, ServiceDescription};

/// Conventional operations in SRM are "read", "update", "download", however,
/// "read" and "download" are ambiguously close together, and there are no cases
/// as of yet where we read without downloading, so "get", "update" and "insert"
/// are used instead.
pub const ACCESS_OPS: &[&str] = &["get", "insert", "update"];
/// It is not clear that we need this yet. However once messages
/// contain logon information, we will definitely need it.
pub const SECURITY_OPS: &[&str] = &["users"];
const VENDOR: &str = "gazprom.ru";
const SERVICE: &str = "srm";

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

pub fn new_cef_stdout<'a>(
    service: &ServiceDescription,
    ops: &'a [&'a str],
) -> (CEFTracingLayer<'a>, AppenderGuard) {
    service.clone().into_guarded_cef_layer_stdout(
        ops,
        time::UtcOffset::from_hms(3, 0, 0).expect("Moscow exists."),
    )
}

/// Determines whether we trace at all, and if we do, whether it is
/// simple logging or igg-tracing
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum TracingKind {
    None,
    /// This is the preferred output format for during development
    Normal,
    /// For CEF formatting: `stdout` is used for CEF formatted output.
    Cef,
    /// For CEF formatting: `path` is used as the CEF ouput
    /// and JSON-formatted log is sent to `stdout`.
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
            // Technically uses a port to contact plans.
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

/// This type is here so that we do not have to use tracing-appender in every crate.
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
