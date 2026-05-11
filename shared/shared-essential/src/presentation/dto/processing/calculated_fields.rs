use super::*;
use ahash::AHashMap;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::{DbAdaptor, Value};
use asez2_tables::maths::CurrencyValue;
use fieldname_access::FieldnameAccess;

pub const START_LOTTING_DATE: &str = "start_lotting_date";
pub const START_APPROVED_DATE: &str = "start_approved_date";
pub const PRICING_WORKING_DAYS_COUNT_THRESHOLD: &str =
    "pricing_working_days_count_threshold";
pub const PRICING_PROCESS_COUNT: &str = "pricing_process_count";
pub const NUMBER_OF_DAYS_WITH_EXPERT_THRESHOLD: &str =
    "number_of_days_with_expert_threshold";
pub const START_RECEIVED_DATE: &str = "start_received_date";
pub const APPROVERS: &str = "approvers";
pub const AGENDA_ITEM_QUANTITY_THRESHOLD: &str = "agenda_item_quantity_threshold";
pub const AGENDA_ITEM_D647_QUANTITY_THRESHOLD: &str =
    "agenda_item_d647_quantity_threshold";
pub const PROTOCOL_ITEM_QUANTITY_THRESHOLD: &str =
    "protocol_item_quantity_threshold";
pub const PROTOCOL_ITEM_D647_QUANTITY_THRESHOLD: &str =
    "protocol_item_d647_quantity_threshold";
pub const ACTUAL_SUM_EXCLUDED_VAT: &str = "actual_sum_excluded_vat";
pub const COMMISSION_ECONOMY_SUM_EXCLUDED_VAT: &str =
    "commission_economy_sum_excluded_vat";
pub const COMMISSION_PERCENT_ECONOMY: &str = "commission_percent_economy";
pub const IS_COMMISSION_SUM_EQUAL_ACTUAL_SUM: &str =
    "is_commission_sum_equal_actual_sum";
pub const START_PRIMARY_EXPERT_CONTROL_DATE: &str =
    "start_primary_expert_control_date";
pub const START_DETERMINE_PRICE_DATE: &str = "start_determine_price_date";
pub const SAVINGS_IN_PERCENT: &str = "savings_in_percent";
pub const VOTE_ITERACTION_PRICE: &str = "vote_iteraction_price";

const CALCULATED_FIELDS: &[&str] = &[
    START_LOTTING_DATE,
    START_APPROVED_DATE,
    PRICING_WORKING_DAYS_COUNT_THRESHOLD,
    PRICING_PROCESS_COUNT,
    NUMBER_OF_DAYS_WITH_EXPERT_THRESHOLD,
    START_RECEIVED_DATE,
    APPROVERS,
    AGENDA_ITEM_QUANTITY_THRESHOLD,
    AGENDA_ITEM_D647_QUANTITY_THRESHOLD,
    PROTOCOL_ITEM_QUANTITY_THRESHOLD,
    PROTOCOL_ITEM_D647_QUANTITY_THRESHOLD,
    ACTUAL_SUM_EXCLUDED_VAT,
    COMMISSION_ECONOMY_SUM_EXCLUDED_VAT,
    COMMISSION_PERCENT_ECONOMY,
    IS_COMMISSION_SUM_EQUAL_ACTUAL_SUM,
    START_PRIMARY_EXPERT_CONTROL_DATE,
    START_DETERMINE_PRICE_DATE,
    VOTE_ITERACTION_PRICE,
];

pub type CalculatedProtocolItemRep = Calculated<EcProtocolItemRep>;
pub type CalculatedPlanRep = Calculated<PlanOrAmendmentRep>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
pub struct Calculated<T> {
    #[serde(flatten)]
    pub item: T,
    #[serde(flatten)]
    pub calculated: CalculatedPart,
}

#[derive(
    Deserialize, Serialize, Debug, PartialEq, Default, Clone, FieldnameAccess,
)]
pub struct CalculatedPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_working_days_count_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_process_count: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_days_with_expert_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_received_date: Option<AsezTimestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_lotting_date: Option<AsezDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_approved_date: Option<AsezTimestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item_quantity_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item_d647_quantity_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_item_quantity_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_item_d647_quantity_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_sum_excluded_vat: Option<CurrencyValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_economy_sum_excluded_vat: Option<CurrencyValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission_percent_economy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_commission_sum_equal_actual_sum: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_primary_expert_control_date: Option<AsezTimestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_determine_price_date: Option<AsezTimestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub savings_in_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote_iteraction_price: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approvers: Option<Vec<Approver>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Approver {
    pub department_id: Option<i32>,
    pub response_id: Option<SdExpertConclusion>,
}

type Result<T> = std::result::Result<T, ProcessingError>;

impl<T: DbAdaptor + Default> Calculated<T> {
    /// Функция для удобства.
    pub fn from_db_item(x: T::DbItem, fields: Option<&[&str]>) -> Self {
        let item = T::from_item(x, fields);

        Self {
            item,
            ..Default::default()
        }
    }
}

impl<T> Calculated<T> {
    pub fn map_item<F, E>(self, f: F) -> Calculated<E>
    where
        F: FnOnce(T) -> E,
    {
        let Self { calculated, item } = self;

        let new_item = f(item);

        Calculated {
            item: new_item,
            calculated,
        }
    }

    pub fn with_db_item<U, S>(self, fields: &[S]) -> Calculated<U>
    where
        S: AsRef<str>,
        U: DbAdaptor<DbItem = T>,
    {
        let Calculated { calculated, item } = self;

        let new_item = U::from_item(item, Some(fields));

        Calculated {
            item: new_item,
            calculated,
        }
    }
}

// Генерирует внутренние функции которые выглядят так
// (на пример для `pricing_working_days_count`):
// ```
// pub async fn set_pricing_working_days_count_with<'a, F, FutFn>(
//     mut self,
//     fields: &[&str],
//     closure: FutFn,
// ) -> Result<Self>
// where
//     F: futures::Future<Output = Result<u16>> + 'a + Send,
//     FutFn: FnOnce() -> F + Send + 'static,
// {
//     if fields.contains(&"pricing_working_days_count") {
//         self.pricing_working_days_count = Some(closure().await?);
//     }
//     Ok(self)
// }
// pub fn set_pricing_working_days_count(
//     mut self,
//     fields: &[&str],
//     value: u16,
// ) -> Self {
//     if fields.contains(&"pricing_working_days_count") {
//         self.pricing_working_days_count = Some(value);
//     }
//     self
// }
// ```
macro_rules! derive_set_with {
    ($field:ident, $t:ty) => {
        paste::paste! {
            pub async fn [<set_ $field _with>]<'a, F, FutFn>(
                mut self,
                fields: &[&str],
                closure: FutFn,
            ) -> Result<Self>
            where
                F: futures::Future<Output = Result<$t>> + 'a + Send,
                FutFn: FnOnce() -> F + Send + 'static,
            {
                if fields.contains(&stringify!($field)) {
                    self.calculated.$field = Some(closure().await?);
                }
                Ok(self)
            }
            pub fn [<set_ $field _unconditional>](
                mut self,
                value: $t,
            ) -> Self {
                self.calculated.$field = Some(value);
                self
            }
            pub fn [<set_ $field>](
                mut self,
                fields: &[&str],
                value: $t,
            ) -> Self {
                if fields.contains(&stringify!($field)) {
                    self.calculated.$field = Some(value);
                }
                self
            }
        }
    };
}

macro_rules! fill_calculated_values_map {
    ($map:ident, $field_struct:ident, $($field:ident),*) => {
        $(
            $map.insert(stringify!($field).to_string(), $field_struct.calculated.$field.clone().into());
        )*
    };
}

impl<T> Calculated<T> {
    // Функция для того чтобы можно было работать с PlanOrAmendmentRep.
    pub fn new(item: T) -> Self {
        Self {
            item,
            calculated: Default::default(),
        }
    }

    /// Отфильтровать поля которые не являются расчётными.
    pub fn get_calculated_fields(inp_fields: &[String]) -> Vec<&str> {
        inp_fields
            .iter()
            .filter(|x| CALCULATED_FIELDS.iter().any(|n| *n == x as &str))
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
    }

    pub fn get_calculated_values_map(&self) -> AHashMap<String, Value> {
        let mut map = AHashMap::new();
        fill_calculated_values_map!(
            map,
            self,
            start_lotting_date,
            start_approved_date,
            pricing_working_days_count_threshold,
            pricing_process_count,
            number_of_days_with_expert_threshold,
            start_received_date,
            agenda_item_quantity_threshold,
            agenda_item_d647_quantity_threshold,
            protocol_item_quantity_threshold,
            protocol_item_d647_quantity_threshold,
            actual_sum_excluded_vat,
            commission_economy_sum_excluded_vat,
            commission_percent_economy,
            is_commission_sum_equal_actual_sum,
            start_primary_expert_control_date,
            start_determine_price_date,
            vote_iteraction_price
        );
        map
    }

    derive_set_with!(approvers, Vec<Approver>);
    derive_set_with!(pricing_working_days_count_threshold, ColorThreshold);
    derive_set_with!(pricing_process_count, u16);
    derive_set_with!(number_of_days_with_expert_threshold, ColorThreshold);
    derive_set_with!(start_received_date, AsezTimestamp);
    derive_set_with!(start_lotting_date, AsezDate);
    derive_set_with!(start_approved_date, AsezTimestamp);
    derive_set_with!(start_primary_expert_control_date, AsezTimestamp);
    derive_set_with!(start_determine_price_date, AsezTimestamp);
    derive_set_with!(agenda_item_quantity_threshold, ColorThreshold);
    derive_set_with!(agenda_item_d647_quantity_threshold, ColorThreshold);
    derive_set_with!(protocol_item_quantity_threshold, ColorThreshold);
    derive_set_with!(protocol_item_d647_quantity_threshold, ColorThreshold);
    derive_set_with!(actual_sum_excluded_vat, CurrencyValue);
    derive_set_with!(commission_economy_sum_excluded_vat, CurrencyValue);
    derive_set_with!(commission_percent_economy, String);
    derive_set_with!(is_commission_sum_equal_actual_sum, bool);
    derive_set_with!(savings_in_percent, String);
    derive_set_with!(vote_iteraction_price, i64);
}

pub fn has_calculated_field(inp_fields: &[String]) -> bool {
    inp_fields
        .iter()
        .any(|field| CALCULATED_FIELDS.contains(&field.as_str()))
}
