use ahash::AHashMap;
use asez2_tables::master_data::ObjectTypeId;
use uuid::Uuid;

use super::plan_reasons_cancel::PlanReasonCancel;
use super::routes::RouteCriterion;
use crate::domain::enums::master_data::DirectoryType;
use crate::domain::plan_reasons_cancel::PlanReasonCancelHeaderRep;
use crate::presentation::dto::general::ItemList;
use asez2_shared_db::db_item::Select;
use asez2_tables::master_data::routes::{
    CritValue, RouteApprType, RouteDataContent, RouteHeaderRep,
};
use asez2_tables::{PlanOrAmendmentRep, Section};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct MasterDataSearchRequest {
    pub search_type: MasterDataSearchType,
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum MasterDataSearchType {
    SearchById(Vec<i32>, DirectoryType),
    SearchByUserInput(SearchByUserInput, DirectoryType),
    GetFullDirectory(Vec<DirectoryType>),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MasterDataAction {
    RouteStart(RouteStartReq),
    RouteStop(RouteStopReq),
    RouteRemove(RouteRemoveReq),
    RouteList(RouteListReq),
    RouteDetails(RouteDetailsReq),
    RouteCreate(RouteCreateReq),
    RouteUpdate(RouteUpdateReq),
    RouteCopy(RouteCopyReq),
    OrganizationUserAssignmentSearchById(SearchByIdReq),
    OrganizationUserAssignmentSearchByDepartment(SearchByDepartmentReq),
    SearchPlanReasonCancel(SearchPlanReasonsCancelRabbitReq),
}

/// Is used for search_by_id requests
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct SearchById {
    pub id: i32,
}

/// Последовательный поиск, при котором сначала идет
/// попытка найти по `division` подразделению. Если же `division`
/// не был передан или записей по нему не было найдено, то
/// осуществляется попытка найти по `department` департаменту
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct SearchByDepartment {
    pub department: i32,
    pub division: Option<i32>,
}

impl From<i32> for SearchById {
    fn from(id: i32) -> Self {
        SearchById { id }
    }
}

/// search_by_id request body
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct SearchByIdReq(Vec<SearchById>);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchByDepartmentReq(pub Vec<SearchByDepartment>);

impl SearchByIdReq {
    pub fn iter(&self) -> impl '_ + Iterator<Item = &'_ i32> {
        self.0.iter().map(|x| &x.id)
    }
}

impl FromIterator<i32> for SearchByIdReq {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        SearchByIdReq(iter.into_iter().map(Into::into).collect())
    }
}

/// Is used for user input search requests
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Default)]
pub struct SearchByUserInput {
    pub from: u32,
    pub quantity: u32,
    pub search: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Source {
    EstimatedCommission,
    PriceAnalysis,
    SpecializedDepartments,
}

/// Запрос на "action/route_start". В зависимости
/// от типа маршрута, могут быть разные под-действия.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteStartReq {
    pub items: Vec<Uuid>,
    pub type_id: RouteApprType,
}
/// Запрос на "action/route_stop".
pub type RouteStopReq = Vec<Uuid>;
/// Запрос на "action/route_remove".
pub type RouteRemoveReq = Vec<Uuid>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteCopyReq {
    pub uuid: Uuid,
    pub name_short: String,
    pub user_id: i32,
}

/// Запрос на получение маршрутов.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteListReq {
    pub section: Section,
    pub select: Select,
    pub user_id: i32,
    pub type_id: RouteApprType,
}

/// Значение поля для проверки критерия.
///
/// `Plain` -- одиночное значение,
/// `AnyAgg` -- набор значений, из которых хотя бы одно должно удовлетворять критерию,
/// `AllAgg` -- набор значений, все из которых должны удовлетворять критерию.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "values", rename_all = "snake_case")]
pub enum CritArg {
    AnyAgg(Vec<CritValue>),
    AllAgg(Vec<CritValue>),
    #[serde(untagged)]
    Plain(Option<CritValue>),
}

/// Структура для удобства передачи элемента с дополнительными значениями
/// (например, ППЗ/ДС с набором своих полей + аггрегатрые значения полей позиций).
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemWithExtraFields<T> {
    #[serde(flatten)]
    pub item: T,
    #[serde(flatten)]
    pub extra: AHashMap<String, CritArg>,
}

/// Запрос на получение деталей маршрута.
#[derive(Debug, Deserialize, Serialize)]
pub struct RouteDetailsReq {
    pub route_id: i64,
}

/// Запрос на создание/удаление маршрута.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteCreateUpdateReq {
    pub user_id: i32,
    pub header: RouteHeaderRep,
    pub criteria: AHashMap<String, Vec<RouteCriterion>>,
    pub data: RouteDataContent,
}

/// Запрос на создание маршрута.
pub type RouteCreateReq = RouteCreateUpdateReq;

/// Запрос на удаление маршрута.
pub type RouteUpdateReq = RouteCreateUpdateReq;

/// Запрос на поиск маршрутов.
#[derive(Debug, Deserialize, Serialize)]
pub struct RouteFindReq<T> {
    /// Тип маршрутов.
    pub type_id: RouteApprType,
    /// Исходные элементы, по которым производится поиск.
    pub item_list: Vec<RouteFindReqItem<T>>,
}

/// Элемент поиска маршрутов.
#[derive(Debug, Deserialize, Serialize)]
pub struct RouteFindReqItem<T> {
    /// Идентификатор элемента.
    pub id: i64,
    /// Элемент, который будет проверяться на удовлетворение критериям маршрута.
    ///
    /// Набор полей, которые должны присутствовать, зависит от типа маршрута.
    pub item: T,
}

/// Добавляет object_type_id в сериализацию
#[derive(Debug, Deserialize, Serialize)]
pub struct WithObjectTypeId<T> {
    object_type_id: ObjectTypeId,
    #[serde(flatten)]
    item: T,
}

impl<'a> From<&'a PlanOrAmendmentRep> for WithObjectTypeId<&'a PlanOrAmendmentRep> {
    fn from(item: &'a PlanOrAmendmentRep) -> Self {
        let object_type_id = match item {
            PlanOrAmendmentRep::Plan(_) => ObjectTypeId::Plan,
            PlanOrAmendmentRep::Amendment(_) => ObjectTypeId::ContractAmendment,
        };
        WithObjectTypeId {
            object_type_id,
            item,
        }
    }
}

/// Запрос на получение Профильных департаментов по пользователю.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSpecializedDepartmentsReq {
    pub user_id: i32,
}

/// Запрос на получение Управлений по пользователю.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetDivisionsReq {
    pub user_id: i32,
}

pub type CreatePlanReasonsCancelReq = PlanReasonCancel;
pub type UpdatePlanReasonsCancelReq = PlanReasonCancel;

pub type DeleteRestoreIdReq = ItemList<SearchById>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchPlanReasonCancelReq {
    #[serde(flatten)]
    pub search: SearchByUserInput,
    #[serde(flatten)]
    pub header: PlanReasonCancelHeaderRep,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchPlanReasonsCancelRabbitReq {
    pub ids: Option<Vec<i32>>,
    pub check_reason_id: Option<i16>,
}
