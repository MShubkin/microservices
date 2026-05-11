//! Модуль под составляющие разных DTO, которые одинаковы
//! по структуре

use crate::presentation::dto::print_docs::common::{
    TemplateFormat, TemplateStructure,
};

use crate::presentation::dto::response_request::{ApiResponseData, EntityKind};
use asez2_shared_db::db_item::int_array::AsezArray;
use asez2_shared_db::db_item::selection::filters::FilterTrait;
use asez2_shared_db::db_item::{selection, AsezDate, AsezTimestamp, Select};
use asez2_shared_db::result::SharedDbError;
use asez2_shared_db::{IntWithOriginal, Value};
use asez2_tables::maths::*;
use asez2_tables::traits::{HasId, HasUuid};
use asez2_tables::{PlanOrAmendmentRep, Section, WidePlanOrAmendmentRep};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use uuid::Uuid;

use super::export::ReplacementConfig;
use super::value::UiValue;

/// Список с идентификаторами сущности
pub type ObjectIdentifierList = ItemList<ObjectIdentifier>;
pub type ObjectIdentifierWithStatusNoteList =
    ItemList<ObjectIdentifierWithStatusNote>;

/// Идентификатор который однозначно i64. Иногда ФЕ его посылает.
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct Id {
    pub id: i64,
}

/// Идентификатор, который включает в себя uuid и id
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct ObjectIdentifier {
    pub id: i64,
    pub uuid: Uuid,
    /// This field is not used.
    #[serde(default, skip_serializing_if = "EntityKind::undefined")]
    pub object_type: EntityKind,
}

impl ObjectIdentifier {
    pub fn new(id: i64, uuid: Uuid) -> Self {
        Self {
            id,
            uuid,
            object_type: Default::default(),
        }
    }

    pub fn new_with_type(id: i64, uuid: Uuid, object_type: EntityKind) -> Self {
        Self {
            id,
            uuid,
            object_type,
        }
    }
}

/// Идентификатор вместе с комментарием по статусу
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ObjectIdentifierWithStatusNote {
    #[serde(flatten)]
    pub inner: ObjectIdentifier,
    #[serde(default)]
    pub status_note: String,
    pub plan_reason_cancel_id: Option<i32>,
    pub plan_replaced_id: Option<i64>,
}

impl ObjectIdentifierWithStatusNote {
    pub fn new(id: i64, uuid: Uuid, status_note: String) -> Self {
        Self {
            inner: ObjectIdentifier::new(id, uuid),
            status_note,
            plan_reason_cancel_id: None,
            plan_replaced_id: None,
        }
    }

    pub fn new_with_type(
        id: i64,
        uuid: Uuid,
        object_type: EntityKind,
        status_note: String,
    ) -> Self {
        Self {
            inner: ObjectIdentifier::new_with_type(id, uuid, object_type),
            status_note,
            plan_reason_cancel_id: None,
            plan_replaced_id: None,
        }
    }

    pub fn new_with_reason(
        id: i64,
        uuid: Uuid,
        object_type: EntityKind,
        status_note: String,
        plan_reason_cancel_id: Option<i32>,
        plan_replaced_id: Option<i64>,
    ) -> Self {
        Self {
            inner: ObjectIdentifier::new_with_type(id, uuid, object_type),
            status_note,
            plan_reason_cancel_id,
            plan_replaced_id,
        }
    }

    pub fn new_with_reason_only(
        id: i64,
        uuid: Uuid,
        object_type: EntityKind,
        status_note: String,
        plan_reason_cancel_id: Option<i32>,
    ) -> Self {
        Self {
            inner: ObjectIdentifier::new_with_type(id, uuid, object_type),
            status_note,
            plan_reason_cancel_id,
            plan_replaced_id: None,
        }
    }
}

impl From<ObjectIdentifier> for ObjectIdentifierWithStatusNote {
    fn from(value: ObjectIdentifier) -> Self {
        Self {
            inner: value,
            ..Default::default()
        }
    }
}

impl From<ObjectIdentifierWithStatusNote> for ObjectIdentifier {
    fn from(value: ObjectIdentifierWithStatusNote) -> Self {
        value.inner
    }
}

impl Deref for ObjectIdentifierWithStatusNote {
    type Target = ObjectIdentifier;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Квери информация при аутентификаци
#[derive(Debug, Deserialize)]
pub struct UserId {
    pub user_id: i32,
}

/// Запрос на получение планов
#[derive(Deserialize, Serialize, Debug)]
pub struct GetPlansReq<T>
where
    T: Into<Section>,
{
    /// UI секция пользователя
    pub section_id: T,
    #[serde(flatten)]
    /// Селект для запроса определенных полей
    pub select: UiSelect,
}

/// Запрос на обновление ППЗ/ДС
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct UpdatePlansReq<T>
where
    T: Into<Section>,
{
    /// UI секция пользователя
    pub section_id: T,
    /// UUID ППЗ/ДС
    pub uuid: Uuid,
    /// Массив возвращаемых колонок
    pub column_list: Vec<String>,
    /// Обновляемые данные
    #[serde(flatten)]
    pub data: WidePlanOrAmendmentRep,
}

/// Ответ на [запрос обновления ППЗ/ДС](UpdatePlansReq)
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct UpdatePlansResponseData {
    /// Обновленные данные
    #[serde(flatten)]
    pub data: PlanOrAmendmentRep,
    /// Метадата для фронтенда
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<Metadata>,
}

impl ApiResponseData for UpdatePlansResponseData {}

// TODO: Возможно в будущем придется расширить
/// Метаданные для фронтенда
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Metadata {
    /// Запрещенные для редактирования поля на данный момент
    pub disabled_field_list: Vec<String>,
}

/// Структура для переноса идентификаторов с пользователе.
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectIdsWithUser {
    pub user_id: i32,
    pub ids: Vec<ObjectIdentifier>,
}

/// Структура для переноса идентификаторов с пользователе.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlansAmendmentsWithUser {
    pub user_id: i32,
    pub plans: Vec<PlanOrAmendmentRep>,
}

/// Структура для переноса идентификаторов с пользователе.
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectIdsWithUserAndComment {
    pub user_id: i32,
    pub ids: Vec<ObjectIdentifierWithStatusNote>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
/// Defines a complete select from a single table.
pub struct UiSelect {
    #[serde(default)]
    pub chunk: Option<ArrayChunk>,
    #[serde(rename = "column_list")]
    pub field_list: Vec<String>,
    #[serde(default)]
    pub filter_list: Vec<Filters>,
    #[serde(default)]
    pub order_list: Vec<selection::FieldSortOrder>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ArrayChunk {
    pub quantity: usize,
    pub offset: usize,
    #[serde(default)]
    pub count_total: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
/// NB: Serde is derived to map using `impl_serde_map` macro.
pub struct ColumnFilter {
    #[serde(rename = "operator")]
    pub selection_kind: selection::SelectionKind,
    #[serde(rename = "filter_values")]
    pub values: Vec<UiValue>,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Filters {
    #[serde(rename = "column_id")]
    pub field: String,
    #[serde(default)]
    pub is_key: bool,
    #[serde(rename = "value_list")]
    pub values: Vec<ColumnFilter>,
}

/// NB: По правилам ФЕ, фильтры для того же поля ВСЕГДА расширяют выборку,
/// т.е. работаю через OR.
/// Но при этом фильтры для разных полей сужают выборку, работая через AND.
impl TryFrom<Filters> for selection::FilterTree {
    type Error = SharedDbError;

    fn try_from(x: Filters) -> Result<Self, Self::Error> {
        let count = x.values.len();
        let mut values = x
            .values
            .into_iter()
            .map(|v| {
                Ok(selection::Filter {
                    field: x.field.to_owned(),
                    kind: v.selection_kind,
                    values: v
                        .values
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, SharedDbError>>()?,
                })
            })
            .collect::<Result<Vec<_>, SharedDbError>>()?;
        match count {
            0 => Ok(selection::FilterTree::None),
            1 => Ok(selection::FilterTree::Filter(values.pop().expect("1"))),
            _ => Ok(selection::FilterTree::or_from_list(values)),
        }
    }
}

impl TryFrom<UiSelect> for selection::Select {
    type Error = SharedDbError;

    fn try_from(x: UiSelect) -> Result<Self, Self::Error> {
        // Фильтры для разных полей СУЖАЮТ выборку, работая через AND.
        let filter_list = x
            .filter_list
            .into_iter()
            .map(selection::FilterTree::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let filter_list = selection::FilterTree::and_from_list(filter_list);
        let (offset, take_n, count_total) = if let Some(ArrayChunk {
            quantity,
            offset,
            count_total,
            ..
        }) = x.chunk
        {
            (Some(offset), Some(quantity), Some(count_total))
        } else {
            (None, None, None)
        };

        Ok(Self {
            field_list: x.field_list,
            order_list: x.order_list,
            filter_list,
            offset,
            take_n,
            count_total,
            ..Default::default()
        })
    }
}

impl FilterTrait for ColumnFilter {
    type ValueType = UiValue;

    fn kind(&self) -> asez2_shared_db::db_item::SelectionKind {
        self.selection_kind
    }

    fn values(&self) -> &[Self::ValueType] {
        &self.values
    }
}

/// Обобщенный формат запроса на экспорт документа.
#[derive(Deserialize, Serialize, Debug)]
pub struct GeneralExportReq<T, S>
where
    T: Into<Section>,
    S: Into<Select>,
{
    /// UI секция пользователя
    pub section_id: T,
    /// Requested output format, Default: Excel (XSLX)
    pub format: Option<TemplateFormat>,
    /// Requested output template (file name)
    pub template: Option<String>,
    #[serde(flatten)]
    /// SELECT для запроса определенных полей
    pub select: S,
    /// User
    pub user_id: i32,
    /// Наименования полей (метки/labels вместо имен полей из запроса)
    pub captions: Option<Vec<String>>,
    /// Токен монолита
    pub token: String,
}

/// Запрос на получение маршрутов
#[derive(Deserialize, Serialize, Debug)]
pub struct GetRouteListReq<T>
where
    T: Into<Section>,
{
    /// UI секция пользователя
    pub section_id: T,
    pub pricing_organization_unit_id: i32,
    #[serde(flatten)]
    /// Селект для запроса определенных полей
    pub select: UiSelect,
}

/// DTO с подсказкой типа значения (используется для экспорта данных)
/// See `asez2_shared_db::value::Value` for details
#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "t", content = "v")]
pub enum TaggedValue {
    Bool(bool),
    Date(AsezDate),
    DateTime(AsezTimestamp),
    Float(f64),
    FloatWithPrecisionFormat(f64, FloatPrecisionFormat),
    Int(i64),
    Quantity(Quantity),
    CRate(CurrencyRate),
    CValue(CurrencyValue),
    #[serde(rename = "null")]
    Null,
    NullWithFormat(NullValueFormat),
    String(String),
    Uuid(Uuid),
    Vec64(AsezArray<i64>),
    RangeString((String, AsezArray<String>)),
    Error(String),
    VecValue(Vec<TaggedValue>),
}

///  Формат пустого значения. Используется для форматирования xlsx-ячейки
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum NullValueFormat {
    #[default]
    String,
    Bool,
    Int,
    Float(FloatPrecisionFormat),
    Date,
    DateTime,
    RangeString(Vec<String>),
    Uuid,
}

impl TaggedValue {
    pub fn get_string_value(&self) -> String {
        match self {
            TaggedValue::Bool(value) => value.to_string(),
            TaggedValue::Date(value) => value.to_string(),
            TaggedValue::DateTime(value) => value.to_string(),
            TaggedValue::Float(value) => value.to_string(),
            TaggedValue::FloatWithPrecisionFormat(value, _) => value.to_string(),
            TaggedValue::Int(value) => value.to_string(),
            TaggedValue::Quantity(value) => value.to_string(),
            TaggedValue::CRate(value) => value.to_string(),
            TaggedValue::CValue(value) => value.to_string(),
            TaggedValue::Null => "".to_owned(),
            TaggedValue::NullWithFormat(_) => "".to_owned(),
            TaggedValue::String(value) => value.clone(),
            TaggedValue::Uuid(value) => value.to_string(),
            TaggedValue::Vec64(value) => {
                value.0.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
            }
            TaggedValue::RangeString(value) => value.0.to_string(),
            TaggedValue::Error(value) => value.to_string(),
            TaggedValue::VecValue(value) => {
                value.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
            }
        }
    }

    pub fn from_maybe_int_iter<
        'a,
        I: IntoIterator<Item = &'a T>,
        T: Into<i64> + Copy + 'a,
    >(
        maybe_iter: Option<I>,
    ) -> Self {
        maybe_iter.map_or(TaggedValue::Null, TaggedValue::from_int_iter)
    }

    pub fn from_int_iter<
        'a,
        I: IntoIterator<Item = &'a T>,
        T: Into<i64> + Copy + 'a,
    >(
        iter: I,
    ) -> Self {
        TaggedValue::Vec64(AsezArray(
            iter.into_iter().copied().map(Into::into).collect(),
        ))
    }
}

impl From<Option<Value>> for TaggedValue {
    fn from(v: Option<Value>) -> Self {
        v.map(TaggedValue::from).unwrap_or(TaggedValue::Null)
    }
}

impl From<Option<UiValue>> for TaggedValue {
    fn from(v: Option<UiValue>) -> Self {
        v.map(TaggedValue::from).unwrap_or(TaggedValue::Null)
    }
}

// for generated fields...
macro_rules! from_num {
    ($ty:ty) => {
        impl From<$ty> for TaggedValue {
            fn from(value: $ty) -> Self {
                TaggedValue::Int(value.into())
            }
        }

        impl From<Option<$ty>> for TaggedValue {
            fn from(value: Option<$ty>) -> Self {
                value.map_or(TaggedValue::Null, |value| value.into())
            }
        }
    };
}

from_num!(i8);
from_num!(i16);
from_num!(i32);
from_num!(i64);
from_num!(u8);
from_num!(u16);
from_num!(u32);

impl From<AsezDate> for TaggedValue {
    fn from(value: AsezDate) -> Self {
        Self::Date(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum FloatPrecisionFormat {
    Single,
    #[default]
    Double,
    Three,
    Five,
    None,
}

impl From<u8> for FloatPrecisionFormat {
    fn from(value: u8) -> Self {
        match value {
            1 => FloatPrecisionFormat::Single,
            2 => FloatPrecisionFormat::Double,
            3 => FloatPrecisionFormat::Three,
            5 => FloatPrecisionFormat::Five,
            _ => FloatPrecisionFormat::None,
        }
    }
}

impl From<Value> for TaggedValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(x) => TaggedValue::Bool(x),
            Value::Date(x) if x == AsezDate::default() => TaggedValue::Null,
            Value::Date(x) => TaggedValue::Date(x),
            Value::Float(x) => TaggedValue::Float(x),
            Value::Int(x) => TaggedValue::Int(x),
            Value::Null => TaggedValue::Null,
            Value::String(x) if x.is_empty() => TaggedValue::Null,
            Value::String(x) => TaggedValue::String(x),
            Value::Uuid(x) if x == Uuid::default() => TaggedValue::Null,
            Value::Uuid(x) => TaggedValue::Uuid(x),
            Value::Vec64(x) => TaggedValue::Vec64(x),
            Value::Vec32(x) => TaggedValue::Vec64(AsezArray(
                x.0.into_iter().map(|x| x as i64).collect(),
            )),
            Value::Vec16(x) => TaggedValue::Vec64(AsezArray(
                x.0.into_iter().map(|x| x as i64).collect(),
            )),
            Value::Timestamp(x) if x == AsezTimestamp::default() => {
                TaggedValue::Null
            }
            Value::Timestamp(x) => TaggedValue::DateTime(x),
            Value::IntWithOriginal(IntWithOriginal {
                original,
                precision,
                ..
            }) => TaggedValue::FloatWithPrecisionFormat(original, precision.into()),
        }
    }
}

impl From<UiValue> for TaggedValue {
    fn from(value: UiValue) -> Self {
        match value {
            UiValue::Bool(x) => TaggedValue::Bool(x),
            UiValue::Date(x) if x == AsezDate::default() => TaggedValue::Null,
            UiValue::Date(x) => TaggedValue::Date(x),
            UiValue::Float(x) => TaggedValue::Float(x),
            UiValue::Int(x) => TaggedValue::Int(x),
            UiValue::Null => TaggedValue::Null,
            UiValue::String(x) if x.is_empty() => TaggedValue::Null,
            UiValue::String(x) => TaggedValue::String(x),
            UiValue::Uuid(x) if x == Uuid::default() => TaggedValue::Null,
            UiValue::Uuid(x) => TaggedValue::Uuid(x),
            UiValue::Timestamp(x) if x == AsezTimestamp::default() => {
                TaggedValue::Null
            }
            UiValue::Timestamp(x) => TaggedValue::DateTime(x),
            UiValue::VecValue(x) => x.into(),
        }
    }
}

impl From<Vec<UiValue>> for TaggedValue {
    fn from(value: Vec<UiValue>) -> Self {
        TaggedValue::VecValue(value.into_iter().map(|item| item.into()).collect())
    }
}

impl Debug for TaggedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for TaggedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            TaggedValue::Bool(b) => format!("B({})", b),
            TaggedValue::Date(d) => format!("D({})", d),
            TaggedValue::Float(f) | TaggedValue::FloatWithPrecisionFormat(f, _) => {
                format!("F({})", f)
            }
            TaggedValue::Int(i) => format!("I({})", i),
            // TODO: Do we need to write "F(..)" to make it work properly?
            TaggedValue::Quantity(i) => format!("Q({})", i),
            TaggedValue::CRate(i) => format!("CR({})", i),
            TaggedValue::CValue(i) => format!("CV({})", i),
            TaggedValue::Null | TaggedValue::NullWithFormat(_) => "N".to_string(),
            TaggedValue::String(s) => format!("S({})", s),
            TaggedValue::Uuid(u) => format!("U({})", u),
            TaggedValue::Vec64(v) => format!(
                "V([{}])",
                v.0.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
            ),
            TaggedValue::RangeString((s, _)) => format!("S({})", s),
            TaggedValue::Error(s) => format!("S({})", s),
            TaggedValue::DateTime(d) => format!("D({})", d),
            TaggedValue::VecValue(v) => format!(
                "V([{}])",
                v.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
            ),
        };
        write!(f, "{}", str)
    }
}

/// Внешний (пользовательский) запрос на экспорт документов на базе `Select`
#[derive(Deserialize, Serialize, Debug)]
pub struct UiExportTableReq<T>
where
    T: Into<Section>,
{
    /// UI секция пользователя
    pub section_id: T,
    /// Requested output format, Default: Excel (XSLX)
    pub format: Option<TemplateFormat>,
    /// Requested output template (file name)
    pub template: Option<String>,
    #[serde(flatten)]
    /// SELECT для запроса определенных полей
    pub select: UiSelect,
    /// Наименования полей (метки/labels вместо имен полей из запроса)
    pub captions: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UiExportItemsListReq {
    /// Requested output format, Default: Excel (XSLX)
    pub format: Option<TemplateFormat>,
    /// Requested output template (file name)
    pub template: Option<String>,
    /// Список id записей
    pub id_list: Vec<i64>,
}

/// "Файл" отчета передается из сервиса `print-doc` потоком байт.
/// Так сделано в отсутствии сервиса хранения документов (shared-file-storage).
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ExportResponse {
    pub byte_buf: Vec<u8>,
}

impl ApiResponseData for ExportResponse {}

/// Строка данных для вывода в отчет
pub type DataRecord = Vec<TaggedValue>;
/// Полный набор данных для вывода в отчет: Заголовок колонок + Строки данных
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct DataRecords {
    pub captions: Vec<String>,        // Заголовки столбцов
    pub field_list: Vec<String>,      // Идентификаторы столбцов
    pub data: Vec<DataRecord>,        // Данные
    pub entity_kind: Vec<EntityKind>, // Типы сущностей
}

impl ApiResponseData for DataRecords {}

/// Обобщенный (внутренний) формат запроса (между сервисами) на экспорт данных/документа.
/// Сервис `print-doc` получает этот запрос, строит отчет и возвращает результат в виде потока байт
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InternalExportReq {
    /// Requested output format, Default: Excel (XLS/XLSX)
    pub format: Option<TemplateFormat>,
    /// Requested output template (file name)
    pub template: Option<String>,
    /// User
    pub user_id: i32,
    /// Monolith token
    pub monolith_token: String,
    /// Данные для построения отчета
    pub data: DataRecords,
    /// Конфигурация замены полей
    pub replacements: ReplacementConfig,
}

/// Формат запроса на парсинг файла
/// Сервис `print-doc` парсит входной файл, возвращает структуру DataRecords
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InternalParseReq {
    /// Input file format, Default: Excel (XLS/XLSX)
    pub format: Option<TemplateFormat>,
    /// Input file name
    pub template: Option<String>,
    /// User id
    pub user_id: i32,
    /// Данные
    pub data: Vec<u8>,
    /// Структура файла
    pub template_structure: TemplateStructure,
}

/// Сущность для переноса на ФЕ. Она может содержать произвольные поля.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct FeWrapper<T> {
    /// Сущность запроса которую возвращаем не ФЕ.
    #[serde(flatten)]
    pub entity: T,
    /// Дополнительные поля.
    #[serde(flatten)]
    pub extra_fields: HashMap<String, UiValue>,
    /// метаданные по полям которыми не пользуемся.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub _meta: Option<Metadata>,
}

impl<T> FeWrapper<T> {
    pub fn new(entity: T) -> Self {
        let extra_fields = HashMap::new();
        Self {
            entity,
            extra_fields,
            _meta: None,
        }
    }
    pub fn add_field<V: Into<UiValue>>(mut self, field: &str, v: V) -> Self {
        let field = field.to_string();
        let v = v.into();
        self.extra_fields.insert(field, v);
        self
    }

    pub fn get_extra_field<S: AsRef<str>>(&self, field: S) -> Option<&UiValue> {
        self.extra_fields.get(field.as_ref())
    }
}

/// Список элементов, как правило, для запросов, или данных ответов, если отсутсвует паджинация.
#[derive(
    Debug,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    derive_more::Deref,
    derive_more::From,
)]
pub struct ItemList<T> {
    pub item_list: Vec<T>,
}

impl<T> IntoIterator for ItemList<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.item_list.into_iter()
    }
}

impl<T> FromIterator<T> for ItemList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ItemList {
            item_list: iter.into_iter().collect(),
        }
    }
}

impl<T> ApiResponseData for ItemList<T> where
    T: Debug + Default + Serialize + for<'de> Deserialize<'de>
{
}

impl HasId for ObjectIdentifier {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, status: i64) {
        self.id = status
    }
}

impl HasUuid for ObjectIdentifier {
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn set_uuid(&mut self, status: Uuid) {
        self.uuid = status
    }
}
