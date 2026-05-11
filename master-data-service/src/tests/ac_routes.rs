/// Модуль тестировния операций со справочником "Параметры автоназначения эксперта АЦ"
#[cfg(test)]
mod master_data_crit_route {
    use amqprs::channel::BasicPublishArguments;
    use amqprs::BasicProperties;
    use monolith_service::http::MonolithHttpDriver;
    use monolith_service::MonolithService;
    use sqlx::postgres::PgQueryResult;
    use sqlx::{Pool, Postgres};
    use std::sync::Arc;
    use std::time::Duration;

    use asez2_shared_db::db_item::{DbItemDel, Filter, FilterTree};
    use broker::rabbit::consumer::RabbitMessage;
    use broker::Consumer;
    use shared_essential::{
        domain::{enums::master_data::DirectoryType, routes::RouteCrit},
        presentation::dto::{
            master_data::{
                error::MasterDataResult,
                request::{MasterDataSearchRequest, MasterDataSearchType},
                response::MasterDataSearchResponse,
            },
            AsezResult,
        },
    };

    use crate::infrastructure::rabbit::{publish_response, register_consumer};
    use crate::infrastructure::{setup_rabbit_adapter, Env, GlobalConfig};
    use crate::presentation::handlers::rabbit::process_dictionary_request;
    use crate::tests::db_test::run_db_test;
    use crate::tests::{declare_temp_queue, delete_temp_queue, init_master_data};

    /// Модуль тестировния операций со справочником "Параметры автоназначения эксперта АЦ"
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
    async fn crit_route_get_full_data() {
        run_db_test(&[], |db_pool| async move {
            println!("Setup env");

            let temp_queue = "crit_route_ac_get_full_data";
            let record_id = -1;

            let env = Env::setup().expect("Не удалось загрузить переменные среды");

            println!("Setup rabbit adapter");
            let rabbit_adapter = Arc::new(
                setup_rabbit_adapter(&env.rabbit_config)
                    .await
                    .expect("cannot setup rabbit adapter"),
            );
            println!("declare_queues");

            declare_temp_queue(temp_queue, &rabbit_adapter).await;

            println!("Setup postgress pool");
            let db_pool = (*db_pool).clone();
            let monolith = MonolithService::new(
                MonolithHttpDriver::basic_driver(env.monolith_cfg.url.clone())
                    .expect("cannot setup monolith client"),
            );

            let global_config =
                Arc::new(GlobalConfig::new(env, rabbit_adapter, db_pool, monolith));

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

                assert!(res.is_ok(), "{:?}", res);

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

            //External service publish request
            println!("External service publish request");
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
                .expect("cannot setup direct-reply");

            let request = MasterDataSearchRequest {
                search_type: MasterDataSearchType::GetFullDirectory(vec![
                    DirectoryType::AutoAssignmentExpertPa,
                ]),
            };
            let received_message = direct_reply
                .request(&request, Duration::from_millis(1000))
                .await
                .expect("External service consume error");
            let content: AsezResult<MasterDataSearchResponse> =
                received_message.content;
            //println!("Response from MasterData: {:?}", content);

            let api_response = content.expect("response content is empty");
            let route_vec = api_response
                .records
                .auto_assignment_expert_ac
                .expect("response content is empty");

            assert!(!route_vec.is_empty());

            listener_handle.await.unwrap();

            header_delete_by_route_id(&global_config.clone().db_pool, record_id)
                .await
                .expect("route_crit delete error");

            crit_delete_by_route_id(&global_config.clone().db_pool, record_id)
                .await
                .expect("route_list delete error");
            users_delete_by_route_id(&global_config.clone().db_pool, record_id)
                .await
                .expect("route_users delete error");

            delete_temp_queue(temp_queue, &global_config.clone().broker_adapter)
                .await;
        })
        .await
    }

    pub(crate) async fn crit_delete_by_route_id(
        pool: &Pool<Postgres>,
        route_id: i32,
    ) -> MasterDataResult<u64> {
        let filter = FilterTree::filter(Filter::eq(RouteCrit::route_id, route_id));
        let result = RouteCrit::delete_returning(&filter, pool).await?;
        Ok(result.len() as u64)
    }

    pub(crate) async fn header_delete_by_route_id(
        pool: &Pool<Postgres>,
        route_id: i32,
    ) -> MasterDataResult<u64> {
        let result: sqlx::Result<PgQueryResult> =
            sqlx::query("DELETE FROM route_list WHERE id= $1")
                .bind(route_id)
                .execute(pool)
                .await;
        Ok(result.map(|res| res.rows_affected())?)
    }

    pub(crate) async fn users_delete_by_route_id(
        pool: &Pool<Postgres>,
        route_id: i32,
    ) -> MasterDataResult<u64> {
        let result: sqlx::Result<PgQueryResult> =
            sqlx::query("DELETE FROM route_users WHERE route_id= $1")
                .bind(route_id)
                .execute(pool)
                .await;
        Ok(result.map(|res| res.rows_affected())?)
    }
}
