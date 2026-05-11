use ahash::AHashMap;

use actix_web::web::Json;
use serde::{Deserialize, Serialize};
use shared_essential::presentation::dto::{
    general::UiSelect, print_docs::common::TemplateFormat,
};
use shared_essential::{
    domain::{DepartmentLevel, OrganizationalStructureRep},
    presentation::dto::{
        master_data::{
            request::{CritArg, RouteFindReq, SearchByIdReq},
            response::{
                OrgUserAssignmentSearchResponse, RouteFindResponse,
                SearchResultValue,
            },
        },
        response_request::{ApiResponse, ApiResponseData},
        AsezResult,
    },
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OrgUserAssigngnmentReqBody {
    pub from: u32,
    pub quantity: u32,
    pub search: String,
    pub organization_structure_id: Option<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrganizationalStructureSearchReqBody {
    pub from: u32,
    pub quantity: u32,
    pub search: String,
    pub organization_structure_id: Option<i32>,
    pub level: Option<DepartmentLevel>,
    pub is_specialized_department: Option<bool>,
}

pub type OrganizationalStructureSearchResData =
    SearchResultValue<OrganizationalStructureRep>;

pub type OrgUserAssigngnmentByIdReqBody = SearchByIdReq;

pub type OrgUserAssignmentResData = OrgUserAssignmentSearchResponse;

/// Запрос на получение маршрута, критериям которого удовлетворяет список полей.
pub type RouteFindReqBody = RouteFindReq<RouteFindItem>;

/// Представление исходного элемента поиска маршрутов в виде отображения имен полей в значения.
pub type RouteFindItem = AHashMap<String, CritArg>;

/// Данные результата поиска маршрута.
///
/// Для каждого объекта из запроса возвращается список данных,
/// ассоциированных с маршрутами, удовлетворяющими критериям.
pub type RouteFindResData = RouteFindResponse;

pub type TestRoutes = RouteFindReqBody;

/// Данные иерархии многоуровневого справочника.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DicrionaryHierarchyResData {
    pub dictionary_list: Vec<DicrionaryHierarchyResItem>,
}

impl ApiResponseData for DicrionaryHierarchyResData {}

/// Данные элемента иерархии многоуровневого справочника.
#[derive(Debug, Serialize, Deserialize)]
pub struct DicrionaryHierarchyResItem {
    pub id: i32,
    pub code: String,
    pub parent_id: i32,
}

///
pub type NSIHttpRespose<T> = AsezResult<Json<ApiResponse<T, ()>>>;

/// Запрос на экспорт данных из справочника.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ExportRequest {
    pub format: Option<TemplateFormat>,
    pub captions: Option<Vec<String>>,
    #[serde(default)]
    pub is_export_by_email: bool,
    #[serde(flatten)]
    pub select: UiSelect,
}
