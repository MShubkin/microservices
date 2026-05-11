use asez2_shared_db::{
    db_item::{int_array::AsezArray, AsezDate, AsezTimestamp, DbItemExt, DbUpsert},
    impl_join_on, joined, DbAdaptor, DbItem,
};
use fieldname_access::FieldnameAccess;
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;
use uuid::Uuid;

use crate::{
    maths::CurrencyValue, Attachment, ContractAmendment, ContractAmendmentItem,
    Plan, PlanItemFull,
};

#[derive(
    Debug,
    Default,
    Clone,
    DbItem,
    DbItemExt,
    DbAdaptor,
    DbUpsert,
    PartialEq,
    Deserialize,
    Serialize,
)]
#[adaptor_derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Deserialize,
    Serialize,
    FieldnameAccess
)]
#[adaptor_attributes(
    #[fieldname_enum(derive = [Debug, Eq, PartialEq, Ord, PartialOrd])]
)]
#[adaptor_fields_with_values]
pub struct DocumentApprover {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    pub document_uuid: Uuid,
    pub plan_id: i64,
    pub department_id: i32,
    pub number: i32,
    pub planned_date: AsezDate,
    pub started_at: Option<AsezTimestamp>,
    pub division_id: Option<i32>,
    pub division_assigned_at: Option<AsezTimestamp>,
    pub expert_id: Option<i32>,
    pub responded_at: Option<AsezTimestamp>,
    pub response_id: Option<SdExpertConclusion>,
    pub response_note: Option<String>,
    pub total_when_decision: Option<CurrencyValue>,
    pub status_appr: ApprovalStatus,
    pub responsible_person_id: Option<i32>,
    pub is_auto: bool,
    pub route_id: AsezArray<i64>,
    pub send_date_1: Option<AsezTimestamp>,
    pub send_users_1: AsezArray<i32>,
    pub send_date_2: Option<AsezTimestamp>,
    pub send_users_2: AsezArray<i32>,
    pub is_preapproved: bool,
    pub is_removed: bool,
    pub is_actual: bool,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
}

/// Решение Эксперта ПД
#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum SdExpertConclusion {
    #[db_default]
    #[display(fmt = "Не установлено")]
    Undefined = 0,
    #[display(fmt = "Согласовано")]
    Agreed = 1,
    #[display(fmt = "Не согласовано")]
    NotAgreed = 2,
    #[display(fmt = "Не относится к компетенции")]
    NotWithinCompetence = 3,
    #[display(fmt = "Доработка")]
    Revision = 4,
    #[display(fmt = "Срок истек. Решение отсутствует")]
    DeadlineExpired = 5,
}

/// Статус утверждения.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum ApprovalStatus {
    #[db_default]
    #[display(fmt = "Новое")]
    New = 1,
    #[display(fmt = "В работе")]
    Approving = 2,
    #[display(fmt = "Завершено")]
    Approved = 3,
}

impl_join_on!(DocumentApprover:plan_id => Plan:id, left);
impl_join_on!(DocumentApprover:plan_id => ContractAmendment:id, left);
impl_join_on!(DocumentApprover:document_uuid => Attachment:object_uuid, aggr);

joined!(
    !DocumentApproverWithDocs,
    document_approver: DocumentApprover,
    plan: Plan[DocumentApprover => Plan, left],
    contract_amendment: ContractAmendment[DocumentApprover => ContractAmendment, left],
);

joined!(
    !DocumentApproverWithDocsAndItems,
    document_approver: DocumentApprover,
    plan: Plan[DocumentApprover => Plan, left],
    contract_amendment: ContractAmendment[DocumentApprover => ContractAmendment, left],
    plan_items: PlanItemFull[Plan => PlanItemFull, aggr],
    ca_items: ContractAmendmentItem[ContractAmendment => ContractAmendmentItem, aggr],
);

impl AsRef<DocumentApprover> for DocumentApprover {
    fn as_ref(&self) -> &DocumentApprover {
        self
    }
}
