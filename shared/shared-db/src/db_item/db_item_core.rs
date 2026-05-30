use itertools::Itertools;
use sqlx::database::HasArguments;
use sqlx::postgres::PgRow;
use sqlx::query::Query;
use sqlx::{Executor, FromRow, Postgres};

use super::selection::WithCount;
use super::*;
use crate::result::{Result, SharedDbError};

use std::fmt::Debug;

/// Максимальное количество привязанных параметров в одном запросе PostgreSQL.
/// Ограничение протокола: номер placeholder `$N` кодируется u16.
pub const MAX_BINDINGS: usize = u16::MAX as usize;

/// При UNNEST-обновлениях на очень больших наборах данных PostgreSQL может
/// потребить огромное количество RAM. Ограничиваем количество строк на одну
/// операцию этой константой.
pub const UPDATE_UNNEST_VALUES: usize = 1_000_000;
const RETURNING: &str = " returning *";

/// Используется для унификации insert/update -- один и тот же метод умеет
/// возвращать либо количество затронутых строк, либо сами строки,
/// в зависимости от флага `returning`.
#[derive(Debug)]
pub enum ReturningEither<T: DbItem> {
    RowsAffected(u64),
    Items(Vec<T>),
}

impl<T: DbItem> ReturningEither<T> {
    /// Создаёт пустой контейнер нужного варианта по флагу `returning`.
    pub fn new(returning: bool) -> Self {
        match returning {
            false => Self::RowsAffected(0),
            true => Self::Items(Vec::<T>::with_capacity(0)),
        }
    }
}

/// Ответ от БД с поддержкой пагинации.
pub struct Paginated<T> {
    /// Срез элементов текущей страницы.
    pub items: Vec<T>,
    /// Полное количество строк без учёта пагинации (только если запрошено).
    pub total: Option<i64>,
}

/// Псевдоним для типизированного запроса sqlx с параметрами Postgres.
pub type BindQuery<'a> =
    Query<'a, Postgres, <Postgres as HasArguments<'a>>::Arguments>;

#[async_trait::async_trait]
/// Основной трейт CRUD-доступа к таблице PostgreSQL.
///
/// Реализуется через proc-macro `#[derive(DbItem)]`, которая по полям структуры
/// и атрибутам генерирует константы (`TABLE`, `FIELDS`, `PRIMARY_KEYS`, …)
/// и методы привязки значений к запросам.
///
/// Каждая структура привязана ровно к одной таблице. Первичный ключ может быть
/// составным -- несколько полей с атрибутом `#[item_field_pkey]`.
///
/// Ограничение `Unpin` нужно для `Query::map` в операциях выборки.
pub trait DbItem:
    FieldTolerance
    + for<'d> sqlx::FromRow<'d, PgRow>
    + Debug
    + Clone
    + Send
    + Sync
    + Unpin
{
    /// Имя таблицы в PostgreSQL.
    const TABLE: &'static str;
    /// Имена колонок первичного ключа.
    const PRIMARY_KEYS: &'static [&'static str];
    /// Имена всех колонок таблицы.
    const FIELDS: &'static [&'static str];
    /// Индексы первичных ключей в массиве `FIELDS`.
    ///
    /// Нужны для быстрой работы с [`DbFieldMask`] без поиска по строкам.
    const PRIMARY_KEY_INDICES: &'static [usize];
    /// Поля, которые вставляются при INSERT (без autogen-колонок).
    const INSERT_FIELDS: &'static [&'static str];
    /// Поля, доступные для UPDATE (без `GENERATED ALWAYS AS` колонок).
    const UPDATE_FIELDS: &'static [&'static str];
    /// Индексы полей в `FIELDS`, которые нельзя обновлять (GENERATED ALWAYS AS).
    const NO_UPDATE_INDICES: &'static [usize];

    /// Привязывает только первичные ключи структуры к запросу.
    /// Используется в WHERE-части UPDATE/DELETE.
    fn bind_pkeys<'a>(&'a self, query: BindQuery<'a>) -> BindQuery<'a>;

    /// Привязывает все поля структуры к запросу в порядке `FIELDS`.
    fn bind_to_query<'a>(&'a self, query: BindQuery<'a>) -> BindQuery<'a>;

    /// Привязывает только insertable-поля (без autogen).
    /// Вызывается при формировании INSERT.
    fn bind_to_query_insert<'a>(&'a self, query: BindQuery<'a>) -> BindQuery<'a>;

    /// Привязывает только поля, выбранные маской.
    ///
    /// Применяется при частичном обновлении, когда нет смысла передавать
    /// в запрос значения полей, которые не меняются.
    fn bind_limited_fields<'a>(
        &'a self,
        query: BindQuery<'a>,
        mask: &DbFieldMask<Self>,
    ) -> BindQuery<'a>;

    /// Заполняет поля значениями перед INSERT, если они равны `Default::default()`.
    ///
    /// Нужен для полей, у которых в Rust нет валидного дефолта для БД
    /// (например, `Option::None` в NOT NULL колонку) или где нужно
    /// проставить `now()` вместо дефолтной даты.
    fn activate_fields(&mut self);

    /// Возвращает все поля через запятую: `"id,name,address"`.
    /// TODO: Перенести в proc-macro.
    fn fields_string() -> String {
        Self::FIELDS.join(",")
    }

    /// Возвращает insertable-поля через запятую.
    fn insert_fields_string() -> String {
        Self::INSERT_FIELDS.join(",")
    }

    /// Возвращает строку placeholder'ов для n-й строки VALUES.
    /// Например: `"($1,$2,$3)"` для `nth=0`.
    /// TODO: Перенести в proc-macro.
    fn field_counter(nth: usize) -> String {
        field_counter(Self::FIELDS, nth)
    }

    /// Аналог [`field_counter`], но только для insertable-полей.
    ///
    /// Отличается от `field_counter` только когда есть autogen-поля,
    /// которые не вставляются вручную.
    fn insert_field_counter(nth: usize) -> String {
        field_counter(Self::INSERT_FIELDS, nth)
    }

    /// Вставляет одну запись в таблицу.
    ///
    /// Первичный ключ генерируется на стороне клиента. Возвращает
    /// количество затронутых строк (обычно 1).
    async fn insert<'a, Ex>(&mut self, pool: Ex) -> Result<u64>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        match self.insert_inner(pool, false).await? {
            ReturningEither::RowsAffected(x) => Ok(x),
            _ => panic!("Only returns rows."),
        }
    }

    /// Вставляет запись и возвращает её из БД (с заполненными autogen-полями).
    async fn insert_returning<'a, Ex>(&mut self, pool: Ex) -> Result<Self>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        match self.insert_inner(pool, true).await? {
            ReturningEither::Items(mut x) => x.pop().ok_or_else(|| {
                SharedDbError::Other("insert_returning returned nothing".into())
            }),
            _ => panic!("Only returns items."),
        }
    }

    async fn insert_inner<'a, Ex>(
        &mut self,
        pool: Ex,
        returning: bool,
    ) -> Result<ReturningEither<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        self.activate_fields();
        let f = format!(
            "INSERT INTO {table} ({fields}) values{field_count}{ret_clause};",
            table = Self::TABLE,
            fields = Self::insert_fields_string(),
            field_count = Self::insert_field_counter(0),
            ret_clause = if returning { RETURNING } else { "" },
        );
        let q = sqlx::query(&f);

        if returning {
            let r = self
                .bind_to_query_insert(q)
                .try_map(|x| Self::from_row(&x))
                .fetch_all(pool)
                .await?;
            Ok(ReturningEither::Items(r))
        } else {
            let r = self
                .bind_to_query_insert(q)
                .execute(pool)
                .await
                .map(|r| r.rows_affected())?;
            Ok(ReturningEither::RowsAffected(r))
        }
    }

    /// Обновляет запись по первичному ключу.
    ///
    /// `update_fields` -- список полей для обновления; `None` обновляет все
    /// UPDATE_FIELDS.
    async fn update<'a, Ex>(
        &self,
        update_fields: Option<&[&str]>,
        pool: Ex,
    ) -> Result<u64>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        match self.update_inner(update_fields, None, pool).await? {
            ReturningEither::RowsAffected(x) => Ok(x),
            _ => unreachable!("Only returns rows."),
        }
    }

    /// Обновляет запись по маске полей.
    async fn update_masked<'a, Ex>(
        &self,
        update_fields: DbFieldMask<Self>,
        pool: Ex,
    ) -> Result<u64>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        match self.update_inner_masked(update_fields, None, pool).await? {
            ReturningEither::RowsAffected(x) => Ok(x),
            _ => unreachable!("Only returns rows."),
        }
    }

    /// Обновляет запись и возвращает обновлённую версию из БД.
    ///
    /// `return_fields` задаёт, какие поля вернуть. Если `None` -- все поля.
    ///
    /// Важно: если у структуры нет `#[sqlx(default)]`, то `return_fields`
    /// обязаны включать все NOT NULL поля, иначе `FromRow` завершится ошибкой.
    async fn update_returning<'a, Ex, D: std::fmt::Display + Send + Sync>(
        &self,
        update_fields: Option<&[&str]>,
        return_fields: Option<&[D]>,
        pool: Ex,
    ) -> Result<Self>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        // Тип D не совпадает с &str, поэтому конвертируем через String.
        let return_fields = return_fields
            .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<_>>());
        let ret_fields = match return_fields {
            Some(ref f) => f.iter().map(|x| x as &str).collect::<Vec<_>>(),
            None => Self::FIELDS.to_vec(),
        };

        match self.update_inner(update_fields, Some(&ret_fields), pool).await? {
            ReturningEither::Items(mut x) => x.pop().ok_or_else(|| {
                SharedDbError::Other("update_returning returned nothing".into())
            }),
            _ => unreachable!("Only returns items."),
        }
    }

    async fn update_inner<'a, Ex>(
        &self,
        update_fields: Option<&[&str]>,
        return_fields: Option<&[&str]>,
        pool: Ex,
    ) -> Result<ReturningEither<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        // Применяем толерантные имена полей перед формированием запроса.
        let update_fields = update_fields.map(Self::apply_tolerance_to_fields);
        let return_fields = return_fields.map(Self::apply_tolerance_to_fields);

        let update_mask = update_fields
            .as_ref()
            .map(|x| make_single_update_bind_mask::<Self>(x))
            .unwrap_or_else(|| make_bind_mask::<Self>(Self::UPDATE_FIELDS));

        let return_mask = return_fields.map(|x| make_bind_mask::<Self>(&x));

        self.update_inner_masked(update_mask, return_mask, pool).await
    }

    async fn update_inner_masked<'a, Ex>(
        &self,
        update_fields: DbFieldMask<Self>,
        return_fields: Option<DbFieldMask<Self>>,
        pool: Ex,
    ) -> Result<ReturningEither<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        use ReturningEither::*;
        let selected_fields = update_fields_helper::<Self>(&update_fields);
        let field_count = field_counter(&selected_fields, 0);

        let return_clause = return_fields
            .map(|x| {
                let fields = select_fields_helper::<Self>(&x);
                format!(" RETURNING {}", fields.join(","))
            })
            .unwrap_or_default();

        let fields = selected_fields.join(",");

        // Для одного поля синтаксис без скобок: `field=$1`.
        // Для нескольких: `(f1,f2)=($1,$2)`.
        let field_thread = if selected_fields.len() == 1 {
            format!("{}={}", fields, field_count)
        } else {
            format!("({})=({})", fields, field_count)
        };

        let where_clause = Self::PRIMARY_KEYS
            .iter()
            .enumerate()
            .map(|(i, k)| {
                format!("{k}=${n}", k = k, n = i + 1 + selected_fields.len())
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        let f = format!(
            "UPDATE {table} set {field_thread} WHERE {where_clause}{returning};",
            table = Self::TABLE,
            field_thread = field_thread,
            where_clause = where_clause,
            returning = return_clause,
        );
        let mut q = sqlx::query(&f);
        q = self.bind_limited_fields(q, &update_fields);
        q = self.bind_pkeys(q);

        let ret = match return_clause.is_empty() {
            true => RowsAffected(q.execute(pool).await?.rows_affected()),
            false => {
                let x = q.try_map(|r| Self::from_row(&r)).fetch_all(pool).await?;
                Items(x)
            }
        };
        Ok(ret)
    }

    /// Вставляет срез записей в одной транзакции.
    ///
    /// Батч разбивается на чанки так, чтобы не превышать [`MAX_BINDINGS`].
    /// Предпочтителен вместо многократных одиночных `insert`, потому что
    /// стоимость одного вызова БД перекрывает стоимость удержания среза.
    ///
    /// Если `DbItem` сгенерирован макросом и атрибут `item_aggr_insert` задан,
    /// то реализация будет отличаться от этой дефолтной.
    async fn insert_vec(
        items: &mut [Self],
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<u64> {
        match Self::insert_vec_inner(items, pool, false).await? {
            ReturningEither::RowsAffected(x) => Ok(x),
            _ => panic!("Only returns rows."),
        }
    }

    /// Вставляет срез записей и возвращает вставленные строки из БД.
    async fn insert_vec_returning(
        items: &mut [Self],
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<Vec<Self>> {
        match Self::insert_vec_inner(items, pool, true).await? {
            ReturningEither::Items(x) => Ok(x),
            _ => panic!("Only returns items."),
        }
    }

    async fn insert_vec_inner(
        items: &mut [Self],
        pool: &mut sqlx::Transaction<Postgres>,
        returning: bool,
    ) -> Result<ReturningEither<Self>> {
        let mut ret = ReturningEither::<Self>::new(returning);
        if items.is_empty() {
            return Ok(ret);
        }
        let fields = Self::insert_fields_string();

        // Разбиваем на чанки по ограничению числа привязок.
        for items in items.chunks_mut(MAX_BINDINGS / fields.len()) {
            let mut query_string = format!(
                "INSERT INTO {table} ({fields}) values{field_count}",
                table = Self::TABLE,
                fields = fields,
                field_count = Self::insert_field_counter(0),
            );
            for i in 1..items.len() {
                query_string.push(',');
                query_string.push_str(&Self::insert_field_counter(i));
            }
            if returning {
                query_string.push_str(RETURNING);
            }
            query_string.push(';');
            let mut q = sqlx::query(&query_string);

            for item in items {
                item.activate_fields();
                q = item.bind_to_query_insert(q);
            }
            match ret {
                ReturningEither::RowsAffected(ref mut n) => {
                    *n += q.execute(&mut *pool).await?.rows_affected();
                }
                ReturningEither::Items(ref mut items) => {
                    let mut inserted = q
                        .try_map(|x| Self::from_row(&x))
                        .fetch_all(&mut *pool)
                        .await?;
                    items.append(&mut inserted);
                }
            }
        }
        Ok(ret)
    }

    /// Обновляет срез записей в одном запросе через VALUES + FROM.
    ///
    /// Использует паттерн `UPDATE t SET ... FROM (values ...) AS upd WHERE t.pk=upd.pk`.
    /// `update_fields = None` обновляет все UPDATE_FIELDS.
    /// Идентификация строк всегда по первичному ключу.
    async fn update_vec(
        items: &[Self],
        update_fields: Option<&[&str]>,
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<u64> {
        match Self::update_vec_inner(items, update_fields, None, pool).await? {
            ReturningEither::RowsAffected(x) => Ok(x),
            ReturningEither::Items(_) => panic!("`update_vec` returns rows."),
        }
    }

    /// Обновляет срез записей и возвращает обновлённые строки из БД.
    ///
    /// Если структура не имеет `#[sqlx(default)]`, `return_fields` должен
    /// быть `None` -- иначе `FromRow` упадёт на отсутствующих колонках.
    async fn update_vec_returning<D: std::fmt::Display + Send + Sync>(
        items: &[Self],
        update_fields: Option<&[&str]>,
        return_fields: Option<&[D]>,
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<Vec<Self>> {
        let return_fields = return_fields
            .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<_>>());
        let ret_fields = match return_fields {
            Some(ref f) => f.iter().map(|x| x as &str).collect::<Vec<_>>(),
            None => Self::FIELDS.to_vec(),
        };

        let r =
            Self::update_vec_inner(items, update_fields, Some(&ret_fields), pool);

        match r.await? {
            ReturningEither::Items(x) => Ok(x),
            ReturningEither::RowsAffected(_) => {
                panic!("`update_vec_returning` returns items.")
            }
        }
    }

    /// Внутренняя реализация векторного обновления.
    ///
    /// Наличие `return_fields` определяет, возвращать строки или счётчик.
    async fn update_vec_inner(
        items: &[Self],
        update_fields: Option<&[&str]>,
        return_fields: Option<&[&str]>,
        pool: &mut sqlx::Transaction<Postgres>,
    ) -> Result<ReturningEither<Self>> {
        use ReturningEither::*;
        let mut ret = ReturningEither::<Self>::new(return_fields.is_some());
        if items.is_empty() {
            return Ok(ret);
        }

        let t = 't';
        let upd = "upd";
        let update_fields = update_fields.map(Self::apply_tolerance_to_fields);
        let return_fields = return_fields.map(Self::apply_tolerance_to_fields);

        let bind_mask = update_fields
            .as_ref()
            .map(|x| make_bind_mask::<Self>(x))
            .unwrap_or_else(|| make_bind_mask::<Self>(Self::UPDATE_FIELDS));

        // fields_to_set: то, что идёт в SET (без autogen_always).
        // fields_to_select: то, что берётся из подзапроса VALUES (ключи + обновляемые поля).
        let fields_to_set = update_set_fields_helper::<Self>(&bind_mask);
        let fields_to_select = select_fields_helper::<Self>(&bind_mask);

        let return_clause = return_fields
            .map(|x| {
                let bind_mask = make_bind_mask::<Self>(&x);
                let fields = select_fields_helper::<Self>(&bind_mask)
                    .into_iter()
                    .map(|x| format!("{t}.{x}", t = t, x = x))
                    .collect::<Vec<_>>();
                format!(" RETURNING {}", fields.join(","))
            })
            .unwrap_or_default();

        for items in items.chunks(MAX_BINDINGS / fields_to_select.len()) {
            // Строим список placeholder'ов для всех строк.
            let mut field_counts = field_counter(&fields_to_select, 0);
            for i in 1..items.len() {
                field_counts.push(',');
                field_counts.push_str(&field_counter(&fields_to_select, i));
            }

            // `SET field_a=upd.field_a, field_b=upd.field_b, ...`
            let field_mapping = fields_to_set
                .iter()
                .map(|x| format!("{f}={upd}.{f}", upd = upd, f = x))
                .collect::<Vec<_>>()
                .join(",");
            let where_clause = Self::PRIMARY_KEYS
                .iter()
                .map(|k| format!("{t}.{k}={upd}.{k}", t = t, upd = upd, k = k))
                .collect::<Vec<_>>()
                .join(" AND ");

            let query_string = format!(
                r#"UPDATE {table} AS {t} SET
                {field_mappings}
               FROM
                (values{field_counts}) AS {upd}({fields})
               WHERE {where_clause}{returning};"#,
                table = Self::TABLE,
                fields = fields_to_select.join(","),
                field_counts = field_counts,
                field_mappings = field_mapping,
                t = t,
                upd = upd,
                where_clause = where_clause,
                returning = return_clause,
            );
            let mut query = sqlx::query(&query_string);

            for item in items {
                query = item.bind_limited_fields(query, &bind_mask);
            }
            match ret {
                RowsAffected(ref mut n) => {
                    *n += query.execute(&mut *pool).await?.rows_affected();
                }
                Items(ref mut it) => {
                    let mut r = query
                        .try_map(|r| Self::from_row(&r))
                        .fetch_all(&mut *pool)
                        .await?;
                    it.append(&mut r);
                }
            }
        }
        Ok(ret)
    }

    /// Выбирает строки по параметрам из [`Select`].
    ///
    /// Поля указаны через `Select::with_fields([Entity::field_name])` --
    /// compile-time константы, сгенерированные `#[derive(DbItem)]`.
    /// Порядок полей в `field_list` должен совпадать с порядком в структуре,
    /// иначе `FromRow` вернёт ошибку декодирования.
    async fn select<'a, Ex>(q: &Select, pool: Ex) -> Result<Vec<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        selection::SelectMaker::<Self>::start(q)
            .await?
            .stack()
            .await?
            .bind_and_execute(pool)
            .await
    }

    /// Выбирает ровно одну строку. Ошибка если нет строк или больше одной.
    async fn select_single<'a, Ex>(q: &Select, pool: Ex) -> Result<Self>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        let mut items = selection::SelectMaker::<Self>::start(q)
            .await?
            .stack()
            .await?
            .bind_and_execute(pool)
            .await?;
        if let Some(item) = items.pop() {
            if !items.is_empty() {
                Err(SharedDbError::Other("select_single: too many items".into()))
            } else {
                Ok(item)
            }
        } else {
            Err(SharedDbError::Other("select_single: nothing is selected".into()))
        }
    }

    /// Выбирает не более одной строки. Ошибка если больше одной.
    async fn select_option<'a, Ex>(q: &Select, pool: Ex) -> Result<Option<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        let mut items = selection::SelectMaker::<Self>::start(q)
            .await?
            .stack()
            .await?
            .bind_and_execute(pool)
            .await?;
        let item = items.pop();
        if !items.is_empty() {
            Err(SharedDbError::Other("select_option: too many items".into()))
        } else {
            Ok(item)
        }
    }

    /// Выбирает все строки таблицы без фильтров.
    ///
    /// Предназначен для тестов, где нужно быстро получить полное содержимое
    /// таблицы без настройки `Select`.
    async fn select_all<'a, Ex>(pool: Ex) -> Result<Vec<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        let q = Select::full::<Self>();
        selection::SelectMaker::<Self>::start(&q)
            .await?
            .stack()
            .await?
            .bind_and_execute(pool)
            .await
    }

    /// Рекурсивный SELECT через `WITH RECURSIVE`.
    ///
    /// Стартует с `initial_select`, затем итеративно добирает строки,
    /// у которых `new_item_field` совпадает с `existing_item_field` уже
    /// найденных строк. Циклы обнаруживаются через `CYCLE ... SET is_cycle`.
    /// Финальный набор строк обрабатывается через `final_select`.
    async fn select_recursive<'a, Ex>(
        initial_select: &Select,
        existing_item_field: &str,
        new_item_field: &str,
        final_select: &Select,
        pool: Ex,
    ) -> Result<Vec<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        // TODO use all fields for initial
        let mut initial_select = initial_select.clone();
        initial_select.field_list =
            Self::FIELDS.iter().copied().map(String::from).collect();
        let initial_maker = selection::SelectMaker::<Self>::start(&initial_select)
            .await?
            .stack()
            .await?;
        let rec_table = format!("rec_{}", Self::TABLE);
        let final_maker = selection::SelectMaker::<Self>::start_from(
            final_select,
            initial_maker.bind_count(),
            &rec_table,
            false,
        )
        .await?
        .stack()
        .await?;

        let fields =
            Self::FIELDS.iter().map(|f| format!("{}.{f}", Self::TABLE)).join(",");
        let pkeys = Self::PRIMARY_KEYS.iter().join(",");

        let query_string = format!(
            "
WITH RECURSIVE {rec_table} AS (
        {initial_select}
    UNION ALL
        SELECT {fields}
        FROM {rec_table}, {table}
        WHERE {table}.{new_item_field} = {rec_table}.{existing_item_field}
) CYCLE {pkeys} SET is_cycle USING path
{final_select};",
            initial_select = initial_maker.query_string(),
            table = Self::TABLE,
            final_select = final_maker.query_string()
        );

        let query = sqlx::query(&query_string);
        let query = initial_maker.bind_to_query(query);
        let query = final_maker.bind_to_query(query);
        query
            .try_map(|x| Self::from_row(&x))
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    /// Выбирает страницу записей с опциональным подсчётом total через оконную функцию.
    ///
    /// `Select` обязан содержать `offset`, `take_n` и `count_total`.
    /// Если `count_total == true`, запрос включает `COUNT(*) OVER()` и возвращает
    /// полное количество строк без повторного запроса к БД.
    async fn select_paginated<'a, Ex>(
        select: &Select,
        pool: Ex,
    ) -> Result<Paginated<Self>>
    where
        Ex: Executor<'a, Database = Postgres>,
    {
        let Select { offset: Some(offset), take_n: Some(take_n), count_total: Some(count_total), .. } = select else {
            return Err(SharedDbError::Other("select is not suitable for paged query".to_string()));
        };
        let (items, total) = if *count_total {
            let items_with_count: Vec<_> =
                selection::SelectMaker::<Self>::start_with_count(select)
                    .await?
                    .stack()
                    .await?
                    .bind()
                    .try_map(|x| WithCount::<Self>::from_row(&x))
                    .fetch_all(pool)
                    .await?;
            let total_count = match items_with_count.get(0) {
                None if *offset == 0 => Some(0),
                Some(item_with_count) => Some(item_with_count.total_count()),
                _ => {
                    tracing::warn!(
                        "empty chunk at offset={offset} and take_n={take_n}"
                    );
                    None
                }
            };
            let items =
                items_with_count.into_iter().map(WithCount::into_item).collect();
            (items, total_count)
        } else {
            let items = selection::SelectMaker::<Self>::start(select)
                .await?
                .stack()
                .await?
                .bind_and_execute(pool)
                .await?;
            (items, None)
        };
        Ok(Paginated { items, total })
    }
}

/// Маркерный трейт: структура может быть десериализована из неполного набора колонок.
///
/// Генерируется автоматически вместе с `#[derive(DbItem)]`, если ни одно поле
/// не помечено `#[item_field_required]`. При наличии таких полей трейт не генерируется,
/// и попытка использовать структуру в `Select` с неполным списком полей приведёт
/// к ошибке компиляции.
pub trait DbItemPartialSelect {}
