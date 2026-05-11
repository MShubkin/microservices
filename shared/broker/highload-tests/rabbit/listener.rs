use std::{
    sync::{atomic::Ordering, Arc},
    time::{Duration, Instant},
};

use amqprs::{
    channel::{
        BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
    },
    connection::OpenConnectionArguments,
    BasicProperties,
};
use broker::{rabbit::RabbitAdapter, BrokerAdapter, Consumer, Publisher};

mod test_state;
use test_state::{get_config, HighloadTestState, TestReq};

#[tokio::main]
async fn main() {
    let config = get_config().unwrap();

    let connection_args = OpenConnectionArguments::new(
        &config.rabbit_config.addr,
        config.rabbit_config.port,
        &config.rabbit_config.username,
        &config.rabbit_config.password,
    );
    let rabbit_adapter = Arc::new(
        RabbitAdapter::connect(connection_args, Default::default())
            .await
            .expect("Не удалось подключиться к RabbitMQ серверу"),
    );

    let queue_args = QueueDeclareArguments::default()
        .queue(config.rabbit_config.queue_name.clone())
        .finish();
    rabbit_adapter
        .declare_queue(queue_args)
        .await
        .expect("Не удалось декларировать очередь");

    let consume_props = BasicConsumeArguments::new(
        &config.rabbit_config.queue_name,
        &format!("{}-consumer", config.rabbit_config.queue_name),
    );
    let mut basic_consumer = rabbit_adapter
        .register_consumer(consume_props)
        .await
        .expect("Cannot register consumer");

    let test_state = Arc::new(HighloadTestState::default());
    let mut handles = Vec::with_capacity(config.request_count as usize);

    // Получение запросов от паблишера и возвращение ответа
    for _ in 0..config.request_count {
        let request_start = Instant::now();
        let message = basic_consumer.consume().await;

        let message = message.expect("Не удалось получить сообщение");
        let rabbit_adapter = rabbit_adapter.clone();
        let test_state = test_state.clone();

        let handle = tokio::spawn(async move {
            let num =
                handle_message_content(message.content, config.process_time_ms)
                    .await;

            let reply_to = message
                .properties
                .reply_to()
                .expect("`reply_to` метадата не была передана")
                .as_ref();
            let basic_props = BasicProperties::default()
                .with_content_type("application/json")
                .with_persistence(true)
                .finish();
            let publish_props = BasicPublishArguments::new("", reply_to);
            let basic_publisher = rabbit_adapter
                .register_publisher(basic_props, publish_props)
                .await
                .expect("Не удалось зарегестрировать паблишера");

            basic_publisher
                .publish(&num)
                .await
                .expect("Не удалось отправить сообщение");

            let elapsed = request_start.elapsed().as_millis() as u64;
            test_state.update(elapsed, Ordering::SeqCst);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("# Результаты слушателя в миллисекундах :\n\n{:#?}\n\n", test_state);
}

async fn handle_message_content(content: TestReq, sleep_time: u64) -> TestReq {
    tokio::time::sleep(Duration::from_millis(sleep_time)).await;
    content
}
