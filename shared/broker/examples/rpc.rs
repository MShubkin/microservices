//! Базовый пример использования Direct-Reply механизма для RPC паттерна
//! с помощью долгоживущих паблишера и консьюмера, который были заспавнены
//! на одном канале
use std::time::Duration;

use amqprs::{
    channel::BasicPublishArguments, connection::OpenConnectionArguments,
    BasicProperties,
};
use broker::{rabbit::RabbitAdapter, BrokerAdapter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct SomeToReq {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SomeFromReq {
    age: u8,
}

#[tokio::main]
async fn main() {
    // Подключение к RabbitMQ серверу
    let connection_args =
        OpenConnectionArguments::new("localhost", 5672, "guest", "guest");
    let rabbit_adapter =
        RabbitAdapter::connect(connection_args, Default::default())
            .await
            .expect("Не удалось подключиться к RabbitMQ серверу");

    // Регистрация Direct-Reply механизма
    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .with_persistence(true)
        .finish();
    // Роутинг ключ установлен в виде очереди другого сервиса
    let publish_args = BasicPublishArguments::new("", "hello");
    let mut direct_reply = rabbit_adapter
        .direct_reply(basic_props, publish_args, "direct_reply_one-return-consumer")
        .await
        .expect("Не удалось зарегестрировать Direct-Reply механизм");

    // Здесь другой сервис получает сообщение и считывает его метадату с `reply_to` полем.
    // После этого он отправляет сообщение обратно в очередь из `reply_to` поля, и
    // вы получаете его в return_consumer
    let content = SomeToReq {
        name: "Name".into(),
    };
    // Получение сообщения из очереди
    let received_message = direct_reply
        .request(&content, Duration::from_millis(200))
        .await
        .expect("Не удалось получить сообщение");
    let _content: SomeFromReq = received_message.content;
}
