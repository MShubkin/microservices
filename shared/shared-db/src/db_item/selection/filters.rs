use std::fmt::Display;

// use super::*;
use crate::db_item::{AsezTimestamp, BindQuery};
use crate::result::SharedDbError;
use crate::value::{single_value, two_values};
use crate::{IntWithOriginal, Result, Value};

use ahash::AHashSet;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterTree {
    /// группа где каждый фильтр между собой связано OR.
    Or(Vec<FilterTree>),
    /// группа где каждый фильтр между собой связан AND.
    And(Vec<FilterTree>),
    /// Сам фильтр.
    Filter(Filter),
    #[default]
    None,
}

impl FilterTree {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::None => true,
            Self::Filter(x) => x.values.is_empty(),
            Self::Or(x) => x.is_empty() || x.iter().all(|x| x.is_empty()),
            Self::And(x) => x.is_empty() || x.iter().all(|x| x.is_empty()),
        }
    }

    // Gets a vector of references to filters.
    pub fn slice(&self) -> Vec<&Filter> {
        let mut output = Vec::new();
        match self {
            Self::Or(ref x) | Self::And(ref x) => {
                for filter in x {
                    output.append(&mut filter.slice());
                }
            }
            Self::Filter(ref x) => output.push(x),
            Self::None => {}
        }
        output
    }

    // Gets a vector of references to filters which can be edited.
    pub fn slice_mut(&mut self) -> Vec<&mut Filter> {
        let mut output = Vec::new();
        match self {
            Self::Or(ref mut x) | Self::And(ref mut x) => {
                for filter in x {
                    output.append(&mut filter.slice_mut());
                }
            }
            Self::Filter(ref mut x) => output.push(x),
            Self::None => {}
        }
        output
    }

    // Gets a vector of references to filters which can be edited.
    pub fn into_filters(self) -> Vec<Filter> {
        let mut output = Vec::new();
        match self {
            Self::Or(x) | Self::And(x) => {
                for filter in x {
                    output.extend(filter.into_filters());
                }
            }
            Self::Filter(x) => output.push(x),
            Self::None => {}
        }
        output
    }

    // Retrieves the filter and the branch it belongs to by field name.
    pub fn find_with_branch(&mut self, field: &str) -> Option<&mut FilterTree> {
        match self {
            Self::Filter(ref mut x) if x.field == field => Some(self),
            Self::And(ref mut x) | Self::Or(ref mut x) => {
                x.iter_mut().find_map(|x| x.find_with_branch(field))
            }
            _ => None,
        }
    }

    pub fn push_filter(&mut self, filter: Filter) {
        match self {
            FilterTree::None => *self = filter.into(),
            FilterTree::Filter(f) => {
                *self = FilterTree::and_from_list(vec![f.clone(), filter]);
            }
            FilterTree::And(x) | FilterTree::Or(x) => x.push(filter.into()),
        };
    }

    pub fn remove_by_field(&mut self, field: &str) -> Vec<Filter> {
        match self {
            Self::Filter(x) if x.field == field => {
                let x = x.clone();
                *self = Self::None;
                vec![x]
            }
            Self::Or(x) | Self::And(x) => {
                let mut output = Vec::new();
                x.retain_mut(|tree| {
                    let removed = tree.remove_by_field(field);
                    output.extend(removed.into_iter());
                    !tree.is_empty()
                });
                output
            }
            _ => vec![],
        }
    }

    pub fn purge_by_fields(&mut self, fields: &AHashSet<&str>) {
        match self {
            Self::Filter(x) if !fields.contains(&x.field as &str) => {
                *self = Self::None;
            }
            Self::Or(x) | Self::And(x) => {
                x.retain_mut(|tree| {
                    tree.purge_by_fields(fields);
                    !tree.is_empty()
                });
            }
            _ => {}
        }
    }

    /// Splits the filter tree into two, one mentioning specified fields, and
    /// another the rest of them.
    ///
    /// The first filter in the result is the one that restricts `fields`.
    pub fn split_by_fields(self, fields: &AHashSet<&str>) -> (Self, Self) {
        match self {
            FilterTree::None => (FilterTree::None, FilterTree::None),
            FilterTree::Filter(x) if fields.contains(x.field.as_str()) => {
                (FilterTree::Filter(x), FilterTree::None)
            }
            FilterTree::Filter(_) => (FilterTree::None, self),
            FilterTree::Or(x) => {
                let (one, two) =
                    x.into_iter().map(|x| x.split_by_fields(fields)).unzip();
                (FilterTree::Or(one), FilterTree::Or(two))
            }
            FilterTree::And(x) => {
                let (one, two) =
                    x.into_iter().map(|x| x.split_by_fields(fields)).unzip();
                (FilterTree::And(one), FilterTree::And(two))
            }
        }
    }

    pub fn and(self, x: FilterTree) -> Self {
        FilterTree::And(vec![self, x])
    }

    pub fn or(self, x: FilterTree) -> Self {
        Self::Or(vec![self, x])
    }

    pub fn filter(x: Filter) -> Self {
        Self::Filter(x)
    }

    pub fn and_from_list<I, V>(x: I) -> Self
    where
        V: Into<FilterTree>,
        I: IntoIterator<Item = V>,
    {
        let x = x.into_iter().map(Into::into).collect::<Vec<_>>();
        Self::And(x)
    }

    pub fn or_from_list<I, V>(x: I) -> Self
    where
        V: Into<FilterTree>,
        I: IntoIterator<Item = V>,
    {
        let x = x.into_iter().map(Into::into).collect::<Vec<_>>();
        Self::Or(x)
    }

    /// NB: Not async, because we don't have time for async recursion
    /// right now.
    /// NB2: It is CRITICAL that values are binded in the same order as they are binded.
    pub(crate) fn build_sql(
        &self,
        container: &mut String,
        idx: usize,
    ) -> Result<usize> {
        match self {
            Self::None => Ok(idx),
            Self::Filter(x) => x.add_to_string(container, idx),
            Self::Or(or_filters) => build_or(or_filters, container, idx),
            Self::And(and_filters) => build_and(and_filters, container, idx),
        }
    }

    /// Used to bind values to a query. Is an inner function.
    pub(crate) fn bind_vars_to_query<'a>(
        &'a self,
        query: BindQuery<'a>,
    ) -> BindQuery<'a> {
        match self {
            Self::None => query,
            Self::Filter(x) => x.bind_filter_vars_to_query(query),
            Self::Or(or_filters) => bind_filters(or_filters, query),
            Self::And(and_filters) => bind_filters(and_filters, query),
        }
    }
}

fn bind_filters<'a, 'b: 'a>(
    filters: &'b [FilterTree],
    mut query: BindQuery<'a>,
) -> BindQuery<'a> {
    for filter in filters {
        query = filter.bind_vars_to_query(query);
    }
    query
}

/// Если список пустой, мы ее игнорируем.
fn build(
    filters: &[FilterTree],
    container: &mut String,
    joiner: &str,
    mut idx: usize,
) -> Result<usize> {
    let mut f_iter = filters.iter().filter(|x| !x.is_empty());

    if let Some(tree) = f_iter.next() {
        container.push_str(" (");
        idx = tree.build_sql(container, idx)?;
    }
    for tree in f_iter {
        container.push_str(joiner);
        idx = tree.build_sql(container, idx)?;
    }
    if !filters.is_empty() {
        container.push(')');
    }
    Ok(idx)
}

fn build_and(
    filters: &[FilterTree],
    container: &mut String,
    idx: usize,
) -> Result<usize> {
    build(filters, container, " AND", idx)
}

fn build_or(
    filters: &[FilterTree],
    container: &mut String,
    idx: usize,
) -> Result<usize> {
    build(filters, container, " OR", idx)
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Filter {
    pub field: String,
    pub kind: SelectionKind,
    pub values: Vec<Value>,
}

impl From<Filter> for FilterTree {
    fn from(x: Filter) -> Self {
        Self::filter(x)
    }
}

impl FilterTrait for Filter {
    type ValueType = Value;

    fn kind(&self) -> SelectionKind {
        self.kind
    }

    fn values(&self) -> &[Self::ValueType] {
        &self.values
    }
}

impl Filter {
    #[inline(always)]
    pub fn with_value<V>(field: &str, kind: SelectionKind, value: V) -> Self
    where
        V: Into<Value>,
    {
        Self {
            field: field.to_owned(),
            kind,
            values: vec![value.into()],
        }
    }

    #[inline(always)]
    pub fn with_values<V, I>(field: &str, kind: SelectionKind, values: I) -> Self
    where
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        Self {
            field: field.to_owned(),
            kind,
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    pub fn eq<V>(field: &str, value: V) -> Self
    where
        V: Into<Value>,
    {
        Filter::with_value(field, SelectionKind::Equals, value)
    }

    pub fn not_eq<V>(field: &str, value: V) -> Self
    where
        V: Into<Value>,
    {
        Filter::with_value(field, SelectionKind::NotEquals, value)
    }

    pub fn in_any<I, V>(field: &str, values: I) -> Self
    where
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        Filter::with_values(field, SelectionKind::In, values)
    }

    pub fn not_in_any<I, V>(field: &str, values: I) -> Self
    where
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        Filter::with_values(field, SelectionKind::NotIn, values)
    }

    pub fn between<V>(field: &str, from: V, to: V) -> Self
    where
        V: Into<Value>,
    {
        Filter::with_values(
            field,
            SelectionKind::Between,
            vec![from.into(), to.into()],
        )
    }

    /// Produces "!~ $" statement" and should be used for strings
    pub fn not_contains<V>(field: &str, value: V) -> Self
    where
        V: Into<Value>,
    {
        Filter::with_value(field, SelectionKind::NotContains, value)
    }

    pub fn less_eq<V>(field: &str, value: V) -> Self
    where
        V: Into<Value>,
    {
        Filter::with_value(field, SelectionKind::LessEqual, value)
    }

    fn add_to_string(
        &self,
        container: &mut String,
        mut binding: usize,
    ) -> Result<usize> {
        use SelectionKind::*;
        let field_name = &self.field;
        let kind = self.kind;

        // This closure exists for DRY.
        let binding_push = |n: usize, c: &mut String| -> usize {
            c.push('$');
            c.push_str(&n.to_string());
            n + 1
        };

        let not_null_values_count =
            self.values.iter().filter(|value| **value != Value::Null).count();

        if (kind.is_simple() || kind.is_jsonpath()) && self.values.len() > 1 {
            return Err(format!(
                "`{}` не поддерьивает больше одного значение. Найдено {}. (Фильтр на {}: {:?})",
                kind.str(),
                self.values.len(),
                self.field,
                self.values,
            )
            .into());
        } else if !matches!(kind, Equals | NotEquals)
            && !kind.is_in_brackets()
            && not_null_values_count != self.values.len()
        {
            return Err(format!(
                "`{}` не поддерживается для фильтра на нуль",
                kind.str()
            )
            .into());
        }
        container.push(' ');

        // TODO: See if other simple types work with null.
        if kind.is_simple() {
            container.push_str(field_name);
            container.push(' ');
            match (not_null_values_count, kind) {
                (0, Equals) => container.push_str("IS NULL"),
                (0, NotEquals) => container.push_str("IS NOT NULL"),
                _ => {
                    container.push_str(kind.str());
                    binding = binding_push(binding, container);
                    if kind == Jsonpath {
                        // оператор `@?` требует, чтобы правый операнд был типа `jsonpath`
                        container.push_str("::jsonpath");
                    }
                }
            };
            Ok(binding)
        } else if kind.is_jsonpath() {
            container.push_str(field_name);
            container.push(' ');
            container.push_str(kind.str());
            binding = binding_push(binding, container);
            container.push_str("::jsonpath");
            Ok(binding)
        } else if matches!(kind, In | NotIn) {
            // either "field=ANY($1)" or "NOT (field=ANY($1))"
            if kind == NotIn {
                container.push_str("NOT (");
            }
            if not_null_values_count == 0 {
                self.add_null_filter(container, field_name);
            } else {
                // (field=ANY($) OR field IS NULL) открывает скобку
                if not_null_values_count < self.values.len() {
                    container.push('(');
                }
                // create the central "field=ANY($1)"
                container.push_str(field_name);
                container.push_str("=ANY(");
                binding = binding_push(binding, container);
                container.push(')');

                // Закрываем скобку для фильтра с нулем
                if not_null_values_count < self.values.len() {
                    container.push_str(" OR ");
                    self.add_null_filter(container, field_name);
                    container.push(')');
                }
            }
            // Close final bracket if needed
            if kind == NotIn {
                container.push(')');
            }
            Ok(binding)
        } else if kind.is_in_brackets() {
            if not_null_values_count == 0 {
                self.add_null_filter(container, field_name);
            } else {
                // Добавление скобки для фильтра под нуль
                if not_null_values_count < self.values.len() {
                    container.push('(');
                }
                container.push_str(field_name);
                container.push(' ');

                // Here we push something like "<" or "NOT IN" or "!~".
                // For some conditions we need a space, so better safe than sorry.
                container.push_str(kind.str());

                // Here we push a sequence such as "($1,$2,$3)"
                container.push('(');
                binding = binding_push(binding, container);

                for _ in 1..not_null_values_count {
                    container.push(',');
                    binding = binding_push(binding, container);
                }
                container.push(')');

                // Закрываем скобку для фильтра с нулем
                if not_null_values_count < self.values.len() {
                    container.push_str(" OR ");
                    self.add_null_filter(container, field_name);
                    container.push(')');
                }
            }
            Ok(binding)
        } else if matches!(kind, Between) {
            container.push_str(field_name);
            container.push(' ');
            container.push_str(kind.str());

            if self.values.len() != 2 {
                return Err(format!(
                    "BETWEEN filter must have EXACTLY 2 values but has:\n {:?}",
                    self.values
                )
                .into());
            }
            container.push_str(&format!(" ${} AND ${}", binding, binding + 1));
            if matches!(self.values[1], Value::Date(_)) {
                container.push_str("+ interval '23:59:59'");
            }
            binding += 2;
            Ok(binding)
        } else {
            Err(format!("Unsupported Filter: {:?} after:\n{}", self, container)
                .into())
        }
    }

    // TODO: Возможно будет удалено при добавлении
    // null типа в БД
    fn add_null_filter(&self, container: &mut String, field_name: &str) {
        match self.kind {
            SelectionKind::Equals | SelectionKind::In => {
                container.push_str(&format!("{} IS NULL", field_name))
            }
            SelectionKind::NotEquals | SelectionKind::NotIn => {
                container.push_str(&format!("{} IS NOT NULL", field_name))
            }
            _ => {}
        }
    }

    /// Used to bind values to a query. Is an inner function.
    pub(crate) fn bind_filter_vars_to_query<'a>(
        &'a self,
        mut query: BindQuery<'a>,
    ) -> BindQuery<'a> {
        use crate::value::AggrValue;
        // NB: there should be checks elsewhere to ensure that the length
        // of values is not zero.
        if self.values.iter().all(|x| matches!(x, Value::Null))
            || self.values.is_empty()
        {
            return query;
        } else if matches!(self.kind, SelectionKind::In | SelectionKind::NotIn) {
            return match crate::value::AggrValue::build(&self.values) {
                AggrValue::Int(x) => query.bind(x),
                AggrValue::String(x) => query.bind(x),
                AggrValue::Float(x) => query.bind(x),
                AggrValue::Bool(x) => query.bind(x),
                AggrValue::Uuid(x) => query.bind(x),
                AggrValue::Date(x) => query.bind(x),
                AggrValue::Timestamp(x) => query.bind(x),
                // TODO Check how badly this breaks.
                AggrValue::Vec64(x) => query.bind(x),
                AggrValue::Null => query,
            };
        }
        for val in self.values.iter() {
            query = match val {
                Value::Int(x) => query.bind(x),
                Value::IntWithOriginal(IntWithOriginal { int, .. }) => {
                    query.bind(int)
                }
                Value::String(x) => query.bind(x),
                Value::Float(x) => query.bind(x),
                Value::Bool(x) => query.bind(x),
                Value::Uuid(x) => query.bind(x),
                Value::Date(x) => query.bind(x),
                Value::Timestamp(x) => query.bind(x),
                Value::Null => query,
                Value::Vec64(x) => query.bind(&x.0),
                Value::Vec32(x) => query.bind(&x.0),
                Value::Vec16(x) => query.bind(&x.0),
            };
        }
        query
    }

    pub fn matches_number(&self, value: i64) -> bool {
        match self.kind {
            SelectionKind::Equals => self.values.iter().any(|v| match v {
                Value::Int(x) => *x == value,
                Value::String(s) => s == &value.to_string(),
                _ => false,
            }),
            SelectionKind::Contains => self.values.iter().any(|v| match v {
                Value::Int(x) => value.to_string().contains(&x.to_string()),
                Value::String(s) => value.to_string().contains(s),
                _ => false,
            }),
            _ => false,
        }
    }

    pub fn matches_date(&self, date: &AsezTimestamp) -> bool {
        match self.kind {
            SelectionKind::Equals => self.values.iter().any(|v| match v {
                Value::Timestamp(ts) => ts.0.date() == date.0.date(),
                Value::Date(d) => d.to_timestamp().0.date() == date.0.date(),
                _ => false,
            }),
            SelectionKind::Between => {
                if let [start, end] = &self.values[..] {
                    let start_date = match start {
                        Value::Timestamp(ts) => ts.0.date(),
                        Value::Date(d) => d.to_timestamp().0.date(),
                        _ => return false,
                    };
                    let end_date = match end {
                        Value::Timestamp(ts) => ts.0.date(),
                        Value::Date(d) => d.to_timestamp().0.date(),
                        _ => return false,
                    };
                    date.0.date() >= start_date && date.0.date() <= end_date
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
/// Section type of the filter. This defines the logic, giving an `AND`, `NOT` or `OR` clause.
#[serde(rename_all = "snake_case")]
pub enum SelectionKind {
    Equals = 1,
    NotEquals = 2,
    Less = 3,
    LessEqual = 4,
    Greater = 5,
    GreaterEqual = 6,
    Mask = 7,    // TODO
    NotMask = 8, // TODO
    Contains = 9,
    NotContains = 10,
    /// NB: BETWEEN is inclusive as it is in postgres.
    Between = 11,
    In = 12,
    NotIn = 13,
    Jsonpath = 14,
    Overlaps = 15,
}

impl std::fmt::Display for SelectionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl SelectionKind {
    fn str(&self) -> &str {
        match self {
            Self::Equals => "=",
            Self::In => "=ANY",
            Self::NotEquals => "!=",
            Self::NotIn => "=ANY",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Contains => "~",
            Self::NotContains => "!~",
            Self::Between => "BETWEEN",
            Self::Mask => "",
            Self::NotMask => "",
            SelectionKind::Jsonpath => "@?",
            SelectionKind::Overlaps => "&&",
        }
    }

    pub(crate) fn is_simple(&self) -> bool {
        use SelectionKind::*;
        matches!(
            self,
            Less | LessEqual
                | Greater
                | GreaterEqual
                | Contains
                | NotContains
                | Equals
                | NotEquals
                | Overlaps
        )
    }

    pub(crate) fn is_jsonpath(&self) -> bool {
        use SelectionKind::*;
        matches!(self, Jsonpath)
    }

    fn is_in_brackets(&self) -> bool {
        use SelectionKind::*;
        matches!(self, In | NotIn)
    }
}

impl std::ops::BitAnd for Filter {
    type Output = FilterTree;

    fn bitand(self, rhs: Self) -> Self::Output {
        FilterTree::And(vec![FilterTree::Filter(self), FilterTree::Filter(rhs)])
    }
}

impl std::ops::BitOr for Filter {
    type Output = FilterTree;

    fn bitor(self, rhs: Self) -> Self::Output {
        FilterTree::Or(vec![FilterTree::Filter(self), FilterTree::Filter(rhs)])
    }
}

fn jsonpath_op(selection_kind: SelectionKind) -> &'static str {
    match selection_kind {
        SelectionKind::Equals => "==",
        SelectionKind::NotEquals => "!=",
        SelectionKind::Less => "<",
        SelectionKind::LessEqual => "<=",
        SelectionKind::Greater => ">",
        SelectionKind::GreaterEqual => ">=",
        _ => "",
    }
}

/// Типаж фильтра.
pub trait FilterTrait {
    /// Тип значений, хранимых фильтром.
    type ValueType;
    /// Тип фильтра.
    fn kind(&self) -> SelectionKind;
    /// Значения фильтра.
    fn values(&self) -> &[Self::ValueType];
}

// По фильтру `filter`, ограничивающему значения параметра `field`,
// строит фильтр, использующий jsonpath для фильтрации данных в БД.
//
// Для примера, FE оперирует параметрами маршрута `department_id` и
// `division_id`, но в реальности в БД нет таких колонок. Эти значения находятся
// в поле `data` таблицы `RouteData`, причем обернутые в стркутуру с ключем
// `assign_department`.
//
// Для того, чтобы выполнить фильтрацию по этим значениям, нам надо воспользоваться
// оператором `@?`, который проверяет значение на совпадение с jsonpath выражением.
// Соответственно, для фильтра `department_id ? value` мы должны сформировать фильтр
// типа `data @? '$.assign_department[*] ? (@.department_id ? value)'`.
pub(super) fn convert_to_jsonpath<F>(
    db_field: &str,
    jsonpath: &str,
    json_field: &str,
    filter: &F,
) -> Result<Filter>
where
    F: FilterTrait,
    F::ValueType: Display,
{
    let jsonpath_expr = match filter.kind() {
        SelectionKind::Equals
        | SelectionKind::NotEquals
        | SelectionKind::Less
        | SelectionKind::LessEqual
        | SelectionKind::Greater
        | SelectionKind::GreaterEqual => {
            let op = jsonpath_op(filter.kind());
            format!(
                "@.{json_field} {op} {value}",
                value = single_value(filter.values())?
            )
        }
        SelectionKind::Between => {
            let (low, high) = two_values(filter.values())?;
            format!("{low} <= @.{json_field} && @.{json_field} <= {high}")
        }
        SelectionKind::In => filter
            .values()
            .iter()
            .map(|value| format!("@.{json_field} == {value}"))
            .join(" || "),
        SelectionKind::NotIn => filter
            .values()
            .iter()
            .map(|value| format!("@.{json_field} != {value}"))
            .join(" && "),
        _ => {
            return Err(SharedDbError::Other(format!(
                "неверный тип фильтра для поля `{json_field}`"
            )));
        }
    };
    let jsonpath = format!("{jsonpath} ? ({jsonpath_expr})");
    Ok(Filter {
        field: db_field.to_string(),
        kind: SelectionKind::Jsonpath,
        values: vec![jsonpath.into()],
    })
}

/// Утилита для преобразования абстрактных фильтров для `json_field`,
/// являющегося полем JSON структуры, которая хранится в БД в колонке `db_field`,
/// и находится в объекте, доступном по пути `jsonpath`.
///
/// Например, данные маршрута ПД хранятся в поле `data` таблицы `RouteData`:
///
/// ```json
/// { "assign_departments": [{ "department_id": 10, "division_id": 20 }] }
/// ```
///
/// Фильтр FE для параметра `department_id` выглядит как `kind: Equals, values: [10]`.
///
/// По этому фильтру надо получить фильтр, применимый к колонке `data` таблицы `RouteData`:
///
/// `kind: Jsonpath, values: ["$.assign_departments ? ( @.department_id == 10)"]`
pub fn convert_jsonpath_filters<F>(
    db_field: &str,
    jsonpath: &str,
    json_field: &str,
    filters: &[F],
) -> Result<Vec<Filter>>
where
    F: FilterTrait,
    F::ValueType: Display,
{
    filters
        .iter()
        .map(|filter| convert_to_jsonpath(db_field, jsonpath, json_field, filter))
        .collect::<Result<_>>()
}

pub fn convert_filters_equal_to_value<F>(filters: &[F]) -> Result<&F::ValueType>
where
    F: FilterTrait + std::fmt::Debug,
    F::ValueType: PartialEq + From<bool> + std::fmt::Debug,
{
    match filters {
        [filter] if filter.kind() == SelectionKind::Equals => {
            let value = single_value(filter.values())?;
            Ok(value)
        }
        _ => Err(SharedDbError::Other(format!(
            "невозможно извлечь значениe из фильтра `{filters:?}`"
        ))),
    }
}
