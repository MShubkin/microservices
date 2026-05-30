//! Типизированные RPC-клиенты для сервисов АСЭЗ 2.0 через RabbitMQ.
//!
//! Каждый сервис хранит три вещи:
//! - [`RabbitAdapter`] -- подключение к брокеру;
//! - [`Source`] -- идентификатор вызывающего микросервиса (нужен для логирования);
//! - [`AsezRabbitProperties`] -- метаданные запроса (заголовки трассировки).
//!
//! RPC реализован через механизм Direct Reply-to: при каждом вызове
//! `service_request` на канале временно регистрируется консьюмер на
//! псевдоочередь `amq.rabbitmq.reply-to`. Брокер возвращает ответ в эту же
//! псевдоочередь без создания реальной очереди, что исключает накопление мусора.
//!
//! Чтобы сервис можно было получить прямо в Actix-хэндлере через аргумент функции,
//! макрос `from_request!` генерирует реализацию [`FromRequest`](actix_web::FromRequest).
//! Для этого в `App::new()` должны быть зарегистрированы два объекта:
//! - `Arc<RabbitAdapter>` -- через `Data::from(config.rabbit_adapter())`;
//! - `Source` -- идентификатор сервиса-хозяина, например `Source::EstimatedCommission`.
//!
//! Пример регистрации:
//! ```ignore
//! App::new()
//!     .app_data(Source::EstimatedCommission)
//!     .app_data(Data::from(config.rabbit_adapter()))
//! ```
//!
//! Пример хэндлера:
//! ```ignore
//! async fn my_handler(processing: ProcessingService) -> Result<...> {
//!     processing.call_something(...).await
//! }
//! ```

/// Генерирует реализацию `actix_web::FromRequest` для указанного типа сервиса.
///
/// Форма `from_request!(Foo)` использует конструктор `Foo::new`.
/// Форма `from_request!(Foo, custom_new)` позволяет указать другой конструктор.
///
/// Реализация извлекает из запроса:
/// 1. `Arc<RabbitAdapter>` -- обязательно, иначе ошибка при старте хэндлера.
/// 2. `Source` -- идентификатор вызывающего сервиса, обязательно.
/// 3. `AsezRabbitProperties` -- метаданные трассировки; если отсутствуют, берётся `Default`.
///    Затем в них дописываются поля `AsezTracingFieldsCollection`, если они есть в расширениях запроса.
macro_rules! from_request {
    ($ty:ident) => {
        from_request!($ty, new);
    };
    ($ty:ident, $new:ident) => {
        impl actix_web::FromRequest for $ty {
            type Error = actix_web::Error;

            type Future =
                std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

            fn from_request(
                req: &actix_web::HttpRequest,
                _payload: &mut actix_web::dev::Payload,
            ) -> Self::Future {
                let req = req.clone();
                Box::pin(async move {
                    // Отсутствие любой из требуемых вещей в `app_data` — это ошибка конфигурации
                    // сервиса (забыли `.app_data(...)` в `App::new()`), а не клиентская ошибка.
                    // Поэтому отдаём 500 без внутренней диагностики наружу, а причину пишем в лог.
                    let Some(rabbit_adapter) = req.app_data::<actix_web::web::Data<RabbitAdapter>>() else {
                        tracing::error!(
                            kind = "infra",
                            service = stringify!($ty),
                            "RabbitAdapter не зарегистрирован в App::app_data",
                        );
                        return Err(actix_web::error::ErrorInternalServerError("service unavailable"));
                    };
                    let Some(service_caller) = req.app_data::<shared_essential::presentation::dto::Source>().cloned() else {
                        tracing::error!(
                            kind = "infra",
                            service = stringify!($ty),
                            "Source (service_caller) не зарегистрирован в App::app_data",
                        );
                        return Err(actix_web::error::ErrorInternalServerError("service unavailable"));
                    };
                    use actix_web::HttpMessage;
                    // AsezRabbitProperties может быть заполнен HTTP-middleware;
                    // если его нет -- создаём дефолтный (персистентный JSON).
                    let mut rabbit_properties = req.extensions()
                        .get::<AsezRabbitProperties>()
                        .cloned()
                        .unwrap_or_default();
                    // Дописываем поля трассировки, если middleware их положил в расширения запроса.
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
/// Клиент для отправки записей в сервис хранения логов.
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

/// Трейт типизированного RPC-клиента к сервису АСЭЗ 2.0 через RabbitMQ.
///
/// Реализован на основе паттерна Direct Reply-to: вместо создания временной очереди
/// используется псевдоочередь `amq.rabbitmq.reply-to`. Брокер сам возвращает ответ
/// на канал, который отправил запрос, не создавая никаких реальных очередей.
///
/// Каждый вызов `service_request` регистрирует временный консьюмер с уникальным тегом,
/// ждёт ответа с заданным таймаутом и снимает консьюмера. Параллельные вызовы
/// не мешают друг другу, так как теги уникальны (UUID-суффикс).
///
/// Конкретные сервисы (MasterDataService, ProcessingService и т.д.) реализуют
/// этот трейт и добавляют типизированные методы поверх `service_request`.
#[async_trait]
pub trait AsezRabbitService {
    /// Константа с идентификатором целевого сервиса.
    const SERVICE: Source;

    /// Выполняет типизированный RPC-вызов к удалённому сервису.
    ///
    /// Сериализует `dto` в JSON, публикует в очередь `routing_key`,
    /// подписывается на `amq.rabbitmq.reply-to` и ждёт ответа не дольше `timeout`.
    /// Ответ приходит в виде `RabbitMessage<AsezResult<R>>` -- обёртка содержит
    /// свойства ответного сообщения и уже десериализованный результат.
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

    /// Настраивает механизм Direct Reply-to перед отправкой запроса.
    ///
    /// Регистрирует паблишер и консьюмер на `amq.rabbitmq.reply-to`, затем
    /// подключает все заданные коллбэки (например, [`LogStorageCallback`]).
    /// Вызывается внутри `service_request` -- использовать напрямую не нужно.
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

    /// Подключает все зарегистрированные коллбэки к экземпляру DirectReply.
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

    /// Строит аргументы публикации из маршрута: extract exchange + queue и оборачивает в `BasicPublishArguments`.
    fn basic_publish_args(routing_key: AsezRabbitRouting) -> BasicPublishArguments {
        let (exchange, routing_key) = routing_key.as_full_routing();
        BasicPublishArguments::new(exchange, routing_key)
    }

    /// Формирует уникальный тег консьюмера вида `{caller}<-{service}-consumer-{uuid}`.
    ///
    /// UUID-суффикс гарантирует, что одновременные RPC-вызовы одного и того же
    /// сервиса не перехватят чужие ответы.
    fn consumer_tag(&self) -> String {
        format!(
            "{}<-{}-consumer-{}",
            self.service_caller(),
            Self::SERVICE,
            Uuid::new_v4()
        )
    }

    /// Возвращает идентификатор вызывающего сервиса (нужен для логирования и тега консьюмера).
    fn service_caller(&self) -> Source;

    /// Возвращает ссылку на адаптер RabbitMQ.
    fn adapter(&self) -> &RabbitAdapter;

    /// Возвращает список коллбэков, регистрируемых при каждом запросе.
    fn callbacks(&self) -> &[AsezCallback];

    /// Добавляет коллбэк к сервису (builder-паттерн).
    fn with_callback(self, callback: AsezCallback) -> Self;

    /// Добавляет [`LogStorageCallback`] -- автоматически логирует каждую публикацию.
    fn with_log_callback(self) -> Self
    where
        Self: Sized,
    {
        self.with_callback(AsezCallback::LogStorageCallback)
    }
}
