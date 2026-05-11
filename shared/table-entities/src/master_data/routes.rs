use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::Type;
use uuid::Uuid;

use asez2_shared_db::{db_item::*, Value};
use asez2_shared_db::{impl_join_on, joined, DbAdaptor, DbItem};
use shared_db_derive::DbEnum;

use crate::DepartmentLevel;

impl_join_on!(RouteHeader:uuid => RouteData:route_uuid);
impl_join_on!(RouteHeader:uuid => RouteCrit:route_uuid, aggr);

joined!(
    !RouteWithCrits,
    route: RouteHeader,
    crits: RouteCrit[RouteHeader => RouteCrit, aggr],
);
joined!(
    !RouteFull,
    route: RouteHeader,
    crits: RouteCrit[RouteHeader => RouteCrit, aggr],
    data: RouteData[RouteHeader => RouteData],
);

/// Маршрут согласования
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Route {
    /// Заголовок маршрута согласования
    pub route_header: RouteHeader,
    /// Критерии маршрута согласования
    pub route_crit: Vec<RouteCrit>,
    /// Пользователи и периоды действия их записей для маршрута согласования
    pub route_users: Vec<RouteUsers>,
}

/// Перечень маршрутов согласования
#[derive(
    Debug,
    Default,
    Clone,
    DbItem,
    DbItemExt,
    DbAdaptor,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "route_list"]
#[item_skip_field_tolerance]
pub struct RouteHeader {
    /// Типы маршрута
    pub type_id: RouteApprType,
    /// Идентификатор записи в таблице
    #[item_field_pkey]
    #[item_field_activate_with = "Uuid::new_v4()"]
    pub uuid: Uuid,
    /// Номер маршрута
    #[item_field_autogen_always]
    #[adaptor_rename = "route_id"]
    pub id: i64,
    /// Краткая наименование маршрута
    pub name_short: Option<String>,
    /// Исключения
    pub is_exception: bool,
    /// Признак удаления записи
    pub is_removed: bool,
    /// Статус записи
    pub is_active: bool,
    /// Создано
    pub created_at: AsezTimestamp,
    /// Автор создания
    pub created_by: i32,
    /// Изменено
    pub changed_at: AsezTimestamp,
    /// Автор изменения
    pub changed_by: i32,
}

impl DbUpsert for RouteHeader {}

impl FieldTolerance for RouteHeader {
    const TOLERATED: &'static [(&'static str, &'static str)] =
        &[("route_id", RouteHeader::id)];
}

/// Критерии маршрутов согласования
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "route_crit"]
pub struct RouteCrit {
    /// Уникальный идентификатор записи (маршрута)
    #[item_field_pkey]
    pub route_uuid: Uuid,
    /// Имя поля
    #[item_field_pkey]
    pub field_name: String,
    /// Предикат критерия
    pub predicate: Json<CritPredicate>,
    /// Статус записи
    pub is_removed: bool,
    /// Дата/время создания записи
    pub created_at: AsezTimestamp,
    /// Автор записи
    pub created_by: i32,
    /// Дата/время изменения записи
    pub changed_at: AsezTimestamp,
    /// Автор изменения
    pub changed_by: i32,
}

impl DbItemDel for RouteCrit {}

/// Данные автоназначения эксперта АЦ.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoAssignExpertData {
    pub primary_pricing_expert_list: Vec<AutoAssignExpert>,
    pub replacement_pricing_expert_list: Option<Vec<AutoAssignExpert>>,
}

/// Эксперт автоназначения АЦ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoAssignExpert {
    pub expert_id: i32,
    pub date_range: (AsezDate, AsezDate),
}

/// Элемент данных маршрута автоназначения ПД.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoAssignDepartmentItem {
    pub department_id: i32,
    pub division: Option<AutoAssignDepartmentDivision>,
}

/// Подразделения ПД.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoAssignDepartmentDivision {
    pub id: i32,
    pub level: DepartmentLevel,
}

/// Данные маршрута автоназначения ПД.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    derive_more::Deref,
    derive_more::From,
    derive_more::IntoIterator,
)]
#[serde(transparent)]
pub struct AutoAssignDepartmentData(Vec<AutoAssignDepartmentItem>);

/// Данные маршрута автоназначения.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::From)]
#[serde(rename_all = "snake_case")]
pub enum RouteDataContent {
    AssignExpert(AutoAssignExpertData),
    AssignDepartment(AutoAssignDepartmentData),
}

#[derive(Debug, derive_more::Display)]
pub enum RouteDataTag {
    #[display(fmt = "assign_expert")]
    AssignExpert,
    #[display(fmt = "assign_department")]
    AssignDepartment,
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum TryFromRouteDataContentError {
    #[error("ожидаются данные для автоназначения эксперта")]
    AssignExpert,
    #[error("ожидаются данные для автоназначения профильного департамента")]
    AssignDepartment,
}

impl TryFrom<Option<RouteDataContent>> for AutoAssignExpertData {
    type Error = TryFromRouteDataContentError;

    fn try_from(value: Option<RouteDataContent>) -> Result<Self, Self::Error> {
        if let Some(RouteDataContent::AssignExpert(data)) = value {
            Ok(data)
        } else {
            Err(TryFromRouteDataContentError::AssignExpert)
        }
    }
}

impl TryFrom<Option<RouteDataContent>> for AutoAssignDepartmentData {
    type Error = TryFromRouteDataContentError;

    fn try_from(value: Option<RouteDataContent>) -> Result<Self, Self::Error> {
        if let Some(RouteDataContent::AssignDepartment(data)) = value {
            Ok(data)
        } else {
            Err(TryFromRouteDataContentError::AssignDepartment)
        }
    }
}

impl AutoAssignDepartmentData {
    pub fn new(item_list: Vec<AutoAssignDepartmentItem>) -> Self {
        Self(item_list)
    }
}

#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[item_table = "route_data"]
pub struct RouteData {
    /// Уникальный идентификатор записи (маршрута)
    #[item_field_pkey]
    pub route_uuid: Uuid,
    /// Дополнительные данные маршрута.
    ///
    /// NB. `Option` используется, чтобы обойти текущие ограничения,
    /// требующие реализацию `Default`.
    pub data: Json<Option<RouteDataContent>>,
    /// Дата/время создания записи
    pub created_at: AsezTimestamp,
    /// Автор записи
    pub created_by: i32,
    /// Дата/время изменения записи
    pub changed_at: AsezTimestamp,
    /// Автор изменения
    pub changed_by: i32,
}

/// Пользователи и периоды действия их записей для маршрутов согласования
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "route_users"]
pub struct RouteUsers {
    /// Идентификатор записи в таблице
    #[item_field_pkey]
    pub uuid: Uuid,
    /// Уникальный идентификатор записи (маршрута)
    pub route_id: i64,
    /// Тип назначения
    pub type_id: RouteApprType,
    /// Тип записи пользователя
    pub user_type_id: RouteUserType,
    /// Айди пользователя
    pub user_id: i32,
    /// Признак удаления записи
    pub is_removed: bool,
    /// Начало действия записи
    pub start_date: AsezTimestamp,
    /// Окончание действия записи
    pub end_date: AsezTimestamp,
    /// Дата/время создания записи
    pub created_at: AsezTimestamp,
    /// Автор записи
    pub created_by: i32,
    /// Дата/время изменения записи
    pub changed_at: AsezTimestamp,
    /// Автор изменения
    pub changed_by: i32,
}

/// Статус записи
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum RouteStatus {
    /// Не задан
    #[db_default]
    Undefined = 0,
    /// Активная
    Active = 1,
    /// Не активная
    InActive = 2,
}

///Тип назначения
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum RouteApprType {
    /// Не задан
    #[db_default]
    #[display(fmt = "Не установлено")]
    Undefined = 0,
    /// Проф департаменты
    #[display(fmt = "ПД")]
    SpecializedDepartments = 1,
    /// Анализ цены
    #[display(fmt = "АЦ")]
    PriceAnalysis = 2,
    /// Контроль лимита по бюджету
    #[display(fmt = "Контроль лимита по бюджету")]
    BudgetLimitControl = 3,
    /// Контроль лимита
    #[display(fmt = "LimitControl")]
    LimitControl = 4,
}

///Тип назначения
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum RouteUserType {
    /// Не задан
    #[db_default]
    Undefined = 0,
    /// Основной
    Main = 1,
    /// Замещающий
    Substitute = 2,
    /// Руководитель
    Manager = 3,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
/// То, для чего используется маршрут
pub enum RouteActivationType {
    /// Не установлено
    #[db_default]
    Undefined = 0,
    ///Используется для ППЗ
    Plan = 1,
    /// Используется для ДС
    ContractAmendment = 2,
    /// Используется для АЦ
    PriceAnalysis = 3,
}

/// Справочник "Критерии маршрутов согласования"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "route_crit_name"]
pub struct RouteCritName {
    /// Идентификатор критерия
    pub id: CriterionIdentifier,
    /// Наименование критерия
    #[item_field_pkey]
    #[serde(rename = "text")]
    pub name: String,
    /// Автор изменения
    pub changed_by: i32,
    /// Дата изменения
    pub changed_at: AsezTimestamp,
    /// Признак удаления
    pub is_removed: bool,
}

/// Идентификатор критерия (для name_id в route_crit)
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Type,
    Serialize,
    Deserialize,
    DbEnum,
    derive_more::Display,
)]
#[serde(from = "i16", into = "i16")]
#[repr(i16)]
pub enum CriterionIdentifier {
    /// Не установлено
    #[db_default]
    #[display(fmt = "Не установлено")]
    Undefined = 0,
    /// Заказчик
    #[display(fmt = "Заказчик")]
    Customer = 1,
    /// Раздел плана
    #[display(fmt = "Раздел плана")]
    PlanSection = 2,
    /// Стоимость (без НДС) руб
    #[display(fmt = "Стоимость (без НДС) руб.")]
    ContractPriceWithoutVAT = 3,
    /// Вид предмета закупки
    #[display(fmt = "Вид предмета закупки")]
    PurchaseTypeId = 4,
    /// ОКПД2
    #[display(fmt = "ОКПД2")]
    OKPD2 = 5,
    /// Статья бюджета
    #[display(fmt = "Статья бюджета")]
    BudgetItemId = 6,
}

/// Справочник "Типы маршрутов согласования"
#[derive(
    Debug, Default, Clone, DbItem, DbAdaptor, PartialEq, Serialize, Deserialize,
)]
#[adaptor_derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[item_table = "route_type"]
pub struct RouteType {
    /// Идентификатор типа маршрута
    pub id: RouteApprType,
    /// Наименование типа маршрута
    #[item_field_pkey]
    #[serde(rename = "text")]
    pub name: String,
    /// Автор изменения
    pub changed_by: i32,
    /// Дата изменения
    pub changed_at: AsezTimestamp,
    /// Признак удаления
    pub is_removed: bool,
}

impl AsRef<RouteHeader> for RouteHeader {
    fn as_ref(&self) -> &RouteHeader {
        self
    }
}

/// Варианты значений, используемых в критериях маршрутов.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
    derive_more::From,
)]
#[serde(untagged)]
pub enum CritValue {
    Bool(bool),
    Int(i64),
    Timestamp(AsezTimestamp),
    Date(AsezDate),
    String(String),
}

#[derive(Debug, thiserror::Error)]
#[error("значение `{0}` не может быть использовано для критериев маршрутов")]
pub struct TryFromValueError(Value);

impl TryFrom<Value> for CritValue {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(CritValue::String(s)),
            Value::Int(i) => Ok(CritValue::Int(i)),
            Value::Bool(b) => Ok(CritValue::Bool(b)),
            Value::Date(d) => Ok(CritValue::Date(d)),
            Value::Timestamp(t) => Ok(CritValue::Timestamp(t)),
            _ => Err(TryFromValueError(value)),
        }
    }
}

impl From<CritValue> for Value {
    fn from(value: CritValue) -> Self {
        match value {
            CritValue::String(s) => Value::String(s),
            CritValue::Int(i) => Value::Int(i),
            CritValue::Bool(b) => Value::Bool(b),
            CritValue::Date(d) => Value::Date(d),
            CritValue::Timestamp(t) => Value::Timestamp(t),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum CritPredicate {
    /// Неизвестно/ошибка
    #[default]
    Unknown,
    /// Равно. Symbol "="
    Equal { value: CritValue },
    /// Не Равно. Symbol "!="
    NotEqual { value: CritValue },
    /// Symbol "<"
    Less { value: CritValue },
    /// Symbol "<="
    LessEqual { value: CritValue },
    /// Symbol ">"
    Greater { value: CritValue },
    /// Symbol ">="
    GreaterEqual { value: CritValue },
    /// Between
    Between { low: CritValue, high: CritValue },
    /// Symbol *
    Any,
    /// Вхождение в множество
    In { values: Vec<CritValue> },
    /// Конъюнкция предикатов
    And { predicates: Vec<CritPredicate> },
    /// Дизъюнкция предикатов
    Or { predicates: Vec<CritPredicate> },
    /// Отрицание предиката
    Not { predicate: Box<CritPredicate> },
    /// Значение принадлежит поддереву значений из справочника `dictionary` с корнями `roots`.
    ///
    /// Используется для значений многоуровневых справочников (ОКПД2, ...).
    ///
    /// Значение `x` удовлетворяет критерию `in_tree { roots }`, если `x` является одним из `roots`,
    /// либо `x` является узлом одного из поддеревьев `roots`.
    InTree { dictionary: String, roots: Vec<i32> },
    /// Отсутствие проверки
    None,
}
impl CritPredicate {
    pub fn sort_values(&mut self) {
        match self {
            CritPredicate::In { values } => {
                values.sort();
            }
            CritPredicate::InTree { roots, .. } => {
                roots.sort();
            }
            CritPredicate::And { predicates } => {
                predicates.iter_mut().for_each(|predicate| predicate.sort_values())
            }
            CritPredicate::Or { predicates } => {
                predicates.iter_mut().for_each(|predicate| predicate.sort_values())
            }
            _ => {}
        }
    }
}
