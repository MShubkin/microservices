//! This submodule describes a select clause builder.
//!
//! The reason that this is placed inside the `db_item` module is because it is linked
//! to the `DbItem` that it is getting.
//!
//! Currently the module assumes that the selection instructions are the same for all crates
//! that depend on `asez2_db_shared`. However, it is always possible to make an adapter
//! structure on the side of the crate using `asez2_db_shared` for incoming requests that use
//! a format that is not compatible with this `Select` as all of its fields are public.
use super::{BindQuery, DbItem};
use crate::result::Result;
use crate::value::Value;
use crate::DbAdaptor;

use ahash::AHashSet;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{Executor, FromRow, Postgres, Row};

use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

pub mod filters;
#[cfg(test)]
mod tests;
pub use filters::{Filter, FilterTree, SelectionKind};

/// This is designed as a more or less safe interface for creating get queries.
pub struct SelectMaker<T> {
    query_defn: Select,
    query_string: String,
    // The number of bound variables. Mostly for filter values.
    binds: usize,
    phantom_data: PhantomData<T>,
}

// NB: Almost everything is async because some operations might start longer than 10us, which
impl<T: DbItem + Unpin> SelectMaker<T> {
    /// Start the query by checking that the fields are correct and
    /// then making a simple query of `SELECT columns FROM table`.
    /// This can then be used or expanded with sort/filter clauses.
    ///
    /// The `start_bind` should be 1 for a new/single select. However can be should
    /// be set to N+1 where N is the current number of binds in a query, if the select
    /// is part of a multiselect query, eg:
    /// ```sql
    /// SELECT (name, age)
    /// FROM (select (id, name) from names where origin IN($1,$2)) as n
    /// JOIN (select (id, age) from ages where epoch=$3) as a
    /// ON n.id=a.id
    /// WHERE age > $4;
    /// ```
    pub(super) async fn start_from(
        q: &Select,
        start_bind: usize,
        table: &str,
        with_count: bool,
    ) -> Result<SelectMaker<T>> {
        let mut q = q.clone();
        // This step renames frontend fields to backend fields.
        T::apply_tolerance_to_select(&mut q);
        // Check must take place after substitution of fields.
        Self::check_select(&q)?;

        let selection = {
            let mut field_list = String::new();
            // Check that fields are ok.
            for rec_field in q.field_list.iter() {
                field_list.extend(format!("{},", rec_field).chars());
            }
            // If field list is empty we retrieve all fields.
            if field_list.is_empty() {
                field_list.push('*');
            } else {
                field_list.pop();
            }

            if with_count {
                field_list
                    .push_str(&format!(", COUNT(*) OVER() AS {}", TOTAL_COUNT));
            }

            field_list
        };

        let distinct = if q.distinct_on.is_empty() {
            String::default()
        } else {
            format!(" DISTINCT ON({})", q.distinct_on.join(","))
        };

        let query_string = format!("SELECT{distinct} {selection} FROM {table}");
        Ok(Self {
            query_defn: q,
            query_string,
            binds: start_bind, // NB: Index starts at 1 in SQL.
            phantom_data: PhantomData,
        })
    }

    /// Start the query by checking that the fields are correct and
    /// then making a simple query of `SELECT columns FROM table`.
    /// This can then be used or expanded with sort/filter clauses.
    pub(super) async fn start(q: &Select) -> Result<SelectMaker<T>> {
        Self::start_from(q, 1, T::TABLE, false).await
    }

    /// Start the counting query by checking that the fields are correct and
    /// then making a simple query of `SELECT columns FROM table`.
    /// This can then be used or expanded with sort/filter clauses.
    pub(super) async fn start_with_count(q: &Select) -> Result<SelectMaker<T>> {
        Self::start_from(q, 1, T::TABLE, true).await
    }

    /// Достаёт строку с порядком, но не добавляет её к query.
    /// Полезен для aggregate join.
    /// НБ: Порядок этих функций меняет строку, всегда вызывайте add_order после add_filters
    pub(super) fn get_order(&self) -> String {
        FieldSortOrder::as_order_by(&self.query_defn.order_list).unwrap_or_default()
    }

    /// НБ: Порядок этих функций меняет строку, всегда вызывайте add_order после add_filters
    async fn add_order(mut self) -> SelectMaker<T> {
        let sort_order = self.get_order();
        self.query_string.push_str(&sort_order);
        self
    }

    async fn add_filters(mut self) -> Result<SelectMaker<T>> {
        let mut bounds = self.binds;
        // NB: We skip empty values to avoid situations like `WHERE column_a;`
        // This is not a hard error because requests often contain this kind of empty filter_list.
        if self.query_defn.filter_list.is_empty() {
            return Ok(self);
        }

        let container = &mut self.query_string;
        container.push_str(" WHERE");

        bounds = self.query_defn.filter_list.build_sql(container, bounds)?;

        self.binds = bounds;
        Ok(self)
    }

    /// TODO: Change this to chunk...
    fn add_limits(mut self) -> SelectMaker<T> {
        if let Some(n) = self.query_defn.offset {
            self.query_string.push_str(&format!(" OFFSET {n}"));
        }
        if Some(1) == self.query_defn.take_n {
            self.query_string.push_str(" FETCH FIRST ROW ONLY");
        } else if let Some(n) = self.query_defn.take_n {
            self.query_string.push_str(&format!(" FETCH NEXT {n} ROW ONLY"));
        }
        self
    }

    /// ALWAYS call `stack` after start/start_from as it guarantees the correct order of addition.
    pub(super) async fn stack(self) -> Result<SelectMaker<T>> {
        let r = self.add_filters().await?.add_order().await.add_limits();
        Ok(r)
    }

    /// Finalises the query string by pushing a `;` and then returns it into the wild.
    pub(super) fn bind(&mut self) -> BindQuery<'_> {
        self.query_string.push(';');
        let query = sqlx::query(&self.query_string);
        self.query_defn.bind_vars_to_query(query)
    }

    /// Finalises the query string by pushing a `;` and then returns it into the wild.
    pub(super) async fn bind_and_execute<'b, Ex>(
        mut self,
        pool: Ex,
    ) -> Result<Vec<T>>
    where
        Ex: Executor<'b, Database = Postgres>,
    {
        self.query_string.push(';');
        let query = sqlx::query(&self.query_string);
        self.query_defn
            .bind_vars_to_query(query)
            .try_map(|x| T::from_row(&x))
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    /// Binds this select variables to the specified query.
    ///
    /// Used to combine several selects in a single query.
    pub(super) fn bind_to_query<'b>(
        &'b self,
        query: BindQuery<'b>,
    ) -> BindQuery<'b> {
        self.query_defn.bind_vars_to_query(query)
    }

    /// Checks the Select for safety (injections, incompatible tables, etc).
    pub(super) fn check_select(q: &Select) -> Result<()> {
        // If the select is prefiltered, we skip the other checks.
        if q.skip_main_check {
            return Ok(());
        }

        let fields = T::FIELDS.iter().copied().collect::<AHashSet<&str>>();
        let filter_slice = q.filter_list.slice();
        // Check that fields are ok. (NB: We do not need any complex checks here
        // since the fields are predefined by the table.
        let s_fields = q.field_list.iter().map(|x| ("Field", x));
        let d_fields = q.distinct_on.iter().map(|x| ("Distinct field", x));
        let s_orders = q.order_list.iter().map(|o| ("Order key", &o.field));

        let s_filters = filter_slice.iter().map(|f| ("Filter field", &f.field));

        let chain = s_fields.chain(d_fields).chain(s_orders).chain(s_filters);

        for (thing, x) in chain {
            let x: &str = x;
            if !fields.contains(x) {
                return Err(format!(
                    "{thing} `{field}` not in table `{table}`",
                    thing = thing,
                    field = x,
                    table = T::TABLE
                )
                .into());
            }
        }

        Ok(())
    }

    pub(crate) fn query_string(&self) -> &str {
        &self.query_string
    }

    pub(crate) fn bind_count(&self) -> usize {
        self.binds
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
/// Defines a complete select from a single table.
pub struct Select {
    #[serde(rename = "column_list")]
    pub field_list: Vec<String>,
    pub filter_list: FilterTree,
    #[serde(default)]
    pub order_list: Vec<FieldSortOrder>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub take_n: Option<usize>,
    #[serde(default)]
    pub count_total: Option<bool>,
    /// distinct on clause. Gives a list of fields to be distinct on.
    #[serde(default)]
    pub distinct_on: Vec<String>,
    /// Indicates that the select is already filtered and does not need
    /// an extra check.
    #[serde(skip)]
    pub skip_main_check: bool,
}

/// Позиция NULL значений при сортировке
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum NullPosition {
    /// NULLS FIRST
    First,
    /// NULLS LAST
    Last,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
/// NB: Serde is derived to map using `impl_serde_map` macro.
pub struct FieldSortOrder {
    #[serde(rename = "column_id")]
    pub field: String,
    #[serde(rename = "order_variant")]
    pub order: FieldSortKind,
    /// Позиция NULL значений при сортировке. При [`Option::None`]
    /// будет применено дефолтное значение для соответствующего
    /// [`FieldSortKind`]:
    ///
    /// [`FieldSortKind::Asc`] => [`NullPosition::Last`]
    /// [`FieldSortKind::Desc`] => [`NullPosition::First`]
    #[serde(default)]
    pub null_position: Option<NullPosition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
/// NB: Serde is derived to map using `impl_serde_map` macro.
pub enum FieldSortKind {
    #[serde(rename = "a")]
    Asc,
    #[serde(rename = "d")]
    Desc,
}

impl Display for FieldSortOrder {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let ad = match self.order {
            FieldSortKind::Asc => "ASC",
            FieldSortKind::Desc => "DESC",
        };

        if let Some(null_pos) = self.null_position {
            write!(f, "{} {} {}", self.field, ad, null_pos.as_sql())
        } else {
            write!(f, "{} {}", self.field, ad)
        }
    }
}

impl FieldSortOrder {
    /// This creates the ORDER BY clause.
    fn as_order_by(v: &[Self]) -> Option<String> {
        let mut iter = v.iter();

        let first = match iter.next() {
            Some(v) => v,
            None => return None,
        };
        let mut output = String::with_capacity(5 * v.len());
        output.push_str(&format!(" ORDER BY {}", first));

        for field_sort_order in iter {
            output.push_str(&format!(", {}", field_sort_order));
        }
        Some(output)
    }
}

impl NullPosition {
    fn as_sql(&self) -> &'static str {
        match self {
            NullPosition::First => "NULLS FIRST",
            NullPosition::Last => "NULLS LAST",
        }
    }
}

impl Select {
    // Bind internal variables onto a query
    pub(super) fn bind_vars_to_query<'a>(
        &'a self,
        query: BindQuery<'a>,
    ) -> BindQuery<'a> {
        self.filter_list.bind_vars_to_query(query)
    }

    /// A convenience function for making a select with fields.
    pub fn with_fields<F, I>(fields: I) -> Self
    where
        F: Display,
        I: IntoIterator<Item = F>,
    {
        Select {
            field_list: fields
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
            ..Default::default()
        }
    }

    /// This is a convenience function for when you want to select all fields.
    pub fn full<T: DbItem>() -> Self {
        Self::with_fields(T::FIELDS)
    }

    /// Adds the field with specified name to the list of selected fields.
    pub fn and_field<I: Display>(mut self, field: I) -> Self {
        self.field_list.push(field.to_string());
        self
    }

    /// Add filter to a select. The behaviour is as follows:
    /// 1. If the filter already exists it is expanded. Otherwise a fully new filter
    ///    if added.
    /// 2. If a given `selection_kind` value exists for a filter, the value list
    ///    is expanded, but only with values not already in it. Otherwise the
    ///    `selection_kind` is added with all values.
    ///
    /// This also forms the core of the filter injection mechanism.
    pub fn add_expand_filter<V, I>(
        mut self,
        field: &str,
        kind: SelectionKind,
        values: I,
    ) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        // NB: Will always be a FilterTree::Filter
        if let Some(ref mut f) = self.filter_list.find_with_branch(field) {
            match f {
                // TODO: We may need to replace simple filters of the same kind
                // but we need to check in live fire.
                FilterTree::Filter(inner) if inner.kind == kind => {
                    inner.values.extend(values.into_iter().map(Into::into));
                }
                _ => {
                    let new_filter = Filter::with_values(field, kind, values);
                    **f = f.clone().and(new_filter.into());
                }
            };
        } else {
            let new_filter = Filter::with_values(field, kind, values);
            self.filter_list.push_filter(new_filter);
        }
        self
    }

    /// Добавляет фильтры к Select. Если они уже присутствуют, то они полностью
    /// заменяют старые.
    pub fn set_filter_tree(mut self, tree: FilterTree) -> Self {
        self.filter_list = tree;
        self
    }

    /// This is a convenience function which applies the IN field. Since `in` is a
    /// keyword in Rust, we use `in_any` as the function name.
    pub fn in_any<V, I>(self, field: &str, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        Value: From<V>,
    {
        self.add_expand_filter(field, SelectionKind::In, values)
    }

    /// Работает аналогично [`Select::in_any`], но в случае когда values является [`Option::None`]
    /// фильтр не будет применен
    pub fn in_any_maybe<V, I>(self, field: &str, values: Option<I>) -> Self
    where
        I: IntoIterator<Item = V>,
        Value: From<V>,
    {
        if let Some(values) = values {
            self.add_expand_filter(field, SelectionKind::In, values)
        } else {
            self
        }
    }

    /// a convenience function. The opposite to in any.
    pub fn not_in_any<V, I>(self, field: &str, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        Value: From<V>,
    {
        self.add_expand_filter(field, SelectionKind::NotIn, values)
    }

    /// This is a convenience function which checks the field for equality.
    pub fn eq<I: Into<Value>>(self, field: &str, value: I) -> Self {
        self.add_expand_filter(field, SelectionKind::Equals, [value])
    }

    pub fn eq_maybe<I: Into<Value>>(self, field: &str, value: Option<I>) -> Self {
        if let Some(value) = value {
            self.add_expand_filter(field, SelectionKind::Equals, [value])
        } else {
            self
        }
    }

    /// This is a convenience function which checks the field for inequality.
    pub fn ne<I: Into<Value>>(self, field: &str, value: I) -> Self {
        self.add_expand_filter(field, SelectionKind::NotEquals, [value])
    }

    /// Фильтр на [`SelectionKind::LessEqual`]
    pub fn less_eq<I>(self, field: &str, value: I) -> Self
    where
        I: Into<Value>,
    {
        self.add_expand_filter(field, SelectionKind::LessEqual, [value])
    }

    /// Фильтр на [`SelectionKind::Greater`]
    pub fn greater<I>(self, field: &str, value: I) -> Self
    where
        I: Into<Value>,
    {
        self.add_expand_filter(field, SelectionKind::Greater, [value])
    }

    /// This is a convenience function which creates a select for all fields in the
    /// table while applying an in filter.
    pub fn full_in<I: IntoIterator<Item = Value>, T: DbItem>(
        field: &str,
        values: I,
    ) -> Self {
        Self::full::<T>().in_any::<_, I>(field, values)
    }

    pub fn fields_containing<I, T, S>(mut self, fields: I, like: S) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
        S: ToString,
    {
        let filters = fields.into_iter().map(|f| {
            Filter::with_value(
                f.as_ref(),
                SelectionKind::Contains,
                like.to_string(),
            )
        });
        self.filter_list = self.filter_list.and(FilterTree::or_from_list(filters));
        self
    }

    pub fn array_overlaps<V>(mut self, field: &str, values: V) -> Self
    where
        V: Into<Value>,
    {
        self.filter_list.push_filter(Filter::with_value(
            field,
            SelectionKind::Overlaps,
            values.into(),
        ));
        self
    }

    /// This adds an ordering to the select. The behavior should be as follows:
    /// 1. If the ordering exists for a column it replaces it.
    /// 2. If the ordering does not exist, it adds it.
    pub fn add_replace_order(
        mut self,
        field: &str,
        new_order: FieldSortKind,
    ) -> Self {
        let maybe_item = self.order_list.iter_mut().find(|x| x.field == field);

        if let Some(FieldSortOrder { ref mut order, .. }) = maybe_item {
            *order = new_order;
            return self;
        }
        // If we don't already have that sort order, we add ours.
        self.order_list.push(FieldSortOrder {
            field: field.to_owned(),
            order: new_order,
            null_position: None,
        });
        self
    }

    /// Convenience form of `add_replace_order`. ORDER BY field ASC
    pub fn add_replace_order_asc(self, field: &str) -> Self {
        self.add_replace_order(field, FieldSortKind::Asc)
    }

    /// Convenience form of `add_replace_order`. ORDER BY field DESC
    pub fn add_replace_order_desc(self, field: &str) -> Self {
        self.add_replace_order(field, FieldSortKind::Desc)
    }

    /// Определение порядка NULL значений при сортировке по всем полям
    pub fn with_null_position(mut self, pos: NullPosition) -> Self {
        self.order_list.iter_mut().for_each(|o| o.null_position = Some(pos));
        self
    }

    /// Установление правильной позиции для каждой сортировки:
    ///
    /// * При [`FieldSortKind::Asc`] будет установлено [`NullPosition::First`]
    /// * При [`FieldSortKind::Desc`] будет установлено [`NullPosition::Last`]
    pub fn with_approprtiate_null_position(mut self) -> Self {
        self.order_list.iter_mut().for_each(|o| {
            o.null_position = match o.order {
                FieldSortKind::Asc => Some(NullPosition::First),
                FieldSortKind::Desc => Some(NullPosition::Last),
            }
        });
        self
    }

    /// При сортировке по каждому полю NULL значения будут идти первыми
    pub fn with_nulls_first(self) -> Self {
        self.with_null_position(NullPosition::First)
    }

    /// При сортировке по каждому полю NULL значения будут идти последними
    pub fn with_nulls_last(self) -> Self {
        self.with_null_position(NullPosition::Last)
    }

    /// This function exists to extract a filter based on a field.
    /// This is useful for Sections and in general when changing criteria.
    pub fn remove_filters_by_field(&mut self, field: &str) -> Vec<Filter> {
        self.filter_list.remove_by_field(field)
    }

    /// Fetch only the first record.
    pub fn take_first(mut self) -> Self {
        self.take_n = Some(1);
        self
    }

    /// Skip the first N records.
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    /// Take only N records.
    pub fn take_n(mut self, n: usize) -> Self {
        self.take_n = Some(n);
        self
    }

    /// Skip the first N records.
    pub fn offset_maybe(mut self, n: Option<usize>) -> Self {
        self.offset = n;
        self
    }

    pub fn count_total(mut self, v: bool) -> Self {
        self.count_total = Some(v);
        self
    }

    /// Take only N records.
    pub fn take_n_maybe(mut self, n: Option<usize>) -> Self {
        self.take_n = n;
        self
    }

    /// Clears pagination parameters.
    pub fn clear_pagination(&mut self) {
        self.take_n = None;
        self.offset = None;
    }

    /// Gets the field as a `Vec<&str>`/ This is useful for defining the return
    /// fields
    pub fn fields(&self) -> Vec<&str> {
        self.field_list.iter().map(|x| x as &str).collect::<Vec<&str>>()
    }

    /// Add DISTINCT ON clause. If we have an ORDER BY clause, we should remember
    /// that the distinct fields should also be in the ORDER BY clause.
    /// The order of the ORDER BY fields is critically important in getting your query
    /// correct in this case.
    pub fn distinct_on<I: Display>(mut self, fields: &[I]) -> Self {
        self.distinct_on = fields.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        self
    }

    /// Очистка селекта от полей, которых нет в репрезентации сущности
    pub fn filtered_copy_for<T: DbAdaptor>(&self) -> Self {
        self.filtered_copy(T::DbItem::FIELDS.iter().chain(T::DUP_FIELDS).copied())
    }

    /// Разделение селекта на два, в одном -- поля указанной сущности, в другом -- все остальное.
    pub fn split_for<T: DbAdaptor>(self) -> (Self, Self) {
        self.filtered_split(T::DbItem::FIELDS.iter().chain(T::DUP_FIELDS).copied())
    }

    /// Очистка селекта от полей, которых нет в переданном `fields` массиве
    pub fn filtered_copy<'a, I>(&self, fields: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut q = self.clone();
        q.skip_main_check = true;

        let fields = fields.into_iter().collect::<AHashSet<&str>>();
        // Check that fields are ok. (NB: We do not need any complex checks here
        // since the fields are predefined by the table.
        q.field_list = q
            .field_list
            .into_iter()
            .filter(|x| fields.contains(x as &str))
            .collect::<Vec<_>>();
        q.distinct_on = q
            .distinct_on
            .into_iter()
            .filter(|x| fields.contains(x as &str))
            .collect::<Vec<_>>();
        q.order_list = q
            .order_list
            .into_iter()
            .filter(|x| fields.contains(&x.field as &str))
            .collect::<Vec<_>>();
        q.filter_list.purge_by_fields(&fields);

        q
    }

    /// Разбиение селекта на два, в одном -- поля из `fields`, в другом -- остальные.
    pub fn filtered_split<'a, I>(self, fields: I) -> (Self, Self)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let (mut one, mut two) = (Select::default(), Select::default());

        let fields = fields.into_iter().collect::<AHashSet<&str>>();
        // Check that fields are ok. (NB: We do not need any complex checks here
        // since the fields are predefined by the table.

        (one.field_list, two.field_list) =
            self.field_list.into_iter().partition(|x| fields.contains(x.as_str()));
        (one.distinct_on, two.distinct_on) =
            self.distinct_on.into_iter().partition(|x| fields.contains(x.as_str()));
        (one.order_list, two.order_list) = self
            .order_list
            .into_iter()
            .partition(|x| fields.contains(x.field.as_str()));
        (one.filter_list, two.filter_list) =
            self.filter_list.split_by_fields(&fields);

        (one, two)
    }
}

const TOTAL_COUNT: &str = "_total_count";

pub(crate) struct WithCount<T> {
    item: T,
    total_count: i64,
}

impl<'r, T> FromRow<'r, PgRow> for WithCount<T>
where
    T: FromRow<'r, PgRow>,
{
    fn from_row(row: &'r PgRow) -> std::result::Result<Self, sqlx::Error> {
        let item = T::from_row(row)?;
        let total_count: i64 = row.try_get(TOTAL_COUNT)?;
        Ok(WithCount { item, total_count })
    }
}

impl<T> WithCount<T> {
    pub(crate) fn total_count(&self) -> i64 {
        self.total_count
    }

    pub(crate) fn into_item(self) -> T {
        self.item
    }
}
