use super::Result;

use amqprs::connection::OpenConnectionArguments;
use broker::rabbit::RabbitAdapter;
use broker::{BrokerAdapter, RetryArgs};
use serde::Deserialize;

/// A deserializable structure representing the connection settings of
/// our dear rabbit.
/// We would have to use a different config for a different broker,
/// so this code is not universal.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RabbitConfig {
    host: String,
    vhost: String,
    port: u16,
    user: String,
    pw: String,
    retries: u32,
    retry_interval_ms: u64,
}

impl From<env_setup::RabbitCfg> for RabbitConfig {
    fn from(o: env_setup::RabbitCfg) -> Self {
        Self {
            host: o.host,
            vhost: o.vhost,
            port: o.port,
            user: o.user,
            pw: o.pw,
            retries: o.retries,
            retry_interval_ms: o.retry_interval_ms,
        }
    }
}

impl RabbitConfig {
    /// Create the adaptor from the config.
    pub(crate) async fn get_rabbit(&self) -> Result<RabbitAdapter> {
        let args = OpenConnectionArguments::new(
            &self.host, self.port, &self.user, &self.pw,
        )
        .virtual_host(&self.vhost)
        .finish();
        let retry_args = RetryArgs::new(self.retries, self.retry_interval_ms);
        RabbitAdapter::connect(args, retry_args).await.map_err(|e| {
            tracing::error!(
                kind = "broker",
                "Processing error when connecting to RabbitMQ: {}",
                e
            );
            e.into()
        })
    }
}
