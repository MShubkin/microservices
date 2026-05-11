use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem, DbUpsert};
use serde::{Deserialize, Serialize};
use shared_db_derive::{DbEnum, DbItemExt};
use sqlx::Type;
use uuid::Uuid;

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, DbUpsert, PartialEq, DbItemExt,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "attachment"]
#[item_aggr_insert]
pub struct Attachment {
    #[item_field_pkey]
    pub uuid: Uuid,
    /// УУИД файла в opentext
    pub object_uuid: Uuid,
    #[adaptor_rename = "id"]
    pub number: i16,
    #[adaptor_rename = "kind"]
    pub kind_id: AttachmentKind,
    #[adaptor_rename = "text"]
    pub name: String,
    #[adaptor_rename = "parent_id"]
    pub parent_number: Option<i16>,
    pub category_id: CategoryId,
    pub mime_id: i16, // Не приходит с фронта
    pub size: i64,
    pub is_removed: bool,
    pub is_classified: bool,
    pub pricing_version: i16,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    Eq,
    PartialOrd,
    PartialEq,
    Type,
    DbEnum,
)]
#[repr(i16)]
#[serde(into = "i16", from = "i16")]
pub enum AttachmentKind {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    File = 1,
    Directory = 2,
    Link = 3,
}

/// Тип документв
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    Eq,
    PartialOrd,
    PartialEq,
    Type,
    DbEnum,
)]
#[repr(i16)]
#[serde(into = "i16", from = "i16")]
pub enum CategoryId {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    /// Повестка
    Agenda = 1,
    /// Протокол очного заседания СК
    ProtocolInPersonEc = 2,
    /// Протокол заочного заседния СК
    ProtocolCorrespondenceEc = 3,
    /// Бюллетень
    Bulletin = 4,
    /// Сметы
    Estimates = 5,
    /// Справка-обоснование потребности
    JustificationOfDemands = 6,
    /// Документация
    Documentation = 7,
    /// Расчеты АЦ
    EcAccounting = 8,
    /// Документы ТКП
    TkpDocuments = 9,
    /// Доп. документы АЦ
    EcExtraDocuments = 10,
}
