//! Подсистема избранных записей.

use asez2_shared_db::{
    db_item::{Filter, FilterTree, Select},
    DbItem,
};
use itertools::Itertools;
use shared_essential::{
    domain::favorites::{FavoriteDictionary, FavoriteList},
    presentation::dto::master_data::{
        error::{MasterDataError, MasterDataResult},
        favorites::*,
    },
};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

/// Returns a list of favorite items for the specified user.
pub(crate) async fn get_favorite_list(
    user_id: i32,
    pool: &PgPool,
) -> MasterDataResult<FavoriteListData> {
    info!(kind = "get", "Получение списка избранных справочников");

    let favorite_list = FavoriteList::select(
        &Select::default()
            .eq(FavoriteList::user_id, user_id)
            .add_replace_order_asc(FavoriteList::dictionary_id),
        pool,
    )
    .await?;

    let dictionary_list = favorite_list
        .into_iter()
        .map(
            |FavoriteList {
                 dictionary_id,
                 dictionary_item_id,
                 ..
             }| (dictionary_id, dictionary_item_id),
        )
        .group_by(|x| x.0)
        .into_iter()
        .map(|(dictionary_id, dictionary_item_ids)| FavoriteListItem {
            dictionary_id,
            item_list: dictionary_item_ids
                .map(|x| FavoriteDictItem {
                    dictionary_item_id: x.1,
                })
                .collect(),
        })
        .collect();

    Ok(FavoriteListData { dictionary_list })
}

/// Checks that the `dictionary_id` is a valid identifier of an existing _favorite_dictionary_ record.
pub(crate) async fn check_dictionary_id(
    dictionary_id: i32,
    pool: &PgPool,
) -> MasterDataResult<()> {
    let dictionary_id_rec = FavoriteDictionary::select(
        &Select::default().eq(FavoriteDictionary::id, dictionary_id),
        pool,
    )
    .await?;

    if dictionary_id_rec.is_empty() {
        warn!(
            kind = "master-data",
            dictionary_id, "Указан несуществующий справочник"
        );
        return Err(MasterDataError::FavoritesInvalidDictionary(dictionary_id));
    }

    Ok(())
}

/// Creates a favorite item for the specified user and the specified dictionary and its item.
pub(crate) async fn create_favorite_item(
    user_id: i32,
    item_data: FavoriteItemData,
    pool: &PgPool,
) -> MasterDataResult<()> {
    info!(kind = "create", "Создание новой избранной записи");

    let FavoriteItemData {
        dictionary_id,
        dictionary_item_id,
    } = item_data;

    check_dictionary_id(dictionary_id, pool).await?;

    let favorite_item = FavoriteList::select(
        &Select::default()
            .eq(FavoriteList::user_id, user_id)
            .eq(FavoriteList::dictionary_id, dictionary_id)
            .eq(FavoriteList::dictionary_item_id, dictionary_item_id),
        pool,
    )
    .await?;

    if favorite_item.is_empty() {
        FavoriteList {
            uuid: Uuid::new_v4(),
            user_id,
            dictionary_id,
            dictionary_item_id,
        }
        .insert(pool)
        .await?;
    }

    Ok(())
}

/// Deletes the item for the specified user and the specified dictionary and its item
/// from the favorite list, and returns it.
pub(crate) async fn delete_favorite_item(
    user_id: i32,
    item_data: FavoriteItemData,
    pool: &PgPool,
) -> MasterDataResult<FavoriteItemData> {
    use asez2_shared_db::db_item::DbItemDel;
    info!(kind = "delete", "Удаление избранной записи");

    let filter = FilterTree::and_from_list([
        Filter::eq(FavoriteList::user_id, user_id),
        Filter::eq(FavoriteList::dictionary_id, item_data.dictionary_id),
        Filter::eq(FavoriteList::dictionary_item_id, item_data.dictionary_item_id),
    ]);
    FavoriteList::delete_returning(&filter, pool).await?;

    Ok(item_data)
}
