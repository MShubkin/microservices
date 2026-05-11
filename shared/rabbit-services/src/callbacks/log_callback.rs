use async_trait::async_trait;
use broker::rabbit::channel::RabbitChannelCallback;
use shared_essential::presentation::dto::Source;

use crate::{
    properties::AsezRabbitProperties,
    routing::AsezRabbitRouting,
    services::{log_storage::LogStorageService, AsezRabbitService},
};

/// # Описание
///
/// Используется для отправки лога в сервис `log-storage`
/// при отправке сообщения паблишером
///
/// # Важно
///
/// Перед отправкой в [`amqprs::BasicProperties`] обязательно надо
/// указать `user_id` и `message_id`. Если сообщение только только пришло
/// по хттп от фронта, то `message_id` генерируется вручную. Если же
/// к сервису пришел запрос от другого сервиса по amqp, то он должен взять
/// этот `message_id` из запроса сервиса и вставить его в [`amqprs::BasicProperties`],
/// если предполагается, что он будет делать запрос в другой сервис
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
