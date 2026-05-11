use sqlx::Type;
use uuid::Uuid;

use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;

use asez2_shared_db::db_item::AsezTimestamp;

/// Справочник «Тип вложенного документа»

/// Id типа вложенного документа
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum AttachmentTypeId {
    /// Не задано
    #[db_default]
    Undefined = 0,
    /// Повестка
    Agenda = 1,
    /// Протокол очного заседния СК
    ProtocolInPersonEc = 2,
    /// Протокол заочного заседания СК
    ProtocolCorrespondenceEc = 3,
    /// Бюллетень
    Bulletin = 4,
}

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "attachment_type"]
pub struct AttachmentType {
    /// Id типа документам
    #[item_field_pkey]
    #[item_field_autogen]
    pub id: AttachmentTypeId,
    /// Идентификатор записи в таблице
    pub uuid: Uuid,
    /// Наименование
    #[serde(rename = "text")]
    pub name: String,
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
