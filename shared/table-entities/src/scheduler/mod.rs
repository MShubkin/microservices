use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{AsezDate, AsezTimestamp},
    DbAdaptor, DbItem,
};

#[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq, Serialize)]
#[adaptor_derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[item_table = "event"]
pub struct Event {
    #[item_field_pkey]
    pub id: i64,
    pub uuid: Uuid,
    pub username: String,
    pub usergroup: String,
    pub type_id: String,
    pub title: String,
    pub is_periodic: bool,
    pub repeat_rate: i64,
    pub repeat_rate_custom: i64,
    pub start_date: AsezDate,
    pub end_date: AsezDate,
    pub start_time: AsezTimestamp,
    pub end_time: AsezTimestamp,
    pub create_at: bool,
}

#[derive(DbItem, DbAdaptor, Debug, Clone, PartialEq, Serialize)]
#[adaptor_derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[item_table = "calendar_special_day"]
pub struct CalendarSpecialDay {
    /// Дата не рабочего/рабочего дня
    #[item_field_pkey]
    pub date: AsezDate,
    /// Тип для в календаре
    pub day_type_id: DayTypeId,
    /// Описание календарного дня
    pub text: String,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    DbEnum,
    Type,
    Ord,
    PartialOrd,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum DayTypeId {
    #[db_default]
    Holiday = 1,
    Workday = 2,
}
