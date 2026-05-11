#![allow(clippy::map_identity)]
use amqprs::{channel::BasicPublishArguments, BasicProperties};
use async_trait::async_trait;
use serde::Serialize;
use std::time::Duration;

use crate::error::Result;
use crate::Publisher;

use super::channel::RabbitChannelCallback;
use super::RabbitChannel;

/// Базовый долгоживущий паблишер rabbit
pub struct RabbitPublisher {
    /// Канал для общения с RabbitMQ сервером
    channel: RabbitChannel,
    /// Основные свойства
    basic_props: BasicProperties,
    /// Аргументы публикации
    publish_args: BasicPublishArguments,
}

impl RabbitPublisher {
    pub fn new(
        channel: RabbitChannel,
        basic_props: BasicProperties,
        publish_args: BasicPublishArguments,
    ) -> Self {
        Self {
            basic_props,
            channel,
            publish_args,
        }
    }

    /// Возвращает [`RabbitChannel`], который является оберткой над [`amqprs::Channel`]
    pub fn channel(&self) -> &RabbitChannel {
        &self.channel
    }

    /// Используется для получения отладочной информации об основных свойствах.
    pub fn basic_props(&self) -> &BasicProperties {
        &self.basic_props
    }

    /// Используется для получения отладочной информации об аргументах публикации.
    pub fn publish_args(&self) -> &BasicPublishArguments {
        &self.publish_args
    }

    /// Регистрация коллбэков для паблишера
    pub fn register_callback<C>(&mut self, callback: C)
    where
        C: RabbitChannelCallback + Sync + Send + 'static,
    {
        self.channel.register_callback(callback);
    }
}

#[async_trait]
impl<C> Publisher<C> for RabbitPublisher
where
    C: Serialize + Send + Sync,
{
    async fn publish(&self, content: &C) -> Result<()> {
        self.channel
            .basic_publish(
                self.basic_props.clone(),
                self.publish_args.clone(),
                content,
                true,
            )
            .await
    }
}

impl RabbitPublisher {
    /// Для DirectReply лучше чтобы был срок действия сообщения,
    /// так как он сам даёт timeout.
    pub async fn publish_with_expiration<C>(
        &self,
        content: &C,
        expiration: Duration,
    ) -> Result<()>
    where
        C: Serialize + Send + Sync,
    {
        // Изменился подход: Сообщение НЕ должно жить дольше чем таймаут.
        let expiration = (expiration.as_millis()).to_string();

        let msg_props =
            self.basic_props.clone().with_expiration(&expiration).finish();

        self.channel
            .basic_publish(msg_props, self.publish_args.clone(), content, true)
            .await
    }
}
