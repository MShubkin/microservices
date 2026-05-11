use crate::db_item::int_array::AsezArray;
use crate::db_item::{AsezDate, AsezTimestamp};

use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use std::fmt::{self, Display};
use std::result::Result;
use uuid::Uuid;

/// This is a data structure to allow serialization of IntWithOriginal.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct IntWithOriginal {
    /// The fixed precision integer value.
    pub int: i64,
    /// The floating point value it represents.
    pub original: f64,
    /// The precision value that is used to transform the `int` to the `original`.
    /// It also represents the cut-off of precision for the `original` value.
    /// Hence if int=100_234, original=100.234002432 and precision=3, then
    /// original should be written as "100.234".
    pub precision: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // It is nice to be able to test this.]
#[serde(tag = "t", content = "v")]
/// This is a data transfer object (DTO), to enable our filters to work better.
pub enum Value {
    String(String),
    /// This can represent any integer, we can be picky later
    Int(i64),
    /// An integer value along with its original floating-point representation and precision.
    /// It is used for exporting original values such as CurrencyValue, Quantity.
    IntWithOriginal(IntWithOriginal),
    /// Again this can represent any float, and we can be picky later.
    Float(f64),
    Bool(bool),
    Uuid(Uuid),
    Null,
    Vec64(AsezArray<i64>),
    Vec32(AsezArray<i32>),
    Vec16(AsezArray<i16>),
    Date(AsezDate),
    Timestamp(AsezTimestamp),
}

use sqlx::types::time::{Date, PrimitiveDateTime};
/// Used purely for =ANY queries.
#[derive(Debug)]
pub(crate) enum AggrValue<'a> {
    String(Vec<&'a str>),
    Int(Vec<i64>),
    Float(Vec<f64>),
    Bool(Vec<bool>),
    Uuid(Vec<Uuid>),
    Vec64(Vec<i64>),
    Date(Vec<Date>),
    Timestamp(Vec<PrimitiveDateTime>),
    Null,
}

impl<'a> AggrValue<'a> {
    pub(crate) fn build(values: &'a [Value]) -> Self {
        use Value::*;
        let mut output = AggrValue::Null;
        values.iter().filter(|x| !matches!(x, &Value::Null)).for_each(|x| {
            match (x, &mut output) {
                (String(x), AggrValue::String(ref mut out)) => out.push(x.as_ref()),
                (
                    Int(x)
                    | IntWithOriginal(crate::value::IntWithOriginal {
                        int: x, ..
                    }),
                    AggrValue::Int(ref mut out),
                ) => out.push(*x),
                (Float(x), AggrValue::Float(ref mut out)) => out.push(*x),
                (Bool(x), AggrValue::Bool(ref mut out)) => out.push(*x),
                (Uuid(x), AggrValue::Uuid(ref mut out)) => out.push(*x),
                (Vec64(x), AggrValue::Vec64(ref mut out)) => {
                    out.extend(x.0.iter().copied())
                }
                (Date(x), AggrValue::Date(ref mut out)) => out.push(x.0),
                (Timestamp(x), AggrValue::Timestamp(ref mut out)) => out.push(x.0),
                (String(x), AggrValue::Null) => {
                    output = AggrValue::String(vec![x.as_str()])
                }
                (Int(x), AggrValue::Null) => output = AggrValue::Int(vec![*x]),
                (Float(x), AggrValue::Null) => output = AggrValue::Float(vec![*x]),
                (Bool(x), AggrValue::Null) => output = AggrValue::Bool(vec![*x]),
                (Uuid(x), AggrValue::Null) => output = AggrValue::Uuid(vec![*x]),
                (Vec64(x), AggrValue::Null) => {
                    output = AggrValue::Vec64(x.0.clone())
                }
                (Date(x), AggrValue::Null) => output = AggrValue::Date(vec![x.0]),
                (Timestamp(x), AggrValue::Null) => {
                    output = AggrValue::Timestamp(vec![x.0])
                }
                _ => {}
            }
        });
        // println!("input: {:?}", values);
        // println!("output: {:?}", output);
        output
    }
}

impl From<Value> for String {
    fn from(v: Value) -> Self {
        match v {
            Value::String(s) => s,
            Value::Int(s) => s.to_string(),
            Value::IntWithOriginal(IntWithOriginal { int, .. }) => int.to_string(),
            Value::Float(s) => s.to_string(),
            Value::Bool(s) => s.to_string(),
            Value::Uuid(s) => s.to_string(),
            Value::Date(s) => s.to_string(),
            Value::Timestamp(s) => s.to_string(),
            Value::Null => String::from("null"),
            Value::Vec64(AsezArray(x)) => {
                x.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
            }
            Value::Vec32(AsezArray(x)) => {
                x.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
            }
            Value::Vec16(AsezArray(x)) => {
                x.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
            }
        }
    }
}

impl TryFrom<&Value> for Uuid {
    type Error = crate::result::SharedDbError;
    fn try_from(x: &Value) -> Result<Self, Self::Error> {
        match x {
            Value::Uuid(x) => Ok(*x),
            x => {
                Err(Self::Error::ValueError(format!("{:?} is not a valid Uuid", x)))
            }
        }
    }
}

impl Value {
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

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::from_str(&v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::from_str(v)
    }
}
impl From<Json<Value>> for Value {
    fn from(value: Json<Value>) -> Self {
        value.0
    }
}

impl From<&String> for Value {
    fn from(v: &String) -> Self {
        Self::from_str(v as &str)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(val: Option<T>) -> Self {
        match val {
            Some(val) => val.into(),
            None => Value::Null,
        }
    }
}

impl<T> From<&Option<T>> for Value
where
    T: Into<Value> + Clone,
{
    fn from(val: &Option<T>) -> Self {
        match val {
            Some(val) => val.clone().into(),
            None => Value::Null,
        }
    }
}

macro_rules! impl_into_value {
    ($tpe:ty, $var:ident, $map:expr) => {
        impl From<$tpe> for Value {
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
impl_into_value!(AsezArray<i64>, Vec64, |x| x);
impl_into_value!(AsezArray<i32>, Vec32, |x| x);
impl_into_value!(AsezArray<i16>, Vec16, |x| x);
impl_into_value!(Vec<i64>, Vec64, AsezArray);
impl_into_value!(Vec<i32>, Vec32, AsezArray);
impl_into_value!(Vec<i16>, Vec16, AsezArray);
impl_into_value!(&[i64], Vec64, |x: &[i64]| AsezArray(x.to_vec()));
impl_into_value!(&[i32], Vec32, |x: &[i32]| AsezArray(x.to_vec()));
impl_into_value!(&[i16], Vec16, |x: &[i16]| AsezArray(x.to_vec()));

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(str) => write!(f, "{}", str),
            Value::Int(int) => write!(f, "{}", int),
            Value::IntWithOriginal(IntWithOriginal { int, .. }) => {
                write!(f, "{}", int)
            }
            Value::Float(float) => write!(f, "{}", float),
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Uuid(uuid) => write!(f, "{}", uuid),
            Value::Date(d) => write!(f, "{}", d.0),
            Value::Timestamp(t) => write!(f, "{}", t.0),
            Value::Null => write!(f, "null"),
            Value::Vec64(arr) => write!(f, "{:?}", arr),
            Value::Vec32(arr) => write!(f, "{:?}", arr),
            Value::Vec16(arr) => write!(f, "{:?}", arr),
        }
    }
}

/// Проверяет, что контейнер `values` содержит ровно одно значение, и возвращает его.
/// В противном случае возвращает ошибку.
///
/// Используется для операций с фильтрами.
pub fn single_value<V, T>(values: V) -> Result<T, &'static str>
where
    V: IntoIterator<Item = T>,
    V::IntoIter: Iterator<Item = T> + ExactSizeIterator,
{
    let mut iter = values.into_iter();
    let len = iter.len();
    let next = iter.next();
    match (next, len) {
        (Some(value), 1) => Ok(value),
        _ => Err("ожидается ровно одно значение"),
    }
}

/// Проверяет, что контейнер `values` содержит ровно два значения, и возвращает их.
/// В противном случае возвращает ошибку.
///
/// Используется для операций с фильтрами.
pub fn two_values<V, T>(values: V) -> Result<(T, T), &'static str>
where
    V: IntoIterator<Item = T>,
    V::IntoIter: Iterator<Item = T> + ExactSizeIterator,
{
    let mut iter = values.into_iter();
    let len = iter.len();
    let v1 = iter.next();
    let v2 = iter.next();
    match (v1, v2, len) {
        (Some(v1), Some(v2), 2) => Ok((v1, v2)),
        _ => Err("ожидается ровно два значения"),
    }
}
