#![allow(clippy::map_identity)]
use std::time::Duration;

use amqprs::{channel::ConsumerMessage, BasicProperties, Deliver};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::error::Result;
use crate::Consumer;

use super::RabbitChannel;

/// Долгоживущий консьюмер RabbitMQ.
///
/// `rx` получает сообщения от `amqprs` через `mpsc`-канал: библиотека
/// доставляет входящие фреймы во внутренний таск и кладёт результат в этот receiver.
/// `channel` хранится рядом, чтобы отправлять ack/nack по тому же AMQP-каналу,
/// через который пришло сообщение — AMQP требует, чтобы ack шёл по тому же каналу.
pub struct RabbitConsumer {
    /// Получатель сообщений от внутреннего `amqprs`-таска.
    rx: UnboundedReceiver<ConsumerMessage>,
    /// AMQP-канал для отправки ack/nack и взаимодействия с брокером.
    channel: RabbitChannel,
    /// Если `false` — брокер сам считает сообщение подтверждённым после доставки (no-ack режим).
    /// Если `true` — нужно явно вызвать `send_ack` или `send_nack`.
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

/// Входящее AMQP-сообщение с уже десериализованным payload.
///
/// `delivery` содержит `delivery_tag`, который нужен для ack/nack.
/// `properties` даёт доступ к `correlation_id`, `reply_to` и другим заголовкам AMQP —
/// они необходимы на стороне RPC-сервера для формирования ответа.
#[derive(Debug)]
pub struct RabbitMessage<C> {
    /// Десериализованное тело сообщения.
    pub content: C,
    /// Метаданные доставки: `delivery_tag`, `exchange`, `routing_key`.
    pub delivery: Deliver,
    /// Заголовки AMQP: `content_type`, `reply_to`, `correlation_id`, `expiration` и др.
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
