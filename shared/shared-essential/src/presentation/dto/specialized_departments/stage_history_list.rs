use asez2_tables::DocumentApproverRep;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Запрос на получение истории рассмотрения документа.
#[derive(Debug, Serialize, Deserialize)]
pub struct StageHistoryListReq {
    pub uuid: Uuid,
    pub plan_id: Option<i64>,
    pub user_id: i32,
}

/// Результат истории рассмотрения документа.
pub type StageHistoryListRes = Vec<DocumentApproverRep>;
