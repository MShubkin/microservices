use asez2_shared_db::db_item::{int_array::AsezArray, AsezDate, AsezTimestamp};

use asez2_shared_db::result::SharedDbError;
use itertools::Itertools;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display};
use std::result::Result;
use uuid::Uuid;

/// Енам, который описывает общие значения, которые могут прийти с фронта в
/// например [`UiSelect`].
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum UiValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Uuid(Uuid),
    #[serde(rename = "null")]
    Null,
    Date(AsezDate),
    Timestamp(AsezTimestamp),
    VecValue(Vec<UiValue>),
}

impl TryFrom<UiValue> for asez2_shared_db::Value {
    type Error = SharedDbError;

    fn try_from(value: UiValue) -> Result<Self, Self::Error> {
        use asez2_shared_db::Value::*;
        match value {
            UiValue::String(val) => Ok(String(val)),
            UiValue::Int(val) => Ok(Int(val)),
            UiValue::Float(val) => Ok(Float(val)),
            UiValue::Bool(val) => Ok(Bool(val)),
            UiValue::Uuid(val) => Ok(Uuid(val)),
            UiValue::Null => Ok(Null),
            UiValue::Date(val) => Ok(Date(val)),
            UiValue::Timestamp(val) => Ok(Timestamp(val)),
            // NB: При появлении новых возможных Value::VecX придется изменить
            // реализацию, чтобы и они тоже хендлились
            UiValue::VecValue(val) => {
                let arr = val
                    .into_iter()
                    .map(|v| match v {
                        UiValue::Int(v) => Ok(v),
                        _ => Err(SharedDbError::ValueError(
                            "Требуется массив чисел".into(),
                        )),
                    })
                    .collect::<Result<_, _>>()?;

                Ok(Vec64(AsezArray(arr)))
            }
        }
    }
}

impl From<UiValue> for String {
    fn from(v: UiValue) -> Self {
        match v {
            UiValue::String(s) => s,
            UiValue::Int(s) => s.to_string(),
            UiValue::Float(s) => s.to_string(),
            UiValue::Bool(s) => s.to_string(),
            UiValue::Uuid(s) => s.to_string(),
            UiValue::Date(s) => s.to_string(),
            UiValue::Timestamp(s) => s.to_string(),
            UiValue::Null => String::from("null"),
            UiValue::VecValue(values) => {
                values.iter().map(|i| i.to_string()).join(",")
            }
        }
    }
}

impl UiValue {
    fn from_str(v: &str) -> Self {
        if let Ok(t) = AsezTimestamp::try_from_api_format(v) {
            return Self::Timestamp(t);
        }
        if let Ok(d) = AsezDate::try_from_api_format(v) {
            return Self::Date(d);
        };
        match uuid::Uuid::parse_str(v) {
            Ok(u) => Self::Uuid(u),
            Err(_) => Self::String(v.to_string()),
        }
    }
}

impl From<String> for UiValue {
    fn from(v: String) -> Self {
        Self::from_str(&v)
    }
}

impl From<&str> for UiValue {
    fn from(v: &str) -> Self {
        Self::from_str(v)
    }
}

impl From<&String> for UiValue {
    fn from(v: &String) -> Self {
        Self::from_str(v as &str)
    }
}

impl<T> From<Option<T>> for UiValue
where
    T: Into<UiValue>,
{
    fn from(val: Option<T>) -> Self {
        match val {
            Some(val) => val.into(),
            None => UiValue::Null,
        }
    }
}

impl<T> From<&Option<T>> for UiValue
where
    T: Into<UiValue> + Clone,
{
    fn from(val: &Option<T>) -> Self {
        match val {
            Some(val) => val.clone().into(),
            None => UiValue::Null,
        }
    }
}

impl<T> From<Vec<T>> for UiValue
where
    T: Into<UiValue>,
{
    fn from(value: Vec<T>) -> Self {
        UiValue::VecValue(value.into_iter().map(|v| v.into()).collect())
    }
}

macro_rules! impl_into_value {
    ($tpe:ty, $var:ident, $map:expr) => {
        impl From<$tpe> for UiValue {
            fn from(x: $tpe) -> Self {
                Self::$var($map(x))
            }
        }
    };
}

macro_rules! impl_into_value_int {
  ($($tpe:ty),*) => {
      $(
          impl_into_value!($tpe, Int, |x: $tpe| x as i64);
          impl_into_value!(&$tpe, Int, |x: &$tpe| *x as i64);
      )*
  }
}

macro_rules! impl_into_value_float {
  ($($tpe:ty),*) => {
      $(
          impl_into_value!($tpe, Float, |x: $tpe| x as f64);
          impl_into_value!(&$tpe, Float, |x: &$tpe| *x as f64);
      )*
  }
}

// String conversions
impl_into_value!(AsezDate, Date, |x: AsezDate| x);
impl_into_value!(&AsezDate, Date, |x: &AsezDate| x.to_owned());
impl_into_value!(AsezTimestamp, Timestamp, |x: AsezTimestamp| x);
impl_into_value!(&AsezTimestamp, Timestamp, |x: &AsezTimestamp| x.to_owned());

//Integer conversions
impl_into_value_int!(usize, u64, u32, u16, u8, i64, i32, i16, i8);
// Float conversions
impl_into_value_float!(f64, f32);
// The rest.
impl_into_value!(bool, Bool, |x| x);
impl_into_value!(Uuid, Uuid, |x| x);
impl_into_value!(&bool, Bool, |x: &bool| *x);
impl_into_value!(&Uuid, Uuid, |x: &Uuid| *x);
impl_into_value!(AsezArray<i64>, VecValue, |x: AsezArray<i64>| x
    .0
    .into_iter()
    .map(UiValue::Int)
    .collect());
impl_into_value!(&[i64], VecValue, |x: &[i64]| x
    .iter()
    .copied()
    .map(UiValue::Int)
    .collect());
impl_into_value!(AsezArray<i32>, VecValue, |x: AsezArray<i32>| x
    .0
    .into_iter()
    .map(|v| UiValue::Int(v as i64))
    .collect());

struct ValueVisitor;

impl<'de> Deserialize<'de> for UiValue {
    fn deserialize<D>(deserializer: D) -> Result<UiValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl<'d> Visitor<'d> for ValueVisitor {
    type Value = UiValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("Expect a null, string, integer, float or boolean value")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::Int(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::Int(value as i64))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::Float(value))
    }

    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::Float(f64::from(value)))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::Bool(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::from_str(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::from_str(&value))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UiValue::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut output = Vec::new();
        while let Some(e) = seq.next_element::<UiValue>()? {
            output.push(e);
        }
        Ok(UiValue::VecValue(output))
    }
}

impl Display for UiValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiValue::String(str) => write!(f, "{}", str),
            UiValue::Int(int) => write!(f, "{}", int),
            UiValue::Float(float) => write!(f, "{}", float),
            UiValue::Bool(bool) => write!(f, "{}", bool),
            UiValue::Uuid(uuid) => write!(f, "{}", uuid),
            UiValue::Date(d) => write!(f, "{}", d.0),
            UiValue::Timestamp(t) => write!(f, "{}", t.0),
            UiValue::Null => write!(f, "null"),
            UiValue::VecValue(arr) => write!(f, "{:?}", arr),
        }
    }
}

#[cfg(test)]
mod tests {
    use time::macros::{date, datetime};

    use super::*;

    macro_rules! test_de_value {
        ($test_name:ident, $text:literal, $val:expr) => {
            #[test]
            fn $test_name() {
                let fin: UiValue =
                    serde_json::from_str($text).expect("Does not deserialize.");
                assert_eq!(fin, $val);
            }
        };
    }
    macro_rules! test_ser_value {
        ($test_name:ident, $text:literal, $val:expr) => {
            #[test]
            fn $test_name() {
                let fin =
                    serde_json::to_string(&$val).expect("Does not serialize.");
                assert_eq!(fin, $text);
            }
        };
    }

    #[test]
    fn null_test() {
        #[derive(Debug, Serialize)]
        struct NullStruct {
            null_field: UiValue,
        }

        #[derive(Debug, Deserialize)]
        struct OptionStruct {
            option_field: Option<u8>,
        }

        let null_struct = NullStruct {
            null_field: UiValue::Null,
        };
        let null_str = serde_json::to_string(&null_struct).unwrap();
        assert_eq!(null_str, "{\"null_field\":null}");
        let res: OptionStruct = serde_json::from_str(&null_str).unwrap();
        assert!(res.option_field.is_none())
    }

    test_ser_value!(test_ser_value_null, r#"null"#, UiValue::Null);
    test_ser_value!(test_ser_value_true, r#"true"#, UiValue::Bool(true));
    test_ser_value!(test_ser_value_false, r#"false"#, UiValue::Bool(false));
    test_ser_value!(
        test_ser_value_int,
        r#"-4294967296"#,
        UiValue::Int(-4294967296)
    );
    test_ser_value!(test_ser_value_int2, r#"4294967296"#, UiValue::Int(4294967296));
    test_ser_value!(test_ser_value_int3, r#"0"#, UiValue::Int(0));
    test_ser_value!(test_ser_value_int4, r#"100"#, UiValue::Int(100));
    test_ser_value!(test_ser_value_f1, r#"0.0"#, UiValue::Float(0.));
    test_ser_value!(test_ser_value_f2, r#"-999.999"#, UiValue::Float(-999.999));
    test_ser_value!(
        test_se_value_date,
        r#""09.09.1999""#,
        UiValue::Date(date!(1999 - 09 - 09).into())
    );
    // test_ser_value!(_f3, r#"NaN"#, UiValue::Float(f64::NAN));
    // test_ser_value!(_f4, r#"Inf"#, UiValue::Float(f64::INFINITY));
    test_ser_value!(
        test_ser_value_str1,
        r#""Mongeese eat snakes.""#,
        UiValue::String("Mongeese eat snakes.".to_string())
    );
    test_ser_value!(
        test_ser_value_str2,
        r#""Как можно без кириллицы?""#,
        UiValue::String("Как можно без кириллицы?".to_string())
    );
    test_ser_value!(
        test_ser_value_str3,
        r#""中國人""#,
        UiValue::String("中國人".to_string())
    );
    test_ser_value!(
        test_ser_value_uuid1,
        r#""d6229360-06fc-11ee-805c-566ff2f30017""#,
        UiValue::Uuid(
            Uuid::parse_str("d6229360-06fc-11ee-805c-566ff2f30017").unwrap()
        )
    );
    test_ser_value!(
        test_ser_value_uuid2,
        r#""83702159-6418-11ee-8037-566ff2f30017""#,
        UiValue::Uuid(
            Uuid::parse_str("83702159-6418-11ee-8037-566ff2f30017").unwrap()
        )
    );
    test_ser_value!(
        test_ser_value_uuid3,
        r#""566ff2f3-0078-1eee-89bf-a52f40e61a8d""#,
        UiValue::Uuid(
            Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()
        )
    );
    test_ser_value!(
        test_ser_value_uuid4,
        r#""bbbb638e-009d-4024-a037-031896f3f0f7""#,
        UiValue::Uuid(
            Uuid::parse_str("bbbb638e-009d-4024-a037-031896f3f0f7").unwrap()
        )
    );

    test_de_value!(test_de_value_null, r#"null"#, UiValue::Null);
    test_de_value!(test_de_value_true, r#"true"#, UiValue::Bool(true));
    test_de_value!(test_de_value_false, r#"false"#, UiValue::Bool(false));
    test_de_value!(test_de_value_int, r#"-4294967296"#, UiValue::Int(-4294967296));
    test_de_value!(test_de_value_int2, r#"4294967296"#, UiValue::Int(4294967296));
    test_de_value!(test_de_value_int3, r#"0"#, UiValue::Int(0));
    test_de_value!(test_de_value_int4, r#"100"#, UiValue::Int(100));
    test_de_value!(test_de_value_f1, r#"0.0"#, UiValue::Float(0.));
    test_de_value!(test_de_value_f2, r#"-999.999"#, UiValue::Float(-999.999));

    test_de_value!(
        test_de_value_date,
        r#""30.01.2000""#,
        UiValue::Date(AsezDate::try_from("30.01.2000").unwrap())
    );
    test_de_value!(
        test_de_value_timestamp,
        r#""30.01.2000 10:20:30""#,
        UiValue::Timestamp(AsezTimestamp::from(datetime!(2000-01-30 10:20:30)))
    );

    // test_de_value!(_f3, r#"NaN"#, UiValue::Float(f64::NAN));
    // test_de_value!(_f4, r#"Inf"#, UiValue::Float(f64::INFINITY));

    test_de_value!(
        test_de_value_str1,
        r#""Mongeese eat snakes.""#,
        UiValue::String("Mongeese eat snakes.".to_string())
    );
    test_de_value!(
        test_de_value_str2,
        r#""Как можно без кириллицы?""#,
        UiValue::String("Как можно без кириллицы?".to_string())
    );
    test_de_value!(
        test_de_value_str3,
        r#""中國人""#,
        UiValue::String("中國人".to_string())
    );
    test_de_value!(
        test_de_value_uuid1,
        r#""d6229360-06fc-11ee-805c-566ff2f30017""#,
        UiValue::Uuid(
            Uuid::parse_str("d6229360-06fc-11ee-805c-566ff2f30017").unwrap()
        )
    );
    test_de_value!(
        test_de_value_uuid2,
        r#""83702159-6418-11ee-8037-566ff2f30017""#,
        UiValue::Uuid(
            Uuid::parse_str("83702159-6418-11ee-8037-566ff2f30017").unwrap()
        )
    );
    test_de_value!(
        test_de_value_uuid3,
        r#""566ff2f3-0078-1eee-89bf-a52f40e61a8d""#,
        UiValue::Uuid(
            Uuid::parse_str("566ff2f3-0078-1eee-89bf-a52f40e61a8d").unwrap()
        )
    );
    test_de_value!(
        test_de_value_uuid4,
        r#""bbbb638e-009d-4024-a037-031896f3f0f7""#,
        UiValue::Uuid(
            Uuid::parse_str("bbbb638e-009d-4024-a037-031896f3f0f7").unwrap()
        )
    );
}
