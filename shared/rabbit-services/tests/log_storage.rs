use amqprs::channel::{BasicConsumeArguments, QueueDeclareArguments};
use broker::{BrokerAdapter, Consumer};
use shared_essential::presentation::dto::{log_storage::LogDataInsert, Source};
use std::sync::Arc;
use uuid::Uuid;

use rabbit_services::{
    properties::AsezRabbitProperties,
    routing::{LOG_STORAGE_QUEUE, PROCESSING_QUEUE},
    services::{processing::ProcessingService, AsezRabbitService},
};

mod common;
use common::{connect, get_config};

#[tokio::test]
async fn view_storage_callback() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );

    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");
    let rabbit_adapter = Arc::new(rabbit_adapter);

    let processing_queue = QueueDeclareArguments::default()
        .queue(PROCESSING_QUEUE.to_string())
        .durable(true)
        .finish();
    let log_storage_queue = QueueDeclareArguments::default()
        .queue(LOG_STORAGE_QUEUE.to_string())
        .durable(true)
        .finish();

    rabbit_adapter
        .declare_queue(processing_queue)
        .await
        .expect("Не удалось декларировать очередь processing");
    rabbit_adapter
        .declare_queue(log_storage_queue)
        .await
        .expect("Не удалось декларировать очередь log-storage");

    let processing_service = ProcessingService::new(
        rabbit_adapter.clone(),
        AsezRabbitProperties::default().with_user_id(123).with_request_id(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        ),
        Source::EstimatedCommission,
    )
    .with_log_callback();

    let consume_args =
        BasicConsumeArguments::new(LOG_STORAGE_QUEUE, "log_storage_consumer")
            .finish();
    let mut log_storage_consumer = rabbit_adapter
        .register_consumer(consume_args)
        .await
        .expect("Не удалось зарегистрировать консьюмера");

    let _ = processing_service.pre_add_plans_agenda(vec![]).await;

    let message = log_storage_consumer
        .consume()
        .await
        .expect("Не удалось получить сообщение из очереди");
    let content: LogDataInsert = message.content;

    assert_eq!(content.event_id, 1);
    assert!(content
        .request_id
        .unwrap()
        .starts_with("00000000-0000-0000-0000-000000000001"));
    assert_eq!(content.source_id as i16, Source::EstimatedCommission as i16);
    assert_eq!(content.user_id, String::from("123"));
}
