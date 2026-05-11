//! A module to hold currency related types.
use super::*;

use core::ops::{Add, AddAssign, Sub};
use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use std::fmt::{Display, Formatter, Result as FmtResult};

use asez2_shared_db::{IntWithOriginal, Value};

/// A structure that represents a SqlxValue in currency.
/// We do not need this structure for currency calculations, but it
/// does make our coding a little bit less error prone, since  we know immediately
/// if a field is currency related.
/// Currency is stored as a pseudo-fixed point (long long), with two decimal points.
/// In short if the currency is pounds, we store pence. If the currency is rubles, we
/// store kopeikas.
///
/// Notably monetary points are integers that pretend to be values with two decimal places
/// (all values are x100, eg 99.99rub is 9,999).
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct CurrencyValue(pub(crate) i64);

/// A structure to represent currency conversions:
///
/// All currency rates are conversions to RUB pretend to be values with 5 decimal places
/// (all values are x100,000, eg 93.457rub/eur is 9,345,700,  0.20017rub/kzt is 20,017).
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct CurrencyRate(pub(crate) i64);

/// A structure to represent quantities.
///
/// All quantities are integers that pretend to be values with 3 decimal places
/// (all values are x1000, eg 1.534m3 of gas is 1534).
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Quantity(pub(super) i64);

impl Add for CurrencyValue {
    type Output = Self;

    // Required method
    fn add(self, rhs: Self) -> Self::Output {
        CurrencyValue(self.0 + rhs.0)
    }
}

impl AddAssign for CurrencyValue {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl CurrencyRate {
    /// Represents a currency rate of 1.
    pub const ONE: Self = Self(CURRENCY_RATE_RATIO);

    /// Converts a SqlxValue using the rate.
    pub fn convert_value(&self, v: CurrencyValue) -> CurrencyValue {
        CurrencyValue(convert_currency(v.0, self.0))
    }

    /// Get conversion based on currency id. For now it is a stub.
    pub fn get_conversion(&self, currency_id: i16) -> Self {
        let x = get_currency_conversion(self.0, currency_id);
        Self(x)
    }
}
impl CurrencyValue {
    /// Tests if a sum is less than zero. This is a useful comparison
    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }
    // Tests if a sum is greater than zero.
    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }
}

impl Quantity {
    pub fn sum_value(&self, v: CurrencyValue) -> CurrencyValue {
        CurrencyValue(sum_x_quant(v.0, self.0))
    }
}

macro_rules! derive_db_currency {
    ($tpe:ty, $constant:expr, $sf:expr) => {
        // Convert a raw SqlxValue into the types.
        // In this case, for example, 100 will become $100. The inner value will be,
        // for example, 10_000 and this will be recorded in the DB.
        impl From<i64> for $tpe {
            fn from(x: i64) -> Self {
                Self(x * $constant)
            }
        }
        // Convert a raw SqlxValue into the types.
        // In this case, for example, 100 will become $100. The inner value will be,
        // for example, 10_000 and this will be recorded in the DB.
        impl From<f64> for $tpe {
            fn from(x: f64) -> Self {
                Self((x * $constant as f64) as i64)
            }
        }

        impl $tpe {
            /// Convert a raw SqlxValue into the types.
            /// In this case, for example, 100 will become $100. The inner value will be,
            /// for example, 10_000 and this will be recorded in the DB.
            ///
            /// NB: `TryFrom` should not be used since it conflicts with From.
            pub fn from_f64(x: f64) -> std::result::Result<Self, CurrencyError> {
                if x.abs() * $constant as f64 > i64::MAX as f64 {
                    return Err(CurrencyError::Float(x));
                }
                Ok(Self((x * $constant as f64) as i64))
            }
            /// Convert a raw SqlxValue into the types.
            /// In this case, for example, 100 will become $100. The inner value will be,
            /// for example, 10_000 and this will be recorded in the DB.
            ///
            /// NB: `TryFrom` should not be used since it conflicts with From.
            pub fn from_i64(x: i64) -> std::result::Result<Self, CurrencyError> {
                if x.abs() as i128 * $constant as i128 > i64::MAX as i128 {
                    return Err(CurrencyError::Int(x));
                }
                Ok(Self(x * $constant))
            }
        }
        // Convert self into a raw number, so $100, represented as 10_000 will become 100.
        // These functions will round as appropriate
        impl From<$tpe> for i64 {
            fn from(x: $tpe) -> Self {
                roundn(x.0, $constant) / $constant
            }
        }
        impl From<$tpe> for f64 {
            fn from(x: $tpe) -> Self {
                x.0 as f64 / $constant as f64
            }
        }
        impl From<$tpe> for Value {
            fn from(x: $tpe) -> Self {
                Self::IntWithOriginal(IntWithOriginal {
                    int: x.0,
                    original: x.into(),
                    precision: $sf,
                })
            }
        }
        impl Display for $tpe {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                let rem = self.0 % $constant;
                let main = self.0 / $constant;
                let sign = match rem < 0 && main == 0 {
                    true => "-",
                    false => "",
                };
                write!(f, "{sign}{main}.{:0l$}", rem.abs(), l = $sf)
            }
        }
        impl Sub for $tpe {
            type Output = Self;

            // Required method
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<'r> Decode<'r, Postgres> for $tpe {
            fn decode(v: PgValueRef<'r>) -> Result<Self, BoxDynError> {
                <i64 as Decode<'r, Postgres>>::decode(v).map(Self)
            }
        }
        impl Encode<'_, Postgres> for $tpe {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
                self.0.encode_by_ref(buf)
            }
        }
        impl Type<Postgres> for $tpe {
            fn type_info() -> PgTypeInfo {
                i64::type_info()
            }
        }
        impl PgHasArrayType for $tpe {
            fn array_type_info() -> PgTypeInfo {
                i64::array_type_info()
            }
        }
    };
}

derive_db_currency!(CurrencyRate, CURRENCY_RATE_RATIO, 5);
derive_db_currency!(CurrencyValue, CURRENCY_VALUE_RATIO, 2);
derive_db_currency!(Quantity, QUANTITY_RATIO, 3);
