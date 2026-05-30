use crate::db_item::int_array::AsezArray;
use crate::db_item::{AsezDate, AsezTimestamp};

use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use std::fmt::{self, Display};
use std::result::Result;
use uuid::Uuid;

/// Фиксированная точность: хранит целое число `int` вместо `f64`, чтобы
/// избежать ошибок плавающей точки при сравнениях и сортировках в БД.
///
/// `original` и `precision` нужны только для отображения на фронтенде:
/// `original = int / 10^precision` (с усечением до `precision` знаков).
/// Например: `int=100_234`, `precision=3` → показываем `"100.234"`.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct IntWithOriginal {
    /// Целочисленное представление значения.
    pub int: i64,
    /// Оригинальное число с плавающей точкой для отображения.
    pub original: f64,
    /// Количество знаков после запятой у `original`.
    pub precision: u8,
}

/// Универсальный тип значения для фильтров и экспорта.
///
/// Используется как общий язык между фронтендом и запросами к БД:
/// сериализуется в JSON с тегом `t` (тип) и `v` (значение), что позволяет
/// фронтенду передавать произвольные значения фильтров без типобезопасных структур.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum Value {
    String(String),
    /// Любое целое число; при необходимости уточняется позже.
    Int(i64),
    /// Целое число вместе с оригинальным float-представлением.
    /// Нужен для экспорта денежных сумм и количеств с сохранением
    /// исходного вида числа.
    IntWithOriginal(IntWithOriginal),
    /// Любое число с плавающей точкой; уточняется при необходимости.
    Float(f64),
    Bool(bool),
    Uuid(Uuid),
    /// Отсутствие значения -- соответствует SQL NULL.
    Null,
    Vec64(AsezArray<i64>),
    Vec32(AsezArray<i32>),
    Vec16(AsezArray<i16>),
    Date(AsezDate),
    Timestamp(AsezTimestamp),
}

use sqlx::types::time::{Date, PrimitiveDateTime};

/// Агрегированное значение для запросов вида `field = ANY($1)`.
///
/// При фильтре `In` / `NotIn` sqlx требует передать массив однородных значений.
/// `AggrValue::build` принимает срез `Value` и собирает типизированный вектор
/// для последующего `.bind(vec)`. Если значения разных типов -- они игнорируются.
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
    /// Нет ненулевых значений -- массив не нужен.
    Null,
}

impl<'a> AggrValue<'a> {
    /// Строит агрегированное значение из среза [`Value`].
    ///
    /// Проходит по значениям, пропускает `Null`, и накапливает однотипные
    /// значения. Несовпадающие типы молча игнорируются (ветка `_ => {}`).
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
                // Первый ненулевой элемент инициализирует тип аккумулятора.
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
    /// Эвристика для преобразования строки: сначала пробуем timestamp, затем
    /// дату, затем UUID, и только в конце оставляем как строку.
    ///
    /// Это позволяет фронтенду передавать даты и UUID просто строками,
    /// не оборачивая их в специальные JSON-теги.
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

/// Генерирует `From<$tpe> for Value` через переданное отображение `$map`.
macro_rules! impl_into_value {
    ($tpe:ty, $var:ident, $map:expr) => {
        impl From<$tpe> for Value {
            fn from(x: $tpe) -> Self {
                Self::$var($map(x))
            }
        }
    };
}

/// Генерирует конвертации в `Value::Int` для целочисленных типов (и их ссылок).
macro_rules! impl_into_value_int {
  ($($tpe:ty),*) => {
      $(
          impl_into_value!($tpe, Int, |x: $tpe| x as i64);
          impl_into_value!(&$tpe, Int, |x: &$tpe| *x as i64);
      )*
  }
}

/// Генерирует конвертации в `Value::Float` для типов с плавающей точкой.
macro_rules! impl_into_value_float {
  ($($tpe:ty),*) => {
      $(
          impl_into_value!($tpe, Float, |x: $tpe| x as f64);
          impl_into_value!(&$tpe, Float, |x: &$tpe| *x as f64);
      )*
  }
}

// Конвертации дат и временных меток
impl_into_value!(AsezDate, Date, |x: AsezDate| x);
impl_into_value!(&AsezDate, Date, |x: &AsezDate| x.to_owned());
impl_into_value!(AsezTimestamp, Timestamp, |x: AsezTimestamp| x);
impl_into_value!(&AsezTimestamp, Timestamp, |x: &AsezTimestamp| x.to_owned());

// Конвертации целочисленных типов
impl_into_value_int!(usize, u64, u32, u16, u8, i64, i32, i16, i8);
// Конвертации типов с плавающей точкой
impl_into_value_float!(f64, f32);
// Остальные конвертации
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
