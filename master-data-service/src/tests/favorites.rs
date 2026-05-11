#![cfg(test)]

use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbItem;
use shared_essential::domain::favorites::{FavoriteDictionary, FavoriteList};
use shared_essential::presentation::dto::master_data::error::MasterDataError;
use shared_essential::presentation::dto::master_data::favorites::FavoriteItemData;

use crate::application::master_data::favorites::{
    create_favorite_item, delete_favorite_item, get_favorite_list,
};

use crate::tests::db_test::run_db_test;

const USER: i32 = 666;
const USER_NO_FAV: i32 = 555;

#[tokio::test]
async fn list() {
    let list_exp = serde_json::json!({
        "dictionary_list": [
            {
                "dictionary_id": 1,
                "item_list": [{
                    "dictionary_item_id": 10
                }]
            },
            {
                "dictionary_id": 2,
                "item_list": [
                    {
                        "dictionary_item_id": 20
                    },
                    {
                        "dictionary_item_id": 30
                    },
                    {
                        "dictionary_item_id": 40
                    }
                ]
            }
        ]
    });

    run_db_test(&["favorites.sql"], |pool| async move {
        let list = get_favorite_list(USER, &pool).await.expect("no error");
        let list_act = serde_json::to_value(list).unwrap();
        assert_eq!(list_act, list_exp);
    })
    .await
}

#[tokio::test]
async fn list_empty() {
    run_db_test(&["favorites.sql"], |pool| async move {
        let list = get_favorite_list(USER_NO_FAV, &pool).await.expect("no error");
        assert!(list.dictionary_list.is_empty());
    })
    .await
}

#[tokio::test]
async fn create_item() {
    const DICT_ID: i32 = 1;
    const ITEM_ID: i32 = 12;

    run_db_test(&["favorites.sql"], |pool| async move {
        let matches = |item: &FavoriteList| {
            item.dictionary_id == DICT_ID && item.dictionary_item_id == ITEM_ID
        };

        let old_db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert!(!old_db_items.iter().any(|item| matches(item)));

        let new_item = FavoriteItemData {
            dictionary_id: DICT_ID,
            dictionary_item_id: ITEM_ID,
        };

        create_favorite_item(USER, new_item, &pool).await.expect("no error");

        let db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert_eq!(db_items.len(), old_db_items.len() + 1);
        assert!(db_items.iter().any(|item| matches(item)));
    })
    .await
}

#[tokio::test]
async fn create_item_already_exists() {
    const DICT_ID: i32 = 1;
    const ITEM_ID: i32 = 10;

    run_db_test(&["favorites.sql"], |pool| async move {
        let matches = |item: &FavoriteList| {
            item.dictionary_id == DICT_ID && item.dictionary_item_id == ITEM_ID
        };

        let old_db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert!(old_db_items.iter().any(|item| matches(item)));

        let new_item = FavoriteItemData {
            dictionary_id: DICT_ID,
            dictionary_item_id: ITEM_ID,
        };

        create_favorite_item(USER, new_item, &pool).await.expect("no error");

        let db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert_eq!(db_items.len(), old_db_items.len());
        assert!(db_items.iter().any(|item| matches(item)));
    })
    .await
}

#[tokio::test]
async fn create_item_fail_no_dict() {
    const DICT_ID: i32 = 3;
    const ITEM_ID: i32 = 50;

    run_db_test(&["favorites.sql"], |pool| async move {
        let matches = |item: &FavoriteList| {
            item.dictionary_id == DICT_ID && item.dictionary_item_id == ITEM_ID
        };

        let old_db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert!(!old_db_items.iter().any(|item| matches(item)));
        let db_dicts = FavoriteDictionary::select(
            &Select::default().eq(FavoriteDictionary::id, DICT_ID),
            &*pool,
        )
        .await
        .unwrap();
        assert!(db_dicts.is_empty());

        let new_item = FavoriteItemData {
            dictionary_id: DICT_ID,
            dictionary_item_id: ITEM_ID,
        };

        let err =
            create_favorite_item(USER, new_item, &pool).await.expect_err("error");
        assert!(matches!(
            err,
            MasterDataError::FavoritesInvalidDictionary(DICT_ID)
        ));

        let db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert_eq!(db_items.len(), old_db_items.len());
    })
    .await
}

#[tokio::test]
async fn delete_item() {
    const DICT_ID: i32 = 2;
    const ITEM_ID: i32 = 20;

    run_db_test(&["favorites.sql"], |pool| async move {
        let matches = |item: &FavoriteList| {
            item.dictionary_id == DICT_ID && item.dictionary_item_id == ITEM_ID
        };

        let old_db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert!(old_db_items.iter().any(|item| matches(item)));

        let new_item = FavoriteItemData {
            dictionary_id: DICT_ID,
            dictionary_item_id: ITEM_ID,
        };

        let deleted_item =
            delete_favorite_item(USER, new_item, &pool).await.expect("no error");
        assert_eq!(
            serde_json::to_value(&deleted_item).unwrap(),
            serde_json::json!({
                "dictionary_id": DICT_ID,
                "dictionary_item_id": ITEM_ID
            })
        );

        let db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert_eq!(db_items.len(), old_db_items.len() - 1);
        assert!(!db_items.iter().any(|item| matches(item)));
    })
    .await
}

#[tokio::test]
async fn delete_item_no_item() {
    const DICT_ID: i32 = 2;
    const ITEM_ID: i32 = 21;

    run_db_test(&["favorites.sql"], |pool| async move {
        let matches = |item: &FavoriteList| {
            item.dictionary_id == DICT_ID && item.dictionary_item_id == ITEM_ID
        };

        let old_db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert!(!old_db_items.iter().any(|item| matches(item)));

        let new_item = FavoriteItemData {
            dictionary_id: DICT_ID,
            dictionary_item_id: ITEM_ID,
        };

        let deleted_item =
            delete_favorite_item(USER, new_item, &pool).await.expect("no error");
        assert_eq!(
            serde_json::to_value(&deleted_item).unwrap(),
            serde_json::json!({
                "dictionary_id": DICT_ID,
                "dictionary_item_id": ITEM_ID
            })
        );

        let db_items = FavoriteList::select(
            &Select::default().eq(FavoriteList::user_id, USER),
            &*pool,
        )
        .await
        .unwrap();
        assert_eq!(db_items.len(), old_db_items.len());
    })
    .await
}
