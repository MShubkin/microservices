use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};

/// Справочник "Производственный календарь"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "scheduler_catalog"]
pub struct SchedulerRequestUpdateCatalog {
    /// Идентификатор записи в таблице
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: i32,
    /// Значение позиции справочника
    pub event_name: String,
    /// Дата к которой относится позиция
    pub event_date: AsezDate,
    /// Период времени к которому относится позиция (В горизонте "Годы")
    pub period_time: i16,
    /// Запись удалена
    pub is_removed: bool,
    /// Создано
    pub created_at: AsezTimestamp,
    /// Изменено
    pub changed_at: AsezTimestamp,
    /// Создатель
    pub created_by: i32,
    /// Кем изменено
    pub changed_by: i32,
}
