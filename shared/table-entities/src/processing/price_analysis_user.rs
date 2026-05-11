//! Позиции ППЗ, но уже по логике АСЕЗ-2.0
use asez2_shared_db::db_item::AsezTimestamp;
// use asez2_shared_db::db_item::DbItemExt;
use asez2_shared_db::{DbAdaptor, DbItem};
use serde::{Deserialize, Serialize};
use shared_db_derive::DbEnum;
use sqlx::Type;

use crate::PricingUnitId;

#[derive(
    Debug, Default, Clone, PartialEq, DbItem, DbAdaptor, Serialize, Deserialize,
)]
#[adaptor_derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[item_table = "price_analysis_user"]
#[item_aggr_insert]
pub struct PriceAnalysisUser {
    #[item_field_pkey]
    pub id: i32,
    pub pricing_organization_unit_id: PricingUnitId,
    pub subdivision_id: Option<i16>,
    pub type_user_id: UserType,
    pub ppz_type_id: i16,
    pub user_id: i32,
    pub env_type_id: i16,
    pub start_date: AsezTimestamp,
    pub end_date: AsezTimestamp,
    pub is_removed: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Hash,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum UserType {
    /// Не установлен
    #[db_default]
    Undefined = 0,
    /// Руководитель АЦ
    Director = 1,
    /// Эксперт АЦ
    Expert = 2,
    /// Сопровождение АЦ
    Maintenance = 3,
    /// Иные пользователи
    Other = 4,
}
