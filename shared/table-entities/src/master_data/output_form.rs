use crate::master_data::ObjectTypeId;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};

/// Справочник "Выходных форм"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "output_form"]
pub struct OutputForm {
    /// Идентификатор записи в таблице
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: i16,
    /// Тип объекта: НБ: Есть ещё разговоры, должен ли это быть УУИДом
    /// записи из таблицы "object_type"?
    pub object_type: ObjectTypeId,
    /// Код
    pub code: String,
    /// Наименование статуса
    pub text: String,
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
