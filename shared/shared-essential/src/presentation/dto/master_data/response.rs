use super::routes::RouteCriterion;
use crate::presentation::dto::general::ItemList;
use crate::presentation::dto::response_request::PaginatedData;
use crate::{
    domain::master_data::{
        estimated_commission::{
            agenda_status::EstimatedCommissionAgendaStatus,
            protocol_status::EstimatedCommissionProtocolStatus,
            protocol_type::EstimatedCommissionProtocolType,
            results::EstimatedCommissionResult,
        },
        organization::Organization,
        routes::Route,
        scheduler_calendar::scheduler_update_catalog_request::SchedulerRequestUpdateCatalog,
        technical_commercial_proposal::request_type::PriceInformationRequestType,
    },
    presentation::dto::response_request::{
        ApiResponseData, ApiResponseDataWrapper, Messages,
    },
};
use ahash::AHashMap;
use asez2_tables::{
    master_data::{
        assigning_executor_method::AssigningExecutorMethod,
        budget_item::BudgetItem,
        category::Category,
        critical_type_color_scheme::CriticalTypeColorScheme,
        expert_conclusion_type::ExpertConclusionType,
        object_type::ObjectType,
        okpd2::Okpd2,
        payment_conditions::PaymentCondition,
        plan_reasons_cancel::{
            PlanReasonCancelCheckReason, PlanReasonCancelFunctionality,
            PlanReasonCancelImpactArea,
        },
        ppz_type::PpzType,
        price_analysis::{
            analysis_method::AnalysisMethod,
            price_analysis_method::PriceAnalysisMethod, pricing_unit::PricingUnit,
        },
        response::Response,
        routes::{RouteDataContent, RouteHeaderRep},
        technical_commercial_proposal::status_type::TcpStatus,
    },
    OrganizationalStructure,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::plan_reasons_cancel::PlanReasonCancel;

/// Внутренний запрос rabbit'a
#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MasterDataSearchResponse {
    pub messages: Messages,
    pub records: DirectoryRecords,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DirectoryRecords {
    /// Справочник "Способ анализа"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis_method: Option<Vec<AnalysisMethod>>,
    /// Справочник "Способ назначения исполнителя"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigning_executor_method: Option<Vec<AssigningExecutorMethod>>,
    /// Параметры автоназначения эксперта АЦ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_assignment_expert_ac: Option<Vec<Route>>,
    /// Справочник ВПЗ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<Category>>,
    /// Справочник Статья бюджета
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_item: Option<Vec<BudgetItem>>,
    /// Цветовые схемы критичности
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_type_color_scheme: Option<Vec<CriticalTypeColorScheme>>,
    /// Решения комиссии СК по ППЗ/ДС
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_commission_result: Option<Vec<EstimatedCommissionResult>>,
    /// Статусы повестки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_commission_agenda_status:
        Option<Vec<EstimatedCommissionAgendaStatus>>,
    /// Статусы протокола
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_commission_protocol_status:
        Option<Vec<EstimatedCommissionProtocolStatus>>,
    /// Типы заключений эксперта
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expert_conclusion_type: Option<Vec<ExpertConclusionType>>,
    /// Справочник "Типы объектов"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<Vec<ObjectType>>,
    /// Поставщики
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizations: Option<Vec<Organization>>,
    /// Справочник «Условия оплаты»
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_conditions: Option<Vec<PaymentCondition>>,
    /// Справочник «Тип ППЗ»
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ppz_type: Option<Vec<PpzType>>,
    /// Справочник "Метод ценообразования"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_analysis_method: Option<Vec<PriceAnalysisMethod>>,
    /// Справочник «Департамент (организация) АЦ»
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_unit: Option<Vec<PricingUnit>>,
    /// Тип протокола
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_type: Option<Vec<EstimatedCommissionProtocolType>>,
    /// Тип протокола
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Vec<Response>>,
    /// Справочник "Производственный календарь"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduler_request_update_catalog:
        Option<Vec<SchedulerRequestUpdateCatalog>>,
    /// Тип запроса ЗЦИ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_information_request_type: Option<Vec<PriceInformationRequestType>>,
    /// Справочник "Статусы ТКП"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_commercial_proposal_status: Option<Vec<TcpStatus>>,
    /// Справочник "ОКПД2"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okpd2: Option<Vec<Okpd2>>,
    /// Справочние "Организационная структура"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizational_structure: Option<Vec<OrganizationalStructure>>,
    /// Справочник "Основания аннулирования"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_reason_cancel_impact_area: Option<Vec<PlanReasonCancelImpactArea>>,
    /// Справочник "Функциональность"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_reason_cancel_functionality:
        Option<Vec<PlanReasonCancelFunctionality>>,
    /// Справочник "Проверки для ППЗ"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_reason_cancel_check_reason: Option<Vec<PlanReasonCancelCheckReason>>,
}

// NB: Десериализация не будет работать корректно, так как сериализация неоднозначна.
// Нужно это для ответа на запрос, где все варианты енума кладутся в одно и то же поле value
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DirectoryRecord {
    #[default]
    Nothing,
    /// Справочник "Способ анализа"
    AnalysisMethod(Vec<AnalysisMethod>),
    /// Справочник "Способ назначения исполнителя"
    AssigningExecutorMethod(Vec<AssigningExecutorMethod>),
    /// Параметры автоназначения эксперта АЦ
    Route(Vec<Route>),
    /// Справочник ВПЗ
    Category(Vec<Category>),
    /// Справочник Статья бюджета
    BudgetItem(Vec<BudgetItem>),
    /// Цветовые схемы критичности
    CriticalTypeColorScheme(Vec<CriticalTypeColorScheme>),
    /// Решения комиссии СК по ППЗ/ДС
    EstimatedCommissionResult(Vec<EstimatedCommissionResult>),
    /// Статусы повестки
    EstimatedCommissionAgendaStatus(Vec<EstimatedCommissionAgendaStatus>),
    /// Статусы протокола
    EstimatedCommissionProtocolStatus(Vec<EstimatedCommissionProtocolStatus>),
    /// Типы заключений эксперта
    ExpertConclusionType(Vec<ExpertConclusionType>),
    /// Справочник "Типы объектов"
    ObjectType(Vec<ObjectType>),
    /// Поставщики
    Organization(Vec<Organization>),
    /// Справочник «Условия оплаты»
    PaymentConditions(Vec<PaymentCondition>),
    /// Справочник «Тип ППЗ»
    PpzType(Vec<PpzType>),
    /// Справочник "Метод ценообразования"
    PriceAnalysisMethod(Vec<PriceAnalysisMethod>),
    /// Справочник «Департамент (организация) АЦ»
    PricingUnit(Vec<PricingUnit>),
    /// Тип протокола
    EstimatedCommissionProtocolType(Vec<EstimatedCommissionProtocolType>),
    /// Тип протокола
    DepartmentResponseStatus(Vec<Response>),
    /// Справочник "Производственный календарь"
    SchedulerRequestUpdateCatalog(Vec<SchedulerRequestUpdateCatalog>),
    /// Тип запроса ЗЦИ
    PriceInformationRequestType(Vec<PriceInformationRequestType>),
    /// Справочник "Статусы ТКП"
    TcpStatus(Vec<TcpStatus>),
    /// Справочник "ОКПД2"
    Okpd2(Vec<Okpd2>),
    /// Справочние "Организационная структура"
    OrganizationalStructure(Vec<OrganizationalStructure>),
    /// Справочник "Основания аннулирования"
    PlanReasonCancelImpactArea(Vec<PlanReasonCancelImpactArea>),
    /// Справочник "Функциональность"
    PlanReasonCancelFunctionality(Vec<PlanReasonCancelFunctionality>),
    /// Справочник "Проверки для ППЗ"
    PlanReasonCancelCheckReason(Vec<PlanReasonCancelCheckReason>),
}

impl FromIterator<DirectoryRecord> for DirectoryRecords {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = DirectoryRecord>,
    {
        let mut result = Self::default();
        iter.into_iter().for_each(|rec| match rec {
            DirectoryRecord::Nothing => {}
            DirectoryRecord::Route(x) => result.auto_assignment_expert_ac = Some(x),
            DirectoryRecord::AnalysisMethod(x) => result.analysis_method = Some(x),
            DirectoryRecord::AssigningExecutorMethod(x) => {
                result.assigning_executor_method = Some(x)
            }
            DirectoryRecord::Category(x) => result.category = Some(x),
            DirectoryRecord::BudgetItem(x) => result.budget_item = Some(x),
            DirectoryRecord::CriticalTypeColorScheme(x) => {
                result.critical_type_color_scheme = Some(x)
            }
            DirectoryRecord::EstimatedCommissionResult(x) => {
                result.estimated_commission_result = Some(x)
            }
            DirectoryRecord::EstimatedCommissionAgendaStatus(x) => {
                result.estimated_commission_agenda_status = Some(x)
            }
            DirectoryRecord::EstimatedCommissionProtocolStatus(x) => {
                result.estimated_commission_protocol_status = Some(x)
            }
            DirectoryRecord::ExpertConclusionType(x) => {
                result.expert_conclusion_type = Some(x)
            }
            DirectoryRecord::ObjectType(x) => result.object_type = Some(x),
            DirectoryRecord::Organization(x) => result.organizations = Some(x),
            DirectoryRecord::PaymentConditions(x) => {
                result.payment_conditions = Some(x)
            }
            DirectoryRecord::PpzType(x) => result.ppz_type = Some(x),
            DirectoryRecord::PriceAnalysisMethod(x) => {
                result.price_analysis_method = Some(x)
            }
            DirectoryRecord::PricingUnit(x) => result.pricing_unit = Some(x),
            DirectoryRecord::EstimatedCommissionProtocolType(x) => {
                result.protocol_type = Some(x)
            }
            DirectoryRecord::DepartmentResponseStatus(x) => {
                result.response = Some(x)
            }
            DirectoryRecord::SchedulerRequestUpdateCatalog(x) => {
                result.scheduler_request_update_catalog = Some(x)
            }
            DirectoryRecord::PriceInformationRequestType(x) => {
                result.price_information_request_type = Some(x)
            }
            DirectoryRecord::TcpStatus(x) => {
                result.technical_commercial_proposal_status = Some(x)
            }
            DirectoryRecord::Okpd2(x) => result.okpd2 = Some(x),
            DirectoryRecord::OrganizationalStructure(x) => {
                result.organizational_structure = Some(x)
            }
            DirectoryRecord::PlanReasonCancelImpactArea(x) => {
                result.plan_reason_cancel_impact_area = Some(x)
            }
            DirectoryRecord::PlanReasonCancelFunctionality(x) => {
                result.plan_reason_cancel_functionality = Some(x)
            }
            DirectoryRecord::PlanReasonCancelCheckReason(x) => {
                result.plan_reason_cancel_check_reason = Some(x)
            }
        });

        result
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DirectoryRecordResponse {
    pub value: DirectoryRecord,
}

impl ApiResponseData for DirectoryRecordResponse {}

#[macro_export]
macro_rules! try_get_directory_record {
    ($rec:expr, $kind:ident) => {
        match ($rec).value {
            $crate::presentation::dto::master_data::response::DirectoryRecord::$kind(y) => {
                Some(y)
            }
            _ => None,
        }
    };
}

pub type RouteStartResponse = ();
pub type RouteStopResponse = ();
pub type RouteRemoveResponse = ();
/// Ответ на запрос списка маршрутов
pub type RouteListResponse = PaginatedData<RouteItem>;
/// Ответ на запрос Копирования маршрута
pub type RouteCopyResponse = Option<i64>;

/// Ответ на запрос получения деталей маршрута.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RouteDetailsResponse {
    #[serde(flatten)]
    pub header: RouteHeaderRep,
    pub criteria_set: AHashMap<String, Vec<RouteCriterion>>,
    pub data: Option<RouteDataContent>,
}

impl ApiResponseData for RouteDetailsResponse {}

/// Ответ на запрос поиска маршрута.
pub type RouteFindResponse = Vec<FoundRoutes>;

/// Элемент данных ответа на запрос поиска маршрута, для одного входного элемента.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FoundRoutes {
    /// Идентификатор элемента запроса.
    pub id: i64,
    /// Найденные маршруты.
    pub item_list: Vec<FoundRouteData>,
}

/// Данные одного маршрута.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FoundRouteData {
    /// Идентификатор маршрута.
    pub route_id: i64,
    /// Данные маршрута.
    pub data: Option<RouteDataContent>,
}

/// Структура ответа, содержащая маршрут согласования и его атрибуты
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct RouteItem {
    /// Маршрут согласования (основная информация)
    pub header: RouteHeaderRep,
    /// Критерии маршрута.
    pub criteria: AHashMap<String, Vec<RouteCriterion>>,
    /// Применяется для ППЗ.
    pub is_plan: bool,
    /// Применяется для ДС.
    pub is_contract_amendment: bool,
    /// Данные маршрута.
    pub data: Option<RouteDataContent>,
}

/// Результат операции создания маршрута автоназначения.
pub type RouteCreateResponse = ApiResponseDataWrapper<RouteHeaderRep>;

/// Результат операции обновления маршрута автоназначения.
pub type RouteUpdateResponse = ApiResponseDataWrapper<RouteHeaderRep>;

/// Метаданные выбора элементов.
/// Используется в структурах, где возможен выбор элементов, включая
/// возможность выбрать все (`is_selected_all`) или исключить определенные значения (`is_exception_active`).
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct SelectionMeta {
    /// Переменная для определения выбора всех значений
    pub is_selected_all: Option<bool>,
    /// Переменная для определения исключенных значений
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_exception_active: Option<bool>,
}

/// Заказчики, относящиеся к маршруту
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct CustomerQuantity {
    /// Метаданные выбора заказчиков
    #[serde(flatten)]
    pub selection_meta: SelectionMeta,
    /// Список идентификаторов заказчиков
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_quantity: Option<Vec<i64>>,
}

impl CustomerQuantity {
    pub fn new(
        selection_meta: SelectionMeta,
        customer_quantity: Option<Vec<i64>>,
    ) -> Self {
        Self {
            selection_meta,
            customer_quantity,
        }
    }
}

/// Разделы плана, относящиеся к маршруту
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct SectionIdList {
    /// Метаданные выбора разделов
    #[serde(flatten)]
    pub selection_meta: SelectionMeta,
    /// Список идентификаторов разделов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_id_list: Option<Vec<i64>>,
}

impl SectionIdList {
    pub fn new(
        selection_meta: SelectionMeta,
        section_id_list: Option<Vec<i64>>,
    ) -> Self {
        Self {
            selection_meta,
            section_id_list,
        }
    }
}

/// Представление стоимости (без НДС) в маршруте
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct SumExcludedVatRub {
    /// Метаданные выбора
    #[serde(flatten)]
    pub selection_meta: SelectionMeta,
    /// Комплексное значение (оператор + сумма)
    pub complex_value: ComplexValue,
}

impl SumExcludedVatRub {
    pub fn new(
        selection_meta: SelectionMeta,
        operator: String,
        value: i64,
    ) -> Self {
        Self {
            selection_meta,
            complex_value: ComplexValue { operator, value },
        }
    }
}

/// Комплексное значение, содержащее оператор и сумму
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct ComplexValue {
    /// Оператор (`=`, `>`, `<`, `>=`, `<=`)
    pub operator: String,
    /// Значение (сумма)
    pub value: i64,
}

/// Вид предмета закупки (категория)
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct CategoryIdList {
    /// Метаданные выбора категорий
    #[serde(flatten)]
    pub selection_meta: SelectionMeta,
    /// Список идентификаторов категорий
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id_list: Option<Vec<i64>>,
}

impl CategoryIdList {
    pub fn new(
        selection_meta: SelectionMeta,
        category_id_list: Option<Vec<i64>>,
    ) -> Self {
        Self {
            selection_meta,
            category_id_list,
        }
    }
}

/// Статья бюджета
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct BudgetItemList {
    /// Метаданные выбора статей бюджета
    #[serde(flatten)]
    pub selection_meta: SelectionMeta,
    /// Список идентификаторов статей бюджета
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_item_id_list: Option<Vec<i64>>,
}

impl BudgetItemList {
    pub fn new(
        selection_meta: SelectionMeta,
        budget_item_id_list: Option<Vec<i64>>,
    ) -> Self {
        Self {
            selection_meta,
            budget_item_id_list,
        }
    }
}

/// Список Профильных департаменов, к которым относится пользователь
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GetSpecializedDepartmentsResponse {
    pub departments: Vec<i32>,
}

impl ApiResponseData for GetSpecializedDepartmentsResponse {}

/// Список Управлений, к которым относится пользователь
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GetDivisionsResponse {
    pub divisions: Vec<i32>,
}

impl ApiResponseData for GetDivisionsResponse {}

/// Данные результатов запросов search/search_by_id.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultValue<T> {
    pub value: Vec<T>,
}

impl<T> IntoIterator for SearchResultValue<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl<T> Default for SearchResultValue<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl<T> ApiResponseData for SearchResultValue<T> where
    T: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>
{
}

/// Ответ на запрос `organizational_user_assignment/search_by_id`
pub type OrgUserAssignmentSearchResponse =
    SearchResultValue<OrgUserAssignmentResItem>;

/// Элемент ответа на запрос `organizational_user_assignment/search_by_id`
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OrgUserAssignmentResItem {
    pub id: i32,
    pub first_name: String,
    pub patronymic_name: String,
    pub last_name: String,
    pub text: String,
    pub ui_text: String,
    pub email: String,
    pub phone: String,
    pub organization_structure_ids: Vec<Vec<i32>>,
}

pub type PlanReasonCancelResponse = PlanReasonCancel;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PlanReasonCancelDeleteResponse {
    pub id: i32,
    pub is_success: bool,
}

pub type PlanReasonCancelDeleteRestoreResponse =
    ItemList<PlanReasonCancelDeleteResponse>;

pub type PlanReasonCancelSearchResponse = SearchResultValue<PlanReasonCancel>;
