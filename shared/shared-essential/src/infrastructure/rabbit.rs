//! Модуль отвечает за настройку взаимодействия с RabbitMQ сервером
use amqprs::connection::OpenConnectionArguments;
use broker::{
    error::Result as BrokerResult, rabbit::RabbitAdapter, BrokerAdapter, RetryArgs,
};

use env_setup::RabbitCfg;

/// Создаёт подключение к RabbitMQ с повторными попытками.
///
/// `RetryArgs::default()` берёт `retries`/`retry_interval_ms` из конфига.
/// Это нужно при старте в Docker Compose, где RabbitMQ поднимается позже сервиса.
pub async fn setup_rabbit_adapter(
    config: &RabbitCfg,
) -> BrokerResult<RabbitAdapter> {
    let connection_args = OpenConnectionArguments::new(
        &config.host,
        config.port,
        &config.user,
        &config.pw,
    )
    .virtual_host(&config.vhost)
    .finish();

    let rabbit_adapter =
        RabbitAdapter::connect(connection_args, RetryArgs::default()).await?;

    Ok(rabbit_adapter)
}
