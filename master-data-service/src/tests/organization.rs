/// Модуль тестировния операций со справочником "Поставщики"
#[cfg(test)]
mod master_data_organization {
    use std::sync::Arc;
    use std::time::Duration;

    use amqprs::channel::BasicPublishArguments;
    use amqprs::BasicProperties;
    use monolith_service::http::MonolithHttpDriver;
    use monolith_service::MonolithService;
    use sqlx::postgres::PgQueryResult;
    use sqlx::{Pool, Postgres};

    use broker::rabbit::consumer::RabbitMessage;
    use broker::Consumer;
    use shared_essential::domain::enums::master_data::DirectoryType;
    use shared_essential::domain::master_data::organization::Organization;
    use shared_essential::presentation::dto::master_data::error::MasterDataResult;
    use shared_essential::presentation::dto::master_data::request::{
        MasterDataSearchRequest, MasterDataSearchType, SearchByUserInput,
    };
    use shared_essential::presentation::dto::master_data::response::MasterDataSearchResponse;
    use shared_essential::presentation::dto::AsezResult;

    use crate::infrastructure::rabbit::{publish_response, register_consumer};
    use crate::infrastructure::{setup_rabbit_adapter, Env, GlobalConfig};
    use crate::presentation::handlers::rabbit::process_dictionary_request;
    use crate::tests::db_test::run_db_test;
    use crate::tests::{declare_temp_queue, delete_temp_queue, init_master_data};

    /// Тестирование операции поиска записи по id
    /// Тест состоит из следующих шаков:
    /// 1. Инициализауия rabbit-адаптера, инициализация postgres-адаптера
    /// 2. Декларирование временной очереди для прослушивания запроса на поиск записи
    /// 3. Создание в БД временнойх записи, необходимой для выполнения операции поиска
    /// 4. Публикация внешним сервисом запроса на поиск записи по id
    /// 5. Обработка серсивом НСИ запроса
    /// 6. Публикация сервисом НСИ ответа по результатам поиска
    /// 7. Обработка внешним сервисом ответа
    /// 8. Удаление временной очереди и временной записи из БД
    #[tokio::test]
    async fn organization_get_by_id() {
        run_db_test(&[], |db_pool| async move {
            println!("Setup env");

            let temp_queue = "organization_get_by_id";
            let record_id = -1;

            let env = Env::setup().expect("cannot setup env");

            println!("Setup rabbit adapter");
            let rabbit_adapter = Arc::new(
                setup_rabbit_adapter(&env.rabbit_config)
                    .await
                    .expect("cannot setup rabbit adapter"),
            );
            println!("declare_queues");
            declare_temp_queue(temp_queue, &rabbit_adapter).await;
            let monolith_service = MonolithService::new(
                MonolithHttpDriver::basic_driver(env.monolith_cfg.url.clone())
                    .expect("HTTP драйвер монолита"),
            );

            println!("Setup postgress pool");
            let global_config = Arc::new(GlobalConfig::new(
                env,
                rabbit_adapter,
                db_pool,
                monolith_service,
            ));

            println!("init_master_data");
            init_master_data(global_config.clone()).await;

            // Спавн слушателя очереди, который вернет ответ
            let global_config_clone = global_config.clone();
            let listener_handle = tokio::spawn(async move {
                println!("Register consumer");
                let mut dictionary_consumer = register_consumer(
                    &global_config_clone.broker_adapter,
                    temp_queue,
                )
                .await
                .expect("cannot register consumer");

                println!("Master data service consume input request");
                let received_message: RabbitMessage<MasterDataSearchRequest> =
                    dictionary_consumer
                        .consume()
                        .await
                        .expect("Cannot consume message");
                println!("Master data service process request");

                let res =
                    process_dictionary_request(received_message.content).await;

                assert!(res.is_ok());

                //External service publish request
                println!("External service publish request");
                publish_response(
                    res,
                    received_message.properties.reply_to().map(|s| s.as_str()),
                    global_config_clone.clone(),
                )
                .await
                .expect("cannot publish response message");
            });

            let basic_props = BasicProperties::default()
                .with_content_type("application/json")
                .with_persistence(false)
                .finish();
            let publish_args = BasicPublishArguments::new("", temp_queue);
            let mut direct_reply = global_config
                .broker_adapter
                .direct_reply(
                    basic_props,
                    publish_args,
                    "direct_reply_one_return_consumer",
                )
                .await
                .expect("cannot setup direct reply");
            let request = MasterDataSearchRequest {
                search_type: MasterDataSearchType::SearchById(
                    vec![record_id],
                    DirectoryType::Organization,
                ),
            };

            println!("External service consume response from MasterData");
            let received_message = direct_reply
                .request(&request, Duration::from_millis(1000))
                .await
                .expect("External service consume error");
            let content: AsezResult<MasterDataSearchResponse> =
                received_message.content;
            //println!("Response from MasterData: {:?}", content);

            let api_response = content.expect("expect content message");
            let organizations_vec = api_response
                .records
                .organizations
                .expect("expect organizations data");

            let org: &Organization = organizations_vec
                .get(0)
                .expect("expect organizations first record");
            assert_eq!(org.id, record_id);

            listener_handle.await.unwrap();

            delete_by_id(&global_config.clone().db_pool, record_id)
                .await
                .expect("cannot delete organization record");

            delete_temp_queue(temp_queue, &global_config.clone().broker_adapter)
                .await;
        })
        .await
    }

    /// Тестирование операции поиска записи по пользовательскому вводу
    /// Тест состоит из следующих шаков:
    /// 1. Инициализауия rabbit-адаптера, инициализация postgres-адаптера
    /// 2. Декларирование временной очереди для прослушивания запроса на поиск записи
    /// 3. Создание в БД временнойх записи, необходимой для выполнения операции поиска
    /// 4. Публикация внешним сервисом запроса на поиск записи по пользовательскому вводу
    /// 5. Обработка серсивом НСИ запроса
    /// 6. Публикация сервисом НСИ ответа по результатам поиска
    /// 7. Обработка внешним сервисом ответа
    /// 8. Удаление временной очереди и временной записи из БД
    #[tokio::test]
    async fn organization_search_by_user_input() {
        run_db_test(&[], |db_pool| async move {
            println!("Setup env");

            let temp_queue = "organization_search_by_user_input";
            let record_id = -2;

            let env = Env::setup().expect("cannot setup env");

            println!("Setup rabbit adapter");
            let rabbit_adapter = Arc::new(
                setup_rabbit_adapter(&env.rabbit_config)
                    .await
                    .expect("cannot setup rabbit adapter"),
            );
            println!("declare_queues");

            declare_temp_queue(temp_queue, &rabbit_adapter).await;

            println!("Setup postgress pool");
            let monolith_service = MonolithService::new(
                MonolithHttpDriver::basic_driver(env.monolith_cfg.url.clone())
                    .expect("HTTP драйвер монолита"),
            );
            let global_config = Arc::new(GlobalConfig::new(
                env,
                rabbit_adapter,
                db_pool,
                monolith_service,
            ));

            println!("init_master_data");
            init_master_data(global_config.clone()).await;

            // Спавн слушателя очереди, который вернет ответ
            let global_config_clone = global_config.clone();
            let listener_handle = tokio::spawn(async move {
                println!("Register consumer");
                let mut dictionary_consumer = register_consumer(
                    &global_config_clone.broker_adapter,
                    temp_queue,
                )
                .await
                .expect("cannot register consumer");

                println!("Master data service consume input request");
                let received_message: RabbitMessage<MasterDataSearchRequest> =
                    dictionary_consumer
                        .consume()
                        .await
                        .expect("Cannot consume message");
                println!("Master data service process request");

                let res =
                    process_dictionary_request(received_message.content).await;

                assert!(res.is_ok());

                //External service publish request
                println!("External service publish request");
                publish_response(
                    res,
                    received_message.properties.reply_to().map(|s| s.as_str()),
                    global_config_clone.clone(),
                )
                .await
                .expect("cannot publish response message");
            });

            let basic_props = BasicProperties::default()
                .with_content_type("application/json")
                .with_persistence(false)
                .finish();
            let publish_args = BasicPublishArguments::new("", temp_queue);
            let mut direct_reply = global_config
                .broker_adapter
                .direct_reply(
                    basic_props,
                    publish_args,
                    "direct_reply_one_return_consumer",
                )
                .await
                .expect("cannot setup direct reply");

            let request = MasterDataSearchRequest {
                search_type: MasterDataSearchType::SearchByUserInput(
                    SearchByUserInput {
                        from: 0,
                        quantity: 100,
                        search: "тест".to_string(),
                    },
                    DirectoryType::Organization,
                ),
            };
            let received_message = direct_reply
                .request(&request, Duration::from_millis(1000))
                .await
                .expect("External service consume error");
            let content: AsezResult<MasterDataSearchResponse> =
                received_message.content;
            //println!("Response from MasterData: {:?}", content);

            let api_response = content.expect("expect content");
            let organizations_vec =
                api_response.records.organizations.expect("expect organizations");
            let org: &Organization = organizations_vec
                .get(0)
                .expect("expect organizations first record");
            assert_eq!(org.id, record_id);

            listener_handle.await.unwrap();

            delete_by_id(&global_config.clone().db_pool, record_id)
                .await
                .expect("cannot delete organization record");

            delete_temp_queue(temp_queue, &global_config.clone().broker_adapter)
                .await;
        })
        .await
    }

    /// Тестирование операции отдачи всего справочника
    /// Тест состоит из следующих шаков:
    /// 1. Инициализауия rabbit-адаптера, инициализация postgres-адаптера
    /// 2. Декларирование временной очереди для прослушивания запроса на поиск записи
    /// 3. Создание в БД временнойх записи, необходимой для выполнения операции поиска
    /// 4. Публикация внешним сервисом запроса на поиск записи по id
    /// 5. Обработка серсивом НСИ запроса
    /// 6. Публикация сервисом НСИ ответа по результатам поиска
    /// 7. Обработка внешним сервисом ответа
    /// 8. Удаление временной очереди и временной записи из БД
    #[tokio::test]
    async fn organization_get_full_data() {
        run_db_test(&[], |db_pool| async move {
            println!("Setup env");

            let temp_queue = "organization_get_full_data";
            let record_id = -3;

            let env = Env::setup().expect("cannot setup env");

            println!("Setup rabbit adapter");
            let rabbit_adapter = Arc::new(
                setup_rabbit_adapter(&env.rabbit_config)
                    .await
                    .expect("cannot setup rabbit adapter"),
            );
            println!("declare_queues");

            declare_temp_queue(temp_queue, &rabbit_adapter).await;

            println!("Setup postgress pool");
            let monolith_service = MonolithService::new(
                MonolithHttpDriver::basic_driver(env.monolith_cfg.url.clone())
                    .expect("HTTP драйвер монолита"),
            );
            let global_config = Arc::new(GlobalConfig::new(
                env,
                rabbit_adapter,
                db_pool,
                monolith_service,
            ));

            println!("init_master_data");
            init_master_data(global_config.clone()).await;

            // Спавн слушателя очереди, который вернет ответ
            let global_config_clone = global_config.clone();
            let listener_handle = tokio::spawn(async move {
                println!("Register consumer");
                let mut dictionary_consumer = register_consumer(
                    &global_config_clone.broker_adapter,
                    temp_queue,
                )
                .await
                .expect("cannot register consumer");

                println!("Master data service consume input request");
                let received_message: RabbitMessage<MasterDataSearchRequest> =
                    dictionary_consumer
                        .consume()
                        .await
                        .expect("Cannot consume message");
                println!("Master data service process request");

                let res =
                    process_dictionary_request(received_message.content).await;

                assert!(res.is_ok());

                //External service publish request
                println!("External service publish request");
                publish_response(
                    res,
                    received_message.properties.reply_to().map(|s| s.as_str()),
                    global_config_clone.clone(),
                )
                .await
                .expect("cannot publish response message");
            });

            let basic_props = BasicProperties::default()
                .with_content_type("application/json")
                .with_persistence(false)
                .finish();
            let publish_args = BasicPublishArguments::new("", temp_queue);
            let mut direct_reply = global_config
                .broker_adapter
                .direct_reply(
                    basic_props,
                    publish_args,
                    "direct_reply_one_return_consumer",
                )
                .await
                .expect("cannot setup direct reply");

            let request = MasterDataSearchRequest {
                search_type: MasterDataSearchType::GetFullDirectory(vec![
                    DirectoryType::Organization,
                ]),
            };
            let received_message = direct_reply
                .request(&request, Duration::from_millis(1000))
                .await
                .expect("External service consume error");
            let content: AsezResult<MasterDataSearchResponse> =
                received_message.content;
            //println!("Response from MasterData: {:?}", content);

            let api_response = content.expect("expect content");
            let organizations_vec = api_response
                .records
                .organizations
                .expect("expect organizations first record");

            assert!(!organizations_vec.is_empty());

            listener_handle.await.unwrap();

            delete_by_id(&global_config.clone().db_pool, record_id)
                .await
                .expect("cannot delete organization record");

            delete_temp_queue(temp_queue, &global_config.clone().broker_adapter)
                .await;
        })
        .await
    }

    pub(crate) async fn delete_by_id(
        pool: &Pool<Postgres>,
        id: i32,
    ) -> MasterDataResult<u64> {
        let result: sqlx::Result<PgQueryResult> =
            sqlx::query("DELETE FROM organization WHERE id= $1")
                .bind(id)
                .execute(pool)
                .await;
        Ok(result.map(|res| res.rows_affected())?)
    }
}
