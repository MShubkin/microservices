use shared_essential::{
    domain::enums::master_data::DirectoryType,
    presentation::dto::master_data::request::SearchByUserInput,
    try_get_directory_record,
};
use sqlx::PgPool;

use crate::{
    application::{
        directory_search_by_id, directory_search_by_user_input,
        master_data::MasterData,
    },
    domain::MasterDataDirectoryInterface,
    tests::db_test::run_db_test,
};

async fn load_master_data(pool: &PgPool) -> MasterData {
    let mut master_data = MasterData::default();
    master_data
        .scheduler_request_update_catalog
        .load(pool)
        .await
        .expect("scheduler_request_update_catalog loaded");
    master_data
}

#[tokio::test]
async fn search_by_id() {
    run_db_test(&["scheduler_catalog.sql"], |pool| async move {
        let master_data = load_master_data(&pool).await;
        let res = directory_search_by_id(
            &master_data,
            DirectoryType::SchedulerRequestUpdateCatalog,
            &[42],
        )
        .await;
        assert!(res.is_ok(), "{res:?}");
        let (_, record) = res.unwrap();
        assert!(try_get_directory_record!(record, SchedulerRequestUpdateCatalog)
            .is_some());
    })
    .await
}

#[tokio::test]
async fn search() {
    run_db_test(&["scheduler_catalog.sql"], |pool| async move {
        let master_data = load_master_data(&pool).await;
        let res = directory_search_by_user_input(
            &master_data,
            DirectoryType::SchedulerRequestUpdateCatalog,
            &SearchByUserInput {
                from: 0,
                quantity: 100,
                search: "бастили".into(),
            },
        )
        .await;
        assert!(res.is_ok(), "{res:?}");
        let (_, record) = res.unwrap();
        assert!(try_get_directory_record!(record, SchedulerRequestUpdateCatalog)
            .is_some());
    })
    .await
}
