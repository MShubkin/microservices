use std::env::{var, VarError};

use amqprs::connection::OpenConnectionArguments;
use broker::rabbit::RabbitAdapter;
use broker::{BrokerAdapter, BrokerError};
use serde::Deserialize;

/// Конфигурация для тестирования взаимодействия с RabbitMQ
#[derive(Deserialize, Debug)]
pub struct TestRabbitConfig {
    /// Порт, на котором открыт сервер RabbitMQ
    rabbit_port: u16,
    /// Адрес сервера RabbitMQ
    rabbit_addr: String,
    /// Пользователь
    username: String,
    /// Пароль
    password: String,
}

pub fn get_config() -> Result<TestRabbitConfig, VarError> {
    let rabbit_addr = var("RABBITMQ_HOST")?;
    let rabbit_port = var("RABBITMQ_PORT")?
        .parse()
        .expect("RABBIT_PORT env variable has wrong format");
    let username = var("RABBITMQ_USERNAME")?;
    let password = var("RABBITMQ_PASSWORD")?;

    Ok(TestRabbitConfig {
        rabbit_port,
        rabbit_addr,
        username,
        password,
    })
}

pub async fn connect(
    config: TestRabbitConfig,
) -> Result<RabbitAdapter, BrokerError> {
    let connection_args = OpenConnectionArguments::new(
        &config.rabbit_addr,
        config.rabbit_port,
        &config.username,
        &config.password,
    );
    let rabbit_adapter =
        RabbitAdapter::connect(connection_args, Default::default()).await?;

    Ok(rabbit_adapter)
}
