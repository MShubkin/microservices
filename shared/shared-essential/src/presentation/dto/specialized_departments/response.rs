use asez2_tables::DocumentApproverRep;
use serde::{Deserialize, Serialize};

pub type GetApproversForPlansResData = Vec<GetApproversForPlansResItem>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetApproversForPlansResItem {
    pub plan_id: i64,
    pub item_list: Vec<DocumentApproverRep>,
}
