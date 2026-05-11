#![allow(clippy::map_identity)]
use std::time::Duration;

use amqprs::{channel::ConsumerMessage, BasicProperties, Deliver};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::error::Result;
use crate::Consumer;

use super::RabbitChannel;

/// Базовый долгоживущий консьюмер RabbitMQ
pub struct RabbitConsumer {
    /// Канал для получения сообщений
    rx: UnboundedReceiver<ConsumerMessage>,
    /// Канал для общения с RabbitMQ сервером
    channel: RabbitChannel,
    /// Мануальное отправление acknowledgement
    manual_ack: bool,
}

impl RabbitConsumer {
    pub fn new(
        rx: UnboundedReceiver<ConsumerMessage>,
        channel: RabbitChannel,
        manual_ack: bool,
    ) -> Self {
        Self {
            rx,
            channel,
            manual_ack,
        }
    }

    /// Возвращает [`RabbitChannel`], который является оберткой над [`amqprs::Channel`]
    pub fn channel(&self) -> &RabbitChannel {
        &self.channel
    }
    /// Возвращает [`RabbitChannel`], который является оберткой над [`amqprs::Channel`]
    pub fn channel_mut(&mut self) -> &mut RabbitChannel {
        &mut self.channel
    }
    pub fn rx(&self) -> &UnboundedReceiver<ConsumerMessage> {
        &self.rx
    }
}

/// Сообщение RabbitMQ, содержащее контент, метадату доставки и метаданные
#[derive(Debug)]
pub struct RabbitMessage<C> {
    /// Контент сообщения
    pub content: C,
    /// Информация о доставке
    pub delivery: Deliver,
    /// Метаданные сообщения
    pub properties: BasicProperties,
}

#[async_trait]
impl<C> Consumer<C> for RabbitConsumer
where
    C: for<'de> Deserialize<'de> + Send + Sync,
{
    type Message = RabbitMessage<C>;

    async fn consume(&mut self) -> Result<Self::Message> {
        let message = self
            .channel
            .basic_consume_via_receiver(&mut self.rx, self.manual_ack)
            .await?;
        Ok(message)
    }

    async fn consume_with_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Self::Message> {
        let message = self
            .channel
            .basic_consume_via_receiver_with_timeout(
                &mut self.rx,
                self.manual_ack,
                timeout,
            )
            .await?;
        Ok(message)
    }

    async fn send_ack(&self, message: &Self::Message) -> Result<()> {
        self.channel.send_ack(&message.delivery).await
    }

    async fn send_nack(&self, message: &Self::Message) -> Result<()> {
        self.channel.send_nack(&message.delivery).await
    }
}
