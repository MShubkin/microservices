use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

#[derive(
    Default, Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Ord, Eq,
)]
#[serde(transparent)]
/// This a wrapper around Vec<i64> used to overcome orphan rule.
pub struct AsezArray<T>(pub Vec<T>);

impl<T> AsezArray<T> {
    pub fn concat<I>(mut self, rhs: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.0.extend(rhs);
        self
    }
}

impl<T> From<Vec<T>> for AsezArray<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl PgHasArrayType for AsezArray<i64> {
    fn array_type_info() -> PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_int8")
    }
}
impl<'r> Decode<'r, Postgres> for AsezArray<i64> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        <Vec<i64> as Decode<'r, Postgres>>::decode(value).map(Self)
    }
}
impl Encode<'_, Postgres> for AsezArray<i64> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.0.encode_by_ref(buf)
    }
}
impl Type<Postgres> for AsezArray<i64> {
    fn type_info() -> PgTypeInfo {
        Self::array_type_info()
    }
}

impl PgHasArrayType for AsezArray<i32> {
    fn array_type_info() -> PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_int4")
    }
}
impl<'r> Decode<'r, Postgres> for AsezArray<i32> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        <Vec<i32> as Decode<'r, Postgres>>::decode(value).map(Self)
    }
}
impl Encode<'_, Postgres> for AsezArray<i32> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.0.encode_by_ref(buf)
    }
}
impl Type<Postgres> for AsezArray<i32> {
    fn type_info() -> PgTypeInfo {
        Self::array_type_info()
    }
}

impl PgHasArrayType for AsezArray<i16> {
    fn array_type_info() -> PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_int2")
    }
}
impl<'r> Decode<'r, Postgres> for AsezArray<i16> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        <Vec<i16> as Decode<'r, Postgres>>::decode(value).map(Self)
    }
}
impl Encode<'_, Postgres> for AsezArray<i16> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.0.encode_by_ref(buf)
    }
}
impl Type<Postgres> for AsezArray<i16> {
    fn type_info() -> PgTypeInfo {
        Self::array_type_info()
    }
}
