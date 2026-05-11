//! Базовый пример получения сообщения из RabbitMQ очереди
//! с помощью долгоживущего консьюмера
use amqprs::{
    channel::{BasicConsumeArguments, QueueDeclareArguments},
    connection::OpenConnectionArguments,
};
use broker::{rabbit::RabbitAdapter, BrokerAdapter, Consumer};
use serde::Deserialize;

#[derive(Deserialize)]
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

    // Регистрация RabbitMQ консьюмера
    let consume_props = BasicConsumeArguments::new("hello", "hello-consumer");
    let mut basic_consumer = rabbit_adapter
        .register_consumer(consume_props)
        .await
        .expect("Не удалось зарегистрировать консьюмера");

    // Какой то сервис отправил нам сообщение...

    // Получение сообщения из очереди
    let received_message =
        basic_consumer.consume().await.expect("Не удалось получить сообщение");
    // Мы должны явно определить тип, чтобы указать serde, как его десериализовать
    // Мы можем просто извлечь содержимое и определить тип
    let _received_content: &SomeReq = &received_message.content;
    // или просто передайте его в функцию
    handle_message_content(&received_message.content);

    // Закрытие соединения с RabbitMQ сервером
    rabbit_adapter
        .shutdown()
        .await
        .expect("Не удалось закрыть соединение с RabbitMQ сервером");
}

fn handle_message_content(message: &SomeReq) {
    println!("{}", message.name);
}
