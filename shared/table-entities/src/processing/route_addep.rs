use asez2_shared_db::{db_item::AsezTimestamp, DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_aggr_insert]
pub struct RouteAddep {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    pub route_id: i32,
    pub department_id: i32,
    pub division_id: i32,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
