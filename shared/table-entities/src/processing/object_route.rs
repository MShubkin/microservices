//! Отвечает а объекты с таблицы `object_route`.
use asez2_shared_db::db_item::{AsezTimestamp, DbAdaptor, DbItem};
use shared_db_derive::{DbEnum, DbItemExt};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use sqlx::error::BoxDynError;
use sqlx::Type;

/// TODO: Investigate array in array to be able to use:
#[derive(Debug, Default, Clone, DbItem, DbItemExt, DbAdaptor, PartialEq)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "object_route"]
#[item_aggr_insert]
pub struct ObjectRoute {
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    pub route_uuid: Uuid,
    // Should be 'ПД' or 'АЦ'.
    pub designation_type: DesignationType,
    pub responsible_unit_id: i64,
    // Somehow joins `department.price_department_id`.
    pub price_department_id: Option<i64>,
    pub executor_id: i64,
    // Somehow joins `executor_method.id`
    pub executor_method_id: i16,
    pub status: i16,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub created_at: AsezTimestamp,
    #[item_field_activate_with = "AsezTimestamp::now()"]
    pub changed_at: AsezTimestamp,
    pub created_by: i32,
    pub changed_by: i32,
}

/// 'АЦ' или 'SK'
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, DbEnum, Type)]
#[serde(untagged)]
#[repr(i16)]
pub enum DesignationType {
    #[serde(rename = "не установлено")]
    #[db_default]
    Undefined = 0,
    #[serde(rename = "СК")]
    EstimatedCommission = 1,
    #[serde(rename = "АЦ")]
    PriceAnalysis = 2,
}

impl From<DesignationType> for &str {
    fn from(x: DesignationType) -> Self {
        match x {
            DesignationType::Undefined => "не установлено",
            DesignationType::EstimatedCommission => "СК",
            DesignationType::PriceAnalysis => "АЦ",
        }
    }
}

#[derive(Debug, Clone)]
struct DesignationError(String);

impl std::fmt::Display for DesignationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for DesignationError {}

impl TryFrom<&str> for DesignationType {
    type Error = BoxDynError;

    fn try_from(x: &str) -> Result<Self, Self::Error> {
        match x {
            "СК" => Ok(Self::EstimatedCommission),
            "АЦ" => Ok(Self::PriceAnalysis),
            x => {
                let msg = format!(
                    "Only 'СК' and 'АЦ' allowed for designation, got {}",
                    x
                );
                Err(Box::new(DesignationError(msg)))
            }
        }
    }
}
