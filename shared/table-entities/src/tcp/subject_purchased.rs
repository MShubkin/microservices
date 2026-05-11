use asez2_shared_db::db_item::{
    AsezTimestamp, DbAdaptor, DbItem, DbUpdateByFilter, DbUpsert,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::PricingUnitId;

/// Предметы закупки ЗЦИ
#[derive(
    Debug,
    Default,
    Clone,
    DbItem,
    DbAdaptor,
    DbUpsert,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "request_subject_purchased"]
pub struct RequestSubjectPurchased {
    /// Уникальный идентификатор записи
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Идентификатор Предмета закупки/Группы предметов закупки
    #[item_field_autogen_always]
    #[adaptor_rename = "pricing_subject_purchase_id"]
    pub id: i64,
    /// Подразделение
    pub organization_unit_id: PricingUnitId,
    /// Уникальный идентификатор вышестоящей записи
    /// Для Предметов это uuid группы, для Группы её uuid
    pub hierarchy_uuid: Uuid,
    /// Уровень иерархии
    pub hierarchy_id: i16,
    /// Наименование предмета закупки
    #[adaptor_rename = "pricing_subject_purchase_text"]
    pub contract_subject_purchase_text: String,
    /// Уникальный идентификатор родительской записи
    pub parent_uuid: Option<Uuid>,
    /// Признак удаления
    pub is_removed: bool,
    /// Дата изменения
    pub changed_at: AsezTimestamp,
    /// Изменил
    pub changed_by: i32,
    /// Дата создания
    pub created_at: AsezTimestamp,
    /// Создал
    pub created_by: i32,
}

/// Организации предметов закупки
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "partner_subject_purchased"]
pub struct PartnerSubjectPurchased {
    /// Уникальный идентификатор записи
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Уникальный идентификатор предмета закупки
    pub uuid_subject: Uuid,
    /// Организация
    pub supplier_id: i32,
    /// Признак удаления
    pub is_removed: bool,
    /// Дата изменения
    pub changed_at: AsezTimestamp,
    /// Изменил
    pub changed_by: i32,
    /// Дата создания
    pub created_at: AsezTimestamp,
    /// Создал
    pub created_by: i32,
}

impl DbUpdateByFilter for RequestSubjectPurchased {}
impl DbUpdateByFilter for PartnerSubjectPurchased {}
