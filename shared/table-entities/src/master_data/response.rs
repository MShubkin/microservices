use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, DbItem};
use shared_db_derive::DbEnum;

use serde::{Deserialize, Serialize};
use sqlx::Type;

use crate::{ColorCode, SapID, SdExpertConclusion};

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "response"]
pub struct Response {
    #[item_field_pkey]
    pub id: SdExpertConclusion,
    pub text: String,
    pub icon: Option<String>,
    pub color_code: ColorCode,
    pub note_obligation: FillMode,
    pub sap_id: SapID,
    pub is_removed: bool,
    /// Для автоматического использования
    pub is_auto: bool,
    pub created_at: AsezTimestamp,
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

/// Обязательность комментария к решению
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
pub enum FillMode {
    #[db_default]
    Undefined = 0,
    /// не проверяется
    UncheckedField = 1,
    // обязательно к заполнению
    ObligatoryField = 2,
    // запрет заполнения
    ForbiddenField = 3,
}
