pub mod error;
pub use error::SchedulerError;

pub mod calendar;
pub use calendar::*;

use asez2_shared_db::db_item::AsezDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RelationEventObject {
    pub id: i64,
    pub event_id: i64,
    pub object_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Object {
    pub id: i64,
    pub type_id: i64,
    pub object_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelationEventTaskObject {
    pub id: i64,
    pub event_task_id: i64,
    pub object_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvenTask {
    pub id: i64,
    pub event_id: i64,
    pub date: AsezDate,
    pub title: String,
    pub date_type: i64,
    pub date_id: i64,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct GetWeek {
    pub day_from_last_week: AsezDate,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct GetHolidayResponse {
    pub type_day: String,
    pub date: String,
}

#[derive(Serialize, Debug)]
pub struct ProductionDirectoryRequest {
    pub get_directory: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductionDirectoryResponse {
    pub items: Vec<ProductItems>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductItems {
    pub id: i64,
    pub position_text: String,
    pub position_date: AsezDate,
    pub period_time: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct LastWorkday {
    pub last_workday: String,
}
