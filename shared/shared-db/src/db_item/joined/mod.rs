//! Построитель JOIN-запросов к нескольким таблицам.
//!
//! Позволяет декларативно описать соединение нескольких [`DbItem`]-таблиц
//! без написания сырого SQL. Три вида соединений:
//! - [`NormalJoin`] -- INNER JOIN
//! - [`LeftJoin`] -- LEFT JOIN (дополнительная таблица может отсутствовать)
//! - [`AggrJoin`] -- LEFT JOIN с агрегацией через `array_agg()`
//!
//! Для каждой пары таблиц нужно реализовать [`JoinTo`] (обычно через [`impl_join_on!`]).
//! Конечный результат строится макросом [`joined!`], который генерирует
//! структуру результата и селектор с типизированным API.
use super::selection::{FieldSortKind, FieldSortOrder, NullPosition, SelectMaker};
use super::Select;
use crate::result::{Result, SharedDbError};
use crate::DbItem;

use sqlx::postgres::PgRow;
use sqlx::{Executor, Postgres};
use std::marker::PhantomData;

#[cfg(test)]
mod tests;

/// Маркер для агрегирующего JOIN (результаты оборачиваются в `array_agg`).
#[derive(Debug, Default, Clone, Copy)]
pub struct AggrJoin;

/// Маркер для LEFT JOIN (в результате может быть `Option<T>`).
#[derive(Debug, Default, Clone, Copy)]
pub struct LeftJoin;

/// Маркер для обычного INNER JOIN.
#[derive(Debug, Default, Clone, Copy)]
pub struct NormalJoin;

/// Типаж-маркер для вида соединения. Константы `LEFT` и `AGG` управляют
/// генерацией SQL в `JoinedSelect::join`.
pub trait JoinKind: Default {
    const LEFT: bool = false;
    const AGG: bool = false;
}

impl JoinKind for NormalJoin {}

impl JoinKind for LeftJoin {
    const LEFT: bool = true;
}

impl JoinKind for AggrJoin {
    const AGG: bool = true;
}

#[async_trait::async_trait]
/// Трейт соединения: описывает, как таблица `Self` присоединяется к `ForeignTable`.
///
/// Дополнительный параметр `Join: JoinKind` позволяет реализовать `JoinTo`
/// несколько раз для одной пары таблиц с разными видами соединения.
/// Например:
/// ```ignore
/// impl JoinTo<Agenda, AggrJoin> for AgendaItem {}
/// impl JoinTo<RelAgendaProtocolItem, LeftJoin> for AgendaItem {}
/// ```
pub trait JoinTo<ForeignTable: DbItem, Join: JoinKind>: DbItem {
    /// Поле `ForeignTable`, по которому происходит соединение по умолчанию.
    const DEFAULT_FKEY: &'static str;
    /// Поле `Self`, которое ссылается на `ForeignTable` по умолчанию.
    const DEFAULT_PKEY: &'static str;

    /// Проверяет, что `select`, `own_key` и `foreign_table_key` корректны
    /// для данного JOIN -- поля присутствуют в соответствующих таблицах
    /// и включены в `field_list` select'а.
    ///
    /// Без этой проверки мы всё равно упадём при выполнении запроса --
    /// но уже за границей БД, с менее понятным сообщением об ошибке.
    fn check_joinable_select(
        select: &Select,
        own_key: &str,
        foreign_table_key: &str,
    ) -> Result<()> {
        if !Self::FIELDS.iter().any(|x| *x == own_key) {
            return Err(SharedDbError::Join(format!(
                "field '{}' used for join but not found in table '{}'",
                own_key,
                Self::TABLE,
            )));
        }
        if !select.field_list.iter().any(|x| x == own_key) {
            return Err(SharedDbError::Join(format!(
                "field '{}' used for join but not found in selection '{}'",
                own_key,
                select.field_list.join(", ")
            )));
        }

        if !ForeignTable::FIELDS.iter().any(|x| *x == foreign_table_key) {
            return Err(SharedDbError::Join(format!(
                "field '{}' not found in table '{}'",
                foreign_table_key,
                ForeignTable::TABLE,
            )));
        }

        Ok(())
    }

    /// Создаёт данные JOIN по указанному ключу `ForeignTable`.
    ///
    /// Используется когда нужно переопределить ключ соединения (не `DEFAULT_PKEY`).
    /// `own_key` (ключ `Self`) берётся из `DEFAULT_FKEY`.
    fn join_on(f: &str) -> JoinedSelectData<Self, ForeignTable, Join> {
        let mut j = Self::join_default();
        j.foreign_table_key = f.to_string();
        j
    }

    /// Создаёт данные JOIN с ключами по умолчанию.
    fn join_default() -> JoinedSelectData<Self, ForeignTable, Join> {
        JoinedSelectData {
            foreign_table_key: Self::DEFAULT_PKEY.to_string(),
            own_key: Self::DEFAULT_FKEY.to_string(),
            type_data: PhantomData::<Self>,
            foreign_type: PhantomData::<ForeignTable>,
            kind: PhantomData::<Join>,
            select: Select::full::<Self>(),
            distinct: false,
            ordered_aggr: vec![],
            distinct_aggr: false,
            outer_order: vec![],
        }
    }
}

impl<T, O, K> JoinedSelectData<T, O, K>
where
    T: JoinTo<O, K>,
    O: DbItem,
    K: JoinKind,
{
    /// Переопределяет ключ `Self`, по которому происходит соединение.
    pub fn eq_own(mut self, own_field: &str) -> Self {
        self.own_key = own_field.to_string();
        self
    }

    /// Применяет фильтры, порядок и distinct_on из `select` к этому JOIN.
    ///
    /// Список полей не наследуется -- всегда берутся все поля таблицы.
    /// Это гарантирует, что `FromRow` сможет собрать структуру из результата.
    pub fn selecting(mut self, select: Select) -> JoinedSelectData<T, O, K> {
        self.select.filter_list = select.filter_list;
        self.select.order_list = select.order_list;
        self.select.distinct_on = select.distinct_on;
        self
    }

    /// Добавляет DISTINCT к подзапросу этой таблицы.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Добавляет сортировку во внешний ORDER BY составного запроса.
    ///
    /// Не влияет на AGGR JOIN (агрегированные таблицы сортируются внутри
    /// `array_agg` через [`order_aggr_by`]).
    /// Возвращает ошибку, если поле не принадлежит таблице.
    pub fn add_outer_order(
        &mut self,
        field: &str,
        order: FieldSortKind,
    ) -> Result<()> {
        if K::AGG {
            return Ok(());
        }

        if !T::FIELDS.iter().any(|x| *x == field) {
            return Err(format!(
                "Outer ORDER BY clause `{field}` not in table `{table}`",
                field = field,
                table = T::TABLE
            )
            .into());
        }
        self.outer_order.push(FieldSortOrder {
            field: field.to_string(),
            order,
            null_position: None,
        });

        Ok(())
    }

    pub fn with_outer_order_asc(mut self, field: &str) -> Result<Self> {
        self.add_outer_order(field, FieldSortKind::Asc)?;
        Ok(self)
    }

    pub fn with_outer_order_desc(mut self, field: &str) -> Result<Self> {
        self.add_outer_order(field, FieldSortKind::Desc)?;
        Ok(self)
    }

    /// Устанавливает позицию NULL для всех полей внешнего ORDER BY.
    pub fn with_null_position(mut self, pos: NullPosition) -> Self {
        self.outer_order.iter_mut().for_each(|o| o.null_position = Some(pos));
        self
    }

    /// NULL значения идут первыми во внешней сортировке.
    pub fn with_nulls_first(self) -> Self {
        self.with_null_position(NullPosition::First)
    }

    /// NULL значения идут последними во внешней сортировке.
    pub fn with_nulls_last(self) -> Self {
        self.with_null_position(NullPosition::Last)
    }
}

impl<T, O> JoinedSelectData<T, O, AggrJoin>
where
    T: JoinTo<O, AggrJoin>,
    O: DbItem,
{
    /// Задаёт внутренний порядок сортировки для `array_agg(table ORDER BY field)`.
    ///
    /// Влияет только на агрегирующие JOIN'ы -- для остальных видов не имеет смысла.
    fn order_aggr_by(mut self, field: &str, order: FieldSortKind) -> Result<Self> {
        if !T::FIELDS.iter().any(|x| *x == field) {
            return Err(format!(
                "aggregate ORDER BY clause `{field}` not in table `{table}`",
                field = field,
                table = T::TABLE
            )
            .into());
        }
        self.ordered_aggr.push(FieldSortOrder {
            field: field.to_string(),
            order,
            null_position: None,
        });
        Ok(self)
    }

    pub fn add_order_aggr_asc_by(self, field: &str) -> Result<Self> {
        self.order_aggr_by(field, FieldSortKind::Asc)
    }

    pub fn add_order_aggr_desc_by(self, field: &str) -> Result<Self> {
        self.order_aggr_by(field, FieldSortKind::Desc)
    }

    /// Добавляет `DISTINCT` внутрь `array_agg`, чтобы убрать дубликаты.
    ///
    /// NB: требует наличия ORDER BY в подзапросе; не совместим с `order_aggr_by`.
    pub fn distinct_aggr(mut self, value: bool) -> Self {
        self.distinct_aggr = value;
        self
    }
}

/// Хранит все параметры для построения одного JOIN.
///
/// Создаётся через `JoinTo::join_on` или `JoinTo::join_default` и
/// при необходимости настраивается через методы `eq_own`, `selecting` и т.д.
/// Передаётся в `JoinedSelect::join`.
#[derive(Debug, Clone)]
pub struct JoinedSelectData<T, O, K>
where
    T: JoinTo<O, K>,
    O: DbItem,
    K: JoinKind,
{
    select: Select,
    own_key: String,
    foreign_table_key: String,
    distinct: bool,
    ordered_aggr: Vec<FieldSortOrder>,
    distinct_aggr: bool,
    outer_order: Vec<FieldSortOrder>,
    type_data: PhantomData<T>,
    foreign_type: PhantomData<O>,
    kind: PhantomData<K>,
}

/// Промежуточный фрагмент истории построения JOIN.
///
/// Хранит уже построенный подзапрос таблицы: строку SQL и соответствующий `Select`
/// с переменными для привязки. Стирает типовую информацию (`T::TABLE`, `T::FIELDS`),
/// чтобы фрагменты можно было хранить в однородном `Vec`.
struct HistoryFragment {
    // Select хранится, потому что содержит переменные для привязки к запросу.
    select: Select,
    /// Имя таблицы нужно при финализации составного запроса.
    table: &'static str,
    /// Агрегирующие таблицы требуют GROUP BY.
    is_aggregate: bool,
    /// SQL-фрагмент, сгенерированный SelectMaker.
    query_string: String,
    distinct_table: bool,
    ordered_aggr: Vec<FieldSortOrder>,
    distinct_aggr: bool,
    outer_order: Vec<FieldSortOrder>,
}

/// Построитель составного JOIN-запроса.
///
/// Заполняется последовательными вызовами `initiate` + `join`, после чего
/// финализируется через `construct_query` и выполняется через `FinalJoinedSelect::get`.
pub struct JoinedSelect {
    /// Фрагменты по каждой таблице в порядке добавления.
    tables: Vec<HistoryFragment>,
    /// Суммарный счётчик привязок для корректной нумерации `$N`.
    binds: usize,
}

impl JoinedSelect {
    /// Инициализирует JOIN по главной таблице `T`.
    ///
    /// Список полей принудительно заменяется полным набором (`Select::full<T>`),
    /// чтобы `FromRow` мог восстановить структуру из результата.
    pub async fn initiate<T: DbItem>(
        mut select: Select,
        distinct: bool,
        outer_order: Vec<FieldSortOrder>,
    ) -> Result<JoinedSelect> {
        select.field_list = Select::full::<T>().field_list;
        let maker = SelectMaker::<T>::start(&select).await?.stack().await?;

        let q = maker.query_string();
        let query_string = format!("({q}) as {table}", q = q, table = T::TABLE,);

        Ok(JoinedSelect {
            binds: maker.bind_count(),
            tables: vec![HistoryFragment {
                select,
                table: T::TABLE,
                is_aggregate: false,
                query_string,
                distinct_table: distinct,
                ordered_aggr: vec![],
                distinct_aggr: false,
                outer_order,
            }],
        })
    }

    /// Добавляет следующую таблицу к JOIN.
    ///
    /// Тип JOIN (INNER/LEFT/AGG) определяется параметром `K`. Нумерация
    /// placeholder'ов продолжается с того места, где остановилась предыдущая
    /// таблица (`self.binds`).
    pub async fn join<T, O, K>(
        mut self,
        select_box: JoinedSelectData<T, O, K>,
    ) -> Result<Self>
    where
        T: JoinTo<O, K>,
        O: DbItem,
        K: JoinKind,
    {
        T::check_joinable_select(
            &select_box.select,
            &select_box.own_key,
            &select_box.foreign_table_key,
        )?;

        let maker = SelectMaker::<T>::start_from(
            &select_box.select,
            self.binds,
            T::TABLE,
            false,
        )
        .await?
        .stack()
        .await?;
        let q = maker.query_string();
        let query_string = format!(
            " {join_kind} JOIN ({q}) as {table}
                ON {table}.{own_key}={other_t}.{other_key}",
            join_kind = if K::LEFT || K::AGG { "LEFT" } else { "INNER" },
            q = q,
            table = T::TABLE,
            other_t = O::TABLE,
            own_key = select_box.own_key,
            other_key = select_box.foreign_table_key,
        );

        self.binds = maker.bind_count();
        self.tables.push(HistoryFragment {
            select: select_box.select,
            table: T::TABLE,
            is_aggregate: K::AGG,
            query_string,
            distinct_table: select_box.distinct,
            ordered_aggr: select_box.ordered_aggr,
            distinct_aggr: select_box.distinct_aggr,
            outer_order: select_box.outer_order,
        });
        Ok(self)
    }

    /// Строит SELECT-часть: для обычных таблиц -- `table`, для агрегирующих --
    /// `array_agg(table ORDER BY ...) as table`.
    fn construct_fields(&self) -> String {
        self.tables
            .iter()
            .map(|x| {
                if !x.is_aggregate {
                    return x.table.to_string();
                }
                let distinct = if x.distinct_aggr { "distinct " } else { "" };
                let mut order = x
                    .ordered_aggr
                    .iter()
                    .map(|f| format!("{t}.{f}", t = x.table, f = f))
                    .collect::<Vec<_>>()
                    .join(",");
                if !order.is_empty() {
                    order = format!(" ORDER BY {}", order);
                }
                format!(
                    "array_agg({distinct}{t}{order}) as {t}",
                    t = x.table,
                    order = order
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Строит GROUP BY по всем не-агрегирующим таблицам (и полям внешней сортировки).
    fn construct_group_by(&self) -> String {
        let not_aggr_count = self.tables.iter().filter(|x| !x.is_aggregate).count();

        if not_aggr_count != 0 {
            let groups = self
                .tables
                .iter()
                .filter(|x| !x.is_aggregate)
                .flat_map(|x| {
                    let groups_by_orderings = x.outer_order.iter().map(|order| {
                        format!(
                            "{table}.{order_field}",
                            table = x.table,
                            order_field = order.field
                        )
                    });
                    let aggr_group = format!("{table}.*", table = x.table);

                    std::iter::once(aggr_group).chain(groups_by_orderings)
                })
                .collect::<Vec<_>>()
                .join(",");

            format!(" GROUP BY {}", groups)
        } else {
            String::default()
        }
    }

    /// Строит DISTINCT ON по таблицам с флагом `distinct_table == true`.
    fn construct_outer_distincts(&self) -> String {
        let distincts = self
            .tables
            .iter()
            .filter(|x| x.distinct_table)
            .map(|x| x.table)
            .collect::<Vec<_>>()
            .join(",");
        match distincts.is_empty() {
            true => String::with_capacity(0),
            false => format!("DISTINCT ON ({}) ", distincts),
        }
    }

    /// Строит внешний ORDER BY из полей `outer_order` каждой таблицы.
    fn construct_outer_order(&self) -> String {
        let orders = self
            .tables
            .iter()
            .flat_map(|x| {
                x.outer_order
                    .iter()
                    .map(|f| format!("{t}.{o}", t = x.table, o = f))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .join(",");
        match orders.is_empty() {
            true => String::with_capacity(0),
            false => format!(" ORDER BY {}", orders),
        }
    }

    /// Финализирует построитель: собирает итоговую SQL-строку и список `Select`
    /// для привязки переменных. Результат можно исполнить через `FinalJoinedSelect::get`.
    pub fn construct_query(self) -> FinalJoinedSelect {
        let fields = self.construct_fields();
        let distincts = self.construct_outer_distincts();

        let mut query_string =
            format!("SELECT {d}{fields} FROM", d = distincts, fields = fields);
        for x in self.tables.iter() {
            query_string.push_str(&x.query_string);
        }

        let group_by_clause = self.construct_group_by();
        query_string.push_str(&group_by_clause);

        let outer_order = self.construct_outer_order();
        query_string.push_str(&outer_order);

        let source = self.tables.into_iter().map(|x| x.select).collect::<Vec<_>>();

        FinalJoinedSelect {
            query_string,
            source,
        }
    }
}

/// Финальный JOIN-запрос, готовый к исполнению.
///
/// Используется для инспекции SQL строки перед выполнением, если нужна отладка.
/// В обычных случаях используйте `Selector::get` напрямую.
#[derive(Debug)]
pub struct FinalJoinedSelect {
    query_string: String,
    source: Vec<Select>,
}

impl FinalJoinedSelect {
    /// Выполняет запрос и десериализует результаты в тип `J`.
    pub async fn get<'a, J, Ex>(self, pool: Ex) -> Result<Vec<J>>
    where
        J: for<'d> sqlx::FromRow<'d, PgRow> + Send + Unpin,
        Ex: Executor<'a, Database = Postgres>,
    {
        let mut query = sqlx::query(&self.query_string);
        for x in self.source.iter() {
            query = x.bind_vars_to_query(query);
        }
        query
            .try_map(|x| J::from_row(&x))
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }
}

/// Извлекает значение нужного типа из строки результата JOIN.
///
/// Три формы:
/// - `left` -- `Option<T>` (LEFT JOIN, таблица может отсутствовать)
/// - `aggr` -- `Vec<T>` (AGG JOIN, массив агрегированных строк)
/// - без суффикса -- `T` (INNER JOIN, таблица всегда присутствует)
#[macro_export]
macro_rules! convert_row {
    ($row:expr, $subt:ty, left) => {
        $row.try_get(<$subt>::TABLE).map(|x: Option<$subt>| x)?
    };
    ($row:expr, $subt:ty, aggr) => {
        $row.try_get_unchecked(<$subt>::TABLE).map(|x: Vec<Option<$subt>>| {
            x.into_iter().filter_map(|x| x).collect::<Vec<_>>()
        })?
    };
    ($row:expr, $subt:ty, ) => {
        $row.try_get(<$subt>::TABLE).map(|x: $subt| x)?
    };
}

/// Определяет Rust-тип поля результата JOIN в зависимости от вида соединения.
///
/// Принимает тип таблицы и опциональный суффикс вида соединения:
/// - `($t, left)` → `Option<$t>` (LEFT JOIN — присоединённой строки может не быть)
/// - `($t, aggr)` → `Vec<$t>`    (AGG JOIN — массив агрегированных строк)
/// - `($t,)`      → `$t`         (INNER JOIN)
///
/// Сигнатура с двумя аргументами обязательна: call-сайт в `joined!` передаёт
/// `$crate::join_type!($subt, $($kind)?)`, где `$kind` опционален.
#[macro_export]
macro_rules! join_type {
    ($tpe:ty, left) => { Option<$tpe> };
    ($tpe:ty, aggr) => { Vec<$tpe> };
    ($tpe:ty $(,)?) => { $tpe };
}

/// Возвращает маркерный тип [`JoinKind`] по суффиксу вида соединения.
#[macro_export]
macro_rules! join_kind {
    (left) => {
        $crate::db_item::joined::LeftJoin
    };
    (aggr) => {
        $crate::db_item::joined::AggrJoin
    };
    () => {
        $crate::db_item::joined::NormalJoin
    };
}

#[macro_export]
macro_rules! make_type {
    (pub type $name:ident = $stream:tt) => {
        #[allow(dead_code)]
        pub type $name = $stream;
    };
    (pub type  = $stream:tt) => {};
}

/// Реализует трейт [`JoinTo`] для пары таблиц.
///
/// Параметры:
/// - `$main_tpe` -- тип главной таблицы
/// - `$main_default_key` -- ключ главной таблицы (по умолчанию)
/// - `$second_tpe` -- тип присоединяемой таблицы
/// - `$sec_default_key` -- ключ присоединяемой таблицы (по умолчанию)
/// - `$kind` -- `left` | `aggr` | пусто (inner)
///
/// Вызывается отдельно от `joined!`, так как реализует трейт и может вызываться
/// несколько раз для одной пары (разные виды JOIN).
#[macro_export]
macro_rules! impl_join_on {
    ($main_tpe:ty:$main_default_key:ident => $second_tpe:ty:$sec_default_key:ident $(,$kind:tt)?) => {
        impl $crate::db_item::joined::JoinTo<$main_tpe, $crate::join_kind!($($kind)?)>
            for $second_tpe
        {
            const DEFAULT_PKEY: &'static str = stringify!($main_default_key);
            const DEFAULT_FKEY: &'static str = stringify!($sec_default_key);
        }
    };
}

/// Генерирует типизированную структуру результата и селектор для JOIN нескольких таблиц.
///
/// Создаёт:
/// - `Joined{Main}{Sub1}{Sub2}` -- структура с полями по одному на каждую таблицу.
/// - `Joined{Main}{Sub1}{Sub2}Selector` -- построитель запроса с методами
///   `new(select)`, `set_{sub}(...)` для каждой присоединяемой таблицы,
///   `get(&pool)` для выполнения запроса.
///
/// Пример: объединение `Agenda`, `AgendaItem` (aggr) и `AgendaResult` (left):
/// ```ignore
/// impl_join_on!(Agenda:uuid => AgendaItem:agenda_uuid, aggr);
/// impl_join_on!(Agenda:uuid => AgendaResult:agenda_uuid, left);
/// joined!(
///     agenda: Agenda,
///     agenda_items: AgendaItem[Agenda => AgendaItem, aggr],
///     agenda_result: AgendaResult[Agenda => AgendaResult, left],
/// );
/// // Использование:
/// let results = JoinedAgendaAgendaItemAgendaResultSelector::new(my_select)
///     .get(&pool).await?;
/// ```
#[macro_export]
macro_rules! joined {
    ($(!$name:ty,)? $main:ident:$tpe:ty $(,$sub:ident:$subt:ty[$type_a:ty => $type_b:ty $(,$kind:tt)?])+ $(,)?) => {
    $crate::paste::paste!{
        $crate::make_type!(pub type $($name)? = [<Joined $tpe $($subt)+>]);
        $crate::make_type!(pub type $([<$name Selector>])? = [<Joined $tpe $($subt)+ Selector>]);

        #[derive(Debug, Clone, PartialEq)]
        /// Автоматически сгенерированная структура -- результат JOIN нескольких таблиц.
        pub struct [<Joined $tpe $($subt)+>] {
            pub $main: $tpe,
            $(pub $sub: $crate::join_type!($subt, $($kind)?),)+
        }

        /// Построитель запроса для JOIN. Позволяет задать отдельные селекты
        /// для каждой из присоединяемых таблиц через методы `set_{sub}`.
        pub struct [<Joined $tpe $($subt)+ Selector>] {
            distinct: bool,
            order: Vec<$crate::db_item::selection::FieldSortOrder>,
            $main: $crate::db_item::Select,
            $($sub: $crate::db_item::joined::JoinedSelectData<$type_b, $type_a, $crate::join_kind!($($kind)?)>,)+
        }

        /// По умолчанию выбирает все поля всех таблиц без фильтров.
        impl Default for [<Joined $tpe $($subt)+ Selector>] {
            fn default() -> Self {
                use $crate::db_item::joined::JoinTo;
                Self {
                    distinct: false,
                    order: vec![],
                    $main: $crate::db_item::Select::full::<$tpe>(),
                    $($sub: <$type_b as JoinTo<$type_a, $crate::join_kind!($($kind)?)>>::join_default(),)+
                }
            }
        }

        impl [<Joined $tpe $($subt)+ Selector>] {
            #[allow(dead_code)]
            pub fn new(select: $crate::db_item::Select) -> Self {
                let mut new = Self::default();
                new.$main = select;
                new
            }

            /// Переносит ORDER BY из входящего [`Select`] во внешние ордеринги
            /// джойн-селекта, очищая список сортировок у основного select'а.
            #[allow(dead_code)]
            pub fn new_with_order(mut select: $crate::db_item::Select) -> Self {
                let mut new = Self::default();
                new.order = std::mem::take(&mut select.order_list);
                new.$main = select;
                new
            }

            #[allow(dead_code)]
            pub fn add_order_desc(self, f: &str) -> Self {
                self.add_order(f, $crate::db_item::selection::FieldSortKind::Desc)
            }

            #[allow(dead_code)]
            pub fn add_order_asc(self, f: &str) -> Self {
                self.add_order(f, $crate::db_item::selection::FieldSortKind::Asc)
            }

            #[allow(dead_code)]
            pub fn add_order(mut self, f: &str, order: $crate::db_item::selection::FieldSortKind) -> Self {
                self.order.push($crate::db_item::selection::FieldSortOrder {
                    field: f.to_string(),
                    order,
                    null_position: None,
                });
                self
            }

            #[allow(dead_code)]
            pub fn distinct(mut self) -> Self {
                self.distinct = true;
                self
            }

            $(
                #[allow(dead_code)]
                pub fn [<set_ $sub>](
                    mut self,
                    select: $crate::db_item::joined::JoinedSelectData<$type_b, $type_a, $crate::join_kind!($($kind)?)>
                ) -> Self {
                    self.$sub = select;
                    self
                }
            )+

            pub async fn get<'a, Ex>(
                self,
                pool: Ex
            ) -> $crate::result::Result<Vec<[<Joined $tpe $($subt)+>]>>
            where
                Ex: sqlx::Executor<'a, Database = sqlx::Postgres>, {

                    self.finalise()
                        .await?
                        .get(pool)
                        .await
                }

            /// Финализирует построитель без выполнения запроса.
            ///
            /// Полезно для отладки или когда нужен объект `FinalJoinedSelect`
            /// для дальнейшей обработки перед вызовом `get`.
            pub async fn finalise(self) -> $crate::result::Result<$crate::db_item::joined::FinalJoinedSelect> {
                let r = $crate::db_item::joined::JoinedSelect::initiate::<$tpe>(
                    self.$main,
                    self.distinct,
                    self.order
                )
                    .await?
                    $(.join(self.$sub).await?)+
                    .construct_query();
                Ok(r)
            }
        }

        impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for [<Joined $tpe $($subt)+>] {
            fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
                use $crate::sqlx::Row;
                Ok(Self {
                    $main: $crate::convert_row!(row, $tpe,),
                    $($sub: $crate::convert_row!(row, $subt, $($kind)?),)+
                })
            }
        }
    }}
}
