use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::{error::ResponseError, http::header::ContentType};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use thiserror::Error;
use uuid::Uuid;

use asez2_shared_db::db_item::{AsezDate, Select};
use asez2_shared_db::result::SharedDbError;
use asez2_shared_db::Value;
use asez2_tables::maths::CurrencyValue;
use asez2_tables::Section;
use broker::BrokerError;
pub use calculated_fields::*;
pub use errors::*;
pub use legacy_interaction::*;
use price_analysis::{
    ApproveByChiefReq, DeclineByChiefReq, GetPlansWithLastAgendaItemsReq,
    PreDeclineByChiefReq, PreRequestDocumentationReq,
    PreRequestDocumentsForExpertReq, PriceDeterminedReq, RequestDocumentationReq,
};

use super::general::{
    GeneralExportReq, Metadata, ObjectIdentifierList,
    ObjectIdentifierWithStatusNote, PlansAmendmentsWithUser,
};
use super::{
    error::{AsezErrorComplete, AsezErrorDict, ErrorLevel, Level},
    estimated_commission,
    general::{ObjectIdentifier, ObjectIdsWithUser},
    response_request::*,
    AsezError,
};
use crate::domain::tables::*;
use crate::presentation::dto::general::ObjectIdsWithUserAndComment;
use crate::presentation::dto::processing::price_analysis::{
    ExportSpecificationReq, ImportReq,
};

pub use self::price_analysis::{DocumentationCheckedReq, PricingReportRequest};

pub mod calculated_fields;
pub mod errors;
pub mod legacy_interaction;

pub mod get_plan_amendment;
pub use get_plan_amendment::*;

/// Basic type for sending requests to processing microservice.
/// Since processing receives from a single queue, for now,
/// and since our rabbit may or may not work correctly in this case,
/// it is simpler, and in any case more efficient, to use a request enum.
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "request_kind", content = "request_data")]
#[allow(clippy::large_enum_variant)]
pub enum ProcessingRequest {
    /// Обновить планы из монолита
    InsertUpdateLegacyPlans(legacy_interaction::InsertUpdateSrmPlansReq),
    /// Обновить планы из монолита
    InsertUpdateLegacyAmendments(legacy_interaction::InsertUpdateSrmAmendmentsReq),
    /// Used to get Plans without PlanItems
    GetPlans(PlansRequest),
    GetPlansCount(PlansCountRequest),
    /// Used to get Plans with PlanItems. This is usually more useful.
    GetCompletePlans(CompletePlansRequest),
    GetCompleteContractAmendments(CompletePlansRequest),
    GetPlanVersion(PlanVersionRequest),
    GetContractAmendmentVersion(PlanVersionRequest),
    GetAttachmentsMeta(GetAttachmentsMetaRequest),
    /// Для проверки данных для создания повестки.
    PreCreateAgenda(PreCreateAgendaReq),
    /// Предзапрос по удалению Повестки
    PreRequestAgendaRemove(PreRequestAgendaRemoveReq),
    /// Для удаления Повесток
    AgendaRemove(AgendaRemoveReq),
    /// Предзапрос отправки протокола на согласование
    PreProtocolAgreement(PreProtocolAgreementReq),
    /// Отправка протокола на согласование
    ProtocolAgreement(ProtocolAgreementReq),
    /// Для создания повестки.
    CreateAgenda(CreateAgendaReq),
    /// Для проверки данных для создания протокола.
    PreCreateProtocol(PreCreateProtocolReq),
    /// Для создания протокола.
    CreateProtocol(CreateProtocolReq),
    /// Презапрос на добавление новых ППЗ/ДС в Повестку СК
    PreAddPlansAgenda(PreAddPlansAgendaReq),
    /// Презапрос на перемещение ППЗ/ДС между Повестками СК
    PreTransferPlansAgenda(PreTransferPlansAgendaReq),
    /// Презапрос на добавление новых элементов в Протокол
    PreAddPlansProtocol(PreAddPlansProtocolReq),
    /// Запрос на добавление элементов в Протокол СК
    AddPlansProtocol(AddPlansProtocolReq),
    /// Добавление новых ППЗ/ДС в Повестку СК
    AddPlansAgenda(AddPlansAgendaReq),
    /// Перемещение ППЗ/ДС между Повестками СК
    TransferPlansAgenda(TransferPlansAgendaReq),
    /// Презапрос на добавление новых ППЗ/ДС в Повестку СК
    GetAgendaItemsByIdRange(GetAgendaItemsByIdRangeReq),
    /// Получение списка ППЗ/ДС, включенных в Протокол по диапазону идентификаторов ППЗ/ДС
    GetProtocolItemsByIdRange(GetProtocolItemsByIdRangeReq),
    /// Получения данных по не удаленным ППЗ/ДС,
    /// которые включены в Повестку и еще не включены в Протокол.
    GetAgendaItemsForProtocolCreate(GetAgendaItemsForProtocolCreateReq),
    /// Получения разных подробностей для повесток
    GetAgendaDetails(GetAgendaDetailsReq),
    /// Получение списка Повесток СК
    GetAgendaList(GetAgendaListReq),
    /// Получение списка Повесток СК по дате
    GetAgendaListByDate(GetAgendaListByDateReq),
    /// Получение разных подробностей для Протоколов СК
    GetProtocolDetails(GetProtocolDetailsReq),
    /// Получение списка Протоколов СК
    GetProtocolList(GetProtocolListReq),
    /// Получение списка Протоколов СК по Повестке СК
    GetProtocolListByAgenda(GetProtocolListByAgendaReq),
    /// Получение списка Протоколов СК по дате
    GetProtocolListByDate(GetProtocolListByDateReq),
    /// Аннулировать ППЗ/ДС. (Проверка до реальной отмены).
    PreCancelPlan(PreCancelPlansReq),
    /// Аннулировать ППЗ/ДС.
    CancelPlan(CancelPlansReq),
    /// Презапрос на изменение формы
    PreChangeForm(PreChangeFormReq),
    /// Запрос на изменение формы
    ChangeForm(ChangeFormReq),
    /// Утверждение списка ППЗ/ДС
    ApprovePlans(ApprovePlansReq),
    /// Предзапрос списка ППЗ/ДС, которые требуется Утвердить
    PreApprovePlans(PreApprovePlansReq),
    /// Проверка утвердить протокол.
    PreApproveProtocol(Vec<ObjectIdentifier>),
    /// Утвердить протокол
    ApproveProtocol(ApproveProtocolReq),
    /// Подтверждение решения СК в Протоколе
    ConfirmDecision(ConfirmDecisionReq),
    /// Возврат заказчику
    ReturnToCustomer(ReturnToCustomerReq),
    PreReturnToCustomer(PreReturnToCustomerReq),
    /// Получение партнеров СК
    GetPartners(GetPartnersReq),
    /// Назначение Эксперта АЦ
    AssignExpert(AssignExpertReq),
    /// Массовое Назначение Эксперта АЦ
    AssignExpertMass(AssignExpertMassReq),
    /// Возврат Эксперту АЦ
    ReturnToExpert(ReturnToExpertReq),
    PreReturnToExpert(PreReturnToExpertReq),
    /// Устранить протокол.
    RemoveProtocol(RemoveProtocolReq),
    /// Проверка на устранить протокол.
    PreRemoveProtocol(PreRemoveProtocolReq),
    /// Получение элементов Повестки СК
    GetAgendaItemList(GetItemListReq),
    /// Подписание протокола
    SendProtocolForSigning(SignProtocolReq),
    /// Проверка подписания протокола.
    PreSendProtocolForSigning(PreSignProtocolReq),
    /// Изменение даты очной СК
    ChangeCommissionDateReq(ChangeCommissionDateReq),
    /// Предзапрос ППЗ/ДС для изменения Даты очной СК
    PreChangeCommissionDateReq(PreChangeCommissionDateReq),
    /// Отправка повестки
    PreAgendaSend(PreAgendaSendReq),
    AgendaSend(AgendaSendReq),
    /// Проверка на удаление позиций повестки
    PreRemoveAgendaItems(PreRemoveAgendaItemsReq),
    /// Поиск повестки по плану
    GetPlansWithLastAgendaItems(GetPlansWithLastAgendaItemsReq),

    /// Предзапрос на возврат заказчику, АЦ
    PaPreReturnToCustomer(price_analysis::PreReturnToCustomerReq),
    /// Запрос на возврат заказчику, АЦ
    PaReturnToCustomer(price_analysis::ReturnToCustomerReq),
    /// Запрос действия "Документация проверена"
    PaDocumentationChecked(DocumentationCheckedReq),
    /// Получение количества ППЗ/ДС по секциям АЦ
    PaGetSectionsCount(price_analysis::GetSectionsCountRequest),
    /// Получение количества ППЗ/ДС по секциям СК
    EcGetSectionsCount(UserIdWrapper<GetSectionsCountRequest>),
    /// In theory we update one plan at a time without updating individual
    /// items. However it may be useful for services to update several.
    UpdatePlans(PrUpdatePlansReq),
    /// Обновить повестки и их принадлежности.
    UpdateAgenda(estimated_commission::UpdateAgendaReqWithUser),
    /// Обновить протоколы и их принадлежности.
    UpdateProtocol(estimated_commission::UpdateProtocolReqWithUser),
    /// Предзапрос списка ППЗ/ДС, действие "Запросить документацию" в АЦ
    PaPreRequestDocumentation(PreRequestDocumentationReq),
    /// Предзапрос списка ППЗ/ДС, действие "Запросить документацию" в АЦ
    PaPreRequestDocumentsForExpert(PreRequestDocumentsForExpertReq),
    /// Запрос действия "Запросить документацию" в АЦ
    PaRequestDocumentation(RequestDocumentationReq),
    /// Цена определена
    PriceDetermined(PriceDeterminedReq),
    /// Согласовать утверждение цены
    ApproveByChief(ApproveByChiefReq),
    /// Предзапрос списка ППЗ/ДС, действие "Вернуть Эксперту АЦ/Отклонить утверждение цены"
    PaPreDeclineByChief(PreDeclineByChiefReq),
    /// Запрос действия "Вернуть Эксперту АЦ/Отклонить утверждение цены"
    PaDeclineByChief(DeclineByChiefReq),
    /// Редактировать ППЗ
    PaUpdatePlan(UserIdWrapper<price_analysis::UpdatePlanReq>),
    /// Получение данных для отображения в форме "Результат определения цены"
    PaUpdateContractAmendment(
        UserIdWrapper<price_analysis::UpdateContractAmendmentReq>,
    ),
    // Получение данных для отображения в форме "Результат определения цены"
    PricingResult(price_analysis::PricingResultReq),
    /// Получение всех актуальных записей по пользователю АЦ
    PricingUser(price_analysis::GetPriceAnalysisUsersReq),
    /// Экспорт данных
    ExportData(ExportReq),
    /// Экспорт спецификации
    ExportSpecification(ExportSpecificationReq),
    /// Еженедельный отчет
    PricingReportCommon(PricingReportRequest),
    /// Отчет по экономии
    PricingReportSavings(PricingReportRequest),
    /// Бюллетень СК
    PricingReportCommission(PricingReportRequest),
    /// Импорт спецификации
    ImportSpecification(ImportReq),
    /// Импорт на Фронт(Без сохранения в БД) Списка ППЗ/ДС в повестку/protocol
    ImportItemListSpecific(ImportReq),

    /// Получение данных ретроспективных закупок на этапе Определения цены
    GetRetrospective(GetRetrospectiveReq),
    /// "POST /rest/pricing/v1/action/complete_lotting/"
    PaCompleteLotting(price_analysis::CompleteLottingRequest),
    /// Получение Количества поступлений ППЗ/ДС на данный статус/этап/раздел АЦ
    PaReviewProgress(price_analysis::ReviewProgressReq),
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct UserIdWrapper<T: Serialize> {
    pub user_id: i32,
    pub dto: T,
}

/// A plan request which includes a standard request
/// and extra parameters defined by section.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlansRequest {
    pub section: Section,
    pub select: Select,
    pub user_id: i32,
}
/// A plan request which includes a standard request
/// and extra parameters defined by section.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlansCountRequest {
    pub select: Select,
    pub pricing_expert_ids: Vec<i32>,
    pub section: Section,
    pub user_id: i32,
}

/// A plan request which includes a standard request
/// and extra parameters defined by section.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompletePlansRequest {
    pub section: Section,
    pub select: Select,
    pub item_fields: Vec<String>,
    pub user_id: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanVersionRequest {
    pub plan_id: i64,
    pub user_id: i32,
    pub version: i16,
}

impl From<CompletePlansRequest> for PlansRequest {
    fn from(x: CompletePlansRequest) -> Self {
        Self {
            section: x.section,
            select: x.select,
            user_id: x.user_id,
        }
    }
}

/// Запрос мета-данных по аттачам для ППЗ/ДС
pub type GetAttachmentsMetaRequest = ObjectIdentifierList;
/// Ответ на [`GetAttachmentsMetaRequest`]
pub type GetAttachmentsMetaResponseData = Vec<GetAttachmentsMetaResponseItem>;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GetAttachmentsMetaResponseItem {
    #[serde(flatten)]
    pub id: ObjectIdentifier,
    pub attachment_list: Vec<AttachmentMeta>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AttachmentMeta {
    pub uuid: Uuid,
    /// Идентификатор категории, в которой расположен файл.
    pub category_id: CategoryId,
    #[serde(rename = "parent_id")]
    pub parent_number: i16,
}

/// Пре-запрос для проверки возможности аннулирования ППЗ/ДС
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreCancelPlansReq {
    pub section_id: Section,
    pub item_list: Vec<ObjectIdentifier>,
}
/// Ответ на пре-запрос на проверку возможности аннулирования ППЗ/ДС
pub type PreCancelPlansResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Запрос на аннулирование ППЗ/ДС
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CancelPlansReq {
    pub section_id: Section,
    pub item_list: Vec<ObjectIdentifierWithStatusNote>,
    pub user_id: i32,
}
/// Ответ на запрос на аннулирование ППЗ/ДС
pub type CancelPlansResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Пре-запрос на изменение формы СК
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreChangeFormReq {
    pub section_id: Section,
    pub item_list: Vec<ObjectIdentifier>,
}
/// Ответ на пре-запрос на изменение формы СК
pub type PreChangeFormResponseData = PaginatedData<PlanOrAmendmentRep>;
pub type ChangeFormResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Презапрос на добавление ППЗ/ДС в Повестку СК
pub type PreAddPlansAgendaReq = Vec<ObjectIdentifier>;

/// Предзапрос на перемещение ППЗ/ДС между Повестками СК
pub type PreTransferPlansAgendaReq = Vec<ObjectIdentifier>;
pub type PreTransferPlansAgendaResponseData = PaginatedData<PlanOrAmendmentRep>;
/// Ответ на [`PreAddPlansAgendaReq`]
pub type PreTransferPlansAgendaResponse =
    ApiResponse<PreTransferPlansAgendaResponseData, ()>;

/// Предзапрос на перемещение ППЗ/ДС между Повестками СК
pub type TransferPlansAgendaReq = AddPlansAgendaReq;
pub type TransferPlansAgendaResponseData = PaginatedData<i64>;
/// Ответ на [`TransferPlansAgendaReq`]
pub type TransferPlansAgendaResponse =
    ApiResponse<TransferPlansAgendaResponseData, ()>;

/// Предзапрос на возврат заказчику
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PreReturnToCustomerReq {
    pub section_id: Section,
    pub item_list: Vec<ObjectIdentifier>,
}

/// Данные из ответа на запрос на получение планов
pub type GetPlansResponseData = PaginatedData<GetPlansCalculatedItem>;
/// Ответ на запрос на получение планов
pub type GetPlansResponse = ApiResponse<GetPlansResponseData, ()>;

pub type GetExpertPlansCount = ApiResponse<GetExpertPlansCountData, ()>;

///
pub type GetExpertPlansCountData = AHashMap<i32, usize>;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ExpertPlansCount {
    pricing_expert_id: i32,
    plans_count: i32,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq, Clone)]
pub struct GetPlansItem {
    pub plan: PlanOrAmendmentRep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda: Option<EcAgendaRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item: Option<EcAgendaItemRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<EcProtocolRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_item: Option<EcProtocolItemRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<Metadata>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetPlansCalculatedItem {
    pub plan: CalculatedPlanRep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda: Option<EcAgendaRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<EcProtocolRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item: Option<EcAgendaItemRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_item: Option<CalculatedProtocolItemRep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<Metadata>,
}

/// Ответ на запрос [`AddPlansAgendaDto`] с айди успешно добавленных
/// ППЗ/ДС в Повестку СК
pub type AddPlansAgendaResponse = ApiResponse<PaginatedData<i64>, ()>;
pub type AddPlansAgendaResponseData = PaginatedData<i64>;
pub type GetItemListResponse = ApiResponse<GetItemListResponseData, ()>;

/// Ответ на запрос [`PreAddPlansAgendaDto`] с айди ППЗ/ДС,
/// которые можно добавить в Повестку СК
pub type PreAddPlansAgendaResponse = ApiResponse<PreAddPlansAgendaResponseData, ()>;

/// Ответ на запрос [`PreAddPlansAgendaReq`] с айди ППЗ/ДС,
/// которые можно добавить в Повестку СК
pub type PreAddPlansAgendaResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Ответ на запрос списка Повесток СК
pub type GetAgendaListResponse = ApiResponse<GetAgendaListResponseData, ()>;
pub type GetAgendaListResponseData = PaginatedData<GetAgendaListItem>;

pub type GetAgendaListByDateResponse =
    ApiResponse<GetAgendaListByDateResponseData, ()>;
pub type GetAgendaListByDateResponseData = PaginatedData<EcAgendaRep>;
impl ApiResponseData for GetAgendaItemsForProtocolCreateResponseData {}

/// Получение Протоколов по Повестке СК. Передается идентификатор
/// Повестки
pub type GetProtocolListByAgendaReq = ObjectIdentifier;
/// Ответ на [`GetProtocolListByAgendaReq`]
pub type GetProtocolListByAgendaResponse =
    ApiResponse<GetProtocolListByAgendaResponseData, ()>;
/// Ответ на [`GetProtocolListByAgendaReq`]
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct GetProtocolListByAgendaResponseData {
    /// Идентификатор Повестки
    pub id: i64,
    pub commission_date: AsezDate,
    pub item_list: Vec<EcProtocolRep>,
}
impl ApiResponseData for GetProtocolListByAgendaResponseData {}

/// Результат поиска повесток предназначенных для удаления
pub type PreRequestAgendaRemoveResponseData = PaginatedData<EcAgendaRep>;
pub type PreRequestAgendaRemoveResponse =
    ApiResponse<PreRequestAgendaRemoveResponseData, ()>;

pub type UpdateProtocolRes = ();

/// Результат удаления повесток
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AgendaRemoveResponseData {
    pub status_id: EcAgendaStatus,
}
impl ApiResponseData for AgendaRemoveResponseData {}
pub type AgendaRemoveResponse = ApiResponse<AgendaRemoveResponseData, ()>;

/// Ответ на запрос [`ReturnToCustomerReq`]
pub type ReturnToCustomerResponse =
    ApiResponse<PaginatedData<PlanOrAmendmentRep>, ()>;
pub type ReturnToCustomerResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Ответ на запрос [`PreReturnToCustomerReq`]
pub type PreReturnToCustomerResponseData = PaginatedData<PlanOrAmendmentRep>;

///  Ответ на запрос "/rest/estimated_commission/v1/pre_request/protocol_create/"
pub type PreCreateProtocolResponse = ApiResponse<PreCreateProtocolResponseData, ()>;
/// Ответ на предзапрос [`PreCreateProtocolResponse`] на создание Протокола СК
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PreCreateProtocolResponseData {
    /// Всегда является [`Option::Some`] если в [`PreCreateProtocolReq`]
    /// в protocol_type_id было передано 1, в обратном случае будет [`Option::None`]
    pub agenda_list: Option<Vec<AgendaWithItemThreshold>>,
    /// Всегда является [`Option::Some`] если в [`PreCreateProtocolReq`]
    /// в protocol_type_id было передано 2, в обратном случае будет [`Option::None`]
    pub plans: Option<Vec<PlanOrAmendmentRep>>,
}
impl ApiResponseData for PreCreateProtocolResponseData {}

/// Ответ на создание Протокола СК
pub type CreateProtocolResponseData = ();
pub type CreateProtocolResponse = ApiResponse<(), ()>;

/// Ответ на запрос [`PreAddPlansProtocolReq`] с айди ППЗ/ДС,
/// которые можно добавить в Протокол (PreCreateProtocolResponseData)
pub type PreAddPlansProtocolResponse = PreCreateProtocolResponse;
pub type PreAddPlansProtocolResponseData = PreCreateProtocolResponseData;

/// Ответ на запрос [`AddPlansProtocolReq`] с айди ППЗ/ДС,
/// которые можно добавить в Протокол (PreCreateProtocolResponseData)
pub type AddPlansProtocolResponse = CreateProtocolResponse;
pub type AddPlansProtocolResponseData = CreateProtocolResponseData;

pub type PreCreateAgendaResponse =
    ApiResponse<PaginatedData<PlanOrAmendmentRep>, ()>;
pub type PreCreateAgendaResponseData = PaginatedData<PlanOrAmendmentRep>;
/// Ответ на создание Повестки СК
pub type CreateAgendaResponseData = PaginatedData<i64>;
pub type CreateAgendaResponse = ApiResponse<PaginatedData<i64>, ()>;

/// Предзапрос на утверждение протокола
pub type PreApproveProtocolReq = Vec<ObjectIdentifier>;
/// Ответ на предзапрос на утверждение протокола
pub type PreApproveProtocolResponseData = PaginatedData<EcProtocolRep>;

/// Запрос на утверждение протокола
#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveProtocolReq {
    pub user_id: i32,
    pub ids: Vec<ObjectIdentifierWithStatusNote>,
    pub protocol_type_id: ProtocolType,
}

/// Ответ на запрос на утверждение протокола
pub type ApproveProtocolResponseData = PreSignProtocolResponseData;

/// Предзапрос на подписание протокола
pub type PreSignProtocolReq = Vec<ObjectIdentifier>;
/// Ответ на предзапрос на подписание протокола
pub type PreSignProtocolResponseData = PaginatedData<EcProtocolRep>;
/// Запрос на подписание протокола
pub type SignProtocolReq = ObjectIdsWithUserAndComment;
/// Ответ на запрос на подписание протокола
pub type SignProtocolResponseData = PreSignProtocolResponseData;

/// Пре-запрос на удаление Протокола СК
#[derive(Debug, Serialize, Deserialize)]
pub struct PreRemoveProtocolReq {
    pub protocol_type_id: ProtocolType,
    pub user_id: i32,
    pub item_list: Vec<ObjectIdentifier>,
}
/// Ответ на пре-запрос на удаление Протокола СК
pub type PreRemoveProtocolResponseData = PaginatedData<EcProtocolRep>;

/// Запрос на удаление Протокола СК
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveProtocolReq {
    pub protocol_type_id: ProtocolType,
    pub user_id: i32,
    pub item_list: Vec<ObjectIdentifierWithStatusNote>,
}
/// Ответ на удаление Протокола СК
pub type RemoveProtocolResponseData = PreRemoveProtocolResponseData;
/// Ответ на удаление Протокола СК
pub type RemoveProtocolResponse = ApiResponse<RemoveProtocolResponseData, ()>;

/// Запрос (поиск) повесток, предназначенных для удаления
pub type PreRequestAgendaRemoveReq = ObjectIdentifierList;

/// Ответ на предзапрос изменения даты очной СК
pub type PreChangeCommissionDateResponse = PaginatedData<PlanOrAmendmentRep>;
/// Ответ на изменение даты очной СК
pub type ChangeCommissionDateResponse = PreChangeCommissionDateResponse;

/// Ответ на проверку для удаление повесток.
pub type PreRemoveAgendaItemsResponseData = PaginatedData<PlanOrAmendmentRep>;
/// Ответ на удаление Протокола СК
pub type PreRemoveAgendaItemsResponse =
    ApiResponse<PreRemoveAgendaItemsResponseData, ()>;

/// Ответ на предзапрос списка ППЗ/ДС, которые требуется Утвердить
pub type PreApprovePlansResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Ответ на утверждение списка ППЗ/ДС
pub type ApprovePlansResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Предзапрос списка ППЗ/ДС, которые требуется вернуть Эксперту АЦ
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PreReturnToExpertReq {
    pub section_id: Section,
    pub item_list: Vec<ObjectIdentifier>,
}
/// Ответ на запрос [`PreReturnToExpertReq`]
pub type PreReturnToExpertResponseData = PaginatedData<PlanOrAmendmentRep>;

/// Назначение эксперта АЦ
pub type AssignExpertReq = ObjectIdsWithUser;
pub type AssignExpertResponse = ApiResponse<AssignExpertResponseData, ()>;
pub type AssignExpertResponseData = Vec<PlanOrAmendmentRep>;

/// Назначение эксперта АЦ
pub type AssignExpertMassReq = PlansAmendmentsWithUser;
pub type AssignExpertMassResponse = ApiResponse<AssignExpertMassResponseData, ()>;
pub type AssignExpertMassResponseData = Vec<PlanOrAmendmentRep>;

/// Запрос на проверку для удаление повесток.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PreRemoveAgendaItemsReq {
    pub agenda_id: i64,
    pub agenda_uuid: Uuid,
    pub item_list: Vec<PreRemoveAgendaItem>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PreRemoveAgendaItem {
    /// Идентификатор ППЗ / ДС
    pub id: i64,
    /// UUID ППЗ/ДС
    pub source_uuid: Uuid,
    /// UUID позиции Повестки СК. Может быть пустым, если запись не добавлена в БД
    pub uuid: Option<Uuid>,
    pub object_type: EntityKind,
}

/// Запрос на добавление ППЗ/ДС в Повестку СК
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ChangeFormReq {
    /// Тип комиссии
    pub commission_kind_id: CommissionKind,
    /// Игнорировать ли предупреждения
    pub is_force: bool,
    /// Идентификатор пользователя
    pub user_id: i32,
    /// Список добавляемых ППЗ/ДС
    pub item_list: Vec<ObjectIdentifierWithStatusNote>,
    pub section_id: Section,
}

/// Выборка ППЗ/ДС из Повестки для включения в Протокол очной СК
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct GetAgendaItemsForProtocolCreateReq {
    pub agenda_id: i64,
    pub uuid: Uuid,
}
/// Ответ на [`GetAgendaItemsForProtocolCreateReq`]
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetAgendaItemsForProtocolCreateResponseData {
    pub agenda_id: i64,
    pub uuid: Uuid,
    pub meeting_date: AsezDate,
    pub item_list: Vec<GetAgendaItemsForProtocolCreateItem>,
}
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetAgendaItemsForProtocolCreateItem {
    pub plan: PlanOrAmendmentRep,
    pub agenda_item: EcAgendaItemRep,
    pub is_can_be_included_in_protocol: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<Metadata>,
}

/// Удаление повесток
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct AgendaRemoveReq {
    pub user_id: i32,
    pub item_list: Vec<ObjectIdentifier>,
}

/// Отправка повестки (перевод на статус Sent)
pub type PreAgendaSendReq = ObjectIdentifierList;
pub type PreAgendaSendResponseData = PaginatedData<EcAgendaRep>;
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct AgendaSendReq {
    pub user_id: i32,
    pub item_list: Vec<ObjectIdentifier>,
}
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct AgendaSendResponseData {
    pub status_id: EcAgendaStatus,
}
impl ApiResponseData for AgendaSendResponseData {}

/// Запрос на создание нового протокола СК.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CreateProtocolReq {
    /// ID пользователя.
    pub user_id: i32,
    /// 1 or 2.
    pub protocol_type_id: ProtocolType,
    pub protocol_date: AsezDate,
    /// Идентификаторы записей (повесток), вместе со списком
    /// ППЗ/ДС.
    pub item_list: Vec<CreateProtocolItem>,
}

/// Пре-запрос на создание Протокола СК
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PreCreateProtocolReq {
    /// ID пользователя.
    pub user_id: i32,
    /// 1 or 2.
    pub protocol_type_id: ProtocolType,
    /// Идентификаторы записей (повесток или ППЗ/ДС)
    pub item_list: Vec<ObjectIdentifier>,
}

/// Презапрос на добавление элементов в Протокол СК
pub type PreAddPlansProtocolReq = PreCreateProtocolReq;

/// Запрос на добавление элементов в Протокол СК
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AddPlansProtocolReq {
    /// ID пользователя.
    pub user_id: i32,
    /// 1 or 2.
    pub protocol_type_id: ProtocolType,
    pub protocol_date: AsezDate,
    pub protocol_id: i64,
    pub uuid: Uuid,
    /// Идентификаторы записей (повесток), вместе со списком
    /// ППЗ/ДС.
    pub item_list: Vec<CreateProtocolItem>,
}

/// Запрос на получение списка элементов Повестки СК
#[derive(Deserialize, Serialize, Debug)]
pub struct GetItemListReq {
    /// UI секция пользователя
    pub section_id: Section,
    /// Находятся ли в реестре
    pub is_registered_by_d647: Option<bool>,
    /// Айди Повестки СК
    pub id: i64,
    pub uuid: Uuid,
}

/// Запрос на получение списка элементов Повестки СК
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetItemListResponseData {
    /// Идентификатор Повестки/Протокола СК
    pub id: i64,
    pub protocol_date: Option<AsezDate>,
    pub meeting_date: Option<AsezDate>,
    pub item_list: Vec<GetItemListItem>,
}

impl ApiResponseData for GetItemListResponseData {}

/// Запрос на получение списка Повесток СК
#[derive(Deserialize, Serialize, Debug)]
pub struct GetProtocolListReq {
    /// Тип Протокола СК
    pub protocol_type_id: ProtocolType,
    pub select: Select,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct ProtocolRelatedRep {
    /// Plans get flattened in estimated_commission.
    pub plan: PlanOrAmendmentRep,
    /// These get flattened in estimated_commission.
    pub protocol_item: EcProtocolItemRep,
    /// These get flattened in estimated_commission.
    pub agenda_item: EcAgendaItemRep,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetProtocolListResponseItem {
    pub protocol: EcProtocolRep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_item_quantity_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_item_d647_quantity_threshold: Option<ColorThreshold>,
}
/// Ответ на [`GetProtocolListReq`]
pub type GetProtocolListResponseData = PaginatedData<GetProtocolListResponseItem>;

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
/// Можно и 'GetItemListItem' на его месте использовать.
pub struct GetAgendaItemListItem {
    ///ППЗ или ДС
    pub plan: PlanOrAmendmentRep,
    /// Элемент повестки СК
    pub agenda_item: EcAgendaItemRep,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetItemListItem {
    ///ППЗ или ДС
    pub plan: PlanOrAmendmentRep,
    /// Элемент повестки СК
    pub agenda_item: Option<EcAgendaItemRep>,
    /// Элемент протокола СК
    pub protocol_item: Option<EcProtocolItemRep>,
}

/// Презапрос на добавление новых ППЗ/ДС в Повестку СК
#[derive(Deserialize, Serialize, Debug)]
pub struct GetAgendaItemsByIdRangeReq {
    pub agenda_id: i64,
    pub is_registered_by_d647: bool,
    pub uuid: Uuid,
    pub item_list: Vec<Vec<i64>>,
}
/// Ответ на презапрос добавления новых ППЗ/ДС в Повестку СК
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetAgendaItemsByIdRangeResponseData {
    pub item_list: Vec<PlanOrAmendmentRep>,
}

impl ApiResponseData for GetAgendaItemsByIdRangeResponseData {}

/// Получение списка ППЗ/ДС, включенных в Протокол по диапазону идентификаторов ППЗ/ДС
#[derive(Deserialize, Serialize, Debug)]
pub struct GetProtocolItemsByIdRangeReq {
    pub protocol_id: i64,
    pub uuid: Uuid,
    pub protocol_type_id: ProtocolType,
    pub is_registered_by_d647: bool,
    pub item_list: Vec<Vec<i64>>,
}
/// Ответ на [`GetProtocolItemsByIdRangeReq`]
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetProtocolItemsByIdRangeResponseData {
    pub item_list: Vec<GetProtocolItemsByIdRangeItem>,
}
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetProtocolItemsByIdRangeItem {
    #[serde(flatten)]
    pub plan: PlanOrAmendmentRep,
    pub actual_sum_excluded_vat: Option<CurrencyValue>,
    pub commission_sum_excluded_vat: Option<CurrencyValue>,
}
impl ApiResponseData for GetProtocolItemsByIdRangeResponseData {}

/// Внутренний Запрос на создание протокола СК,
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateProtocolItem {
    pub id: ObjectIdentifier,
    pub all_items: Option<bool>,
    pub item_list: Option<Vec<ObjectIdentifier>>,
}

/// Запрос на получение списка Повесток СК
#[derive(Deserialize, Serialize, Debug)]
pub struct GetAgendaListReq {
    /// UI секция пользователя
    pub section_id: Section,
    #[serde(flatten)]
    /// Селект для запроса определенных полей
    pub select: Select,
}

/// Запрос на получение списка Повесток СК по дате
#[derive(Deserialize, Serialize, Debug)]
pub struct GetAgendaListByDateReq {
    /// Дата встречи СК
    pub date: AsezDate,
    /// Типа даты
    pub date_type: String,
}

/// Запрос на получение списка Протоколов СК по дате
#[derive(Deserialize, Serialize, Debug)]
pub struct GetProtocolListByDateReq {
    /// Дата встречи СК.
    pub date: AsezDate,
    /// Типа даты.
    pub date_type: String,
    /// Тип протокола.
    pub protocol_type_id: ProtocolType,
}

pub type GetProtocolListByDateResponse =
    ApiResponse<GetProtocolListByDateResponseData, ()>;
pub type GetProtocolListByDateResponseData = PaginatedData<EcProtocolRep>;

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct AgendaWithItemThreshold {
    #[serde(flatten)]
    pub agenda: EcAgendaRep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item_quantity_threshold: Option<ColorThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item_d647_quantity_threshold: Option<ColorThreshold>,
}

impl ApiResponseData for AgendaWithItemThreshold {}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GetAgendaListItem {
    #[serde(flatten)]
    pub agenda: EcAgendaRep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item_quantity_threshold: Option<ColorFullThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agenda_item_d647_quantity_threshold: Option<ColorFullThreshold>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_quantity: Option<usize>,
}

/// Обозначение количества элементов
#[derive(Clone, Copy, Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct ColorThreshold {
    /// Количество элементов, которые имеют определенный признак
    pub value: usize,
    /// Отношение этих элементов ко всему множеству элементов
    pub color_scheme_id: ColorScheme,
}

/// Имеет такое же значение, как и [`ColorThreshold`], но в value имеет массив
/// значений
#[derive(Clone, Copy, Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct ColorFullThreshold {
    /// Первое значение отвечает за общее количество элементов.
    /// Второе значение отвечает за количество элементов с определенным признаком
    pub value: [usize; 2],
    /// Отношение элементов с определенным признаком ко всему множеству элементов
    pub color_scheme_id: ColorScheme,
}

impl From<ColorFullThreshold> for ColorThreshold {
    fn from(value: ColorFullThreshold) -> Self {
        Self {
            color_scheme_id: value.color_scheme_id,
            value: value.value[0],
        }
    }
}

impl From<ColorThreshold> for asez2_shared_db::Value {
    fn from(value: ColorThreshold) -> Self {
        value.value.into()
    }
}

impl From<ColorFullThreshold> for Value {
    fn from(value: ColorFullThreshold) -> Self {
        format!("{} ({})", value.value[0], value.value[1]).into()
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(from = "u8", into = "u8")]
#[repr(u8)]
pub enum ColorScheme {
    Undefined = 0,
    Green = 1,
    Yellow = 2,
    Red = 3,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::Undefined
    }
}

impl From<u8> for ColorScheme {
    fn from(value: u8) -> Self {
        match value {
            1 => ColorScheme::Green,
            2 => ColorScheme::Yellow,
            3 => ColorScheme::Red,
            _ => ColorScheme::Undefined,
        }
    }
}

impl From<ColorScheme> for u8 {
    fn from(value: ColorScheme) -> Self {
        value as u8
    }
}

/// Предзапрос "Отправить Протокол на согласование"
#[derive(Deserialize, Serialize, Debug)]
pub struct PreProtocolAgreementReq {
    /// Тип протокола
    pub protocol_type_id: ProtocolType,
    /// Уникальные идентификаторы протоколов для запроса отправки на согласование
    pub item_list: Vec<ObjectIdentifier>,
}

/// Результат поиска "Отправить Протокол на согласование"
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct PreProtocolAgreementResponse {
    /// кол-во Протоколов
    pub total: u32,
    /// Список протоколов
    pub item_list: Vec<EcProtocolRep>,
}
impl ApiResponseData for PreProtocolAgreementResponse {}

/// Запрос "Отправить Протокол на согласование"
#[derive(Deserialize, Serialize, Debug)]
pub struct ProtocolAgreementReq {
    pub user_id: i32,
    /// Тип протокола
    pub protocol_type_id: ProtocolType,
    /// Уникальные идентификаторы протоколов для отправки на согласование
    pub item_list: Vec<ObjectIdentifierWithStatusNote>,
}

/// A request for Plan Updates contains a list of fields and a list of PlanReps.
#[derive(Debug, Serialize, Deserialize)]
pub struct PrUpdatePlansReq {
    pub user_id: i32,
    pub fields: Vec<String>,
    pub plans: Vec<PlanOrAmendmentRep>,
}
pub type PrUpdatePlansResponseData = PaginatedData<PlanOrAmendmentRep>;

/// This structure represents what the contract
/// ("v1/action/create_agenda")
/// expects. However, we do not want it.
pub struct EsUpdatePlansReq {
    uuid: String,
    data: PlanOrAmendmentRep,
    fields: Vec<String>,
}

/// Предзапрос на создание Повестки СК
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PreCreateAgendaReq {
    pub user_id: i32,
    pub item_list: Vec<ObjectIdentifier>,
}

/// Запрос на создание Повестки СК
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct CreateAgendaReq {
    pub user_id: i32,
    pub is_force: bool,
    pub meeting_date: AsezDate,
    pub item_list: Vec<ObjectIdentifier>,
}

/// Запрос на добавление ППЗ/ДС в Повестку СК
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AddPlansAgendaReq {
    /// Игнорировать ли предупреждения
    pub is_force: bool,
    /// Идентификатор пользователя
    pub user_id: i32,
    /// Список добавляемых ППЗ/ДС
    pub item_list: Vec<ObjectIdentifier>,
    /// Айди Повестки СК
    pub agenda_id: i64,
}

/// Запрос на возврат заказчику
#[derive(Deserialize, Serialize, Debug)]
pub struct ReturnToCustomerReq {
    pub section_id: Section,
    pub action_type: ActionType,
    pub is_force: bool,
    pub item_list: Vec<ReturnToSomeoneItem>,
    pub user_id: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReturnToSomeoneItem {
    pub id: i64,
    pub uuid: Uuid,
    pub status_note: String,
    /// This field is not used.
    #[serde(default, skip_serializing_if = "EntityKind::undefined")]
    pub object_type: EntityKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_excluded: Option<bool>,
}

impl From<ReturnToSomeoneItem> for ObjectIdentifierWithStatusNote {
    fn from(value: ReturnToSomeoneItem) -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new_with_type(
            value.id,
            value.uuid,
            value.object_type,
            value.status_note,
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Revision,
    Documentation,
}

// Изменить дату очной СК
#[derive(Deserialize, Serialize, Debug)]
pub struct ChangeCommissionDateReq {
    pub item_list: Vec<ChangeCommissionDateItem>,
    pub is_force: bool,
    pub user_id: i32,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct ChangeCommissionDateItem {
    #[serde(flatten)]
    pub item: ObjectIdentifier,
    pub commission_date: AsezDate,
}

// Предзапрос списка ППЗ/ДС для изменения Даты очной СК
#[derive(Deserialize, Serialize, Debug)]
pub struct PreChangeCommissionDateReq {
    pub item_list: Vec<ObjectIdentifier>,
}

/// Предзапрос списка ППЗ/ДС, которые требуется Утвердить
#[derive(Deserialize, Serialize, Debug)]
pub struct PreApprovePlansReq {
    pub section_id: Section,
    pub item_list: Vec<ObjectIdentifier>,
}

/// Утверждение списка ППЗ/ДС
#[derive(Deserialize, Serialize, Debug)]
pub struct ApprovePlansReq {
    pub section_id: Section,
    pub item_list: Vec<ObjectIdentifierWithStatusNote>,
    pub user_id: i32,
    pub is_force: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfirmDecisionReq {
    pub protocol_id: i64,
    pub protocol_uuid: Uuid,
    pub is_registered_by_d647: bool,
    pub user_id: i32,
    pub item_list: Vec<ConfirmDecisionItem>,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ConfirmDecisionItem {
    /// uuid элемента Протокола
    pub uuid: Uuid,
    pub source_uuid: Uuid,
    pub plan_id: i64,
    pub object_type: EntityKind,
    pub result_id: ResultId,
    pub status_note: String,
}
pub type ConfirmDecisionResponseData = ();

/// Запрос на подробности повестки. UUID и ID повестки.
#[derive(Deserialize, Serialize, Debug)]
pub struct GetAgendaDetailsReq {
    pub id: i64,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct GetAgendaDetailsRes {
    pub agenda: EcAgendaRep,
    pub agenda_item_list: Vec<MergedAgendaItem>,
    pub agenda_item_d647_list: Vec<MergedAgendaItem>,
    pub partner_list: Vec<EcPartnerRep>,
    pub attachment_list: Vec<AttachmentRep>,
    pub status_histories: Vec<StatusHistoryRep>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MergedAgendaItem {
    pub plan: PlanOrAmendmentRep,
    pub agenda_item: EcAgendaItemRep,
}

impl MergedAgendaItem {
    pub fn from_items(
        agenda_item: EcAgendaItem,
        plan: PlanOrAmendment,
        agenda_item_fields: Option<&[&str]>,
        plan_fields: Option<&[&str]>,
    ) -> Result<Self, SharedDbError> {
        use asez2_shared_db::DbAdaptor;
        Ok(MergedAgendaItem {
            agenda_item: EcAgendaItemRep::from_item(
                agenda_item,
                agenda_item_fields,
            ),
            plan: PlanOrAmendmentRep::from_item(plan, plan_fields),
        })
    }
}

impl ApiResponseData for GetAgendaDetailsRes {}

/// Запрос на подробности протокола. UUID и ID протокола.
#[derive(Deserialize, Serialize, Debug)]
pub struct GetProtocolDetailsReq {
    pub id: i64,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct ProtocolItemRelated {
    pub protocol_item: Calculated<EcProtocolItemRep>,
    pub plan: PlanOrAmendmentRep,
    pub agenda_item: Option<EcAgendaItemRep>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq, Clone)]
pub struct ProtocolDetailsItem {
    pub plan: PlanOrAmendmentRep,
    pub protocol_item: Calculated<EcProtocolItemRep>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct GetProtocolDetailsRes {
    pub protocol: EcProtocolRep,
    pub protocol_item_list: Vec<ProtocolDetailsItem>,
    pub protocol_item_d647_list: Vec<ProtocolDetailsItem>,
    pub partner_list: Vec<EcPartnerRep>,
    pub attachment_list: Vec<AttachmentRep>,
}

impl ApiResponseData for GetProtocolDetailsRes {}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UpdateAgendaRes {
    pub agenda: EcAgendaRep,
    pub items: Vec<MergedAgendaItem>,
    pub d647_items: Vec<MergedAgendaItem>,
    pub partner_list: Vec<EcPartnerRep>,
    pub attachment_list: Vec<AttachmentRep>,
}
impl ApiResponseData for UpdateAgendaRes {}

/// Вернуть Эксперту АЦ
#[derive(Deserialize, Serialize, Debug)]
pub struct ReturnToExpertReq {
    pub user_id: i32,
    pub section_id: Section,
    pub item_list: Vec<ReturnToSomeoneItem>,
    pub is_force: bool,
}

/// Ответ на запрос [`ReturnToExpertReq`]
pub type ReturnToExpertResponseData = PaginatedData<PlanOrAmendmentRep>;

impl TryFrom<(EsUpdatePlansReq, i32)> for PrUpdatePlansReq {
    type Error = uuid::Error;

    fn try_from(
        (s, user_id): (EsUpdatePlansReq, i32),
    ) -> Result<Self, Self::Error> {
        // The update data comes without a UUID.
        // However, the update functionality wants a uuid.
        let mut new_plan = s.data;
        *new_plan.uuid_mut() = Some(uuid::Uuid::parse_str(&s.uuid)?);

        Ok(Self {
            user_id,
            fields: s.fields,
            plans: vec![new_plan],
        })
    }
}

pub type ExportReq = GeneralExportReq<Section, Select>;

//запрос количества сущностей для секций СК
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSectionsCountRequest {
    pub section_list: Vec<Section>,
}
//ответ на запрос количества сущностей для секций СК
#[derive(Default, Deserialize, Serialize, Debug, sqlx::FromRow, PartialEq)]
pub struct GetSectionsCountResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_person_commission: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correspondence_commission: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_commission_required: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preparation_for_in_person_commission: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summing_up_in_person_commission_results: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summing_up_correspondence_commission_results: Option<usize>,
}
impl ApiResponseData for GetSectionsCountResponse {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPartnersReq {
    pub protocol_type_id: ProtocolType,
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GetPartnersResponseData {
    pub item_list: Vec<GetPartnersResponseItem>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GetPartnersResponseItem {
    /// UUID партнера
    pub uuid: Uuid,
    /// Код пользователя
    pub user_id: i32,
    /// Роль пользователя на Сметной комиссии
    pub commission_role_id: i16,
}

impl ApiResponseData for GetPartnersResponseData {}

pub mod price_analysis {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use asez2_shared_db::db_item::{AsezDate, AsezTimestamp, Select};
    use asez2_tables::legacy::plans::PlanStatus;
    use asez2_tables::plan_amendment::PlanOrAmendmentItemsRep;
    use asez2_tables::processing::price_analysis_user::{
        PriceAnalysisUser, UserType,
    };
    use asez2_tables::{
        ExpertConclusionId, PlanOrAmendmentRep, PlanRep, PlanVersionRep,
    };
    use asez2_tables::{PricingUnitId, Section};

    use crate::presentation::dto::general::DataRecords;
    use crate::presentation::dto::{
        general::{
            ObjectIdentifier, ObjectIdentifierList, ObjectIdentifierWithStatusNote,
        },
        processing::{
            AttachmentRep, CalculatedPlanRep, ContractAmendmentItemRep,
            ContractAmendmentRep, ContractAmendmentVersionRep, PlanItemFullRep,
        },
    };

    use super::{ApiResponseData, UserIdWrapper};

    // Согласовать утверждение цены
    #[derive(Deserialize, Serialize, Debug)]
    pub struct ApproveByChiefReq {
        pub user_id: i32,
        pub item_list: Vec<ObjectIdentifierWithStatusNote>,
    }
    pub type ApproveByChiefResponseData = Vec<PlanOrAmendmentRep>;

    /// Предзапрос на действие "Вернуть Заказчику"
    pub type PreReturnToCustomerReq = ObjectIdentifierList;
    /// Ответ на [`ReturnToCustomerReq`]
    pub type PreReturnToCustomerResponseData = Vec<PlanOrAmendmentRep>;

    /// Запрос на действие "Вернуть Заказчику"
    #[derive(Deserialize, Serialize, Debug)]
    pub struct ReturnToCustomerReq {
        pub user_id: i32,
        pub item_list: Vec<ObjectIdentifierWithStatusNote>,
    }
    /// Ответ на [`ReturnToCustomerReq`]
    pub type ReturnToCustomerResponseData = Vec<PlanOrAmendmentRep>;

    /// Запрос действия "Документация проверена"
    #[derive(Deserialize, Serialize, Debug)]
    pub struct DocumentationCheckedReq {
        pub user_id: i32,
        pub item_list: Vec<ObjectIdentifier>,
    }
    /// Ответ на [`DocumentationCheckedReq`]
    pub type DocumentationCheckedResponseData = Vec<PlanOrAmendmentRep>;

    /// Предзапрос списка ППЗ/ДС для "Запроса документации"
    pub type PreRequestDocumentationReq = ObjectIdentifierList;
    /// Ответ на [`PreRequestDocumentationReq`]
    pub type PreRequestDocumentationResponseData = Vec<PlanOrAmendmentRep>;

    /// Запрос действия "Запросить документацию"
    #[derive(Deserialize, Serialize, Debug)]
    pub struct RequestDocumentationReq {
        pub user_id: i32,
        pub item_list: Vec<ObjectIdentifierWithStatusNote>,
    }

    /// Ответ на [`RequestDocumentationReq`]
    pub type RequestDocumentationResponseData = Vec<PlanOrAmendmentRep>;

    /// Предзапрос списка ППЗ/ДС для "Массового назначения"
    pub type PreRequestDocumentsForExpertReq = ObjectIdentifierList;
    /// Ответ на [`PreRequestDocumentsForExpertReq`]
    pub type PreRequestDocumentsForExpertResponseData = Vec<PlanOrAmendmentRep>;

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct PriceDeterminedReq {
        pub user_id: i32,
        pub item_list: Vec<ObjectIdentifier>,
    }
    pub type PriceDeterminedResponseData = Vec<PlanOrAmendmentRep>;

    /// Предзапрос списка ППЗ/ДС для "Отклонить утверждение цены/Вернуть эксперту АЦ"
    pub type PreDeclineByChiefReq = ObjectIdentifierList;
    /// Ответ на [`PreDeclineByChiefReq`]
    pub type PreDeclineByChiefResponseData = Vec<PlanOrAmendmentRep>;

    /// Запрос действия "Отклонить утверждение цены/Вернуть эксперту АЦ"
    #[derive(Deserialize, Serialize, Debug)]
    pub struct DeclineByChiefReq {
        pub user_id: i32,
        pub item_list: Vec<ObjectIdentifierWithStatusNote>,
    }
    /// Ответ на [`DeclineByChiefReq`]
    pub type DeclineByChiefResponseData = Vec<PlanOrAmendmentRep>;

    /// Запрос на получение повестки по плану
    #[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
    pub struct GetPlansWithLastAgendaItemsReq {
        pub plans_uuid: Vec<Uuid>,
    }

    #[derive(Default, Deserialize, Serialize, Debug)]
    pub struct GetPlansWithLastAgendaItemsRes {
        /// Ключ - UUID ППЗ/ДС, значение - UUID agenda item
        pub last_agenda_item_hashmap: HashMap<Uuid, Uuid>,
    }
    impl ApiResponseData for GetPlansWithLastAgendaItemsRes {}

    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct UpdatePlanReq {
        #[serde(flatten)]
        pub plan: PlanRep,
        pub item_list: Vec<PlanItemFullRep>,
        pub pricing_attachment_list: Vec<AttachmentRep>,
    }

    pub type UpdatePlanResponseData = ();

    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct UpdateContractAmendmentReq {
        #[serde(flatten)]
        pub contract_amendment: ContractAmendmentRep,
        pub item_list: Vec<ContractAmendmentItemRep>,
        pub pricing_attachment_list: Vec<AttachmentRep>,
    }
    pub type UpdateContractAmendmentResponseData = ();

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct GetSectionsCountRequest {
        pub section_list: Vec<Section>,
        pub user_type: UserType,
        pub user_id: i32,
        pub departments: Vec<PricingUnitId>,
    }

    #[derive(Default, Deserialize, Serialize, Debug, sqlx::FromRow, PartialEq)]
    pub struct GetSectionsCountResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub assign_expert: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub determine_price: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub primary_expert_control: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub approve_price: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lotting_mtr: Option<i64>,
    }
    impl ApiResponseData for GetSectionsCountResponse {}

    pub type PricingResultReq = ObjectIdentifier;
    pub type PricingResultResponseData = CalculatedPlanRep;
    impl ApiResponseData for PricingResultResponseData {}

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct GetPriceAnalysisUsersReq {
        /// Выборка пользователей по определенных user_id
        pub user_ids: Option<Vec<i32>>,
        pub user_types: Option<Vec<UserType>>,
        pub unit_ids: Option<Vec<PricingUnitId>>,
    }
    pub type GetPriceAnalysisUsersResponseData = Vec<PriceAnalysisUser>;

    /// Запрос на экспорт позиций спецификации
    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct ExportSpecificationReq {
        #[serde(flatten)]
        pub object_identifier: ObjectIdentifier,
        #[serde(default)]
        pub item_id_list: Vec<ExportSpecificationItem>,
        #[serde(default)]
        pub field_configuration: Vec<ExportSpecificationField>,
        #[serde(default)]
        pub token: String,
        #[serde(default)]
        pub user_id: i32,
    }
    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct ExportSpecificationItem {
        pub uuid: Uuid,
    }
    #[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
    pub struct ExportSpecificationField {
        /// Идентификатор поля
        pub field_id: String,
        /// Заголовок столбца
        pub header_name: String,
        /// Участие поля в селекте
        pub is_select_field: bool,
    }

    /// Запрос на импорт.
    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct ImportReq {
        #[serde(flatten)]
        pub object_identifier: ObjectIdentifier,
        pub file_name: String,
        pub token: String,
        pub user_id: i32,
        pub data_records: DataRecords,
        #[serde(default)]
        pub is_registered_by_d647: Option<bool>,
    }
    /// Ответ на запрос импорта позиций спецификации
    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct ImportSpecificationResponseData {
        pub item_list: PlanOrAmendmentItemsRep,
    }

    impl ApiResponseData for ImportSpecificationResponseData {}

    /// Запрос на отчет по процедурам АЦ
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    pub struct PricingReportRequest {
        pub section: Section,
        pub select: Select,
        pub user_id: i32,
        pub start_date: AsezDate,
        pub end_date: AsezDate,
    }

    /// Ответ для отчета по процедурам АЦ.
    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct PricingReportResData {
        pub plans: Vec<PlanRep>,
        pub plans_versions: Vec<PlanVersionRep>,
        pub contract_amendments: Vec<ContractAmendmentRep>,
        pub contract_amendments_versions: Vec<ContractAmendmentVersionRep>,
    }

    impl ApiResponseData for PricingReportResData {}

    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    pub struct UpdatePlanRetrospectiveReq {
        pub uuid: Uuid,
        pub id: i64,
        pub plng_year: i16,
        pub status: PlanStatus,
        pub id_ly: i64,
        pub uuid_ly: Uuid,
    }

    pub type UpdatePlanRetrospectiveResponseData = ();

    /// "POST /rest/pricing/v1/action/complete_lotting/"
    pub type CompleteLottingRequest = UserIdWrapper<ObjectIdentifierList>;

    /// Данные которые передаются из процессинга в сервис АЦ, в основном для
    /// создания оповещения. ("POST /rest/pricing/v1/action/complete_lotting/")
    #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
    pub struct CompleteLottingData {
        /// Смесь руководителей, исполнительный, и тех кого на оповестить.
        pub users: Vec<PriceAnalysisUser>,
        /// ППЗ из которых нада черпать данные.
        pub plan_data: Vec<PlanRep>,
    }

    impl ApiResponseData for CompleteLottingData {}

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct ReviewProgressReq {
        pub id: i64,
        pub uuid: Uuid,
    }

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct ReviewProgressItem {
        /// ППЗ/ДС
        #[serde(flatten)]
        pub object: ObjectIdentifier,
        /// Идентификатор эксперта АЦ
        pub pricing_expert_id: i32,
        /// Дата поступления на рассмотрение
        pub receipt_date: AsezTimestamp,
        /// Дата рассмотрения
        pub consideration_date: AsezTimestamp,
        /// Комментарий, текущая ситуация
        pub comment: String,
        /// Решение эксперта АЦ
        pub expert_conclusion_id: ExpertConclusionId,
    }

    pub type ReviewProgressResp = Vec<ReviewProgressItem>;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MergedAgendaOrProtocolItem {
    AgendaItems(Vec<MergedAgendaItem>),
    ProtocolItems(Vec<ProtocolDetailsItem>),
}
impl Default for MergedAgendaOrProtocolItem {
    fn default() -> Self {
        MergedAgendaOrProtocolItem::AgendaItems(vec![])
    }
}

/// Ответ на запрос импорта списка ППЗ/ДС, реестра ППЗ/ДС в Повестку и Протокол
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ImportItemListSpecificResponseData {
    pub item_list: MergedAgendaOrProtocolItem,
}
impl ApiResponseData for ImportItemListSpecificResponseData {}

pub type GetRetrospectiveReq = ObjectIdentifierList;

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct GetRetrospectiveResponseData {
    pub item_list: Vec<MergedPlanRetrospective>,
}
impl ApiResponseData for GetRetrospectiveResponseData {}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct MergedPlanRetrospective {
    pub plan: PlanOrAmendmentRep,
    pub retrospective: PlanRetrospectiveRep,
    pub status_history: StatusHistoryRep,
}
