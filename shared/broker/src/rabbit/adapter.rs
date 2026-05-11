#![allow(clippy::map_identity)]
use std::time::Duration;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicConsumeArguments, BasicPublishArguments, Channel,
        QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
#[cfg(feature = "traces")]
use tracing::error;

use super::{
    channel::RabbitChannelCallback,
    publisher::RabbitPublisher,
    RabbitMessage,
    {consumer::RabbitConsumer, RabbitChannel},
};
use crate::{
    error::Result,
    retry::{RetryArgs, Retryable},
    BrokerAdapter, BrokerError, Consumer,
};

/// Адаптер подключения к RabbitMQ серверу
#[derive(Clone)]
pub struct RabbitAdapter {
    /// Подключение к RabbitMQ серверу
    connection: Connection,
    /// Аргументы для механизма ретрая
    retry_args: RetryArgs,
}

#[async_trait]
impl BrokerAdapter for RabbitAdapter {
    type Channel = RabbitChannel;
    type ConnectionArgs = OpenConnectionArguments;
    type BasicProperties = BasicProperties;
    type QueueArgs = QueueDeclareArguments;
    type ConsumeArgs = BasicConsumeArguments;
    type PublishArgs = BasicPublishArguments;

    type Consumer = RabbitConsumer;
    type Publisher = RabbitPublisher;

    async fn connect(
        args: Self::ConnectionArgs,
        retry_args: RetryArgs,
    ) -> Result<Self> {
        let connection = Connection::open(&args).await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(
                kind = "broker",
                "Ошибка при подключении к RabbitMQ серверу: {:?}", err
            );
            BrokerError::from(err)
        })?;

        connection.register_callback(DefaultConnectionCallback).await.map_err(
            |err| {
                #[cfg(feature = "traces")]
                error!(
                    kind = "broker",
                    "Ошибка при регистрации колбэка для соединения: {:?}", err
                );
                BrokerError::from(err)
            },
        )?;

        Ok(Self {
            connection,
            retry_args,
        })
    }

    async fn open_channel(&self, channel_id: Option<u16>) -> Result<Self::Channel> {
        let channel_open_op = || self.connection.open_channel(channel_id);
        let channel =
            channel_open_op.retry(&self.retry_args).await.map_err(|err| {
                #[cfg(feature = "traces")]
                error!(kind = "broker", "Ошибка при открытии канала: {}", err);
                BrokerError::from(err)
            })?;

        let register_callback_op =
            || channel.register_callback(DefaultChannelCallback);
        register_callback_op.retry(&self.retry_args).await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(
                kind = "broker",
                "Ошибка при регистрации колбэка для канала: {:?}", err
            );
            err
        })?;

        let rabbit_channel =
            RabbitChannel::new(channel, self.retry_args.clone(), Vec::new());
        Ok(rabbit_channel)
    }

    async fn declare_queue(&self, args: Self::QueueArgs) -> Result<()> {
        let rabbit_channel = self.open_channel(None).await?;
        rabbit_channel.channel.queue_declare(args).await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(kind = "broker", "Ошибка при декларации очереди: {}", err);
            err
        })?;

        rabbit_channel.close().await
    }

    async fn register_consumer(
        &self,
        consume_args: Self::ConsumeArgs,
    ) -> Result<Self::Consumer> {
        let channel = self.open_channel(None).await?;
        self.register_consumer_on_channel(consume_args, channel.channel).await
    }

    async fn register_publisher(
        &self,
        basic_props: Self::BasicProperties,
        publish_args: Self::PublishArgs,
    ) -> Result<Self::Publisher> {
        let channel = self.open_channel(None).await?;
        self.register_publisher_on_channel(
            basic_props,
            publish_args,
            channel.channel,
        )
        .await
    }

    async fn shutdown(self) -> Result<()> {
        self.connection.close().await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(kind = "broker", "Ошибка при закрытии соединения: {}", err);
            err.into()
        })
    }
}

/// Обертка для воссоздания RPC паттерна с помощью Direct-Reply механизма.
/// Создает [RabbitPublisher] и [RabbitConsumer] на одном
/// и том же канале и возвращает их как долгоживущие сущности.
pub struct DirectReply {
    consumer: RabbitConsumer,
    publisher: RabbitPublisher,
}

impl DirectReply {
    /// # Описание
    ///
    /// Remote Protocol Call к другому сервису
    ///
    /// # Аргументы
    /// * `dto` - Отправляемое другому сервису тело сообщения
    /// * `timeout` - Таймаут, по истечении которого запрос будет принудительно
    /// закончен
    ///
    /// # Возвращает
    /// * Ok([`RabbitMessage<R>`]) - Успешное получение ответа от сервиса
    /// * Err([`BrokerError`]) - Что-то пошло не так при отправке или получении сообщения
    pub async fn request<T, R>(
        &mut self,
        dto: &T,
        timeout: Duration,
    ) -> Result<RabbitMessage<R>>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de> + Send + Sync,
    {
        self.publisher.publish_with_expiration(dto, timeout).await?;
        let result = self.consumer.consume_with_timeout(timeout).await;
        if let Err(BrokerError::WaitingTooLong) = &result {
            #[cfg(feature = "traces")]
            error!(
                kind = "broker",
                error = %BrokerError::WaitingTooLongForReply,
                "Не получен ответ от консьюмера, таймаут {:?}",
                timeout
            );
            return Err(BrokerError::WaitingTooLongForReply);
        }
        result
    }
}

impl RabbitAdapter {
    /// # Описание
    ///
    /// Метод для воссоздания паттерна RPC с помощью Direct-Reply механизма.
    /// Создает [RabbitPublisher] и [RabbitConsumer] на одном
    /// и том же канале и возвращает их как долгоживущие сущности.
    ///
    /// Вам не следует указывать reply_to, потому что этот метод сделает это за вас.
    ///
    /// Для получения дополнительной информации ознакомьтесь с https://www.rabbitmq.com/docs/direct-reply-to.
    ///
    /// Также вы можете посмотреть пример использования [RabbitAdapter::direct_reply] в каталоге examples.
    /// # Аргументы
    /// * `basic_props` - Основные свойства для отправки сообщения
    /// * `publish_args` - Аргументы отправления сообщения
    /// * `consumer_tag` - Специфический тег потребителя для лучшей инфографики
    ///
    /// # Возвращает
    /// * Ok([`DirectReply`]) - Способ обращения к другому сервису для получения сообщения
    /// * Err([`BrokerError`]) - Что-то пошло не так при регистрации паблишера или консьюмера
    pub async fn direct_reply(
        &self,
        mut basic_props: BasicProperties,
        publish_args: BasicPublishArguments,
        consumer_tag: &str,
    ) -> Result<DirectReply> {
        let channel = self.open_channel(None).await?;

        let basic_props =
            basic_props.with_reply_to("amq.rabbitmq.reply-to").finish();
        let publisher = self
            .register_publisher_on_channel(
                basic_props,
                publish_args,
                channel.channel.clone(),
            )
            .await?;

        let consume_args =
            BasicConsumeArguments::new("amq.rabbitmq.reply-to", consumer_tag)
                // Важный аспект Direct Reply-To механизма
                .auto_ack(true)
                .finish();
        let consumer = self
            .register_consumer_on_channel(consume_args, channel.channel)
            .await?;

        Ok(DirectReply {
            consumer,
            publisher,
        })
    }

    /// # Описание
    ///
    /// Вспомогательный метод для регистрации консьюмера на конкретном канале
    ///
    /// # Аргументы
    /// * `consume_args` - Аргументы для регистрации консьюмера и консьюма сообщений
    /// * `channel` - Канал, в котором будет зарегистрирован [RabbitConsumer]
    ///
    /// # Возвращает
    /// * Ok([`Self::Consumer`]) - Успешная регистрация консьюмера
    /// * Err([`BrokerError`]) - Ошибка при регистрация консьюмера
    pub async fn register_consumer_on_channel(
        &self,
        consume_args: BasicConsumeArguments,
        channel: Channel,
    ) -> Result<RabbitConsumer> {
        let op = || channel.basic_consume_rx(consume_args.clone());
        let (_, rx) = op.retry(&self.retry_args).await.map_err(|err| {
            #[cfg(feature = "traces")]
            error!(
                kind = "broker",
                "Ошибка при регистрации консьюмера `{}` очереди: {}",
                consume_args.queue,
                err
            );
            err
        })?;

        let manual_ack = !consume_args.no_ack;
        let rabbit_channel =
            RabbitChannel::new(channel, self.retry_args.clone(), Vec::new());

        Ok(RabbitConsumer::new(rx, rabbit_channel, manual_ack))
    }

    /// # Описание
    ///
    /// Вспомогательный метод для регистрации паблишера на конкретном канале
    ///
    /// # Аргументы
    /// * `basic_props` - Основные свойства для отправки сообщения
    /// * `publish_args` - Аргументы отправления сообщения
    /// * `channel` - Канал, в котором будет зарегистрирован [RabbitConsumer]
    ///
    /// # Возвращает
    /// * Ok([`Self::Consumer`]) - Успешная регистрация паблишера
    /// * Err([`BrokerError`]) - Ошибка при регистрация паблишера
    pub async fn register_publisher_on_channel(
        &self,
        basic_props: BasicProperties,
        publish_args: BasicPublishArguments,
        channel: Channel,
    ) -> Result<RabbitPublisher> {
        let rabbit_channel =
            RabbitChannel::new(channel, self.retry_args.clone(), Vec::new());
        Ok(RabbitPublisher::new(rabbit_channel, basic_props, publish_args))
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

impl DirectReply {
    pub fn register_publisher_callback<C>(&mut self, callback: C)
    where
        C: RabbitChannelCallback + Sync + Send + 'static,
    {
        self.publisher.register_callback(callback)
    }
}

impl AsRef<RabbitPublisher> for DirectReply {
    fn as_ref(&self) -> &RabbitPublisher {
        &self.publisher
    }
}

impl AsRef<RabbitConsumer> for DirectReply {
    fn as_ref(&self) -> &RabbitConsumer {
        &self.consumer
    }
}

impl std::fmt::Debug for RabbitAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = &self.connection;
        f.debug_struct("RabbitAdapter")
            .field("Наименование подключения - ", &c.connection_name())
            .field("Открыто - ", &c.is_open())
            .field("Максимальное кол-во каналов - ", &c.channel_max())
            .field("Максимальный размер фрейма - ", &c.frame_max())
            .field("Интервал heartbeat в секундах - ", &c.heartbeat())
            .field("Свойства сервера - ", &c.server_properties())
            .finish()
    }
}
