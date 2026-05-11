use std::{
    env::{var, VarError},
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};

/// Конфигурация highload теста
#[derive(Deserialize, Debug)]
pub struct TestConfig {
    /// Количество запросов
    pub request_count: u64,
    /// Время обработки запроса
    pub process_time_ms: u64,
    /// Время интервала между запросами в миллисекундах
    pub request_interval_ms: u64,
    /// Количество каналов
    pub channel_count: u64,
    /// Конфигурация для взаимодействия с RabbitMQ
    pub rabbit_config: RabbitConfig,
}

/// Конфигурация для взаимодействия с RabbitMQ
/// сервером при highload тестах
#[derive(Deserialize, Debug)]
pub struct RabbitConfig {
    /// Порт RabbitMQ сервера
    pub port: u16,
    /// Адрес RabbitMQ сервера
    pub addr: String,
    /// Пользователь RabbitMQ сервера
    pub username: String,
    /// Пароль пользователя RabbitMQ сервера
    pub password: String,
    /// Наименование очереди для highload теста
    pub queue_name: String,
}

/// Получение конфигурации из .env файла, название которого передается
/// при запуске теста
pub fn get_config() -> Result<TestConfig, VarError> {
    let request_count = var("REQUEST_COUNT")?
        .parse()
        .expect("`REQUEST_COUNT` переменная имеет неверный формат");
    let process_time_ms = var("REQUEST_INTERVAL_MS")?
        .parse()
        .expect("`REQUEST_INTERVAL_MS` переменная имеет неверный формат");
    let request_interval_ms = var("PROCESS_TIME_MS")?
        .parse()
        .expect("`PROCESS_TIME_MS` переменная имеет неверный формат");
    let channel_count = var("CHANNEL_COUNT")?
        .parse()
        .expect("`CHANNEL_COUNT` переменная имеет неверный формат");
    let port = var("RABBITMQ_PORT")?
        .parse()
        .expect("`RABBITMQ_PORT` переменная имеет неверный формат");
    let addr = var("RABBIT_RABBITMQ_VHOSTADDR")?
        .parse()
        .expect("`RABBITMQ_VHOST` переменная имеет неверный формат");
    let username = var("RABBITMQ_USERNAME")?;
    let password = var("RABBITMQ_PASSWORD")?;
    let queue_name = var("QUEUE_NAME")?;

    Ok(TestConfig {
        request_count,
        process_time_ms,
        request_interval_ms,
        channel_count,
        rabbit_config: RabbitConfig {
            port,
            addr,
            username,
            password,
            queue_name,
        },
    })
}

/// Тестовая структура
#[derive(Deserialize, Serialize, Debug)]
pub struct TestReq {
    pub count: u64,
}

impl TestReq {
    #[allow(dead_code)]
    pub fn new(count: u64) -> Self {
        Self { count }
    }
}

/// Отслеживаемый стейт теста
pub struct HighloadTestState {
    /// Максимальное время обработки действия
    max: AtomicU64,
    /// Минимальное время обработки действия
    min: AtomicU64,
    /// Общее время выполнения теста
    sum: AtomicU64,
    /// Общее количество выполненных действий
    count: AtomicU64,
}

impl Default for HighloadTestState {
    fn default() -> Self {
        Self {
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }
}

impl HighloadTestState {
    pub fn update(&self, val: u64, order: Ordering) {
        self.max.fetch_max(val, order);
        self.min.fetch_min(val, order);
        self.sum.fetch_add(val, order);
        self.count.fetch_add(1, order);
    }

    pub fn avg(&self, order: Ordering) -> u64 {
        self.sum.load(order) / self.count.load(order)
    }
}

impl Debug for HighloadTestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "* MIN время запрос-ответ - {}\n\
            * MAX время запрос-ответ - {}\n\
            * AVG время запрос-ответ - {}",
            &self.min.load(Ordering::Relaxed),
            &self.max.load(Ordering::Relaxed),
            &self.avg(Ordering::Relaxed)
        )
    }
}
