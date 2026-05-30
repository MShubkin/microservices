//! Клиент RabbitMQ для сервиса хранения логов (log-storage). Используется LogStorageCallback.
use std::sync::Arc;

use broker::{rabbit::RabbitAdapter, BrokerAdapter, Publisher};

use shared_essential::{
    common::AsezTimestamp,
    presentation::dto::{
        log_storage::{LogDataInsert, LogStorageError},
        {AsezResult, Source},
    },
};
use uuid::Uuid;

use crate::{callbacks::AsezCallback, properties::AsezRabbitProperties};

use super::{AsezRabbitRouting, AsezRabbitService};

/// # Описание
///
/// Сервис логов
///
/// # API
/// 1. [`LogStorageService::insert_log`] - Запись лога
#[derive(Debug, Clone)]
pub struct LogStorageService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for LogStorageService {
    const SERVICE: Source = Source::LogStorage;

    fn adapter(&self) -> &RabbitAdapter {
        &self.rabbit_adapter
    }

    fn service_caller(&self) -> Source {
        self.service_caller
    }

    fn callbacks(&self) -> &[AsezCallback] {
        &self.callbacks
    }

    fn with_callback(mut self, callback: AsezCallback) -> Self {
        self.callbacks.push(callback);
        self
    }
}

impl LogStorageService {
    pub fn new(
        rabbit_adapter: Arc<RabbitAdapter>,
        rabbit_properties: AsezRabbitProperties,
        service_caller: Source,
    ) -> Self {
        Self {
            rabbit_adapter,
            rabbit_properties,
            service_caller,
            callbacks: Vec::new(),
        }
    }

    /// # Описание
    ///
    /// Обращение к `log-storage` сервиса для записи лога.
    ///
    /// # Принимает
    /// * `user_id` - Пользователь, который инициировал действие
    /// * `message_id` - Уникальный идентификатор запроса, который будет сопровождаться по всей цепочки запросов
    /// * `basic_props` - Базовые свойства взаимодействия с RabbitMQ
    ///
    /// # Возвращает
    /// * Ok(()) - Ничего не будет возвращено, так как `log-storage` только принимает запросы, но не отвечает на них
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ
    pub async fn insert_log(
        &self,
        user_id: String,
        message_id: Uuid,
    ) -> AsezResult<()> {
        let publish_args = Self::basic_publish_args(AsezRabbitRouting::Log);
        let publisher = self
            .adapter()
            .register_publisher(
                self.rabbit_properties.clone().finish(),
                publish_args,
            )
            .await
            .map_err(LogStorageError::from)?;

        let dto = Self::generate_dto(self.service_caller(), user_id, message_id);

        publisher.publish(&dto).await.map_err(LogStorageError::from)?;
        Ok(())
    }

    pub(crate) fn generate_dto(
        source_id: Source,
        user_id: String,
        message_id: Uuid,
    ) -> LogDataInsert {
        let now = AsezTimestamp::now();
        LogDataInsert {
            user_id,
            // TODO: пока базовый ивент с кроликом обозначается с `event_id=1`, потом это будет переопределено
            event_id: 1,
            request_id: Some(format!("{} {}", message_id, now)),
            source_id,
        }
    }
}

from_request!(LogStorageService);
