use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{sync::Arc, time::Instant};

use amqprs::{
    channel::BasicPublishArguments, connection::OpenConnectionArguments,
    BasicProperties,
};
use broker::RetryArgs;
use broker::{rabbit::RabbitAdapter, BrokerAdapter};

mod test_state;
use test_state::{get_config, HighloadTestState, TestReq};

#[tokio::main]
async fn main() {
    let config = get_config().expect("Не удалось получить конфигурацию");

    let connection_args = OpenConnectionArguments::new(
        &config.rabbit_config.addr,
        config.rabbit_config.port,
        &config.rabbit_config.username,
        &config.rabbit_config.password,
    );
    let rabbit_adapter = Arc::new(
        RabbitAdapter::connect(connection_args, RetryArgs::new(5, 200))
            .await
            .expect("Не удалось подключиться к RabbitMQ серверу"),
    );

    let test_state = Arc::new(HighloadTestState::default());
    let mut handles = Vec::with_capacity(config.request_count as usize);

    // Отправка запросов к слушателю очереди
    for i in 0..config.request_count {
        let rabbit_adapter = rabbit_adapter.clone();
        let queue_name = config.rabbit_config.queue_name.clone();
        let test_state = test_state.clone();

        let handle = tokio::spawn(async move {
            let request_start = Instant::now();

            let basic_props = BasicProperties::default()
                .with_content_type("application/json")
                .with_persistence(true)
                .finish();
            let publish_props = BasicPublishArguments::new("", &queue_name);
            let mut direct_reply = rabbit_adapter
                .direct_reply(
                    basic_props,
                    publish_props,
                    &format!("{}-{}-return-consumer", queue_name, i),
                )
                .await
                .expect("Не удалось зарегестрировать Direct-Reply механизм");

            let dto = TestReq::new(i);
            let res = direct_reply
                .request(&dto, Duration::from_millis(200))
                .await
                .expect("Не удалось получить сообщение");
            let dto: TestReq = res.content;
            assert_eq!(dto.count, i);

            let elapsed = request_start.elapsed().as_millis() as u64;
            test_state.update(elapsed, Ordering::SeqCst);
        });

        handles.push(handle);

        // Если вы закомментируете эту строку, она завершится ошибкой, так как
        // RabbitMQ каналы будут спавниться слишком много и быстро
        tokio::time::sleep(Duration::from_millis(config.request_interval_ms)).await;
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("# Результаты паблишера в миллисекундах:\n\n{:#?}\n\n", test_state);
}
