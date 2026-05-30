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

/// Обёртка над [`amqprs::Channel`] с ретраем, коллбэками и удобными методами ack/nack/publish.
///
/// Поля `pub(crate)`, а не `pub`, чтобы `consumer` и `publisher` могли читать их напрямую,
/// но внешний код не мог нарушить инварианты (например, подменить `retry_args` на ходу).
pub struct RabbitChannel {
    /// Базовый AMQP-канал из `amqprs`.
    pub(crate) channel: Channel,
    /// Аргументы ретрая унаследованы от `RabbitAdapter` при открытии канала.
    pub(crate) retry_args: RetryArgs,
    /// Коллбэки, вызываемые после каждой успешной публикации.
    /// Хранятся как `Box<dyn Trait>`, чтобы поддерживать разные типы коллбэков одновременно.
    pub(crate) callbacks: Vec<Box<Callback>>,
}

/// Псевдоним для удобства: убирает дублирование `dyn RabbitChannelCallback + Send`.
pub type Callback = dyn RabbitChannelCallback + Send;

/// Хук, вызываемый после публикации сообщения в канал.
///
/// Если коллбэк вернёт `Err`, то `basic_publish` тоже вернёт `Err` —
/// это позволяет реализовывать логику вроде логирования метрик с возможностью
/// сигнализировать об ошибке.
///
/// Метод `on_publish` имеет дефолтную реализацию `Ok(())`, чтобы не обязывать
/// каждого имплементора переопределять все хуки.
#[async_trait]
#[allow(unused_variables)]
pub trait RabbitChannelCallback: Sync {
    /// Вызывается после того, как сообщение успешно отправлено в брокер.
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
    /// Конструктор намеренно `pub(crate)`: канал должен создаваться только через
    /// [`RabbitAdapter`](super::RabbitAdapter), который наследует свои `retry_args`
    /// и контролирует жизненный цикл базового `amqprs::Channel`. Внешнее создание
    /// позволило бы обойти политику ретрая (например, `attempts = 0`).
    pub(crate) fn new(
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
        // `recv()` возвращает `None`, когда все стороны-отправители `mpsc` канала дропнуты —
        // это значит, что `amqprs` закрыл внутренний канал доставки.
        // Ретрай здесь бессмысслен: если отправителей нет, повторный вызов тоже вернёт `None`.
        let msg = rx.recv().await.ok_or(BrokerError::NoSenders)?;
        // По текущей реализации `amqprs` все три поля обычно `Some`, но это не
        // часть стабильного контракта (поля объявлены как `Option`). При смене
        // версии `amqprs` или нестандартном фрейме брокера лучше получить
        // структурированную ошибку, а не панику в worker-таске.
        let delivery = msg.deliver.ok_or_else(|| {
            BrokerError::Internal("amqprs: ConsumerMessage без deliver".to_owned())
        })?;
        let content_bytes = msg.content.ok_or_else(|| {
            BrokerError::Internal("amqprs: ConsumerMessage без content".to_owned())
        })?;
        let basic_properties = msg.basic_properties.ok_or_else(|| {
            BrokerError::Internal(
                "amqprs: ConsumerMessage без basic_properties".to_owned(),
            )
        })?;
        let content = match serde_json::from_slice(&content_bytes) {
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
            properties: basic_properties,
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

    /// Запускает все коллбэки параллельно через [`FuturesUnordered`] и ждёт их завершения.
    ///
    /// `FuturesUnordered` выбран намеренно: коллбэки не зависят друг от друга,
    /// поэтому последовательный запуск был бы медленнее без какой-либо выгоды.
    /// При первой ошибке выполнение прерывается, остальные фьючеры дропаются.
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
            // `multiple = false`: подтверждаем только это конкретное сообщение.
            // `multiple = true` подтвердил бы все предыдущие unacked сообщения в канале,
            // что нарушило бы гарантии обработки при параллельном consume.
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
            // `multiple = false, requeue = false`: отклоняем только это сообщение без повторной
            // постановки в очередь. Если нужна dead-letter очередь — её настраивают на уровне
            // декларации очереди, а не здесь.
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
        // `channel.clone()` нужен потому, что `Channel::close` принимает `self`,
        // но внутри `amqprs` это Arc-хэндл — клонирование не дублирует сокет.
        let op = || self.channel.clone().close();
        op.retry(&self.retry_args).await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(kind = "broker", "Ошибка при закрытии канала: {}", err);
            err.into()
        })
    }
}
