use ahash::AHashMap;
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
    assert_found,
    domain::MasterDataDirectoryInterface,
    tests::db_test::run_db_test,
};

#[tokio::test]
async fn search() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.okpd2.load(&pool).await.expect("loaded");
        let dir_type = DirectoryType::Okpd2;
        let search = "рыБа".to_string();

        let all = directory_get_full_data(&master_data, vec![dir_type])
            .await
            .expect("full_data")
            .records
            .okpd2
            .expect("full_data.okpd2");

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
        let found = try_get_directory_record!(record, Okpd2).unwrap();

        let matches = |s: &str| s.to_lowercase().contains(&search.to_lowercase());
        let expected = all
            .iter()
            .filter(|x| matches(&x.text))
            .map(|x| (x.id, x))
            .collect::<AHashMap<_, _>>();
        found.iter().for_each(|x| assert_eq!(expected.get(&x.id), Some(&x)));
    })
    .await
}

#[tokio::test]
async fn search_code() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.okpd2.load(&pool).await.expect("loaded");
        let dir_type = DirectoryType::Okpd2;
        let search = "03.1".to_string();

        let all = directory_get_full_data(&master_data, vec![dir_type])
            .await
            .expect("full_data")
            .records
            .okpd2
            .expect("full_data.okpd2");

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
        let found = try_get_directory_record!(record, Okpd2).unwrap();

        let matches = |s: &str| s.starts_with(&search);
        let expected = all
            .iter()
            .filter(|x| matches(&x.code))
            .map(|x| (x.id, x))
            .collect::<AHashMap<_, _>>();
        found.iter().for_each(|x| assert_eq!(expected.get(&x.id), Some(&x)));
    })
    .await
}

#[tokio::test]
async fn search_by_id() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.okpd2.load(&pool).await.expect("loaded");
        let dir_type = DirectoryType::Okpd2;

        let ids = [736, 2432, 3808];

        let (_, record) = directory_search_by_id(&master_data, dir_type, &ids).await.unwrap();
        let found = try_get_directory_record!(record, Okpd2).unwrap();

        assert_eq!(found.len(), ids.len());

        assert_found!(found, 736, "01.45.11.211", "Бараны-производители тонкорунных пород, кроме чистопородных племенных овец");
        assert_found!(found, 2432, "02.20.11.185", "Хлысты пихтовые");
        assert_found!(found, 3808, "05.10.10.133", "Уголь марки Г - газовый");
    })
    .await
}
