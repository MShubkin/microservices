use asez2_shared_db::Value;
use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
    Decode, Encode, Postgres, Type,
};
use std::fmt;
use thiserror::Error;
#[cfg(test)]
mod tests;

/// Представление RGB цвета
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ColorCode {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SapID {
    pub id: [char; 20],
}

impl fmt::Display for ColorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl From<ColorCode> for Value {
    fn from(value: ColorCode) -> Self {
        value.to_string().into()
    }
}
impl From<&ColorCode> for Value {
    fn from(value: &ColorCode) -> Self {
        value.to_string().into()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ColorCodeError {
    #[error("Невалидная длина кода: ожидается {expected}, получено {found}")]
    InvalidLength { expected: usize, found: usize },
    #[error("Невалидный формат кода: {msg}")]
    InvalidFormat { msg: String },
}

impl TryFrom<&str> for ColorCode {
    type Error = ColorCodeError;

    fn try_from(hex_str: &str) -> Result<Self, Self::Error> {
        if hex_str.len() != 6 {
            return Err(ColorCodeError::InvalidLength {
                expected: 6,
                found: hex_str.len(),
            });
        }

        let res = u32::from_str_radix(hex_str, 16).map_err(|_| {
            ColorCodeError::InvalidFormat {
                msg: format!("код `{}` содержит невалидные hex символы", hex_str),
            }
        })?;

        Ok(Self {
            r: ((res & 0x00FF0000) >> 16) as u8,
            g: ((res & 0x0000FF00) >> 8) as u8,
            b: (res & 0x000000FF) as u8,
        })
    }
}

impl TryFrom<String> for ColorCode {
    type Error = ColorCodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for SapID {
    type Error = String;

    fn try_from(x: &str) -> Result<Self, Self::Error> {
        let mut id = ['0'; 20];

        if x.len() != 20 {
            return Err(format!("SAP ID '{x}' format is incorrect"));
        }
        for (idx, i) in x.chars().enumerate() {
            id[idx] = i;
        }
        Ok(Self { id })
    }
}

impl TryFrom<String> for SapID {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl ColorCode {
    /// Работает как [`TryFrom::try_from`] из строки, но с учетом `#`
    pub fn try_from_hex(x: &str) -> Result<Self, ColorCodeError> {
        if !x.starts_with('#') {
            return Err(ColorCodeError::InvalidFormat {
                msg: format!("код `{x}` должен начинаться с #"),
            });
        }

        Self::try_from(&x[1..])
    }
}

impl From<&SapID> for String {
    fn from(t: &SapID) -> String {
        t.id.iter().collect::<String>()
    }
}
impl From<SapID> for String {
    fn from(t: SapID) -> String {
        t.id.iter().collect::<String>()
    }
}
impl From<&ColorCode> for String {
    fn from(t: &ColorCode) -> String {
        t.to_string()
    }
}
impl From<ColorCode> for String {
    fn from(t: ColorCode) -> String {
        t.to_string()
    }
}

impl From<SapID> for Value {
    fn from(value: SapID) -> Self {
        value.id.iter().collect::<String>().into()
    }
}
impl From<&SapID> for Value {
    fn from(value: &SapID) -> Self {
        value.id.iter().collect::<String>().into()
    }
}

macro_rules! fixed_string_sqlx {
    ($tpe:ty, $field:ident) => {
        impl<'r> Decode<'r, Postgres> for $tpe {
            fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
                let s = <&str as Decode<'r, Postgres>>::decode(value)?;
                Self::try_from(s).map_err(Into::into)
            }
        }
        impl Encode<'_, Postgres> for $tpe {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
                String::from(self).encode_by_ref(buf)
            }
        }
        impl Type<Postgres> for $tpe {
            fn type_info() -> PgTypeInfo {
                String::type_info()
            }

            fn compatible(ty: &PgTypeInfo) -> bool {
                String::compatible(ty)
            }
        }
    };
}

fixed_string_sqlx!(ColorCode, hex_code);
fixed_string_sqlx!(SapID, id);
