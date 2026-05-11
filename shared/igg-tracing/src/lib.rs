//! The rcg-tracing library is designed as a local replacement for gas_tracing.
//!
//! The reason for making this clone is that the maintenance of the code is not
//! certain, and access to it is not complete. This makes creating our own version
//! of the the library a sensible choice.
//!
//! The logs themselves are kept in the CEF Arclight format or some variant
//! thereof. The format presumes several fields separated by the `|` (pipe)
//! character.
//!
//! In principle it should be fairly easy to extend the library to use other
//! tracing formats. The simplest way to achieve this is to make additional
//! `write_event_as_X` calls and a switch `cef::CEFTracingLayer::write_event`.
//! If on the other hand we require more complete changes in format, we can
//! add additional modules.
//!
//! The library is designed to work with applications based around the Actix
//! framework, but this is not required.
//!
//! TODO: Decide whether we want to keep the &str references or use something
//! a little bit more sensible.
use std::net;
use std::{fmt::Debug, net::Ipv4Addr};

use tracing::{
    field::{Field, Visit},
    Level,
};
pub use tracing_subscriber::Registry;

pub mod tracing_fields;

pub mod cef;
// TODO: resolve features in /shared
#[cfg(test)]
mod test;
#[cfg(feature = "with-actix-web")]
pub mod with_actix;

pub use cef::*;
#[cfg(feature = "with-actix-web")]
pub use with_actix::*;

/// Setup logger for development. Don't forget to set `RUST_LOG`
/// env variable for proper tracing filter
pub fn setup_dev_logger() {
    setup_loggers!(tracing_subscriber::fmt::layer())
}

/// The SRM system uses logging on four layers:
/// 1. fmt::layer()
/// 2. json::layer()
/// 3. CEF-authentication layer
/// 4. CEF security layer.
///
/// It is likely that the client will want a similar system in asez2.
/// Therefore a unified macro for all kinds of combination exists for utility.
#[macro_export]
macro_rules! setup_loggers {
    ($($extra_layer:expr),*$(,)?) => {{
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        $crate::Registry::default()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            $(.with($extra_layer))*
            .init()
    }}
}

#[derive(Debug, Clone, Copy)]
/// I am not sure where this is used yet.
pub enum TracingKind {
    User = 1,
    Document = 2,
    Expert = 3,
    /// TODO: Reread original library and see what this was.
    Something = 4,
}

/// This represents some base data that is collected.
///
/// NB2: A `new` function is not made because it does not help in any way here,
/// and default can be used instead.
#[derive(PartialEq, Debug, Clone)]
pub struct ReqBaseData {
    pub category: String,
    pub uri: String,
    /// It is not clear why the original writer chose to use a string format
    /// instead of something from `std::net::`.
    /// This is the `src` field. NB: In some places "source" is used instead.
    /// For example, the field name is "source_ip"
    pub src_ip: String,
    /// This is the `spt` field.
    pub src_port: u16,
    /// This is the `requestClientApplication` field.
    pub user_agent: String,
    /// This is the `suser` field.
    pub user_code: String,
    pub request_id: String,
    pub trace_id: String,
}

impl Default for ReqBaseData {
    fn default() -> Self {
        Self {
            category: "default".to_string(),
            uri: Default::default(),
            src_ip: Ipv4Addr::UNSPECIFIED.to_string(),
            src_port: Default::default(),
            user_agent: Default::default(),
            user_code: "unknown".to_string(),
            request_id: Default::default(),
            trace_id: Default::default(),
        }
    }
}

/// This exists to make creating a `CEFTracingLayer` less ugly.
#[derive(Debug, Clone)]
pub struct ServiceDescription {
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub host: net::IpAddr,
}

impl ReqBaseData {
    /// This is a utility function written for DRY.
    /// NB: If the value is empty, it does NOT overwrite the original field of the
    /// ReqBaseData instance. This mimics the behaviour of the original
    /// `ReqBaseData::update_from` function.
    pub fn record(&mut self, field: &str, v: &str) {
        // We do not rewrite fields with blank values.
        if v.is_empty() {
            return;
        }
        // NB: Port is a number and not recorded here. But perhaps it should be.
        match field {
            "category" => self.category = v.to_owned(),
            "uri" => self.uri = v.to_owned(),
            "source_ip" => self.src_ip = v.to_owned(),
            "user_agent" => self.user_agent = v.to_owned(),
            "user_code" => self.user_code = v.to_owned(),
            "trace_id" => self.trace_id = v.to_owned(),
            // TODO: Check whether we need to replace anything in `trace_id`.
            "request_id" => self.request_id = v.replace('-', ""),
            _ => {}
        }
    }

    /// This function does not need to exist, however, it exists
    /// in order to be able to easily test the `record_u64` function below.
    pub fn record_u16(&mut self, field: &str, v: u16) {
        if v != 0 && field == "source_port" {
            self.src_port = v;
        }
    }
}

impl Visit for ReqBaseData {
    fn record_debug(&mut self, field: &Field, v: &dyn Debug) {
        let fname = field.name();

        self.record(fname, &format!("{:?}", v))
    }

    // Overloading the `record_u64` method. NB: This isn't that safe because one
    // day someone will send us an impossible value.
    /// NB: If the value==0, it does NOT overwrite the original field of the
    /// ReqBaseData instance. This mimics the behaviour of the original
    /// `ReqBaseData::update_from` function.
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_u16(field.name(), value as u16)
    }

    // Overloading the `record_str` method.
    fn record_str(&mut self, field: &Field, v: &str) {
        self.record(field.name(), v)
    }
}

impl ServiceDescription {
    pub fn new(vendor: &str, name: &str, version: &str, host: net::IpAddr) -> Self {
        Self {
            vendor: vendor.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
            host,
        }
    }
}

/// `tracing::Level` has a size of 8 bytes, so we might as well reference.
pub fn event_severity(lvl: &Level) -> u8 {
    match *lvl {
        Level::TRACE => 1,
        Level::DEBUG => 2,
        Level::INFO => 3,
        Level::WARN => 5,
        Level::ERROR => 8,
    }
}
