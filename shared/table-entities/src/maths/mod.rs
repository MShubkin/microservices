//! This implements some basic maths that follows the logic of the original "planning"
//! module that uses pseudo-fixed point logic.
//!
//! Notably monetary points are integers that pretend to be values with two decimal places
//! (all values are x100, eg 99.99rub is 9,999).
//!
//! All quantities are integers that pretend to be values with 3 decimal places
//! (all values are x1000, eg 1.534m3 of gas is 1534).
//!
//! All currency rates are conversions to RUB pretend to be values with 5 decimal places
//! (all values are x100,000, eg 93.457rub/eur is 9,345,700,  0.20017rub/kzt is 20,017).
pub use currency::{CurrencyRate, CurrencyValue, Quantity};
pub use vat_id::VatId;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CurrencyError {
    #[error("'{0}' слишком велико чтобы задействовать в валютах.")]
    Int(i64),
    #[error("'{0}' слишком велико чтобы задействовать в валютах.")]
    Float(f64),
}

/// Currency id for Russian rub.
pub const CURRENCY_ID_RUB: i16 = 643;
pub const CURRENCY_RATE_RATIO: i64 = 100_000;
pub const CURRENCY_VALUE_RATIO: i64 = 100;
pub(super) const QUANTITY_RATIO: i64 = 1000;

/// A function to round a number, X, to the nearest N.
/// For sanity, N should be a multiple of ten, although it can be
/// whole number.
pub fn roundn(x: i64, n: i64) -> i64 {
    let rem = x % n;
    let x = x - rem;
    if rem < n / 2 {
        x
    } else {
        x + n
    }
}

/// Gets a total, multiplying a monetary sum, S, by a quantity, Q.
/// Since in the old system a value is multiplied by a 100 to act like
/// a fixed-decimal type, and the final type is also a monetary sum we round
/// to the nearest meaningful value (100).
/// We also divide by a 1000 since quantity Q is always a multiple of 1000.
pub fn sum_x_quant<I: Into<i64>>(s: i64, q: I) -> i64 {
    roundn(s * q.into(), QUANTITY_RATIO) / QUANTITY_RATIO // Quantity is always x1000.
}
/// Gets a total, which is still a sum (no division by 1000 is needed)
pub fn sum_x_quant_opt<I: Into<i64>>(s: Option<i64>, q: Option<I>) -> Option<i64> {
    let s = s.unwrap_or(0);
    let q = q.map(Into::into).unwrap_or(0);
    Some(sum_x_quant(s, q))
}

/// 643 is RUB. In this case we do not convert.
/// Since rates are always multipled x100,000, we use 100_000 as the base rate.
pub(super) fn get_currency_conversion(rate: i64, currency: i16) -> i64 {
    match currency {
        CURRENCY_ID_RUB => CURRENCY_RATE_RATIO,
        _ => rate,
    }
}

/// convert a monetary value V to RUB based on `rate`.
/// NB: Currency rate is always x100,000.
pub fn convert_currency(v: i64, rate: i64) -> i64 {
    roundn(v * rate, CURRENCY_RATE_RATIO) / CURRENCY_RATE_RATIO
}

mod currency;
#[cfg(test)]
mod tests;
mod vat_id;
