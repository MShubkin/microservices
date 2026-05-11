use asez2_shared_db::db_item::AsezDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "request")]
pub enum CalendarReq {
    DateAfterWorkdays(DateAfterWorkdaysReq),
    WorkdaysBetweenDates(WorkdaysBetweenDatesReq),
}

/// Для создания связи между элементами запроса
/// и ответа. Идентификатор используется только в
/// множественной форме
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Identifiable<T> {
    Id(Uuid, T),
    None(T),
}

impl<T> Identifiable<T> {
    pub fn value(&self) -> &T {
        match self {
            Identifiable::Id(_, v) => v,
            Identifiable::None(v) => v,
        }
    }

    pub fn with_same_id<U>(self, val: U) -> Identifiable<U> {
        match self {
            Identifiable::Id(id, _) => Identifiable::Id(id, val),
            Identifiable::None(_) => Identifiable::None(val),
        }
    }
}

/// Запрос на получение даты спустя
/// определенное количество рабочих дней
pub type DateAfterWorkdaysReq = Vec<Identifiable<DateAfterWorkdaysReqItem>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct DateAfterWorkdaysReqItem {
    /// Точка отсчета
    pub date: AsezDate,
    /// Количество рабочих дней,
    /// может быть отрицательным
    pub work_days: i32,
}

/// Ответ на [`DateAfterWorkdaysReq`]
///
/// Дата спустя определенное количество рабочих дней
pub type DateAfterWorkdaysRes = Vec<Identifiable<AsezDate>>;

/// Вычислить кол-во рабочих дней между датами
pub type WorkdaysBetweenDatesReq = Vec<Identifiable<WorkdaysBetweenDatesReqItem>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkdaysBetweenDatesReqItem {
    pub from_date: AsezDate,
    pub to_date: AsezDate,
}

/// Ответ на [`WorkdaysBetweenDatesReq`]
///
/// Количество рабочих дней между датами
pub type WorkdaysBetweenDatesRes = Vec<Identifiable<u32>>;
