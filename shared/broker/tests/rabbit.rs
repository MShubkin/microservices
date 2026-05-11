//! Этот модуль отвечает за тестирование адаптера RabbitMQ
//! ВАЖНО: Перед запуском тестов вам необходимо запустить экземпляр сервера RabbitMQ
//! и сконфигуроваить .env.test
use std::{
    env::{var, VarError},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use amqprs::{
    channel::{
        BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
    },
    connection::OpenConnectionArguments,
    BasicProperties,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use broker::{
    rabbit::{
        channel::RabbitChannelCallback, RabbitAdapter, RabbitChannel, RabbitMessage,
    },
    BrokerAdapter, BrokerError, Consumer, Publisher,
};

/// Конфигурация для тестирования взаимодействия с RabbitMQ
#[derive(Deserialize, Debug)]
struct TestRabbitConfig {
    /// Порт, на котором открыт сервер RabbitMQ
    rabbit_port: u16,
    /// Адрес сервера RabbitMQ
    rabbit_addr: String,
    /// Пользователь
    username: String,
    /// Пароль
    password: String,
}

/// Тестовая структура для отправки в сообщении
#[derive(Debug, Serialize, Deserialize)]
struct ToReq {
    name: String,
}

/// Тестовая структура для отправки в сообщении обратно
#[derive(Debug, Serialize, Deserialize)]
struct FromReq {
    age: u8,
}

/// # Описание
///
/// Получение конфигурации общения с RabbitMQ
///
/// # Возвращает
/// * Ok([`TestRabbitConfig`]) - Успешное получение конфигурации
/// * Err([`dotenv::Error`]) - Ошибка при неудачном получении env переменной из внешной среды
fn get_config() -> Result<TestRabbitConfig, VarError> {
    let rabbit_addr = var("RABBITMQ_HOST")?;
    let rabbit_port = var("RABBITMQ_PORT")?
        .parse()
        .expect("RABBIT_PORT env variable has wrong format");
    let username = var("RABBITMQ_USERNAME")?;
    let password = var("RABBITMQ_PASSWORD")?;

    Ok(TestRabbitConfig {
        rabbit_port,
        rabbit_addr,
        username,
        password,
    })
}

/// # Описание
///
/// Настройка подключения к серверу RabbitMQ
///
/// # Аргументы
/// * `config` - [Конфигурация](TestRabbitConfig) взаимодействия с RabbitMQ сервером
/// # Возвращает
/// * Ok([`RabbitAdapter`]) - Успешное подключение к RabbitMQ серверу
/// * Err([`BrokerError`]) - Ошибка при подключении
async fn connect(config: TestRabbitConfig) -> Result<RabbitAdapter, BrokerError> {
    let connection_args = OpenConnectionArguments::new(
        &config.rabbit_addr,
        config.rabbit_port,
        &config.username,
        &config.password,
    );
    let rabbit_adapter =
        RabbitAdapter::connect(connection_args, Default::default())
            .await
            .map_err(|e| {
                println!("{config:?}:{e}");
                e
            })?;

    Ok(rabbit_adapter)
}

/// Тестирует базовый кейс на подключение к RabbitMQ серверу
#[tokio::test]
async fn rabbit_adapter_connect() {
    let config = get_config().unwrap();
    let _rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");
}

/// Тестирует базовый кейс на декларацию очередей
#[tokio::test]
async fn declare_queue() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let declare_args = QueueDeclareArguments::default()
        .queue(String::from("declare_queue"))
        .finish();
    rabbit_adapter
        .declare_queue(declare_args)
        .await
        .expect("Не удалось декларировать очередь");
}

/// Тестирует базовый кейс на регистрацию консьюмера
#[tokio::test]
async fn register_consumer() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let declare_args = QueueDeclareArguments::default()
        .queue(String::from("register_consumer"))
        .finish();
    rabbit_adapter
        .declare_queue(declare_args)
        .await
        .expect("Не удалось декларировать очередь");

    let consume_args = BasicConsumeArguments::new(
        "register_consumer",
        "register_consumer_consumer",
    );
    let _consumer = rabbit_adapter
        .register_consumer(consume_args)
        .await
        .expect("Не удалось зарегистрировать консьюмера");
}

/// Тестирует базовый кейс на регистрацию паблишера
#[tokio::test]
async fn register_publisher() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let declare_args = QueueDeclareArguments::default()
        .queue(String::from("register_publisher"))
        .finish();
    rabbit_adapter
        .declare_queue(declare_args)
        .await
        .expect("Не удалось декларировать очередь");

    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .finish();
    let publish_props = BasicPublishArguments::new("", "register_publisher");
    let _publisher = rabbit_adapter
        .register_publisher(basic_props, publish_props)
        .await
        .expect("Не удалось зарегистрировать паблишера");
}

/// Тестирует кейс c получением сообщения с таймаутом
#[tokio::test]
async fn timeout_consume() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let declare_args = QueueDeclareArguments::default()
        .queue(String::from("timeout_consume"))
        .finish();
    rabbit_adapter
        .declare_queue(declare_args)
        .await
        .expect("Не удалось декларировать очередь");

    let consume_args =
        BasicConsumeArguments::new("timeout_consume", "timeout_consume_consumer")
            .finish();
    let mut consumer = rabbit_adapter
        .register_consumer(consume_args)
        .await
        .expect("Не удалось зарегистрировать консьюмера");

    let now = Instant::now();
    let message: Result<RabbitMessage<String>, BrokerError> =
        consumer.consume_with_timeout(Duration::from_millis(200)).await;
    let elapsed = now.elapsed();
    assert!(
        elapsed >= Duration::from_millis(100)
            && elapsed <= Duration::from_millis(300)
    );
    assert!(matches!(message, Err(BrokerError::WaitingTooLong)));
}

// Тестирует базовый кейс на отправку и получение сообщения
#[tokio::test]
async fn publish_consume_roundtrip() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let declare_args = QueueDeclareArguments::default()
        .queue(String::from("publish_consume_roundtrip"))
        .finish();
    rabbit_adapter
        .declare_queue(declare_args)
        .await
        .expect("Не удалось декларировать очередь");

    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .finish();
    let publish_props = BasicPublishArguments::new("", "publish_consume_roundtrip");
    let publisher = rabbit_adapter
        .register_publisher(basic_props, publish_props)
        .await
        .expect("Не удалось зарегистрировать паблишера");

    let name = String::from("Some amazing name");
    let content = ToReq { name: name.clone() };
    publisher
        .publish(&content)
        .await
        .expect("Не удалось отправить сообщение");

    let consume_args = BasicConsumeArguments::new(
        "publish_consume_roundtrip",
        "publish_consume_roundtrip_consumer",
    )
    .finish();
    let mut consumer = rabbit_adapter
        .register_consumer(consume_args)
        .await
        .expect("Не удалось зарегистрировать консьюмера");

    let message = consumer
        .consume()
        .await
        .expect("Не удалось получить сообщение из очереди");
    let content: ToReq = message.content;

    assert_eq!(content.name, name);
}

/// Тестирует базовый кейс на отправку и получение сообщения с неверными данными
#[tokio::test]
#[should_panic]
async fn publish_consume_wrong_roundtrip() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let declare_args = QueueDeclareArguments::default()
        .queue(String::from("publish_consume_wrong_roundtrip"))
        .finish();
    rabbit_adapter
        .declare_queue(declare_args)
        .await
        .expect("Не удалось декларировать очередь");

    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .finish();
    let publish_props =
        BasicPublishArguments::new("", "publish_consume_wrong_roundtrip");
    let publisher = rabbit_adapter
        .register_publisher(basic_props, publish_props)
        .await
        .expect("Не удалось зарегистрировать паблишера");

    let name = String::from("Some amazing name");
    let content = ToReq { name: name.clone() };
    publisher
        .publish(&content)
        .await
        .expect("Не удалось отправить сообщение");

    let consume_props = BasicConsumeArguments::new(
        "publish_consume_wrong_roundtrip",
        "publish_consume_wrong_roundtrip_consumer",
    );
    let mut basic_consumer = rabbit_adapter
        .register_consumer(consume_props)
        .await
        .expect("Не удалось зарегистрировать консьюмера");
    let received_message = basic_consumer
        .consume()
        .await
        .expect("Должно запаниковать, так в очередь пришли невалидные данные");
    let _content: FromReq = received_message.content;
}

// Тестирует Direct Reply-To механизм для RPC паттерна
#[tokio::test]
async fn direct_reply_one() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let queue_args = QueueDeclareArguments::default()
        .queue("direct_reply_one".into())
        .finish();
    rabbit_adapter
        .declare_queue(queue_args)
        .await
        .expect("Не удалось декларировать очередь");

    // Представим, что это другой сервис, которые принимает от нашего сервиса сообщение
    // и возвращает ответ
    ///////////////
    let rabbit_adapter_clone = rabbit_adapter.clone();
    tokio::spawn(async move {
        let consume_args = BasicConsumeArguments::new(
            "direct_reply_one",
            "direct_reply_one_consumer",
        );
        let mut consumer =
            rabbit_adapter_clone.register_consumer(consume_args).await.unwrap();

        let message = consumer
            .consume()
            .await
            .expect("Не удалось получить сообщение из очереди");
        let content: ToReq = message.content;
        let reply_to = message
            .properties
            .reply_to()
            .expect("В метадате сообщения отсутствует поле `reply_to`");

        let basic_props = BasicProperties::default()
            .with_content_type("application/json")
            .finish();
        let publish_props = BasicPublishArguments::new("", reply_to);
        let basic_publisher = rabbit_adapter_clone
            .register_publisher(basic_props, publish_props)
            .await
            .expect("Не удалось зарегистрировать паблишера");

        basic_publisher
            .publish(&content)
            .await
            .expect("Не удалось отправить сообщение");
    });
    /////////////

    // Наш сервис, который посылает запрос и получает ответ
    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .finish();
    let publish_args = BasicPublishArguments::new("", "direct_reply_one");
    let mut direct_reply = rabbit_adapter
        .direct_reply(
            basic_props,
            publish_args,
            &String::from("direct_reply_one_return_consumer"),
        )
        .await
        .expect("Не удалось зарегестрировать Direct-Reply механизм");

    let content = ToReq {
        name: String::from("Awesome Name"),
    };

    let received_message = direct_reply
        .request(&content, Duration::from_millis(2000))
        .await
        .expect("Не удалось получить сообщение");
    let received_content: ToReq = received_message.content;

    assert_eq!(received_content.name, String::from("Awesome Name"));
}

/// Тестирует Direct Reply-To механизм для RPC паттерна при нескольких запросах
#[tokio::test]
async fn direct_reply_many() {
    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let queue_args = QueueDeclareArguments::default()
        .queue("direct_reply_many".into())
        .finish();
    rabbit_adapter
        .declare_queue(queue_args)
        .await
        .expect("Не удалось декларировать очередь");

    // Представим, что это другой сервис, которые принимает от нашего сервиса сообщение
    // и возвращает ответ
    ///////////////
    let rabbit_adapter_clone = rabbit_adapter.clone();
    tokio::spawn(async move {
        let consume_args = BasicConsumeArguments::new(
            "direct_reply_many",
            "direct_reply_many_consumer",
        );
        let mut consumer =
            rabbit_adapter_clone.register_consumer(consume_args).await.unwrap();

        for _ in 0..20 {
            let message = consumer
                .consume()
                .await
                .expect("Не удалось получить сообщение из очереди");
            let content: ToReq = message.content;
            let reply_to = message
                .properties
                .reply_to()
                .expect("В метадате сообщения отсутствует поле `reply_to`");

            let basic_props = BasicProperties::default()
                .with_content_type("application/json")
                .finish();
            let publish_props = BasicPublishArguments::new("", reply_to);
            let basic_publisher = rabbit_adapter_clone
                .register_publisher(basic_props, publish_props)
                .await
                .expect("Не удалось зарегистрировать паблишера");

            basic_publisher
                .publish(&content)
                .await
                .expect("Не удалось отправить сообщение");
        }
    });
    /////////////

    // Наш сервис, который посылает несколько запросов и получает ответы
    let mut join_handles = Vec::with_capacity(20);
    for i in 0..20 {
        let rabbit_adapter = rabbit_adapter.clone();
        let handle = tokio::spawn(async move {
            let basic_props = BasicProperties::default()
                .with_content_type("application/json")
                .finish();
            let publish_args = BasicPublishArguments::new("", "direct_reply_many");
            let mut direct_reply = rabbit_adapter
                .direct_reply(
                    basic_props,
                    publish_args,
                    &format!("direct_reply_many_return_{}_consumer", i),
                )
                .await
                .expect("Не удалось зарегестрировать Direct-Reply механизм");

            let content = ToReq {
                name: format!("Name {}", i),
            };

            let received_message = direct_reply
                .request(&content, Duration::from_millis(5000))
                .await
                .expect("Не удалось получить сообщение");
            received_message.content
        });
        join_handles.push(handle);
    }

    let mut results = Vec::with_capacity(20);
    for j in join_handles {
        results.push(j.await.expect(
            "Один из-запросов завершился с ошибкой, так и не получив ответ",
        ));
    }

    let res = results
        .into_iter()
        .enumerate()
        .all(|(i, x): (usize, ToReq)| x.name.eq(&format!("Name {}", i)));
    assert!(res);
}

#[tokio::test]
async fn publish_callbacks() {
    let counter = Arc::new(AtomicU64::new(0));

    #[derive(Clone, Debug)]
    struct CountCallback {
        counter: Arc<AtomicU64>,
    }

    #[async_trait]
    impl RabbitChannelCallback for CountCallback {
        async fn on_publish(
            &self,
            _channel: &RabbitChannel,
            basic_props: &BasicProperties,
            _publish_args: &BasicPublishArguments,
        ) -> broker::Result<()> {
            self.counter.fetch_add(
                basic_props.correlation_id().unwrap().parse().unwrap(),
                Ordering::Relaxed,
            );
            Ok(())
        }
    }

    let config = get_config().expect(
        "Не удалось получить конфигурацию для обращения к RabbitMQ серверу",
    );
    let rabbit_adapter = connect(config)
        .await
        .expect("Не удалось подключиться к RabbitMQ серверу");

    let declare_args = QueueDeclareArguments::default()
        .queue(String::from("publish_callbacks"))
        .finish();
    rabbit_adapter
        .declare_queue(declare_args)
        .await
        .expect("Не удалось декларировать очередь");

    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .with_correlation_id("3")
        .finish();
    let publish_props = BasicPublishArguments::new("", "publish_callbacks");
    let mut publisher = rabbit_adapter
        .register_publisher(basic_props, publish_props)
        .await
        .expect("Не удалось зарегистрировать паблишера");

    publisher.register_callback(CountCallback {
        counter: counter.clone(),
    });
    publisher.register_callback(CountCallback {
        counter: counter.clone(),
    });
    publisher.register_callback(CountCallback {
        counter: counter.clone(),
    });

    publisher
        .publish(&String::from("message"))
        .await
        .expect("Не удалось отправить сообщение");

    assert_eq!(counter.load(Ordering::Relaxed), 9)
}
