//! Отвечает а объекты с таблицы `favourite_plan_by_id`.
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use shared_db_derive::DbItemExt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "favourite_plans_by_id"]
#[item_aggr_insert]
pub struct FavouritePlan {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    pub plan_uuid: Uuid,
    pub user_id: i32,
    pub status: i16,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
