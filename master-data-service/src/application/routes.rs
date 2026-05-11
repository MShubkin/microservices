use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    num::{ParseIntError, TryFromIntError},
    sync::Arc,
};

use ahash::{AHashMap, AHashSet, AHasher};
use itertools::{Either, Itertools};
use sqlx::{Executor, PgPool, Postgres};
use thiserror::Error;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{joined::JoinTo, FieldTolerance, Filter, FilterTree, Select},
    result::SharedDbError,
};
use shared_essential::{
    domain::routes::{
        CritPredicate, CritValue, RouteApprType, RouteCrit, RouteData,
        RouteDataContent, RouteFull, RouteFullSelector, RouteHeader,
    },
    presentation::dto::{
        master_data::{
            error::{MasterDataError, MasterDataResult},
            request::CritArg,
        },
        response_request::{Message, Messages, ParamItem},
    },
};

use crate::presentation::dto::RouteFindItem;

use super::hierarchical_values::{
    get_hierarchical_values, HierarchicalValues, HierarchicalValuesError,
};

/// Контекст проверки критериев.
#[derive(Debug, Clone)]
pub(crate) struct EvalContext {
    /// Иерархии многоуровневых справочников, для `in_tree`.
    pub(crate) hierarchical_values: Arc<HierarchicalValues>,
}

impl EvalContext {
    pub(crate) async fn create() -> MasterDataResult<Self> {
        Ok(EvalContext::new(get_hierarchical_values().await?))
    }

    pub(crate) fn new(hierarchical_values: Arc<HierarchicalValues>) -> Self {
        EvalContext {
            hierarchical_values,
        }
    }

    pub(crate) fn eval_in_tree<'a>(
        &'a self,
        name: &'a str,
        roots: &[i32],
        value: i32,
    ) -> Result<bool, HierarchicalValuesError<'a>> {
        roots
            .iter()
            .map(|root| {
                Ok(value == *root
                    || self
                        .hierarchical_values
                        .is_in_subtree(name, *root, value)?)
            })
            .fold_ok(false, |res, eval| res || eval)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("невозможно сравнить значение {0:?} со значением {1:?}")]
pub struct ValueCmpError<'a>(&'a CritValue, &'a CritValue);

fn checked_cmp<'a>(
    left: &'a CritValue,
    right: &'a CritValue,
) -> Result<std::cmp::Ordering, ValueCmpError<'a>> {
    match (left, right) {
        (CritValue::Int(l), CritValue::Int(r)) => Ok(l.cmp(r)),
        (CritValue::Date(l), CritValue::Date(r)) => Ok(l.cmp(r)),
        (CritValue::Timestamp(l), CritValue::Timestamp(r)) => Ok(l.cmp(r)),
        (CritValue::Bool(l), CritValue::Bool(r)) => Ok(l.cmp(r)),
        (CritValue::String(l), CritValue::String(r)) => Ok(l.cmp(r)),
        (l, r) => Err(ValueCmpError(l, r)),
    }
}

macro_rules! crit_value_cmp {
    ($value:expr, $pred_value:expr, $($ord:path)|*) => (
        $value.as_ref().map_or(Ok(false), |value| {
            crit_value_cmp!(Some value, $pred_value, $($ord)|*)
        })
    );
    (Some $value:expr, $pred_value:expr, $($ord:path)|*) => (
        checked_cmp($value, $pred_value).map(|v| $(v == $ord)||*).map_err(CritPredicateError::Cmp)
    );
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CritPredicateError<'a> {
    #[error(transparent)]
    Cmp(ValueCmpError<'a>),
    #[error(transparent)]
    InTree(HierarchicalValuesError<'a>),
    #[error("ошибка преобразования значения для критерия in_tree")]
    InTreeValueConv(#[from] TryFromIntError),
    #[error("ошибка преобразования значения для критерия in_tree")]
    InTreeValueParse(#[from] ParseIntError),
    #[error("неподдерживаемое значение для критерия in_tree: {0:?}")]
    InTreeValue(&'a CritValue),
    #[error("Неизвестный предикат")]
    Unknown,
}

impl<'a> From<HierarchicalValuesError<'a>> for CritPredicateError<'a> {
    fn from(error: HierarchicalValuesError<'a>) -> Self {
        CritPredicateError::InTree(error)
    }
}

fn eval<'a>(
    value: &'a CritArg,
    predicate: &'a CritPredicate,
    ctx: &'a EvalContext,
) -> Result<bool, CritPredicateError<'a>> {
    match value {
        CritArg::Plain(value) => eval_plain(value.as_ref(), predicate, ctx),
        CritArg::AnyAgg(values) => values
            .iter()
            .map(|v| eval_plain(Some(v), predicate, ctx))
            .fold_ok(false, |v, r| v || r),
        CritArg::AllAgg(values) => values
            .iter()
            .map(|v| eval_plain(Some(v), predicate, ctx))
            .fold_ok(true, |v, r| v && r),
    }
}

fn eval_plain<'a>(
    value: Option<&'a CritValue>,
    predicate: &'a CritPredicate,
    ctx: &'a EvalContext,
) -> Result<bool, CritPredicateError<'a>> {
    let eval_plain = |predicate| eval_plain(value, predicate, ctx);
    match predicate {
        CritPredicate::Unknown => Err(CritPredicateError::Unknown),
        CritPredicate::Equal { value: v } => {
            crit_value_cmp!(value, v, Ordering::Equal)
        }
        CritPredicate::NotEqual { value: v } => {
            crit_value_cmp!(value, v, Ordering::Less | Ordering::Greater)
        }
        CritPredicate::Less { value: v } => {
            crit_value_cmp!(value, v, Ordering::Less)
        }
        CritPredicate::LessEqual { value: v } => {
            crit_value_cmp!(value, v, Ordering::Less | Ordering::Equal)
        }
        CritPredicate::Greater { value: v } => {
            crit_value_cmp!(value, v, Ordering::Greater)
        }
        CritPredicate::GreaterEqual { value: v } => {
            crit_value_cmp!(value, v, Ordering::Greater | Ordering::Equal)
        }
        CritPredicate::Between { low, high } => {
            value.as_ref().map_or(Ok(false), |value| {
                Ok(crit_value_cmp!(Some value, low, Ordering::Greater | Ordering::Equal)?
                    && crit_value_cmp!(Some value, high, Ordering::Less | Ordering::Equal)?)
            })
        }
        CritPredicate::Any => Ok(value.is_some()),
        CritPredicate::In { values } => {
            if let Some(value) = value {
                for v in values {
                    if crit_value_cmp!(Some value, v, Ordering::Equal)? {
                        return Ok(true)
                    }
                }
            }
            Ok(false)
        }
        CritPredicate::Not { predicate } => Ok(!eval_plain(predicate)?),
        CritPredicate::And { predicates } => predicates.iter().map(eval_plain).fold_ok(true, |v, r| v && r),
        CritPredicate::Or { predicates } => predicates.iter().map(eval_plain).fold_ok(false, |v, r| v || r),
        CritPredicate::InTree { dictionary, roots } => value.map_or(Ok(false), |value| eval_in_tree(dictionary, roots, value, ctx)),
        CritPredicate::None => Ok(true),
    }
}

/// Вычисляет предикат `in_tree(roots)` для значения `value`,
/// т.е. является ли `value` подзначением одного из `roots`.
fn eval_in_tree<'a>(
    name: &'a str,
    roots: &[i32],
    value: &'a CritValue,
    ctx: &'a EvalContext,
) -> Result<bool, CritPredicateError<'a>> {
    let value: i32 = match value {
        CritValue::Int(value) => (*value).try_into()?,
        CritValue::String(value) => value.parse()?,
        _ => return Err(CritPredicateError::InTreeValue(value)),
    };
    Ok(ctx.eval_in_tree(name, roots, value)?)
}

pub(crate) async fn fetch_routes(
    select: Select,
    pool: &PgPool,
) -> MasterDataResult<Vec<RouteFull>> {
    let mut select = select.eq(RouteHeader::is_removed, false);
    // TODO: apply tolerance in join
    RouteHeader::apply_tolerance_to_select(&mut select);

    let data_select = Select::full::<RouteData>();
    let crit_select = Select::full::<RouteCrit>()
        .eq(RouteCrit::is_removed, false)
        .add_replace_order_asc(RouteCrit::route_uuid)
        .add_replace_order_asc(RouteCrit::field_name)
        .distinct_on(&[RouteCrit::route_uuid, RouteCrit::field_name]);

    let route_selector = RouteFullSelector::new_with_order(select)
        .set_data(RouteData::join_default().selecting(data_select))
        .set_crits(
            RouteCrit::join_default().selecting(crit_select).distinct_aggr(true),
        );

    let joined_routes = route_selector.get(pool).await?;

    Ok(joined_routes)
}

pub(crate) fn route_predicate<'a>(
    item_fields: &'a RouteFindItem,
    ctx: EvalContext,
    messages: &'a mut Messages,
) -> impl 'a + FnMut(&RouteFull) -> bool {
    move |route| route_matches(route, item_fields, &ctx, messages)
}

fn route_matches(
    route: &RouteFull,
    item_fields: &RouteFindItem,
    ctx: &EvalContext,
    messages: &mut Messages,
) -> bool {
    tracing::debug!(kind = "route_crit", route = %route.route.id, "matching route");
    route.crits.iter().all(|crit| {
        tracing::debug!(kind = "route_crit", field = %crit.field_name, predicate = ?crit.predicate, "starting criterion");
        item_fields
            .get(&crit.field_name)
            .map_or(true, |value| criterion_matches(crit, value, ctx, messages))
    })
}

fn criterion_matches(
    crit: &RouteCrit,
    value: &CritArg,
    ctx: &EvalContext,
    messages: &mut Messages,
) -> bool {
    tracing::debug!(kind = "route_crit", field = %crit.field_name, value = ?value, pred = ?crit.predicate, "matching criterion");
    let res = eval(value, &crit.predicate, ctx);
    tracing::debug!(kind = "route_crit", result = ?res, "evaluation result");
    match res {
        Ok(v) => v,
        Err(error) => {
            tracing::warn!(field = %crit.field_name, %error, "Ошибка приминения критерия");
            messages.add_prepared_message(Message::warn(format!(
                "Ошибка приминения критерия {field}: {error}",
                field = crit.field_name
            )));
            false
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum RouteDupCheckError {
    #[error("Невалидные данные по маршруту {route_id}. Ожидается маршрут {expected}, найден маршрут {found}")]
    InvalidRouteData {
        route_id: i64,
        expected: RouteApprType,
        found: RouteApprType,
    },
    #[error("Не найдены данные по маршруту {route_id}")]
    NotFoundRouteData { route_id: i64 },
    #[error(transparent)]
    CannotGetDupRoutes(#[from] SharedDbError),
}

impl From<RouteDupCheckError> for MasterDataError {
    fn from(val: RouteDupCheckError) -> MasterDataError {
        MasterDataError::RouteError(val.to_string())
    }
}

/// Существует для сравнивания маршрутов.
/// Если же все поля совпадают, то маршруты считаются равными
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RouteDuplicateCheck(u64);

#[derive(Hash)]
struct CritDuplicateCheck<'a> {
    field_name: &'a str,
    predicate: &'a CritPredicate,
}

impl RouteDuplicateCheck {
    fn new<'c, C, Div, Dep>(crits: C, divisions: Div, departments: Dep) -> Self
    where
        C: IntoIterator<Item = &'c RouteCrit>,
        Div: IntoIterator<Item = i32>,
        Dep: IntoIterator<Item = i32>,
    {
        let mut hasher = AHasher::default();

        crits.into_iter().sorted_by_key(|c| &c.field_name).for_each(|c| {
            let mut predicate = c.predicate.clone();
            predicate.sort_values();

            CritDuplicateCheck {
                field_name: &c.field_name,
                predicate: &predicate,
            }
            .hash(&mut hasher);
        });
        departments.into_iter().sorted().for_each(|d| d.hash(&mut hasher));
        divisions.into_iter().sorted().for_each(|d| d.hash(&mut hasher));

        Self(hasher.finish())
    }
}

/// Дополнительная проверка на совпадающие маршруты ПД
///
/// Принимает
/// `to_check_routes` - Маршруты, которые требуется проверить
/// `check_only_inactive` - При true будут проверены только неактивные `to_check_routes`,
/// иначе же будут проверены только активные `to_check_routes`
///
/// Возвращает: (Маршруты без дупликатов, Маршруты с дупликатами, Дупликаты)
pub(crate) async fn check_duplicated_routes<
    'a,
    E: Executor<'a, Database = Postgres>,
    I: IntoIterator<Item = Uuid> + Clone,
>(
    to_check_routes: I,
    check_only_inactive: bool,
    tx: E,
) -> Result<
    (Vec<RouteHeader>, Vec<RouteHeader>, Vec<CoincidentRoute>),
    RouteDupCheckError,
> {
    let to_check_routes = to_check_routes.into_iter().collect::<AHashSet<_>>();

    let mut to_check = Vec::with_capacity(to_check_routes.len());
    let mut full_map = AHashMap::new();

    let routes = get_dup_check_routes(to_check_routes.iter().copied(), tx).await?;
    // Построить хэш-карту "условий" против ИД-ишников маршрутов которые
    // эти условия удовлетворяют.
    // В то же самое время перестраиваем список маршрутов которые нам надо
    // активировать так как надо прикрепить условия.
    for route in routes {
        let header = route.route;

        let route_data = match route.data.data.as_ref() {
            Some(RouteDataContent::AssignDepartment(data)) => data,
            Some(RouteDataContent::AssignExpert(_)) => {
                return Err(RouteDupCheckError::InvalidRouteData {
                    route_id: header.id,
                    expected: RouteApprType::SpecializedDepartments,
                    found: RouteApprType::PriceAnalysis,
                })
            }
            _ => {
                return Err(RouteDupCheckError::NotFoundRouteData {
                    route_id: header.id,
                })
            }
        };

        let duplicate_check = RouteDuplicateCheck::new(
            &route.crits,
            route_data
                .iter()
                .filter_map(|d| d.division.as_ref().map(|division| division.id)),
            route_data.iter().map(|d| d.department_id),
        );

        full_map
            .entry(duplicate_check)
            .or_insert(AHashSet::new())
            .insert(header.id);

        match (header.is_active, check_only_inactive) {
            // Так как в `get_dup_check_routes` мы получаем is_active=true OR uuid IN to_activate_routes,
            // то проверяем тут мы только неактивные to_activate_routes маршруты
            (false, true) => to_check.push((header, duplicate_check)),
            (true, false) if to_check_routes.contains(&header.uuid) => {
                to_check.push((header, duplicate_check))
            }
            _ => {}
        }
    }

    let mut coincident_ids = Vec::new();
    // Маршруты для которых есть
    let (checked_routes, duplicate_routes): (_, Vec<_>) =
        to_check.into_iter().partition_map(|(r, criteria)| {
            match full_map.get(&criteria) {
                // If we only have one similar record, it is ourselves.
                // If we have more than one similar record, then we have at
                // least one genuine similar record which is not ours.
                Some(ids) if ids.len() > 1 && ids.contains(&r.id) => {
                    let mut dup: Vec<_> =
                        ids.iter().filter(|id| **id != r.id).copied().collect();
                    // to guarantee orderer of parameters in messages.
                    dup.sort_unstable();
                    coincident_ids.push(CoincidentRoute::new(r.id, dup, r.type_id));
                    Either::Right(r)
                }
                _ => Either::Left(r),
            }
        });

    Ok((checked_routes, duplicate_routes, coincident_ids))
}

async fn get_dup_check_routes<
    'a,
    E: Executor<'a, Database = Postgres>,
    I: IntoIterator<Item = Uuid>,
>(
    to_activate_routes: I,
    tx: E,
) -> Result<Vec<RouteFull>, SharedDbError> {
    // проводим выборку всех маршрутов таблицы route_list у которых
    // route_type_id = 1 (ПД), is_active= true и is_removed = false
    // список сравниваемых маршрутов включаются уже ранее запущенные маршруты
    // + маршруты, которые находятся в списке на запуск
    let dept_filter =
        Filter::eq(RouteHeader::type_id, RouteApprType::SpecializedDepartments)
            .into();
    let is_removed_filter = Filter::eq(RouteHeader::is_removed, false).into();

    let is_active_filter = Filter::eq(RouteHeader::is_active, true);
    let existing_ids_filter = Filter::in_any(RouteHeader::uuid, to_activate_routes);
    let or_filter =
        FilterTree::or_from_list([is_active_filter, existing_ids_filter]);

    // Final filter is `WHERE type_id=1 AND is_removed=false AND (is_active=true OR uuid IN([active_routes]))`
    let header_filter =
        FilterTree::and_from_list([dept_filter, is_removed_filter, or_filter]);

    let crit_selection =
        Select::full::<RouteCrit>().eq(RouteHeader::is_removed, false);
    let route_data_selection =
        Select::with_fields([RouteData::route_uuid, RouteData::data]);
    let route_selection =
        Select::full::<RouteHeader>().set_filter_tree(header_filter);

    // name, low, option_id должны полностью совпадать по всем критериям, чтобы
    // маршрут считался одинаковым.
    RouteFullSelector::new(route_selection)
        .set_crits(RouteCrit::join_default().selecting(crit_selection))
        .set_data(RouteData::join_default().selecting(route_data_selection))
        .get(tx)
        .await
        .map_err(Into::into)
}

#[derive(Debug, Clone)]
pub(crate) struct CoincidentRoute {
    pub own: i64,
    /// Теоретический может совпадать только один маршрут. Теоретический.
    pub others: Vec<i64>,
    /// Тип маршрута.
    pub type_id: RouteApprType,
}

impl CoincidentRoute {
    pub(crate) fn new(own: i64, others: Vec<i64>, type_id: RouteApprType) -> Self {
        Self {
            own,
            others,
            type_id,
        }
    }
    /// Кто то придумал так делать:
    /// "​<системный номер запускаемого маршрута1> - <системный номер совпавшего маршрута1>"
    fn to_params(&self) -> impl Iterator<Item = ParamItem> + '_ {
        let own = self.own;
        self.others.iter().map(move |other| {
            // Здесь не указывается id, вся информация для пользователя в text
            ParamItem::from_id(String::new())
                .with_type(self.type_id.into())
                .with_text(format!("{own} - {other}"))
        })
    }
}

pub(crate) fn params_from_routes(routes: &[CoincidentRoute]) -> Vec<ParamItem> {
    routes.iter().flat_map(CoincidentRoute::to_params).collect()
}

pub(crate) struct RoutesNotFound {
    uuids: AHashSet<Uuid>,
}

impl FromIterator<Uuid> for RoutesNotFound {
    fn from_iter<T: IntoIterator<Item = Uuid>>(iter: T) -> Self {
        RoutesNotFound {
            uuids: iter.into_iter().collect(),
        }
    }
}

impl RoutesNotFound {
    pub(crate) fn found(&mut self, uuid: &Uuid) {
        self.uuids.remove(uuid);
    }
    pub(crate) fn into_message(self) -> Message {
        Message::error(format!(
            "Маршрут(ы) {} не найден(ы)",
            self.uuids.into_iter().sorted().join(", ")
        ))
    }
    pub(crate) fn append(self, messages: &mut Messages) {
        if self.uuids.is_empty() {
            return;
        }
        messages.add_prepared_message(self.into_message());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use asez2_shared_db::db_item::AsezDate;
    use shared_essential::domain::routes::{CritPredicate, CritValue};

    use crate::application::hierarchical_values::HierarchyData;

    use super::eval_plain;

    fn hierarchy_data(parents: Vec<i32>) -> HierarchyData {
        HierarchyData {
            parents,
            code: "".into(),
            is_removed: false,
            from_date: AsezDate::today(),
            to_date: AsezDate::today(),
        }
    }

    #[test]
    fn in_tree_match() {
        let hierarchy = [(100, vec![10, 1])]
            .into_iter()
            .map(|(id, parents)| (id, hierarchy_data(parents)))
            .collect();
        let ctx = super::EvalContext {
            hierarchical_values: Arc::new(
                [("name", hierarchy)].into_iter().collect(),
            ),
        };

        let value = CritValue::Int(100);
        let predicate = CritPredicate::InTree {
            dictionary: "name".to_string(),
            roots: vec![10, 20],
        };
        let res = eval_plain(Some(&value), &predicate, &ctx);

        assert!(res.is_ok(), "{res:?}");
        assert!(res.unwrap());
    }

    #[test]
    fn in_tree_same() {
        let hierarchy = [(100, vec![10, 1]), (10, vec![1]), (30, vec![1])]
            .into_iter()
            .map(|(id, parents)| (id, hierarchy_data(parents)))
            .collect();
        let ctx = super::EvalContext {
            hierarchical_values: Arc::new(
                [("name", hierarchy)].into_iter().collect(),
            ),
        };

        let value = CritValue::Int(10);
        let predicate = CritPredicate::InTree {
            dictionary: "name".to_string(),
            roots: vec![10, 30],
        };
        let res = eval_plain(Some(&value), &predicate, &ctx);

        assert!(res.is_ok(), "{res:?}");
        assert!(res.unwrap());
    }

    #[test]
    fn in_tree_mismatch() {
        let hierarchy = [(100, vec![10, 1, 0]), (200, vec![20, 2, 0])]
            .into_iter()
            .map(|(id, parents)| (id, hierarchy_data(parents)))
            .collect();
        let ctx = super::EvalContext {
            hierarchical_values: Arc::new(
                [("name", hierarchy)].into_iter().collect(),
            ),
        };

        let value = CritValue::Int(100);
        let predicate = CritPredicate::InTree {
            dictionary: "name".to_string(),
            roots: vec![20, 40],
        };
        let res = eval_plain(Some(&value), &predicate, &ctx);

        assert!(res.is_ok());
        assert!(!res.unwrap());
    }
}
