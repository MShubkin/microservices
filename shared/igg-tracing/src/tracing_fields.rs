use std::{fmt::Display, net::SocketAddr, ops::Deref, str::FromStr};
use time::OffsetDateTime;
use uuid::Uuid;

pub const USER_ID: &str = "asez.user_id";
pub const USER_NAME: &str = "asez.user_name";
pub const REQUEST_ID: &str = "asez.request_id";
pub const OBJECT_IDS: &str = "asez.object_ids";
pub const OBJECT_UUIDS: &str = "asez.object_uuids";
pub const CATEGORY: &str = "asez.category";
pub const URI: &str = "asez.uri";
pub const USER_AGENT: &str = "asez.user_agent";
pub const SOURCE_IP: &str = "asez.source_ip";
pub const SOURCE_PORT: &str = "asez.source_port";
pub const TIMESTAMP: &str = "asez.timestamp";

/// Unique request id.
///
/// Might be some user friendly, like String (e.g. uri + timestamp), or Uuid.
pub type RequestId = Uuid;

/// Utility type that wraps vec of items and displays them comma-separated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommaList<T>(pub Vec<T>);

const SEPARATOR: &str = ",";

impl<T> Display for CommaList<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sep = "";
        for item in &self.0 {
            f.write_str(sep)?;
            sep = SEPARATOR;
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl<T> Default for CommaList<T> {
    fn default() -> Self {
        Self(Vec::with_capacity(1))
    }
}

impl<T> Deref for CommaList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: FromStr> FromStr for CommaList<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s.split(SEPARATOR).map(str::parse).collect::<Result<_, _>>()?;
        Ok(CommaList(items))
    }
}

/// Collection of data that is used for tracing events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsezTracingFieldsCollection {
    pub request_id: RequestId,
    pub uri: String,
    pub source: SocketAddr,
    pub user_agent: String,

    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub object_ids: CommaList<i64>,
    pub object_uuids: CommaList<Uuid>,
    pub timestamp: OffsetDateTime,
}

impl AsezTracingFieldsCollection {
    pub fn new(uri: String, source: SocketAddr, user_agent: String) -> Self {
        let request_id = AsezTracingFieldsCollection::new_request_id();
        AsezTracingFieldsCollection {
            request_id,
            uri,
            source,
            user_agent,
            user_id: None,
            user_name: None,
            object_ids: Default::default(),
            object_uuids: Default::default(),
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    fn new_request_id() -> RequestId {
        Uuid::new_v4()
    }

    pub fn set_request_id(&mut self, request_id: RequestId) {
        self.request_id = request_id;
    }

    pub fn set_user_id(&mut self, user_id: i32) -> &mut Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn set_user_name(&mut self, user_name: String) -> &mut Self {
        self.user_name = Some(user_name);
        self
    }

    pub fn set_object_ids(&mut self, object_ids: CommaList<i64>) -> &mut Self {
        self.object_ids = object_ids;
        self
    }

    pub fn set_object_uuids(&mut self, object_uuids: CommaList<Uuid>) -> &mut Self {
        self.object_uuids = object_uuids;
        self
    }
}

#[macro_export]
macro_rules! span_with_fields {
    ($fields:expr, $name:expr) => {
        if let Some(fields) = $fields {
            tracing::info_span!(
                $name,
                "asez.request_id" = %fields.request_id,
                "asez.uri" = %fields.uri,
                "asez.user_agent" = %fields.user_agent,
                "asez.source_ip" = %fields.source.ip(),
                "asez.source_port" = %fields.source.port(),
                "asez.timestamp" = %fields.timestamp,
                "asez.user_id" = %fields.user_id.unwrap_or_default(),
                "asez.user_name" = %fields.user_name.unwrap_or_else(|| "<unknown>".to_string()),
                "asez.object_ids" = %fields.object_ids.to_string(),
                "asez.object_uuids" = %fields.object_uuids.to_string(),
            )
        } else {
            tracing::info_span!(
                $name
            )
        }
    };
}
