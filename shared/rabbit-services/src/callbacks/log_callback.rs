use async_trait::async_trait;
use broker::rabbit::channel::RabbitChannelCallback;
use shared_essential::presentation::dto::Source;

use crate::{
    properties::AsezRabbitProperties,
    routing::AsezRabbitRouting,
    services::{log_storage::LogStorageService, AsezRabbitService},
};

/// Коллбэк, который при каждой публикации отправляет запись в сервис `log-storage`.
///
/// Регистрируется через `AsezRabbitService::with_log_callback()`.
///
/// Для корректной записи лога в [`BasicProperties`] публикуемого сообщения должны быть заполнены:
/// - `user_id` (заголовок [`REQUEST_USER_ID_HEADER`]) -- кто инициировал действие;
/// - `message_id` (заголовок [`REQUEST_ID_HEADER`]) -- UUID запроса для сквозной трассировки.
///
/// Если запрос пришёл по HTTP, `message_id` генерируется middleware.
/// Если запрос пришёл по AMQP от другого сервиса, надо скопировать `message_id`
/// из входящего сообщения, иначе цепочка логов прервётся.
#[derive(Clone, Debug)]
pub struct LogStorageCallback {
    service_caller: Source,
}

impl LogStorageCallback {
    pub fn new(service_caller: Source) -> Self {
        LogStorageCallback { service_caller }
    }
}

#[async_trait]
impl RabbitChannelCallback for LogStorageCallback {
    /// Вызывается брокер-адаптером сразу после публикации каждого сообщения.
    ///
    /// Читает `user_id` и `request_id` из заголовков опубликованного сообщения
    /// и формирует запись в `log-storage`. Если заголовки отсутствуют --
    /// запись всё равно отправляется, но с пустыми полями идентификации.
    async fn on_publish(
        &self,
        channel: &broker::rabbit::RabbitChannel,
        basic_props: &amqprs::BasicProperties,
        _publish_args: &amqprs::channel::BasicPublishArguments,
    ) -> broker::Result<()> {
        let asez_basic_props = AsezRabbitProperties::from(basic_props.clone());

        let user_id =
            asez_basic_props.user_id().map(ToOwned::to_owned).unwrap_or_default();
        let message_id = asez_basic_props.request_id().unwrap_or_default();
        let source_id = self.service_caller;

        let dto = LogStorageService::generate_dto(source_id, user_id, message_id);
        channel
            .basic_publish(
                basic_props.clone(),
                LogStorageService::basic_publish_args(AsezRabbitRouting::Log),
                &dto,
                false,
            )
            .await
    }
}
