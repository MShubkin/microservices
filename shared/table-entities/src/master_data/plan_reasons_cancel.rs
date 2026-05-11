use asez2_shared_db::db_item::{
    int_array::AsezArray, AsezTimestamp, DbAdaptor, DbItem, DbUpdateByFilter,
    DbUpsert,
};
use asez2_shared_db::{impl_join_on, joined};
use serde::{Deserialize, Serialize};

impl_join_on!(PlanReasonCancelHeader:id => PlanReasonCancelCustomer:plan_reason_cancel_id, aggr);

joined!(
    !JoinedPlanReasonsCancel,
    header: PlanReasonCancelHeader,
    customers: PlanReasonCancelCustomer[PlanReasonCancelHeader => PlanReasonCancelCustomer, aggr],
);

impl AsRef<JoinedPlanReasonsCancel> for JoinedPlanReasonsCancel {
    fn as_ref(&self) -> &JoinedPlanReasonsCancel {
        self
    }
}

///Справочник "Причины аннулирования"
#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize, DbItem, DbAdaptor,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[adaptor_fields_with_values]
#[item_table = "plan_reason_cancel"]
pub struct PlanReasonCancelHeader {
    #[item_field_pkey]
    #[item_field_autogen_always]
    /// ID причины аннулирования
    pub id: i32,
    /// Наименование причины
    pub text: String,
    /// ID сферы влияния (PlanReasonCancelImpactArea)
    pub impact_area_id: i16,
    /// Объективная причина
    pub is_objective_reason: bool,
    /// Новая ППЗ/ДС
    pub is_new_plan: bool,
    /// Признак удаления
    pub is_removed: bool,
    ///Автоматическое заполнение причины
    pub is_reason_fill_type: bool,
    /// Функциональность (PlanReasonCancelFunctionality)
    pub functionality_id_list: AsezArray<i16>,
    /// Проверки для ППЗ (PlanReasonCancelCheckReason)
    pub check_reason_id: i16,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
}

impl DbUpdateByFilter for PlanReasonCancelHeader {}

///Заказчики для причин аннулирования
#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize, DbItem, DbAdaptor,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "plan_reason_customer"]
pub struct PlanReasonCancelCustomer {
    #[item_field_autogen_always]
    pub id: i32,
    #[item_field_pkey]
    pub plan_reason_cancel_id: i32,
    #[item_field_pkey]
    pub customer_id: i32,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
}

impl DbUpsert for PlanReasonCancelCustomer {}

impl DbUpdateByFilter for PlanReasonCancelCustomer {}

///Справочник "Основания аннулирования"
#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize, DbItem, DbAdaptor,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "plan_reason_cancel_impact_area"]
pub struct PlanReasonCancelImpactArea {
    #[item_field_pkey]
    #[item_field_autogen_always]
    pub id: i16,
    pub text: String,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
}

///Справочник "Функциональность"
#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize, DbItem, DbAdaptor,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "plan_reason_cancel_functionality"]
pub struct PlanReasonCancelFunctionality {
    #[item_field_pkey]
    #[item_field_autogen_always]
    pub id: i16,
    pub text: String,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
}

///Справочник "Проверки для ППЗ"
#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize, DbItem, DbAdaptor,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "plan_reason_cancel_check_reason"]
pub struct PlanReasonCancelCheckReason {
    #[item_field_pkey]
    #[item_field_autogen_always]
    pub id: i16,
    pub text: String,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_at: AsezTimestamp,
    pub changed_by: i32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum CheckReason {
    Publication,
    PriceSchedule,
    Protocol,
    Unknown(i16),
}

impl From<i16> for CheckReason {
    fn from(value: i16) -> Self {
        match value {
            1 => CheckReason::Publication,
            2 => CheckReason::PriceSchedule,
            3 => CheckReason::Protocol,
            _ => CheckReason::Unknown(value),
        }
    }
}

impl From<CheckReason> for i16 {
    fn from(reason: CheckReason) -> Self {
        match reason {
            CheckReason::Publication => 1,
            CheckReason::PriceSchedule => 2,
            CheckReason::Protocol => 3,
            CheckReason::Unknown(value) => value,
        }
    }
}
