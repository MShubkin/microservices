//! Построитель SELECT-запросов.
//!
//! Модуль намеренно живёт внутри `db_item`, потому что тесно связан с [`DbItem`]:
//! при сборке запроса проверяет, что все запрошенные поля реально существуют
//! в таблице (по константе `T::FIELDS`).
//!
//! Предполагается, что структура [`Select`] одинакова для всех крейтов, которые
//! зависят от `asez2_db_shared`. Если формат запросов фронтенда отличается,
//! его адаптируют до [`Select`] снаружи -- все поля структуры публичны.
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

/// Внутренний строитель SQL-строки запроса.
///
/// Не создаётся напрямую -- используется через `DbItem::select` и
/// `DbItem::select_paginated`. Хранит текущую строку запроса и счётчик
/// привязок, чтобы корректно нумеровать placeholder'ы `$N` при соединении
/// нескольких selects в один запрос (joined queries).
pub struct SelectMaker<T> {
    query_defn: Select,
    query_string: String,
    /// Текущий счётчик привязанных переменных. Начинается с 1 для одиночных,
    /// или с N+1 при вложении в составной запрос.
    binds: usize,
    phantom_data: PhantomData<T>,
}

// NB: Почти всё async, потому что некоторые операции могут занимать >10 мкс.
impl<T: DbItem + Unpin> SelectMaker<T> {
    /// Инициализирует построитель из заданной таблицы `table` с начальным
    /// смещением счётчика `start_bind`.
    ///
    /// Применяется при вложенных запросах (joined queries), где нумерация
    /// `$N` должна продолжаться после предыдущего select'а.
    /// Пример вложенного запроса:
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
        // Переименовываем поля фронтенда в реальные имена колонок.
        T::apply_tolerance_to_select(&mut q);
        // Проверка допустимости полей должна идти ПОСЛЕ подстановки.
        Self::check_select(&q)?;

        let selection = {
            let mut field_list = String::new();
            for rec_field in q.field_list.iter() {
                field_list.extend(format!("{},", rec_field).chars());
            }
            // Пустой список полей означает SELECT *.
            if field_list.is_empty() {
                field_list.push('*');
            } else {
                field_list.pop();
            }

            if with_count {
                // Добавляем оконную функцию для подсчёта полного числа строк
                // без второго запроса к БД.
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
            binds: start_bind, // NB: нумерация SQL начинается с 1.
            phantom_data: PhantomData,
        })
    }

    /// Инициализирует построитель для таблицы `T::TABLE` с нумерацией с 1.
    pub(super) async fn start(q: &Select) -> Result<SelectMaker<T>> {
        Self::start_from(q, 1, T::TABLE, false).await
    }

    /// Инициализирует построитель с добавлением `COUNT(*) OVER()` для пагинации.
    pub(super) async fn start_with_count(q: &Select) -> Result<SelectMaker<T>> {
        Self::start_from(q, 1, T::TABLE, true).await
    }

    /// Возвращает ORDER BY-строку без добавления её к запросу.
    ///
    /// Нужен для агрегированных JOIN'ов, где порядок сортировки передаётся
    /// снаружи, а не встраивается в подзапрос.
    /// НБ: порядок вызовов важен -- `add_order` всегда после `add_filters`.
    pub(super) fn get_order(&self) -> String {
        FieldSortOrder::as_order_by(&self.query_defn.order_list).unwrap_or_default()
    }

    /// Дописывает ORDER BY к строке запроса.
    /// НБ: порядок вызовов важен -- `add_order` всегда после `add_filters`.
    async fn add_order(mut self) -> SelectMaker<T> {
        let sort_order = self.get_order();
        self.query_string.push_str(&sort_order);
        self
    }

    async fn add_filters(mut self) -> Result<SelectMaker<T>> {
        let mut bounds = self.binds;
        // Пустой filter_list -- не ошибка, просто запрос без WHERE.
        if self.query_defn.filter_list.is_empty() {
            return Ok(self);
        }

        let container = &mut self.query_string;
        container.push_str(" WHERE");

        bounds = self.query_defn.filter_list.build_sql(container, bounds)?;

        self.binds = bounds;
        Ok(self)
    }

    /// Добавляет OFFSET и FETCH NEXT к строке запроса.
    /// TODO: Заменить на chunk-based подход.
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

    /// Финализирует построитель, добавляя фильтры, сортировку и лимиты.
    ///
    /// Всегда вызывать после `start`/`start_from`. Метод гарантирует правильный
    /// порядок добавления частей запроса: WHERE -> ORDER BY -> OFFSET/FETCH.
    pub(super) async fn stack(self) -> Result<SelectMaker<T>> {
        let r = self.add_filters().await?.add_order().await.add_limits();
        Ok(r)
    }

    /// Добавляет `;` и возвращает готовый запрос с привязанными переменными.
    pub(super) fn bind(&mut self) -> BindQuery<'_> {
        self.query_string.push(';');
        let query = sqlx::query(&self.query_string);
        self.query_defn.bind_vars_to_query(query)
    }

    /// Добавляет `;`, привязывает переменные и выполняет запрос.
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

    /// Привязывает переменные этого select'а к внешнему запросу.
    ///
    /// Нужно для объединения нескольких select'ов в один запрос --
    /// каждый вносит свои переменные в общий `BindQuery`.
    pub(super) fn bind_to_query<'b>(
        &'b self,
        query: BindQuery<'b>,
    ) -> BindQuery<'b> {
        self.query_defn.bind_vars_to_query(query)
    }

    /// Проверяет корректность [`Select`] для данной таблицы.
    ///
    /// Убеждается, что все поля из `field_list`, `order_list`, `filter_list`
    /// и `distinct_on` реально существуют в `T::FIELDS`. Это защита от
    /// SQL-инъекций через имена полей и от опечаток.
    ///
    /// Пропускает проверку если `skip_main_check == true` (для производных
    /// selects, где поля уже проверены ранее).
    pub(super) fn check_select(q: &Select) -> Result<()> {
        if q.skip_main_check {
            return Ok(());
        }

        let fields = T::FIELDS.iter().copied().collect::<AHashSet<&str>>();
        let filter_slice = q.filter_list.slice();
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

/// Описание одного SELECT-запроса к таблице.
///
/// Сериализуется/десериализуется для передачи с фронтенда. Поля публичны,
/// чтобы снаружи можно было собрать адаптер если формат API отличается.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
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
    /// Поля для DISTINCT ON. Если используется с ORDER BY, эти поля обязаны
    /// присутствовать в ORDER BY первыми.
    #[serde(default)]
    pub distinct_on: Vec<String>,
    /// Пропустить проверку полей в `SelectMaker::check_select`.
    /// Выставляется у производных select'ов, где поля уже проверены.
    #[serde(skip)]
    pub skip_main_check: bool,
}

/// Позиция NULL значений при сортировке.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum NullPosition {
    /// NULLS FIRST
    First,
    /// NULLS LAST
    Last,
}

/// Порядок сортировки по одному полю.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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

/// Направление сортировки: возрастающее или убывающее.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
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
    /// Строит строку вида `ORDER BY field1 ASC, field2 DESC`.
    /// Возвращает `None` для пустого списка.
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
    // Привязывает переменные фильтров к запросу.
    pub(super) fn bind_vars_to_query<'a>(
        &'a self,
        query: BindQuery<'a>,
    ) -> BindQuery<'a> {
        self.filter_list.bind_vars_to_query(query)
    }

    /// Создаёт Select с заданным списком полей.
    ///
    /// Поля обычно передаются как константы, сгенерированные `#[derive(DbItem)]`:
    /// `Select::with_fields([MyEntity::name, MyEntity::created_at])`.
    /// Это даёт проверку имён на этапе компиляции.
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

    /// Создаёт Select, включающий все поля таблицы `T`.
    pub fn full<T: DbItem>() -> Self {
        Self::with_fields(T::FIELDS)
    }

    /// Добавляет ещё одно поле к списку выборки.
    pub fn and_field<I: Display>(mut self, field: I) -> Self {
        self.field_list.push(field.to_string());
        self
    }

    /// Добавляет или расширяет фильтр для поля.
    ///
    /// Логика:
    /// 1. Если фильтр по этому полю уже есть и тип совпадает -- добавляем значения.
    /// 2. Если тип не совпадает -- добавляем новый фильтр через AND.
    /// 3. Если фильтра ещё нет -- добавляем новый.
    ///
    /// Это основа механизма инъекции фильтров: сервис может добавить свои
    /// фильтры поверх фильтров фронтенда не ломая логику.
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

    /// Полностью заменяет дерево фильтров. Старые фильтры удаляются.
    pub fn set_filter_tree(mut self, tree: FilterTree) -> Self {
        self.filter_list = tree;
        self
    }

    /// Фильтр `field IN (values)`. Использует `=ANY($1)` в PostgreSQL.
    /// Имя `in_any` вместо `in` -- чтобы не конфликтовать с ключевым словом Rust.
    pub fn in_any<V, I>(self, field: &str, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        Value: From<V>,
    {
        self.add_expand_filter(field, SelectionKind::In, values)
    }

    /// Аналог [`Select::in_any`], но пропускает фильтр если `values == None`.
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

    /// Фильтр `field NOT IN (values)`.
    pub fn not_in_any<V, I>(self, field: &str, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        Value: From<V>,
    {
        self.add_expand_filter(field, SelectionKind::NotIn, values)
    }

    /// Фильтр `field = value`.
    pub fn eq<I: Into<Value>>(self, field: &str, value: I) -> Self {
        self.add_expand_filter(field, SelectionKind::Equals, [value])
    }

    /// Аналог [`Select::eq`], но пропускает фильтр если `value == None`.
    pub fn eq_maybe<I: Into<Value>>(self, field: &str, value: Option<I>) -> Self {
        if let Some(value) = value {
            self.add_expand_filter(field, SelectionKind::Equals, [value])
        } else {
            self
        }
    }

    /// Фильтр `field != value`.
    pub fn ne<I: Into<Value>>(self, field: &str, value: I) -> Self {
        self.add_expand_filter(field, SelectionKind::NotEquals, [value])
    }

    /// Фильтр [`SelectionKind::LessEqual`]: `field <= value`.
    pub fn less_eq<I>(self, field: &str, value: I) -> Self
    where
        I: Into<Value>,
    {
        self.add_expand_filter(field, SelectionKind::LessEqual, [value])
    }

    /// Фильтр [`SelectionKind::Greater`]: `field > value`.
    pub fn greater<I>(self, field: &str, value: I) -> Self
    where
        I: Into<Value>,
    {
        self.add_expand_filter(field, SelectionKind::Greater, [value])
    }

    /// Создаёт Select со всеми полями таблицы и фильтром `field IN (values)`.
    pub fn full_in<I: IntoIterator<Item = Value>, T: DbItem>(
        field: &str,
        values: I,
    ) -> Self {
        Self::full::<T>().in_any::<_, I>(field, values)
    }

    /// Фильтр ILIKE/`~` по нескольким полям одновременно (OR между полями).
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

    /// Фильтр `field && values` (перекрытие массивов PostgreSQL).
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

    /// Добавляет или заменяет сортировку по полю.
    ///
    /// Если поле уже есть в `order_list` -- заменяет направление.
    /// Если нет -- добавляет в конец.
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
        self.order_list.push(FieldSortOrder {
            field: field.to_owned(),
            order: new_order,
            null_position: None,
        });
        self
    }

    /// Добавляет/заменяет сортировку по возрастанию.
    pub fn add_replace_order_asc(self, field: &str) -> Self {
        self.add_replace_order(field, FieldSortKind::Asc)
    }

    /// Добавляет/заменяет сортировку по убыванию.
    pub fn add_replace_order_desc(self, field: &str) -> Self {
        self.add_replace_order(field, FieldSortKind::Desc)
    }

    /// Устанавливает позицию NULL для всех полей сортировки.
    pub fn with_null_position(mut self, pos: NullPosition) -> Self {
        self.order_list.iter_mut().for_each(|o| o.null_position = Some(pos));
        self
    }

    /// Устанавливает позицию NULL в соответствии с направлением сортировки:
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

    /// При сортировке по каждому полю NULL значения будут идти первыми.
    pub fn with_nulls_first(self) -> Self {
        self.with_null_position(NullPosition::First)
    }

    /// При сортировке по каждому полю NULL значения будут идти последними.
    pub fn with_nulls_last(self) -> Self {
        self.with_null_position(NullPosition::Last)
    }

    /// Удаляет все фильтры по заданному полю и возвращает их.
    ///
    /// Используется когда нужно перехватить фильтры фронтенда и заменить
    /// их на другие (например, переконвертировать в jsonpath).
    pub fn remove_filters_by_field(&mut self, field: &str) -> Vec<Filter> {
        self.filter_list.remove_by_field(field)
    }

    /// Ограничивает выборку первой записью.
    pub fn take_first(mut self) -> Self {
        self.take_n = Some(1);
        self
    }

    /// Пропускает первые `n` записей.
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    /// Берёт не более `n` записей.
    pub fn take_n(mut self, n: usize) -> Self {
        self.take_n = Some(n);
        self
    }

    /// Устанавливает offset из Option -- удобно когда источник сам опциональный.
    pub fn offset_maybe(mut self, n: Option<usize>) -> Self {
        self.offset = n;
        self
    }

    /// Включает или выключает подсчёт общего числа строк при пагинации.
    pub fn count_total(mut self, v: bool) -> Self {
        self.count_total = Some(v);
        self
    }

    /// Устанавливает take_n из Option.
    pub fn take_n_maybe(mut self, n: Option<usize>) -> Self {
        self.take_n = n;
        self
    }

    /// Сбрасывает параметры пагинации (offset и take_n).
    pub fn clear_pagination(&mut self) {
        self.take_n = None;
        self.offset = None;
    }

    /// Возвращает список полей как срез строк.
    pub fn fields(&self) -> Vec<&str> {
        self.field_list.iter().map(|x| x as &str).collect::<Vec<&str>>()
    }

    /// Добавляет DISTINCT ON по указанным полям.
    ///
    /// При наличии ORDER BY поля DISTINCT ON обязаны быть первыми в нём,
    /// иначе PostgreSQL вернёт ошибку.
    pub fn distinct_on<I: Display>(mut self, fields: &[I]) -> Self {
        self.distinct_on = fields.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        self
    }

    /// Возвращает копию Select, оставив только поля, известные адаптору `T`.
    pub fn filtered_copy_for<T: DbAdaptor>(&self) -> Self {
        self.filtered_copy(T::DbItem::FIELDS.iter().chain(T::DUP_FIELDS).copied())
    }

    /// Разбивает Select на два: в первом -- поля сущности `T`, во втором -- остальное.
    ///
    /// Полезно при обработке составного запроса с несколькими сущностями.
    pub fn split_for<T: DbAdaptor>(self) -> (Self, Self) {
        self.filtered_split(T::DbItem::FIELDS.iter().chain(T::DUP_FIELDS).copied())
    }

    /// Возвращает копию Select, оставив только поля из `fields`.
    pub fn filtered_copy<'a, I>(&self, fields: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut q = self.clone();
        q.skip_main_check = true;

        let fields = fields.into_iter().collect::<AHashSet<&str>>();
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

    /// Разбивает Select на два: в первом -- поля из `fields`, во втором -- остальные.
    pub fn filtered_split<'a, I>(self, fields: I) -> (Self, Self)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let (mut one, mut two) = (Select::default(), Select::default());

        let fields = fields.into_iter().collect::<AHashSet<&str>>();

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

/// Имя псевдоколонки для результата `COUNT(*) OVER()` при пагинации.
const TOTAL_COUNT: &str = "_total_count";

/// Обёртка для строки с добавленным полем `_total_count`.
///
/// `FromRow` читает и сам элемент и счётчик из одной строки результата,
/// чтобы не делать второй запрос `SELECT COUNT(*)`.
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
