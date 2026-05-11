use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef},
    types::time::PrimitiveDateTime,
    Decode, Encode, Postgres, Type,
};

use asez2_shared_db::db_item::AsezTimestamp;

/// Представление [`AsezTimestamp`], которое приходит от монолита
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlanningTimestamp(AsezTimestamp);

impl Deref for PlanningTimestamp {
    type Target = AsezTimestamp;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<AsezTimestamp> for PlanningTimestamp {
    fn from(value: AsezTimestamp) -> Self {
        Self(value)
    }
}

impl From<PlanningTimestamp> for AsezTimestamp {
    fn from(value: PlanningTimestamp) -> Self {
        value.0
    }
}

impl From<PlanningTimestamp> for asez2_shared_db::Value {
    fn from(value: PlanningTimestamp) -> Self {
        Self::Timestamp(value.0)
    }
}

impl Serialize for PlanningTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.unix_timestamp() * 1_000_000)
    }
}

impl<'de> Deserialize<'de> for PlanningTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Монолит присылает таймстемп в микросекундах
        let number = i64::deserialize(deserializer)? / 1_000_000;
        Ok(Self(AsezTimestamp::from_unix_timestamp(number)))
    }
}

impl<'r> Decode<'r, Postgres> for PlanningTimestamp {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        AsezTimestamp::decode(value).map(PlanningTimestamp)
    }
}
impl Encode<'_, Postgres> for PlanningTimestamp {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.0.encode_by_ref(buf)
    }
}
impl Type<Postgres> for PlanningTimestamp {
    fn type_info() -> PgTypeInfo {
        PrimitiveDateTime::type_info()
    }
}
impl PgHasArrayType for PlanningTimestamp {
    fn array_type_info() -> PgTypeInfo {
        PrimitiveDateTime::array_type_info()
    }
}

#[cfg(test)]
mod timestamp {
    use asez2_shared_db::db_item::AsezTimestamp;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::PlanningTimestamp;

    #[test]
    fn serde_timestamp() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            timestamp: PlanningTimestamp,
        }

        // 1900-1-1 00:00:00
        let timestamp = AsezTimestamp::from_unix_timestamp(-2_208_988_800);
        let test = Test {
            timestamp: timestamp.into(),
        };

        let expected = json!({
            "timestamp": -2_208_988_800_000_000i64
        });
        assert_eq!(expected, serde_json::to_value(test).unwrap());

        let deser = serde_json::from_value::<Test>(expected).unwrap();
        assert_eq!(*deser.timestamp, timestamp);
    }
}
