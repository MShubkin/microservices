//! Клиент RabbitMQ для сервиса уведомлений (Notifications).
use std::{sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;

use shared_essential::presentation::dto::{notification::*, AsezResult, Source};

use crate::callbacks::AsezCallback;
use crate::properties::AsezRabbitProperties;

use super::{AsezRabbitRouting, AsezRabbitService};

/// # Описание
///
/// Сервис уведомлений
///
/// # API
/// 1. [`NotificationService::send_notification`] - отправление уведомления
#[derive(Debug, Clone)]
pub struct NotificationService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for NotificationService {
    const SERVICE: Source = Source::Notification;

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

impl NotificationService {
    const DEFAULT_TIMEOUT: u64 = 5_000;

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
    /// Отправление уведомления
    ///
    /// # Возвращает
    /// * Ok([`SendNotificationResponse`]) - Успешное отправление уведомления
    /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса в `notification`
    pub async fn send_notifications(
        &self,
        dto: &SendNotificationReq,
    ) -> AsezResult<SendNotificationResponse> {
        // Большой таймаут связан с тем, что уведомление отправляется через SMTP, что ужасно медленно
        let response = self
            .service_request(
                dto,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Notifications,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(NotificationError::from)?;
        response.content
    }
}

from_request!(NotificationService);
