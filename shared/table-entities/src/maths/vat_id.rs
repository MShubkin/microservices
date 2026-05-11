use super::*;
use shared_db_derive::DbEnum;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    sqlx::Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
#[display(fmt = "{}", self as i16)]
pub enum VatId {
    #[db_default]
    Unspecified = 0,
    NoVat = 1,
    R0 = 2,
    R10 = 3,
    R18 = 4,
    Compound = 5,
    R20 = 6,
    R12 = 7,
    R21 = 8,
    R13 = 9,
    R25 = 10,
    R15 = 11,
    R11 = 12,
    R5 = 14,
    R7 = 15,
}

impl VatId {
    /// This converts VAT code ids to VAT percentage (as a multiplier between 0 and 1)
    /// NB: THis function should be used in conjunction with [`calculate_vat`](shared_essential::common::math::calculate_vat).
    /// The naming convention for variants is as follows.
    ///
    /// Rate = XX% -> VatId::RXX, so Rate = 20% -> VatId::R20
    pub fn rate(self) -> currency::CurrencyRate {
        let x = match self {
            VatId::NoVat => 0,
            VatId::R0 => 0,
            VatId::R10 => 10_000,
            VatId::R18 => 18_000,
            VatId::R20 => 20_000,
            VatId::R12 => 12_000,
            VatId::R21 => 21_000,
            VatId::R13 => 13_000,
            VatId::R25 => 25_000,
            VatId::R15 => 15_000,
            VatId::R11 => 11_000,
            VatId::R5 => 5_000,
            VatId::R7 => 7_000,
            _ => 0,
        };
        currency::CurrencyRate(x)
    }

    pub fn vat(self, sum: CurrencyValue) -> CurrencyValue {
        self.rate().convert_value(sum)
    }

    pub fn with_vat(self, sum: CurrencyValue) -> CurrencyValue {
        sum + self.vat(sum)
    }
}

impl FromStr for VatId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<i16>()?.into())
    }
}
