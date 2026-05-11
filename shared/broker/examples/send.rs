//! Базовый пример отправления сообщения в RabbitMQ очередь
//! с помощью долгоживущего паблишера
use amqprs::{
    channel::{BasicPublishArguments, QueueDeclareArguments},
    connection::OpenConnectionArguments,
    BasicProperties,
};
use broker::{rabbit::RabbitAdapter, BrokerAdapter, Publisher};
use serde::Serialize;

#[derive(Serialize)]
struct SomeReq {
    name: String,
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

    // Декларация RabbitMQ очереди
    let queue_args = QueueDeclareArguments::default()
        .queue(String::from("hello"))
        .durable(true)
        .finish();
    rabbit_adapter
        .declare_queue(queue_args)
        .await
        .expect("Не удалось декларировать очередь");

    // Регистрация RabbitMQ паблишера
    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .with_persistence(true)
        .finish();
    let publish_props = BasicPublishArguments::new("", "hello");
    let basic_publisher = rabbit_adapter
        .register_publisher(basic_props, publish_props)
        .await
        .expect("Не удалось зарегистрировать паблишера");

    // Отправление сообщения
    let payload = SomeReq {
        name: String::from("Amazing Name"),
    };
    basic_publisher
        .publish(&payload)
        .await
        .expect("Не удалось отправить сообщение");

    // Закрытие соединения с RabbitMQ сервером
    rabbit_adapter
        .shutdown()
        .await
        .expect("Не удалось закрыть соединение с RabbitMQ сервером");
}
