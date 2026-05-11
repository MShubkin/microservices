//! Cервисы АСЕЗ2, доступные для удаленных вызовов через RabbitMQ.
//!
//! Каждый сервис содержит
//! - Адаптер Rabbit
//! - Идентификатор микросервиса-пользователя
//! - Мета-информацию для вызова (`AsezRabbitProperties`)
//!
//! Для констуирования сервисов в коде, основанном на actix_web,
//! каждый из них реализует [`FromRequest`](actix_web::FromRequest).
//! Для корректной работы этой реализации в запросе должны быть доступны
//! - адаптер Rabbit сервера, как `Arc<RabbitAdapter>`
//! - идентификатор микросервиса-пользователя, например, `Source::EstimatedCommission`.
//!
//! Так же из запроса берется метаинформация в виде `AsezRabbitProperties` (в
//! настоящее время -- только для логгирования). См. [../../../http-middleware/README.md].
//!
//! Создание web-приложения:
//! ```ignore
//! App::new()
//!     ...
//!     .app_data(Source::EstimatedCommission)
//!     .app_data(Data::from(config.rabbit_adapter()))
//! ```
//!
//! Декларация хендлера запроса:
//! ```ignore
//! fn service_handler(
//!     ...
//!     processing: ProcessingService
//!     ...
//! ) -> Result<...> {
//!     ...
//! }
//! ```

macro_rules! from_request {
    ($ty:ident) => {
        from_request!($ty, new);
    };
    ($ty:ident, $new:ident) => {
        impl actix_web::FromRequest for $ty {
            type Error = Box<dyn std::error::Error + 'static>;

            type Future =
                std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

            fn from_request(
                req: &actix_web::HttpRequest,
                _payload: &mut actix_web::dev::Payload,
            ) -> Self::Future {
                let req = req.clone();
                Box::pin(async move {

                    let Some(rabbit_adapter) = req.app_data::<actix_web::web::Data<RabbitAdapter>>() else {
                        return Err("не установлен адаптер кролика".into())
                    };
                    let Some(service_caller) = req.app_data::<shared_essential::presentation::dto::Source>().cloned() else {
                        return Err("не установлен признак исходного сервиса".into())
                    };
                    use actix_web::HttpMessage;
                    let mut rabbit_properties = req.extensions()
                        .get::<AsezRabbitProperties>()
                        .cloned()
                        .unwrap_or_default();
                    if let Some(fields) = req.extensions().get::<igg_tracing::tracing_fields::AsezTracingFieldsCollection>() {
                        rabbit_properties.add_tracing_fields(fields);
                    }
                    Ok(Self::$new(
                        Arc::clone(rabbit_adapter),
                        rabbit_properties,
                        service_caller,
                    ))
                })
            }
        }
    }
}

#[cfg(feature = "integration")]
pub mod integration;
/// Более или менее универсальный.
pub mod log_storage;
#[cfg(feature = "master-data")]
pub mod master_data;
#[cfg(feature = "notification")]
pub mod notification;
#[cfg(feature = "print-doc")]
pub mod print_doc;
#[cfg(feature = "processing")]
pub mod processing;
#[cfg(feature = "scheduler")]
pub mod scheduler;
#[cfg(feature = "specialized-departments")]
pub mod specialized_departments;
#[cfg(feature = "technical-commercial-proposal")]
pub mod technical_commercial_proposal;
#[cfg(feature = "view-storage")]
pub mod view_storage;

use std::time::Duration;

use amqprs::channel::BasicPublishArguments;
use async_trait::async_trait;
use broker::{
    error::Result as BrokerResult,
    rabbit::{adapter::DirectReply, RabbitAdapter, RabbitMessage},
};
use serde::{Deserialize, Serialize};

use shared_essential::presentation::dto::{error::AsezResult, Source};
use uuid::Uuid;

use crate::{
    callbacks::{log_callback::LogStorageCallback, AsezCallback},
    properties::AsezRabbitProperties,
    routing::AsezRabbitRouting,
};

#[async_trait]
pub trait AsezRabbitService {
    /// Наименование сервиса
    const SERVICE: Source;

    /// # Описание
    ///
    /// Обращение к сервису АСЭЗ 2.0 по AMQP, используя RabbitMQ.
    /// Определяет общее использование RPC паттерна
    ///
    /// # Аргументы
    /// * `dto` - Отправляемое другому сервису тело сообщения
    /// * `routing_key` - Очередь, куда будет отправлено сообщение
    /// * `basic_props` - Базовые свойства взаимодействия с RabbitMQ
    /// * `timeout` - Таймаут, по истечении которого запрос будет принудительно закончен
    ///
    /// # Возвращает
    /// * Ok([`RabbitMessage<AsezResult<R>>`]) - Успешное получение ответа от сервиса
    /// * Err([`BrokerError`]) - Что-то пошло не так при отправке или получении сообщения
    async fn service_request<T, R>(
        &self,
        dto: T,
        basic_props: AsezRabbitProperties,
        routing_key: AsezRabbitRouting,
        timeout: Duration,
    ) -> BrokerResult<RabbitMessage<AsezResult<R>>>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de> + Send + Sync,
    {
        let mut direct_reply =
            self.setup_direct_reply(routing_key, basic_props).await?;
        direct_reply.request(&dto, timeout).await
    }

    /// # Описание
    ///
    /// Настройка Direct-Reply механизма для RPC паттерна при
    /// испрользовании RabbitMQ
    ///
    /// # Аргументы
    /// * `routing_key` - Отправляемое другому сервису тело сообщения
    /// * `basic_props` - Базовые свойства взаимодействия с RabbitMQ
    ///
    /// # Возвращает
    /// * Ok([`DirectReply`]) - Успешная регистрация Direct-Reply механизма
    /// * Err([`BrokerError`]) - Что-то пошло не так при регистрации консьюмера или паблишера
    async fn setup_direct_reply(
        &self,
        routing_key: AsezRabbitRouting,
        basic_props: AsezRabbitProperties,
    ) -> BrokerResult<DirectReply> {
        let publish_args = Self::basic_publish_args(routing_key);
        let rabbit_adapter = self.adapter();
        let consumer_tag = self.consumer_tag();

        let mut direct_reply = rabbit_adapter
            .direct_reply(basic_props.finish(), publish_args, &consumer_tag)
            .await?;

        self.register_callbacks(&mut direct_reply);

        Ok(direct_reply)
    }

    /// Регистрация коллбэков
    fn register_callbacks(&self, direct_reply: &mut DirectReply) {
        for callback in self.callbacks() {
            match callback {
                AsezCallback::LogStorageCallback => direct_reply
                    .register_publisher_callback(LogStorageCallback::new(
                        self.service_caller(),
                    )),
            }
        }
    }

    /// Базовые свойства для отправки AMQP сообщений в RabbitMQ
    fn basic_publish_args(routing_key: AsezRabbitRouting) -> BasicPublishArguments {
        let (exchange, routing_key) = routing_key.as_full_routing();
        BasicPublishArguments::new(exchange, routing_key)
    }

    /// Создание консьюмер тэга для идентификации консьюмера.
    /// Тэг создается в формате `{ВЫЗЫВАЮЩИЙ_СЕРВИС}-{ВЫЗЫВАЕМЫЙ СЕРВИС}-consumer-{UUID}`
    fn consumer_tag(&self) -> String {
        format!(
            "{}<-{}-consumer-{}",
            self.service_caller(),
            Self::SERVICE,
            Uuid::new_v4()
        )
    }

    /// Определение того, кто вызывает имплементируемый сервис
    fn service_caller(&self) -> Source;

    /// Получение [`RabbitAdapter`] для взаимодействия с имплементируемым
    /// сервисом
    fn adapter(&self) -> &RabbitAdapter;

    /// Список коллбэков, которые должен регистрировать сервис
    /// при каждом запросе
    fn callbacks(&self) -> &[AsezCallback];

    /// Сеттер коллбэка
    fn with_callback(self, callback: AsezCallback) -> Self;

    /// Сеттер [`LogStorageCallback`] коллбэка
    fn with_log_callback(self) -> Self
    where
        Self: Sized,
    {
        self.with_callback(AsezCallback::LogStorageCallback)
    }
}
