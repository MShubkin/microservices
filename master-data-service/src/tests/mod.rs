use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;
use shared_essential::infrastructure::rabbit::setup_rabbit_adapter;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

use amqprs::channel::{QueueDeclareArguments, QueueDeleteArguments};
use lazy_static::lazy_static;

use asez2_shared_db::db_item::DbItem;
use broker::rabbit::RabbitAdapter;
use broker::BrokerAdapter;
use shared_essential::domain::master_data::organization::Organization;

use crate::application::master_data::load_master_data;
use crate::infrastructure::{Env, GlobalConfig};

mod db_test;
mod dictionary;
mod favorites;
mod get_route_details;
mod get_route_list;
mod org_user_assignment;
mod organization;
mod organizational_structure_search;
mod organizational_structure_search_by_id;
mod route_copy;
mod route_create;
mod route_find;
mod route_remove;
mod route_start;
mod route_stop;
mod route_update;

lazy_static! {
    static ref IS_MASTER_DATA_INITIALIZED: Mutex<bool> = Mutex::new(false);
}
/// Подготовка тестовых данных для всех тестов
/// Загрузка данных в память
async fn init_master_data(config: Arc<GlobalConfig>) {
    {
        // Запись в БД и загрузка тестовых данных в память должна выполнятся один раз и предшествовать выполнению тестов
        let mut guard = IS_MASTER_DATA_INITIALIZED.lock().await;
        if !(*guard) {
            *guard = true;
            init_organizations(config.clone()).await;
            init_conclusion_templates(config.clone()).await;
            load_master_data(config.clone()).await.unwrap();
            println!("Master data is initialized");
        }
    }
}

#[allow(dead_code)]
async fn init_organizations(config: Arc<GlobalConfig>) {
    let mut record1 = Organization {
        uuid: uuid::Uuid::new_v4(),
        id: -1,
        ..Default::default()
    };
    let mut record2 = Organization {
        uuid: uuid::Uuid::new_v4(),
        id: -2,
        text: "тест".to_string(),
        ..Default::default()
    };
    let mut record3 = Organization {
        uuid: uuid::Uuid::new_v4(),
        id: -3,
        ..Default::default()
    };

    record1.insert(config.db_pool.as_ref()).await.unwrap();
    record2.insert(config.db_pool.as_ref()).await.unwrap();
    record3.insert(config.db_pool.as_ref()).await.unwrap();
}

#[allow(dead_code)]
async fn init_conclusion_templates(config: Arc<GlobalConfig>) {
    sqlx::query("delete from conclusion_templates")
        .execute(config.db_pool.as_ref())
        .await
        .unwrap();
    sqlx::query("delete from conclusion_templates_criteria")
        .execute(config.db_pool.as_ref())
        .await
        .unwrap();
    sqlx::query("delete from conclusion_templates_variables")
        .execute(config.db_pool.as_ref())
        .await
        .unwrap();
    sqlx::query("ALTER SEQUENCE conclusion_templates_id_seq RESTART WITH 1")
        .execute(config.db_pool.as_ref())
        .await
        .unwrap();
    sqlx::query(
        "ALTER SEQUENCE conclusion_templates_variables_id_seq RESTART WITH 1",
    )
    .execute(config.db_pool.as_ref())
    .await
    .unwrap();
    sqlx::query(
        "ALTER SEQUENCE conclusion_templates_criteria_id_seq RESTART WITH 1",
    )
    .execute(config.db_pool.as_ref())
    .await
    .unwrap();

    sqlx::query("INSERT INTO public.conclusion_templates(uuid, created_at, changed_at) VALUES ('00000000-0000-0000-0000-000000000001', now(), now())").execute(config.db_pool.as_ref()).await.unwrap();
    sqlx::query("INSERT INTO public.conclusion_templates_variables(uuid, template_id, created_at, changed_at) VALUES ('00000000-0000-0000-0000-000000000001', 1, now(), now())").execute(config.db_pool.as_ref()).await.unwrap();
    sqlx::query("INSERT INTO public.conclusion_templates_criteria(uuid, template_id, created_at, changed_at) VALUES ('00000000-0000-0000-0000-000000000001', 1, now(), now())").execute(config.db_pool.as_ref()).await.unwrap();
}

#[allow(dead_code)]
async fn declare_temp_queue(queue: &str, rabbit_adapter: &RabbitAdapter) {
    rabbit_adapter
        .declare_queue(
            QueueDeclareArguments::default()
                .queue(queue.into())
                .durable(false)
                .finish(),
        )
        .await
        .unwrap();
}

#[allow(dead_code)]
async fn delete_temp_queue(queue: &str, rabbit_adapter: &RabbitAdapter) {
    let ch = rabbit_adapter.open_channel(None).await.unwrap();
    ch.channel()
        .queue_delete(QueueDeleteArguments {
            queue: queue.to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
}

async fn init_master_data_from_pool(db_pool: &PgPool) {
    let env = Env::setup().expect("Не удалось загрузить env");
    let rabbit_adapter = Arc::new(
        setup_rabbit_adapter(&env.rabbit_config)
            .await
            .expect("Не удалось установить соединение с RabbitMQ"),
    );
    let db_pool = db_pool.clone();
    let monolith_service = MonolithService::new(
        MonolithHttpDriver::basic_driver(env.monolith_cfg.url.clone())
            .expect("HTTP драйвер монолита"),
    );
    let global_config = Arc::new(GlobalConfig::new(
        env,
        rabbit_adapter,
        db_pool.into(),
        monolith_service,
    ));

    init_master_data(global_config).await;
}
