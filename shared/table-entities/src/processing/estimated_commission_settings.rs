//! Отвечает а объекты с таблицы `estimated_commission_settings`.
use asez2_shared_db::db_item::{DbAdaptor, DbItem};

use serde::{Deserialize, Serialize};

use crate::CommissionKind;

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "estimated_commission_settings"]
#[item_aggr_insert]
pub struct EsSettings {
    #[item_field_pkey]
    pub commission_kind_id: CommissionKind,
    pub parameter: i32,
    pub selection_option: Option<String>,
    pub content_field_high: Option<String>,
    pub content_field_low: Option<String>,
}
