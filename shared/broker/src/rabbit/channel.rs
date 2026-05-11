#![allow(clippy::map_identity)]
use std::time::Duration;

use amqprs::channel::{
    BasicAckArguments, BasicConsumeArguments, BasicNackArguments,
    BasicPublishArguments, Channel, ConsumerMessage,
};
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::{Future, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

#[cfg(feature = "traces")]
use tracing::error;

use crate::error::{BrokerError, Result};
use crate::retry::Retryable;
use crate::RetryArgs;

use super::RabbitMessage;

/// Обертка над [amqprs::Channel] канал RabbitMQ
pub struct RabbitChannel {
    /// [amqprs::Channel] канал RabbitMQ
    pub(crate) channel: Channel,
    /// Аргументы для механизма ретрая
    pub(crate) retry_args: RetryArgs,
    /// Коллбэки
    pub(crate) callbacks: Vec<Box<Callback>>,
}

pub type Callback = dyn RabbitChannelCallback + Send;

/// Коллбэк для операций на канале
///
/// Если же коллбэк вернет ошибку, то операция вернет ошибку
///
/// [`DynClone`] используется для того, чтобы коллбэки можно было легко
/// клонировать
#[async_trait]
#[allow(unused_variables)]
pub trait RabbitChannelCallback: Sync {
    /// Коллбэк, который будет вызван при успешной отправке сообщения
    async fn on_publish(
        &self,
        channel: &RabbitChannel,
        basic_props: &BasicProperties,
        publish_args: &BasicPublishArguments,
    ) -> Result<()> {
        Ok(())
    }
}

impl RabbitChannel {
    pub fn new(
        channel: Channel,
        retry_args: RetryArgs,
        callbacks: Vec<Box<Callback>>,
    ) -> Self {
        RabbitChannel {
            channel,
            retry_args,
            callbacks,
        }
    }

    /// Возвращает [amqprs::Channel] канал RabbitMQ
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Регистрация коллбэков для канала
    pub fn register_callback<C>(&mut self, callback: C)
    where
        C: RabbitChannelCallback + Sync + Send + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    /// # Описание
    ///
    /// Извлекает сообщение из очереди
    ///
    /// # Аргументы
    /// * `consume_args` - Аргументы для регистрации консьюмера и консьюма сообщений
    ///
    /// # Возвращает
    /// * Ok([`RabbitMessage<C>`]) - Сообщение успешно получено
    /// * Err([`BrokerError::Internal`]) - Ошибка при получении сообщения из очереди
    /// * Err([`BrokerError::InvalidReceivedMessage`]) - Ошибка при десериализации его содержимого
    /// * Err([`BrokerError::NoSenders`]) - Нет ни единого паблишера для данной очереди
    pub async fn basic_consume<C>(
        &mut self,
        consume_args: BasicConsumeArguments,
    ) -> Result<RabbitMessage<C>>
    where
        C: for<'de> Deserialize<'de>,
    {
        let consume_op = || self.channel.basic_consume_rx(consume_args.clone());
        let (_ctag, mut rx) =
            consume_op.retry(&self.retry_args).await.map_err(|err| {
                #[cfg(feature = "traces")]
                error!(kind = "broker", "Ошибка при получении сообщения: {}", err);
                BrokerError::from(err)
            })?;

        // Должен ли быть отправлен manual acknowledgement?
        let manual_ack = !consume_args.no_ack;
        let res = self.basic_consume_via_receiver(&mut rx, manual_ack).await?;
        Ok(res)
    }

    /// # Описание
    ///
    /// Вспомогательный метод для использования непосредственно из [UnboundedReceiver] с [ConsumerMessage]
    /// в качестве сообщения
    ///
    /// # Аргументы
    /// `rx` - Получатель сообщений
    /// `manual_ack` - Отправить ли manual acknowledgement
    ///
    /// # Возвращает
    /// * Ok([`RabbitMessage<C>`]) - Сообщение успешно получено
    /// * Err([`BrokerError::Internal`]) - Ошибка при получении сообщения из очереди
    /// * Err([`BrokerError::InvalidReceivedMessage`]) - Ошибка при десериализации его содержимого
    /// * Err([`BrokerError::NoSenders`]) - Нет ни единого паблишера для данной очереди
    pub(crate) async fn basic_consume_via_receiver<C>(
        &mut self,
        rx: &mut UnboundedReceiver<ConsumerMessage>,
        manual_ack: bool,
    ) -> Result<RabbitMessage<C>>
    where
        C: for<'de> Deserialize<'de>,
    {
        // None возвращается, когда все половинки-отправители удаляются, указывая на то,
        // что никакие другие значения не могут быть отправлены по каналу.
        // Механизм ретрая здесь не используется, так как ошибка возникает только
        // в случае отсутствия отправителей и механизм ретрая тут не поможет
        let msg = rx.recv().await.ok_or(BrokerError::NoSenders)?;
        // Хотя все поля имеют тип Option<T>, библиотека гарантирует, что когда
        // пользователь получит сообщение от получателя, все поля будут иметь значение Some<T>
        let delivery = msg.deliver.unwrap();
        let content = match serde_json::from_slice(&msg.content.unwrap()) {
            Ok(content) => content,
            Err(_err) => {
                #[cfg(feature = "traces")]
                error!(
                    kind = "broker",
                    "Ошибка при десериализации сообщения, отправление сообщения об ошибке: {:?}", _err
                );

                if manual_ack {
                    self.send_nack(&delivery).await?;
                }

                return Err(BrokerError::InvalidReceivedMessage(_err.to_string()));
            }
        };

        if manual_ack {
            self.send_ack(&delivery).await?;
        }

        let message = RabbitMessage {
            content,
            delivery,
            properties: msg.basic_properties.unwrap(),
        };
        Ok(message)
    }

    /// # Описание
    ///
    /// Вспомогательный метод для использования непосредственно из [UnboundedReceiver] с [ConsumerMessage]
    /// в качестве сообщения
    ///
    /// # Аргументы
    /// `rx` - Получатель сообщений
    /// `manual_ack` - Отправить ли manual acknowledgement
    /// * `millis` - Таймаует в миллисекундах
    ///
    /// # Возвращает
    /// * Ok([`RabbitMessage<C>`]) - Сообщение успешно получено
    /// * Err([`BrokerError::Internal`]) - Ошибка при получении сообщения из очереди
    /// * Err([`BrokerError::WaitingTooLong`]) - Слишком долгое ожидание сообщения
    /// * Err([`BrokerError::InvalidReceivedMessage`]) - Ошибка при десериализации его содержимого
    /// * Err([`BrokerError::NoSenders`]) - Нет ни единого паблишера для данной очереди
    pub(crate) async fn basic_consume_via_receiver_with_timeout<C>(
        &mut self,
        rx: &mut UnboundedReceiver<ConsumerMessage>,
        manual_ack: bool,
        timout: Duration,
    ) -> Result<RabbitMessage<C>>
    where
        C: for<'de> Deserialize<'de>,
    {
        tokio::time::timeout(
            timout,
            self.basic_consume_via_receiver(rx, manual_ack),
        )
        .await
        .map_err(|_err| BrokerError::WaitingTooLong)?
    }

    /// # Описание
    ///
    /// Публикует сообщение в очередь
    ///
    /// # Аргументы
    /// * `basic_props` - Основные свойства для отправки сообщения
    /// * `publish_args` - Аргументы отправления сообщения
    /// * `content` - Любое содержимое, которое может быть сериализовано и отправлено
    /// * `content` - Вызвать ли коллбэки
    ///
    /// # Возвращает
    /// * Ok([`()`]) - Сообщение успешно отправлено
    /// * Err([`BrokerError::Internal`]) - Ошибка при отправке сообщения в очередь
    /// * Err([`BrokerError::InvalidSentMessage`]) - Ошибка при сериализации отправляемых данных
    pub async fn basic_publish<C>(
        &self,
        basic_props: BasicProperties,
        publish_args: BasicPublishArguments,
        content: &C,
        with_callbacks: bool,
    ) -> Result<()>
    where
        C: Serialize,
    {
        let content = serde_json::to_string(&content).map_err(|err| {
            #[cfg(feature = "traces")]
            error!(kind = "broker", "Ошибка при сериализации сообщения: {:?}", err);

            BrokerError::InvalidSentMessage(err.to_string())
        })?;

        let op = || {
            self.channel.basic_publish(
                basic_props.clone(),
                content.clone().into_bytes(),
                publish_args.clone(),
            )
        };
        op.retry(&self.retry_args).await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(
                kind = "broker",
                "Ошибка при публикации сообщения в `{}` пути: {:?}",
                publish_args.routing_key,
                err
            );
            err
        })?;

        if with_callbacks {
            RabbitChannel::handle_callbacks(self.callbacks.iter().map(
                |callback| callback.on_publish(self, &basic_props, &publish_args),
            ))
            .await?;
        }

        Ok(())
    }

    async fn handle_callbacks<I, F>(futs: I) -> Result<()>
    where
        I: IntoIterator<Item = F>,
        F: Future<Output = Result<()>> + Send,
    {
        let mut handles = FuturesUnordered::from_iter(futs.into_iter());

        while let Some(res) = handles.next().await {
            res.map_err(|err| {
                #[cfg(feature = "traces")]
                error!(
                    kind = "broker",
                    "Ошибка при выполнении коллбэка: {:?}", err
                );
                err
            })?;
        }

        Ok(())
    }

    /// # Описание
    ///
    /// Отправляет подтверждение об успешном получении сообщения.
    /// Не забывайте, что [`RabbitChannel::basic_consume`] также может отправлять подтверждение при успешном получении
    /// Сообщения. Двойная отправка подтверждения с тем же [`Deliver`] может привести к ошибке.
    ///
    /// # Аргументы
    ///  `delivery` - Полученные метаданные о доставке
    ///
    /// # Возвращает
    /// * Ok([`()`]) - Подтверждение успешно отправлено
    /// * Err([`BrokerError`]) - Ошибка при отправлении подтверждения
    pub async fn send_ack(&self, delivery: &Deliver) -> Result<()> {
        let op = || {
            self.channel
                .basic_ack(BasicAckArguments::new(delivery.delivery_tag(), false))
        };
        op.retry(&self.retry_args).await.map_err(|_err| {
            #[cfg(feature = "traces")]
            error!(
                kind = "broker",
                "Ошибка при отправлении acknowledgement, delivery={}: {}",
                delivery,
                _err
            );

            BrokerError::SendAck
        })
    }

    /// # Описание
    ///
    /// Отправляет уведомление об ошибке, указывая на то, что сообщение не было доставлено.    
    /// Не забывайте, что [`RabbitChannel::basic_consume`] также может отправить nack,
    /// если произошла непредвиденная ошибка при получении сообщения, например при неудачной десериализации.
    /// Двойная отправка с одним и тем же [`Deliver`] может привести к ошибке.
    ///
    /// # Аргументы
    ///  `delivery` - Полученные метаданные о доставке
    ///
    /// # Возвращает
    /// * Ok([`()`]) - Подтверждение успешно отправлено
    /// * Err([`BrokerError`]) - Ошибка при отправлении подтверждения
    pub async fn send_nack(&self, delivery: &Deliver) -> Result<()> {
        let op = || {
            self.channel.basic_nack(BasicNackArguments::new(
                delivery.delivery_tag(),
                false,
                false,
            ))
        };
        op.retry(&self.retry_args).await.map_err(|_err| {
            #[cfg(feature = "traces")]
            error!(
                kind = "broker",
                "Ошибка при отправлении negative acknowledgement, delivery={}: {}",
                delivery,
                _err
            );

            BrokerError::SendNack
        })
    }

    /// # Описание
    ///
    /// Грейсфул закрытие канала
    ///
    /// # Возвращает
    /// * Ok([`()`]) - Успешное закрытие канала
    /// * Err([`BrokerError`]) - Ошибка при закрытии канала
    pub async fn close(self) -> Result<()> {
        let op = || self.channel.clone().close();
        op.retry(&self.retry_args).await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(kind = "broker", "Ошибка при закрытии канала: {}", err);
            err.into()
        })
    }
}
