use asez2_shared_db::{db_item::DbItemDel, DbAdaptor, DbItem};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "favorite_list"]
pub struct FavoriteList {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub user_id: i32,
    pub dictionary_id: i32,
    pub dictionary_item_id: i32,
}

impl DbItemDel for FavoriteList {}

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "favorite_dictionary"]
pub struct FavoriteDictionary {
    #[item_field_pkey]
    pub uuid: Uuid,
    pub id: i32,
    pub text: String,
    pub name: String,
}
