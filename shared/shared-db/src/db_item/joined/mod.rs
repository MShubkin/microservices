//! The `joined` module is responsible for making selects where join is enacted.
//!
//! This is needed to effectively select items from multiple tables without
//! resorting to a lot of boiler plate. This approach is of limited flexibility
//! but should suffice for our initial needs. In more exotic cases we can use
//! raw sql.
//!
//! For a start we use one table as a main table and left join other tables onto
//! other tables that refer to it, or that it refers to.
use super::selection::{FieldSortKind, FieldSortOrder, NullPosition, SelectMaker};
use super::Select;
use crate::result::{Result, SharedDbError};
use crate::DbItem;

use sqlx::postgres::PgRow;
use sqlx::{Executor, Postgres};
use std::marker::PhantomData;

#[cfg(test)]
mod tests;

#[derive(Debug, Default, Clone, Copy)]
/// represents the creation of an aggregate join.
pub struct AggrJoin;

#[derive(Debug, Default, Clone, Copy)]
/// Represents the creation of a left join
pub struct LeftJoin;

#[derive(Debug, Default, Clone, Copy)]
/// Represents the creation of a normal inner join.
pub struct NormalJoin;

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
/// The trait bound is for the OTHER table you are joining to.
/// NB: The extra trait bound here allows us to implement `JoinTo` multiple
/// times for a single type, easch time using different tables.
/// eg, `impl JoinTo<Agenda> for AgendaItem {}`,
///     `impl JoinTo<RelAgendaProtocolItem> for AgendaItem {}`.
pub trait JoinTo<ForeignTable: DbItem, Join: JoinKind>: DbItem {
    /// This gives us the default foreign key. There is no compile time check for
    /// correctness here, although there are run time checks.
    const DEFAULT_FKEY: &'static str;
    const DEFAULT_PKEY: &'static str;

    /// Essentially all this does is to carry out a series of checks as to
    /// whether the two tables can be joined and that all criteria are fulfilled.
    ///
    /// TODO: Decide whether we need this check at all. Reason: Whether the check fails or not
    /// we will still fail at runtime if the join is "bad". However if we do not have a check
    /// we fail across the DB-boundary.
    fn check_joinable_select(
        select: &Select,
        own_key: &str,
        foreign_table_key: &str,
    ) -> Result<()> {
        // We perform a series of sanity checks before making a select statement
        // otherwise we risk ending up with a joined select that makes no sense,
        // eg, having a selection of fields that does not include its own key.
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

    /// Creates a default joined select. By default:
    /// - The whole table is selected with no filters.
    /// - The join will join this type's default foreign key
    ///   onto the specified key. This can be changed with
    ///
    /// `own_key` can be changed using `JoinSelectData::eq_own` and a select
    /// can be changed using `JoinSelectData::selecting`.
    fn join_on(f: &str) -> JoinedSelectData<Self, ForeignTable, Join> {
        let mut j = Self::join_default();
        j.foreign_table_key = f.to_string();
        j
    }

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
    /// Determines what foreign key the table is being joined onto.
    pub fn eq_own(mut self, own_field: &str) -> Self {
        self.own_key = own_field.to_string();
        self
    }

    /// Finalise the JoinDataBuilder. It should be noted, that the field selection is
    /// never inherited here and all fields specified by default are always inherited.
    /// Conversely, filters, chunking and ordering are inherited.
    pub fn selecting(mut self, select: Select) -> JoinedSelectData<T, O, K> {
        self.select.filter_list = select.filter_list;
        self.select.order_list = select.order_list;
        self.select.distinct_on = select.distinct_on;
        self
    }

    /// Adds distinct clause for that table.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Добавляет внешний набор сортировок. Это может вызвать проблемы в сочетании с
    /// предложением outer distinct.
    /// Ничего не дает, если таблица AGGR джойнится.
    ///
    /// Возвращает ошибку, если какое поле не принадлежит таблице
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

    /// Определение порядка NULL значений при сортировке по всем полям во
    /// внешем ORDER BY
    pub fn with_null_position(mut self, pos: NullPosition) -> Self {
        self.outer_order.iter_mut().for_each(|o| o.null_position = Some(pos));
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
}

impl<T, O> JoinedSelectData<T, O, AggrJoin>
where
    T: JoinTo<O, AggrJoin>,
    O: DbItem,
{
    /// If the joined table is aggregated, it will be internally ordered with
    /// `agg_array(table ORDER_BY field)`
    /// This function is only available for aggregate type joins.
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

    /// Adds `DISTINCT` to the `array_agg` to remove duplicates from the array.
    ///
    /// NB. It seems that this requires inner select to have `ORDER BY` clause,
    /// and it does not work with `[Self::order_aggr_by()]`.
    pub fn distinct_aggr(mut self, value: bool) -> Self {
        self.distinct_aggr = value;
        self
    }
}

/// This exists to store all the instructions to make a joined select.
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

/// A history fragment records a semi-completed fragment of the joined select.
/// it needs to store the query string in order to have the parts of the query
/// that relate to the table. (since it cannot store type information including
/// constants such as T::TABLE or T::FIELDS.
/// In addition it must store the actual select as the variables which are bound
/// to each individual select are stored there.
struct HistoryFragment {
    // The select must be stored as it holds the vars to bind to the query.
    select: Select,
    /// The table must be stored as it is used when finalising the joined query.
    table: &'static str,
    /// Aggregate queries are treated with group by.
    is_aggregate: bool,
    /// Query string is extracted from the select maker.
    query_string: String,
    distinct_table: bool,
    ordered_aggr: Vec<FieldSortOrder>,
    distinct_aggr: bool,
    outer_order: Vec<FieldSortOrder>,
}

pub struct JoinedSelect {
    /// Table name and whether it is aggregate or not.
    tables: Vec<HistoryFragment>,
    /// The total number of binds thus far.
    binds: usize,
}

impl JoinedSelect {
    pub async fn initiate<T: DbItem>(
        mut select: Select,
        distinct: bool,
        outer_order: Vec<FieldSortOrder>,
    ) -> Result<JoinedSelect> {
        // Data internally tests our select for consistency: Most importantly it
        // disallows abuse of fields (currently our joins need all fields of a structure)
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

    pub async fn join<T, O, K>(
        mut self,
        select_box: JoinedSelectData<T, O, K>,
    ) -> Result<Self>
    where
        T: JoinTo<O, K>,
        O: DbItem,
        K: JoinKind,
    {
        // The select is checked at this level rather than the level of `JoinedSelectData`
        // as the default selection box from `join_on` may in fact not be entirely
        // correct.
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

    fn construct_group_by(&self) -> String {
        // We must group by all fields that are not aggregate.
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

    fn construct_outer_order(&self) -> String {
        let orders = self
            .tables
            .iter()
            .flat_map(|x| {
                x.outer_order
                    .iter()
                    // Notation is strange but works see impl Display for FieldSortOrder
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

    /// Finalise the joined select, leaving only table names and to bind and
    /// and the query string.
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

/// Represents a final joined select with the sql query and the source
/// Selects.
#[derive(Debug)]
pub struct FinalJoinedSelect {
    query_string: String,
    source: Vec<Select>,
}

impl FinalJoinedSelect {
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

#[macro_export]
// Convert rows correctly for the three join types
macro_rules! convert_row {
    // Left join.
    ($row:expr, $subt:ty, left) => {
        $row.try_get(<$subt>::TABLE).map(|x: Option<$subt>| x)?
    };
    // Aggregate join
    ($row:expr, $subt:ty, aggr) => {
        $row.try_get_unchecked(<$subt>::TABLE).map(|x: Vec<Option<$subt>>| {
            x.into_iter().filter_map(|x| x).collect::<Vec<_>>()
        })?
    };
    // Normal join
    ($row:expr, $subt:ty, ) => {
        $row.try_get(<$subt>::TABLE).map(|x: $subt| x)?
    };
}

#[macro_export]
macro_rules! join_type {
    // Left join.
    ($tpe:ty, left) => { Option<$tpe> };
    // Aggregate join
    ($tpe:ty, aggr) => { Vec<$tpe> };
    // Normal join
    ($tpe:ty, ) => { $tpe };
}

#[macro_export]
macro_rules! join_kind {
    // Left join.
    (left) => {
        $crate::db_item::joined::LeftJoin
    };
    // Aggregate join
    (aggr) => {
        $crate::db_item::joined::AggrJoin
    };
    // Normal join
    () => {
        $crate::db_item::joined::NormalJoin
    };
}

#[macro_export]
macro_rules! make_type {
    // user name & generated name. -> user name
    (pub type $name:ident = $stream:tt) => {
        #[allow(dead_code)]
        pub type $name = $stream;
    };
    // Only the generated name. -> generated_name
    (pub type  = $stream:tt) => {};
}

/// This macro implements the `JoinOn` trait for two DbItems.
/// It is separate from the `joined` macro, as it implements a trait, and it is possible
/// that this will be implemented multiple times, which would cause problems.
/// It should be noted that without `joined`, this is of limited use.
///
/// *main_type*: The DbItem type of the main table being joined.
/// *second_tpe*: The DbItem type that it is being joined to.
/// *sec_default_key*: The key(field) of the `second_tpe` that is normally used.
/// *main_default_key*: The key(field) of the `main_tpe` that is normally used.
/// *kind*: The kind of join that is being formed permitted values are 'left' or 'agg. If left empty
/// an inner join will be formed.
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

/// This provides a more or less safe and sane interface for DbItems (tables) in simple
/// ways. It generates two structures `Joinedsomething` and `JoinedsomethingSelector`.
/// `JoinedsomethingSelector` provides an interface for making a selection within each join.
/// Thus if we are joining `Agenda`, `AgendaItem` (aggregate), AgendaResult (left), we
/// can generate:
/// ```ignore
/// Joinedagenda {
///     agenda: Agenda,
///     agenda_items: Vec<AgendaItem>,
///     agenda_result: Option<AgendaResult>,
/// }
/// ```
/// We also generate the selector, which gives us an interface such that we can
/// make a selection based on some agenda field, eg. id:
/// ```ignore
/// let selection = some_make_select_function();
/// let joined_agendas = JoinedagendaSelector::new(selection).get(&db_pool).await?;
/// ```
/// We can also make a more complex selection, such as.
/// ```ignore
/// let selection = some_make_select_function();
/// let res_selection = some_make_res_select_function();
/// let result_select: AgendaResult::join_default().selection(res_selection);
///
/// let joined_agendas = JoinedagendaSelector::new(selection)
///     .set_agenda_result(result_selector)
///     .get(&pg_pool)
///     .await?;
/// ```
/// The macro also requires `impl_joined_on` to be called for the combination. So in
/// this example we call:
/// ```ignore
/// impl_join_on!(Agenda:uuid => AgendaItems:agenda_uuid, aggr);
/// impl_join_on!(Agenda:uuid => AgendaResult:agenda_uuid, left);
/// joined!(
///     agenda: Agenda,
///     agenda_items: AgendaItem[Agenda => AgendaItems, aggr],
///     agenda_result: AgendaItem[Agenda => AgendaResult, left],
/// );
/// ```
/// (Таблицы и сущности к примеру. Могут не соответствовать действительности.)

#[macro_export]
macro_rules! joined {
    ($(!$name:ty,)? $main:ident:$tpe:ty $(,$sub:ident:$subt:ty[$type_a:ty => $type_b:ty $(,$kind:tt)?])+ $(,)?) => {
    $crate::paste::paste!{
        $crate::make_type!(pub type $($name)? = [<Joined $tpe $($subt)+>]);
        $crate::make_type!(pub type $([<$name Selector>])? = [<Joined $tpe $($subt)+ Selector>]);

        #[derive(Debug, Clone, PartialEq)]
        /// This is an automatically generated structure representing a join
        /// of several tables in accordance with set rules.
        /// Each table (or some of its fields) are represented by a single field,
        /// which can easily be accessed and extracted.
        pub struct [<Joined $tpe $($subt)+>] {
            pub $main: $tpe,
            $(pub $sub: $crate::join_type!($subt, $($kind)?),)+
        }

        /// This represents the process of joining together several tables in
        /// accordance with set rules.
        /// The fields are set, but the `set_` functions allow selections from
        /// specific fields to be set.
        pub struct [<Joined $tpe $($subt)+ Selector>] {
            distinct: bool,
            order: Vec<$crate::db_item::selection::FieldSortOrder>,
            $main: $crate::db_item::Select,
            $($sub: $crate::db_item::joined::JoinedSelectData<$type_b, $type_a, $crate::join_kind!($($kind)?)>,)+
        }

        /// By default select everything from the table.
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

            /// Добавляет все ордеринги из входящего [`Select`]
            /// во внешние ордеринги джойн-селекта
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

            /// This function is used to construct a `FinalJoinedSelect` that
            /// can be easily inspected. It can then be executed by calling `get`
            /// on the resulting `FinalJoinedSelect`.
            /// When no inspection is needed, use `get` directly.
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
