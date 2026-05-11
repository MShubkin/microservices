//! Отвечает а объекты с таблицы `plan_retrospective`.
use crate::legacy::plans::PlanStatus;

use asez2_shared_db::db_item::{DbAdaptor, DbItem, DbItemDel, DbUpsert};

use crate::{ContractAmendment, Plan, StatusHistory};
use asez2_shared_db::{impl_join_on, joined};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

impl_join_on!(PlanRetrospective:uuid_ly => Plan:uuid, left);
impl_join_on!(PlanRetrospective:uuid_ly => ContractAmendment:uuid, left);
impl_join_on!(PlanRetrospective:uuid_ly => StatusHistory:object_uuid, aggr);

joined!(
    !PlanRetrospectiveDetails,
    plan_retrospective: PlanRetrospective,
    plan: Plan[PlanRetrospective => Plan, left],
    amendment: ContractAmendment[PlanRetrospective => ContractAmendment, left],
    status_history: StatusHistory[PlanRetrospective => StatusHistory, aggr],
);

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbAdaptor, DbUpsert, PartialEq)]
#[adaptor_derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Deserialize,
    Serialize,
    PartialOrd
)]
#[item_table = "plan_retrospective"]
#[item_aggr_insert]
pub struct PlanRetrospective {
    #[item_field_autogen]
    #[adaptor_rename = "plan_retrospective_id"]
    pub id: i64,
    pub plan_uuid: Uuid,
    #[item_field_pkey]
    pub plan_id: i64,
    pub plan_year: i16,
    pub plan_status: PlanStatus,
    #[item_field_pkey]
    pub id_ly: i64,
    pub uuid_ly: Uuid,
    pub is_removed: bool,
}

impl DbItemDel for PlanRetrospective {}
