use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::sample_conclusion::SampleConclusionAccessId;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};

/// Справочник "Предметы/Группы Предметов закупки АЦ"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "pricing_subject_purchase"]
pub struct PricingSubjectPurchase {
    /// Уникальный идентификатор записи
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Идентификатор предмета закупки/группы предмета закупки
    #[item_field_autogen]
    pub id: i32,
    /// Наименование предмета закупки/группы предмета
    pub text: String,
    /// Орг.единица АЦ
    pub pricing_organization_unit_id: i16,
    /// ID направления закупки
    pub purchasing_trend_id: i16,
    /// Доступ к записи
    pub access_id: SampleConclusionAccessId,
    /// Уровень иерархии
    pub hierarchy_id: i16,
    /// Уникальный идентификатор вышестоящей записи
    pub hierarchy_uuid: Uuid,
    /// Уникальный идентификатор родительской записи
    pub parent_uuid: Uuid,
    /// Запись удалена
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}
