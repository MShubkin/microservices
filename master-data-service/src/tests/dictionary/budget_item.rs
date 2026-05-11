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
        master_data.budget_item.load(&pool).await.expect("loaded");
        let dir_type = DirectoryType::BudgetItem;
        let search = "Амортизация нематериальных активов".to_string();

        let all = directory_get_full_data(&master_data, vec![dir_type])
            .await
            .expect("full_data")
            .records
            .budget_item
            .expect("full_data.budget_item");

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
        let found = try_get_directory_record!(record, BudgetItem).unwrap();

        assert_search_result!(search, 3, found, all);
    })
    .await
}

#[tokio::test]
async fn search_by_id() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.budget_item.load(&pool).await.expect("loaded");
        let dir_type = DirectoryType::BudgetItem;

        let ids = [2258, 3515, 2239];

        let (_, record) =
            directory_search_by_id(&master_data, dir_type, &ids).await.unwrap();
        let found = try_get_directory_record!(record, BudgetItem).unwrap();

        assert_eq!(found.len(), ids.len());

        assert_found!(found, 2258, "0500010146", "Акцизы на УВ");
        assert_found!(found, 3515, "0501370061", "Льготы и выплаты пенсионерам");
        assert_found!(
            found,
            2239,
            "0500010130",
            "Взносы в государственные внебюджетные фонды РФ"
        );
    })
    .await
}
