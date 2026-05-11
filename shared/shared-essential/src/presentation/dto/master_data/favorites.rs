use serde::{Deserialize, Serialize};

use crate::presentation::dto::response_request::ApiResponseData;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FavoriteListData {
    pub dictionary_list: Vec<FavoriteListItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FavoriteItemData {
    pub dictionary_id: i32,
    pub dictionary_item_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FavoriteListItem {
    pub dictionary_id: i32,
    pub item_list: Vec<FavoriteDictItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FavoriteDictItem {
    pub dictionary_item_id: i32,
}

impl ApiResponseData for FavoriteListData {}
