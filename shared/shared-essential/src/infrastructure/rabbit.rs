//! Модуль отвечает за настройку взаимодействия с RabbitMQ сервером
use amqprs::connection::OpenConnectionArguments;
use broker::{
    error::Result as BrokerResult, rabbit::RabbitAdapter, BrokerAdapter, RetryArgs,
};

use env_setup::RabbitCfg;

/// # Описание
///
/// Создание [адаптера RabbitMQ](RabbitAdapter)
///
/// # Аргументы
/// * `config` - [Конфиг](RabbitCfg) для создания [адаптера](RabbitAdapter) RabbitMQ
///
/// # Возвращает
/// * Ok([`RabbitAdapter`]) - Успешное создание [адаптера](RabbitAdapter) RabbitMQ
/// * Err([`BrokerError`]) - Ошибка при создании соединения с RabbitMQ сервером
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
