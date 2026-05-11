use shared_essential::{
    domain::enums::master_data::DirectoryType,
    presentation::dto::master_data::request::SearchByUserInput,
    try_get_directory_record,
};

use crate::{
    application::{
        directory_get_full_data, directory_search_by_id,
        directory_search_by_user_input, master_data::MasterData,
    },
    assert_found, assert_search_result,
    domain::MasterDataDirectoryInterface,
    tests::db_test::run_db_test,
};

#[tokio::test]
async fn search() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.category.load(&pool).await.expect("loaded");
        let dir_type = DirectoryType::Category;
        let search = "Магистральные трубопроводы".to_string();

        let all = directory_get_full_data(&master_data, vec![dir_type])
            .await
            .expect("full_data")
            .records
            .category
            .expect("full_data.category");

        let (_, record) = directory_search_by_user_input(
            &master_data,
            dir_type,
            &SearchByUserInput {
                from: 0,
                quantity: 100,
                search: search.clone(),
            },
        )
        .await
        .unwrap();
        let found = try_get_directory_record!(record, Category).unwrap();

        assert_search_result!(search, 6, found, all);
    })
    .await
}

#[tokio::test]
async fn search_by_id() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.category.load(&pool).await.expect("loaded");
        let dir_type = DirectoryType::Category;

        let ids = [122, 526, 971];

        let (_, record) = directory_search_by_id(&master_data, dir_type, &ids).await.unwrap();
        let found = try_get_directory_record!(record, Category).unwrap();

        assert_eq!(found.len(), ids.len());

        assert_found!(found, 122, "0030106006", "325 мм");
        assert_found!(found, 526, "0040608000", "Магистральные трубопроводы");
        assert_found!(found, 971, "0180702140", "СС КонсультантАрбитраж: Арбитражные суды всех округов (большая сетевая версия 50 ОД)");
    })
    .await
}
