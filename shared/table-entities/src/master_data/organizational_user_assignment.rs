use asez2_shared_db::{
    db_item::AsezTimestamp, impl_join_on, joined, DbAdaptor, DbItem,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::OrganizationalStructure;

#[derive(
    Debug, Clone, Default, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct OrganizationalUserAssignment {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    pub user_id: i32,
    pub department_id: i32,
    pub customer_id: Option<i32>,
    pub position_id: Option<i32>,
    pub organizer_id: Option<i32>,
    pub purchasing_group_id: Option<i32>,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

impl_join_on!(OrganizationalUserAssignment:department_id => OrganizationalStructure:id);
joined!(
    !UserAssignmentAndOrg,
    user_assignment: OrganizationalUserAssignment,
    org_structure: OrganizationalStructure[OrganizationalUserAssignment => OrganizationalStructure],
);
