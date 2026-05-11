use std::fmt::Debug;
use std::{sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;

use crate::callbacks::AsezCallback;
use crate::properties::AsezRabbitProperties;
use shared_essential::{
    domain::EcProtocolRep,
    presentation::dto::{
        estimated_commission::{
            UpdateAgendaReqWithUser, UpdateProtocolReqWithUser,
        },
        general::DataRecords,
        processing::{
            price_analysis::{
                CompleteLottingData, ExportSpecificationReq, ImportReq,
                ImportSpecificationResponseData, PricingReportRequest,
                PricingReportResData,
            },
            *,
        },
        response_request::{ApiResponse, PaginatedData},
        AsezResult, Source,
    },
};

use super::{AsezRabbitRouting, AsezRabbitService};

/// # Описание
///
/// Сервис процессинга
///
/// # API
///
/// ## Общие операции по ППЗ/ДС
/// * [`ProcessingService::get_plans`] - Получение ППЗ/ДС
/// * [`ProcessingService::get_complete_plans`] - Получение полных ППЗ/ДС с их элементами(plan_item)
/// * [`ProcessingService::get_plans_with_last_agenda_items`] - Поиск повесток по ППЗ/ДС
/// * [`ProcessingService::update_plans`] - Обновление ППЗ/ДС с возвращением обновленных ППЗ/ДС
///
/// ## Общие операции по прочим данным
///
/// [`ProcessingService::get_price_analysis_user`] - Получение всех записей по пользователю АЦ
/// [`ProcessingService::get_attachments_meta`] - Запрос мета-данных по аттачам для ППЗ/ДС для дальнейшего формирования запроса в монолит и получения архива
///
/// ## Частные операции по ППЗ/ДС
///
/// ### СК
///
/// * [`ProcessingService::pre_cancel_plans`] - Предзапрос на аннулирование ППЗ/ДС
/// * [`ProcessingService::cancel_plans`] - Аннулирования ППЗ/ДС
/// * [`ProcessingService::pre_change_form`] - Предзапрос на изменение формы
/// * [`ProcessingService::change_form`] - Запрос на изменение формы
/// * [`ProcessingService::pre_return_to_customer`] - Предзапрос на возврат заказчику
/// * [`ProcessingService::return_to_customer`] - Возврат заказчику
/// * [`ProcessingService::pre_change_commission_date`] - Предзапрос списка ППЗ/ДС, для которых требуется изменить дату очной СК
/// * [`ProcessingService::change_commission_date`] - Изменить дату очной СК
/// * [`ProcessingService::pre_approve_plans`] - Предзапрос списка ППЗ/ДС, которые требуется Утвердить
/// * [`ProcessingService::approve_plans`] - Запрос на утверждение списка ППЗ/ДС
///
/// ## Операции с Повесткой СК
///
/// * [`ProcessingService::pre_create_agenda`] - Предзапрос на создание Повестки СК
/// * [`ProcessingService::create_agenda`] - Создание Повестки СК
/// * [`ProcessingService::pre_add_plans_agenda`] - Предзапрос на добавление ППЗ/ДС в Повестку СК
/// * [`ProcessingService::add_plans_agenda`] - Добавление ППЗ/ДС в Повестку СК
/// * [`ProcessingService::update_agenda`] - Обновить Повестку
/// * [`ProcessingService::pre_transfer_plans_agenda`] - Предзапрос на перемещение ППЗ/ДС между Повестками СК
/// * [`ProcessingService::pre_request_agenda_remove`] - Предзапрос списка ППЗ/ДС на удаление
/// * [`ProcessingService::agenda_remove`] - Запрос списка ППЗ/ДС на удаление
/// * [`ProcessingService::pre_agenda_items_remove`] - Предзапрос удаления позиций повестки.
/// * [`ProcessingService::get_agenda_list`] - Получение списка Повесток СК
/// * [`ProcessingService::get_agenda_list_by_date`] - Получение списка Повесток СК по дате
/// * [`ProcessingService::get_agenda_items_for_protocol_create`] - Получение данных по не удаленным ППЗ/ДС, которые включены в Повестку и еще не включены в Протокол.
/// * [`ProcessingService::get_agenda_items_by_id_range`] - Предзапрос данных по ППЗ/ДС для последующего включения в Повестку СК
/// * [`ProcessingService::get_agenda_details`] - Запрос на подробности по одной повестки.
/// * [`ProcessingService::get_item_list`] - Получение списка элементов Повестки или Протокола СК
///
/// ## Операции с Протоколом СК
///
/// * [`ProcessingService::pre_create_protocol`] - Предзапрос на создание Протокола СК
/// * [`ProcessingService::create_protocol`] - Создание Протокола СК
/// * [`ProcessingService::pre_add_plans_protocol`] - Предзапрос на добавлениe ППЗ/ДС в Протокол
/// * [`ProcessingService::add_plans_protocol`] - Запрос на добавлениe ППЗ/ДС в Протокол
/// * [`ProcessingService::update_protocol`] - Обновить Протокол
/// * [`ProcessingService::pre_remove_protocol`] - Предзапрос на удаление Протокола СК
/// * [`ProcessingService::remove_protocol`] - Удаление Протокола СК
/// * [`ProcessingService::pre_send_protocol_for_signing`] - Предзапрос на отправку протокола на подписание
/// * [`ProcessingService::send_protocol_for_signing`] - Отправка протокола на подписание
/// * [`ProcessingService::pre_approve_protocol`] - Предзапрос на утверждение протокола
/// * [`ProcessingService::approve_protocol`] - Отправка протокола на утверждение
/// * [`ProcessingService::confirm_decision`] - Подтверждение решения СК в Протоколе очной СК
/// * [`ProcessingService::pre_request_protocol_agreement`] - Предзапрос отправки протокола на согласование
/// * [`ProcessingService::action_protocol_agreement`] - Отправить протокол на согласование
/// * [`ProcessingService::get_item_list`] - Получение списка элементов Повестки или Протокола СК
/// * [`ProcessingService::get_protocol_list`] - Получение списка Протоколов СК
/// * [`ProcessingService::get_protocol_list_by_agenda`] - Получение списка Протоколов СК по Повестке СК
/// * [`ProcessingService::get_protocol_details`] - Запрос на подробности по одному протоколу.
/// * [`ProcessingService::get_protocol_items_by_id_range`] - Получение списка ППЗ/ДС, включенных в Протокол по диапазону идентификаторов ППЗ/ДС
///
/// ## Операции с Партнером СК
/// * [`ProcessingService::get_partners`] - Получение партнеров СК по типу Протокола СК
///
/// ### АЦ
///
/// * [`ProcessingService::pa_return_to_customer`] - Возврат заказчику, АЦ
/// * [`ProcessingService::pa_price_determined`] - Цена утверждена
/// * [`ProcessingService::pa_approve_by_chief`] - Согласовать утверждение цены
/// * [`ProcessingService::pa_pre_decline_by_chief`] - Предзапрос "Вернуть Эксперту АЦ/Отклонить утверждение цены"
/// * [`ProcessingService::pa_decline_by_chief`] - Запрос "Вернуть Эксперту АЦ/Отклонить утверждение цены"
/// * [`ProcessingService::pa_pricing_result`] - Получение ППЗ/ДС для отображения в форме "Результат определения цены"
/// * [`ProcessingService::pa_pre_return_to_customer`] - Предзапрос на возврат заказчику, АЦ
/// * [`ProcessingService::pa_documentation_checked`] - Запрос действия "Документация проверена"
/// * [`ProcessingService::pa_complete_lotting`] - Завершить 'лотирование'.
///
/// ## Экспорт ППЗ/ДС
/// * [`ProcessingService::request_export_data`] - Запрос на экспорт данных
///
#[derive(Debug, Clone)]
pub struct ProcessingService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

type ProcessingRes<T> = AsezResult<ApiResponse<T, ()>>;
type ProcessingResAdv<T, P> = AsezResult<ApiResponse<T, P>>;

impl AsezRabbitService for ProcessingService {
    const SERVICE: Source = Source::Processing;

    fn adapter(&self) -> &RabbitAdapter {
        &self.rabbit_adapter
    }

    fn service_caller(&self) -> Source {
        self.service_caller
    }

    fn callbacks(&self) -> &[AsezCallback] {
        &self.callbacks
    }

    fn with_callback(mut self, callback: AsezCallback) -> Self {
        self.callbacks.push(callback);
        self
    }
}

impl ProcessingService {
    const DEFAULT_TIMEOUT: u64 = 60_000;
    pub const DEFAULT_EXPIRATION: u64 = 70_000;

    pub fn new(
        rabbit_adapter: Arc<RabbitAdapter>,
        rabbit_properties: AsezRabbitProperties,
        service_caller: Source,
    ) -> Self {
        Self {
            rabbit_adapter,
            rabbit_properties,
            service_caller,
            callbacks: Vec::new(),
        }
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PaginatedData<PlanRep>`], ()>) - Массив ППЗ/ДС с конкретными колонками, которые были запрошены
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_plans(
        &self,
        dto: PlansRequest,
    ) -> ProcessingRes<GetPlansResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetPlans(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PaginatedData<PlanRep>`], ()>) - Массив ППЗ/ДС с конкретными колонками, которые были запрошены
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_plans_count(
        &self,
        dto: PlansCountRequest,
    ) -> ProcessingRes<GetExpertPlansCountData> {
        let response = self
            .service_request(
                ProcessingRequest::GetPlansCount(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения ППЗ/ДС c их элементами(plan_item)
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PaginatedData<CompletePlanRep>`], ()>) - Массив ППЗ/ДС с конкретными колонками, которые были запрошены
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_complete_plans(
        &self,
        dto: CompletePlansRequest,
    ) -> ProcessingRes<GetPlanResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetCompletePlans(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения ДС c их элементами(contract_amendment_item)
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PaginatedData<CompleteContractAmendmentRep>`], ()>) - Массив ДС с конкретными колонками, которые были запрошены
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_complete_contract_amendments(
        &self,
        dto: CompletePlansRequest,
    ) -> ProcessingRes<GetContractAmendmentResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetCompleteContractAmendments(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn get_plan_version(
        &self,
        dto: PlanVersionRequest,
    ) -> ProcessingRes<GetPlanVersionResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetPlanVersion(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn get_contract_amendment_version(
        &self,
        dto: PlanVersionRequest,
    ) -> ProcessingRes<GetContractAmendmentVersionResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetContractAmendmentVersion(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Запрос мета-данных по аттачам для ППЗ/ДС для дальнейшего формирования запроса в монолит и получения архива
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetAttachmentsMetaResponseData`], ()>) - Массив ППЗ/ДС с аттачментами по каждому
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_attachments_meta(
        &self,
        dto: GetAttachmentsMetaRequest,
    ) -> ProcessingRes<GetAttachmentsMetaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetAttachmentsMeta(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения списка Повесток СК
    pub async fn get_agenda_details(
        &self,
        dto: GetAgendaDetailsReq,
    ) -> ProcessingRes<GetAgendaDetailsRes> {
        let response = self
            .service_request(
                ProcessingRequest::GetAgendaDetails(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения списка Повесток СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetAgendaListResponseData`], ()>) - Массив Повесток СК с конкретными колонками, которые были запрошены
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке при процессинге в `processing`
    pub async fn get_agenda_list(
        &self,
        dto: GetAgendaListReq,
    ) -> ProcessingRes<GetAgendaListResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetAgendaList(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения списка Повесток СК по дате
    ///
    /// # Возвращает
    ///
    pub async fn get_agenda_list_by_date(
        &self,
        dto: GetAgendaListByDateReq,
    ) -> ProcessingRes<GetAgendaListByDateResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetAgendaListByDate(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;

        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения списка элементов Повестки или Протокола СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetItemListResponseData`], ()>) - Массив уже отфильтрованных данных под определенную
    /// секцию
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке при процессинге в `processing`
    pub async fn get_item_list(
        &self,
        dto: GetItemListReq,
    ) -> ProcessingRes<GetItemListResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetAgendaItemList(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT + 2000),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения подробностей одного протокола СК
    pub async fn get_protocol_details(
        &self,
        dto: GetProtocolDetailsReq,
    ) -> ProcessingRes<GetProtocolDetailsRes> {
        let response = self
            .service_request(
                ProcessingRequest::GetProtocolDetails(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для обновления одного протокола СК
    pub async fn update_protocol(
        &self,
        dto: UpdateProtocolReqWithUser,
    ) -> ProcessingRes<UpdateProtocolRes> {
        let response = self
            .service_request(
                ProcessingRequest::UpdateProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для обновления одной повестки СК
    pub async fn update_agenda(
        &self,
        dto: UpdateAgendaReqWithUser,
    ) -> ProcessingRes<UpdateAgendaRes> {
        let response = self
            .service_request(
                ProcessingRequest::UpdateAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения перечня Протоколов
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetItemListResponseData`], ()>) - Массив уже отфильтрованных данных под определенную
    /// секцию
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке при процессинге в `processing`
    pub async fn get_protocol_list(
        &self,
        dto: GetProtocolListReq,
    ) -> ProcessingRes<GetProtocolListResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetProtocolList(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Получение списка Протоколов СК по Повестке сК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetProtocolListByAgendaResponseData`], ()>) - Список Протоколов по Повестке СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке при процессинге в `processing`
    pub async fn get_protocol_list_by_agenda(
        &self,
        dto: GetProtocolListByAgendaReq,
    ) -> ProcessingRes<GetProtocolListByAgendaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetProtocolListByAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для получения перечня Протоколов на заданную дату.
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetItemListResponseByDateData`], ()>) - todo
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке при процессинге в `processing`
    pub async fn get_protocol_list_by_date(
        &self,
        dto: GetProtocolListByDateReq,
    ) -> ProcessingRes<GetProtocolListByDateResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetProtocolListByDate(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Предзапрос данных по ППЗ/ДС для последующего включения в Повестку СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetAgendaItemsByIdRangeResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_agenda_items_by_id_range(
        &self,
        dto: GetAgendaItemsByIdRangeReq,
    ) -> ProcessingRes<GetAgendaItemsByIdRangeResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetAgendaItemsByIdRange(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Предзапрос данных по ППЗ/ДС для последующего включения в Повестку СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetAgendaItemsByIdRangeResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_protocol_items_by_id_range(
        &self,
        dto: GetProtocolItemsByIdRangeReq,
    ) -> ProcessingRes<GetProtocolItemsByIdRangeResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetProtocolItemsByIdRange(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Получение данных по не удаленным ППЗ/ДС, которые
    /// включены в Повестку и еще не включены в Протокол.
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetAgendaItemsForProtocolCreateResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_agenda_items_for_protocol_create(
        &self,
        dto: GetAgendaItemsForProtocolCreateReq,
    ) -> ProcessingRes<GetAgendaItemsForProtocolCreateResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetAgendaItemsForProtocolCreate(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Предзапрос данных по ППЗ/ДС для последующего включения в Повестку СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreAddPlansAgendaResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn pre_add_plans_agenda(
        &self,
        dto: PreAddPlansAgendaReq,
    ) -> ProcessingRes<PreAddPlansAgendaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreAddPlansAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для добавления ППЗ/ДС в Повестку СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`AddPlansAgendaResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn add_plans_agenda(
        &self,
        dto: AddPlansAgendaReq,
    ) -> ProcessingRes<AddPlansAgendaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::AddPlansAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Предзапрос на перемещение ППЗ/ДС между Повестками СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreTransferPlansAgendaReq`], ()>) - Массив ППЗ/ДС, которые могут быть перемещены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn pre_transfer_plans_agenda(
        &self,
        dto: PreTransferPlansAgendaReq,
    ) -> ProcessingRes<PreTransferPlansAgendaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreTransferPlansAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Перемещение ППЗ/ДС между Повестками СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`TransferPlansAgendaResponseData`], ()>) - Массив ППЗ/ДС, которые были перемещены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn transfer_plans_agenda(
        &self,
        dto: TransferPlansAgendaReq,
    ) -> ProcessingRes<TransferPlansAgendaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::TransferPlansAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Предзапрос данных по ППЗ/ДС для последующего включения в Протокол
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreAddPlansProtocolResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Протокол
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn pre_add_plans_protocol(
        &self,
        dto: PreAddPlansProtocolReq,
    ) -> ProcessingRes<PreAddPlansProtocolResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreAddPlansProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Запрос по добавлению элементов в Протокол СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`AddPlansProtocolResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Протокол
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn add_plans_protocol(
        &self,
        dto: AddPlansProtocolReq,
    ) -> ProcessingRes<AddPlansProtocolResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::AddPlansProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для проверки создания Повестки СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreCreateAgendaResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn pre_create_agenda(
        &self,
        dto: PreCreateAgendaReq,
    ) -> ProcessingRes<PreCreateAgendaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreCreateAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для создания Повестки СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`CreateAgendaResponseData`], ()>) - Массив айди ППЗ/ДС, которые были добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn create_agenda(
        &self,
        dto: CreateAgendaReq,
    ) -> ProcessingRes<CreateAgendaResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::CreateAgenda(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для проверки создания Протокола СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreCreateProtocolResponseData`], ()>) - Массив ППЗ/ДС, которые могут быть добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn pre_create_protocol(
        &self,
        dto: PreCreateProtocolReq,
    ) -> ProcessingRes<PreCreateProtocolResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreCreateProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для создания Повестки СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`CreateProtocolResponseData`], ()>) - Массив айди ППЗ/ДС, которые были добавлены в Повестку СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn create_protocol(
        &self,
        dto: CreateProtocolReq,
    ) -> ProcessingRes<CreateProtocolResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::CreateProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для обновления ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PrUpdatePlansResponseData`], ()>) - Массив ППЗ/ДС с конкретными полями, которые были запрошены
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn update_plans(
        &self,
        dto: PrUpdatePlansReq,
    ) -> ProcessingRes<PrUpdatePlansResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::UpdatePlans(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Пре-запрос к `Processing` для проверки возможности изменения формы СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreChangeFormResponseData`], ()>) - Массив с ануллированными ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_change_form(
        &self,
        dto: PreChangeFormReq,
    ) -> ProcessingRes<PreChangeFormResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreChangeForm(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для запроса изменения формы
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`ChangeFormResponseData`], ()>) - Список ППЗ/ДС для изменения формы
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn change_form(
        &self,
        dto: ChangeFormReq,
    ) -> ProcessingRes<ChangeFormResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ChangeForm(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Пре-запрос к `Processing` для проверки возможности ануллирования ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreCancelPlansResponseData`], ()>) - Массив с ануллированными ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_cancel_plans(
        &self,
        dto: PreCancelPlansReq,
    ) -> ProcessingRes<PreCancelPlansResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreCancelPlan(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для ануллирования ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`CancelPlansResponseData`], ()>) - Массив с ануллированными ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn cancel_plans(
        &self,
        dto: CancelPlansReq,
    ) -> ProcessingRes<CancelPlansResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::CancelPlan(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` чтобы утвердить протокол
    pub async fn approve_protocol(
        &self,
        dto: ApproveProtocolReq,
    ) -> ProcessingRes<PaginatedData<EcProtocolRep>> {
        let response = self
            .service_request(
                ProcessingRequest::ApproveProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` чтобы утвердить протокол, предварительная проверка.
    pub async fn pre_approve_protocol(
        &self,
        dto: PreApproveProtocolReq,
    ) -> ProcessingRes<PaginatedData<EcProtocolRep>> {
        let response = self
            .service_request(
                ProcessingRequest::PreApproveProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` чтобы Отправить протокол на подписание
    pub async fn send_protocol_for_signing(
        &self,
        dto: SignProtocolReq,
    ) -> ProcessingRes<PaginatedData<EcProtocolRep>> {
        let response = self
            .service_request(
                ProcessingRequest::SendProtocolForSigning(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(5000),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` чтобы Отправить протокол на подписание, предварительная проверка.
    pub async fn pre_send_protocol_for_signing(
        &self,
        items: PreSignProtocolReq,
    ) -> ProcessingRes<PaginatedData<EcProtocolRep>> {
        let response = self
            .service_request(
                ProcessingRequest::PreSendProtocolForSigning(items),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(200),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для возврата заказчику
    ///
    /// # Возвращает
    /// * Ok([`ReturnToCustomerResponse`]) - Массив ППЗ/ДС в статусе "Доработка Заказчиком"
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn return_to_customer(
        &self,
        dto: ReturnToCustomerReq,
    ) -> ProcessingRes<ReturnToCustomerResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ReturnToCustomer(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для возврата заказчику
    ///
    /// # Возвращает
    /// * Ok([`ReturnToCustomerResponse`]) - Массив ППЗ/ДС в статусе "Доработка Заказчиком"
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_return_to_customer(
        &self,
        dto: PreReturnToCustomerReq,
    ) -> ProcessingRes<PreReturnToCustomerResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreReturnToCustomer(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса списка ППЗ/ДС на удаление
    ///
    /// # Возвращает
    /// * Ok([`PreAgendaRemoveResponse`]) - Список ППЗ/ДС на удаление
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_request_agenda_remove(
        &self,
        dto: PreRequestAgendaRemoveReq,
    ) -> ProcessingRes<PreRequestAgendaRemoveResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreRequestAgendaRemove(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для удаления ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok([`AgendaRemoveResponse`]) - Список UUID удаленных ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn agenda_remove(
        &self,
        dto: AgendaRemoveReq,
    ) -> ProcessingRes<AgendaRemoveResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::AgendaRemove(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса списка повесток на отправку
    ///
    /// # Возвращает
    /// * Ok([`PreAgendaSendResponse`]) - Список повесток на отправку
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_agenda_send(
        &self,
        dto: PreAgendaSendReq,
    ) -> ProcessingRes<PreAgendaSendResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreAgendaSend(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для отправки повесток
    ///
    /// # Возвращает
    /// * Ok([`AgendaSendResponse`]) - Список отправленных повесток
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn agenda_send(
        &self,
        dto: AgendaSendReq,
    ) -> ProcessingRes<AgendaSendResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::AgendaSend(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для удаления ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok([`AgendaRemoveResponse`]) - Список UUID удаленных ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_agenda_items_remove(
        &self,
        dto: PreRemoveAgendaItemsReq,
    ) -> ProcessingRes<PreRemoveAgendaItemsResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreRemoveAgendaItems(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса на удаление Протокола СК
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`PreRemoveProtocolResponseData`], ()>) - Список Протоколов СК, которые могут быть удалены
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_remove_protocol(
        &self,
        dto: PreRemoveProtocolReq,
    ) -> ProcessingRes<PreRemoveProtocolResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreRemoveProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для удаления Протокола СК
    ///
    /// # Возвращает
    /// * Ok([`RemoveProtocolResponse`]) - Список удаленных Протоколов СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn remove_protocol(
        &self,
        dto: RemoveProtocolReq,
    ) -> ProcessingRes<RemoveProtocolResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::RemoveProtocol(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса списка ППЗ/ДС, для которых требуется изменить дату очной СК
    ///
    /// # Возвращает
    /// * Ok([`PreChangeCommissionDateReq`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_change_commission_date(
        &self,
        dto: PreChangeCommissionDateReq,
    ) -> ProcessingRes<PreChangeCommissionDateResponse> {
        let response = self
            .service_request(
                ProcessingRequest::PreChangeCommissionDateReq(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для изменения даты очной СК
    ///
    /// # Возвращает
    /// * Ok([`ChangeCommissionDateReq`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn change_commission_date(
        &self,
        dto: ChangeCommissionDateReq,
    ) -> ProcessingRes<ChangeCommissionDateResponse> {
        let response = self
            .service_request(
                ProcessingRequest::ChangeCommissionDateReq(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса списка ППЗ/ДС, которые требуется Утвердить
    ///
    /// # Возвращает
    /// * Ok([`PreApprovePlansResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_approve_plans(
        &self,
        dto: PreApprovePlansReq,
    ) -> ProcessingRes<PreApprovePlansResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreApprovePlans(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для утверждения списка ППЗ/ДС
    ///
    /// # Возвращает
    /// * Ok([`ApprovePlansResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn approve_plans(
        &self,
        dto: ApprovePlansReq,
    ) -> ProcessingRes<ApprovePlansResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ApprovePlans(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Подтверждение решения СК в Протоколе очной СК
    ///
    /// # Возвращает
    /// * Ok([`()`]) - Удачное подтверждение решения СК
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn confirm_decision(
        &self,
        dto: ConfirmDecisionReq,
    ) -> ProcessingRes<ConfirmDecisionResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ConfirmDecision(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` для предзапроса списка ППЗ/ДС для последующего возврата Эксперту АЦ
    ///
    /// # Возвращает
    /// * Ok([`PreReturnToExpertResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_return_to_expert(
        &self,
        dto: PreReturnToExpertReq,
    ) -> ProcessingRes<PreReturnToExpertResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PreReturnToExpert(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` для запроса назначения Эксперта АЦ
    ///
    /// # Возвращает
    /// * Ok([`AsssignExpertResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn assign_expert(
        &self,
        dto: AssignExpertReq,
    ) -> ProcessingRes<AssignExpertResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::AssignExpert(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` для запроса массового назначения Эксперта АЦ
    ///
    /// # Возвращает
    /// * Ok([`AsssignExpertMassResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn assign_expert_mass(
        &self,
        dto: AssignExpertMassReq,
    ) -> ProcessingRes<AssignExpertMassResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::AssignExpertMass(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Обращение к `Processing` для возврата Эксперту АЦ
    ///
    /// # Возвращает
    /// * Ok([`ReturnToExpertResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn return_to_expert(
        &self,
        dto: ReturnToExpertReq,
    ) -> ProcessingRes<ReturnToExpertResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ReturnToExpert(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// Получение партнеров СК по типу Протокола СК
    pub async fn get_partners(
        &self,
        dto: GetPartnersReq,
    ) -> ProcessingRes<GetPartnersResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetPartners(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса на Возврат Заказчику, АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::ReturnToCustomerResponseData`]) - Список ППЗ/ДС, которые можно перевести на новый статус
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_pre_return_to_customer(
        &self,
        dto: price_analysis::PreReturnToCustomerReq,
    ) -> ProcessingRes<price_analysis::PreReturnToCustomerResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaPreReturnToCustomer(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для запроса на Возврат Заказчику, АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::ReturnToCustomerResponseData`]) - Список ППЗ/ДС, которые были переведены на новый статус
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_return_to_customer(
        &self,
        dto: price_analysis::ReturnToCustomerReq,
    ) -> ProcessingRes<price_analysis::ReturnToCustomerResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaReturnToCustomer(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для запроса действия "Документация проверена"
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::DocumentationCheckedResponseData`]) - Список ППЗ/ДС, у которых была проверена документация
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_documentation_checked(
        &self,
        dto: price_analysis::DocumentationCheckedReq,
    ) -> ProcessingRes<price_analysis::DocumentationCheckedResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaDocumentationChecked(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса отправки протоколов на согласование
    ///
    /// # Возвращает
    /// * Ok([`PreProtocolAgreementResponse`]) - Список протоколов на согласование
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pre_request_protocol_agreement(
        &self,
        dto: PreProtocolAgreementReq,
    ) -> ProcessingRes<PreProtocolAgreementResponse> {
        let response = self
            .service_request(
                ProcessingRequest::PreProtocolAgreement(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для отправки протоколов на согласование
    ///
    /// # Возвращает
    /// * Ok() - Список протоколов на согласование в messages
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn action_protocol_agreement(
        &self,
        dto: ProtocolAgreementReq,
    ) -> ProcessingRes<()> {
        let response = self
            .service_request(
                ProcessingRequest::ProtocolAgreement(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса ППЗ/ДС, для действия "Запросить документацию" в АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::PreRequestDocumentationResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_pre_request_documentation(
        &self,
        dto: price_analysis::PreRequestDocumentationReq,
    ) -> ProcessingRes<price_analysis::PreRequestDocumentationResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaPreRequestDocumentation(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для предзапроса ППЗ/ДС, для действия "Массовое назначение" в АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::PreRequestDocumentsForExpertResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_pre_request_documents_for_expert(
        &self,
        dto: price_analysis::PreRequestDocumentsForExpertReq,
    ) -> ProcessingRes<price_analysis::PreRequestDocumentsForExpertResponseData>
    {
        let response = self
            .service_request(
                ProcessingRequest::PaPreRequestDocumentsForExpert(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для действия "Запросить документацию" в АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::RequestDocumentationResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_request_documentation(
        &self,
        dto: price_analysis::RequestDocumentationReq,
    ) -> ProcessingRes<price_analysis::RequestDocumentationResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaRequestDocumentation(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для действия "Цена определена" в АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::PriceDeterminedResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_price_determined(
        &self,
        dto: price_analysis::PriceDeterminedReq,
    ) -> ProcessingRes<price_analysis::PriceDeterminedResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PriceDetermined(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` для действия "Согласовать утверждение цены" в АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::ApproveByChiefResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_approve_by_chief(
        &self,
        dto: price_analysis::ApproveByChiefReq,
    ) -> ProcessingRes<price_analysis::ApproveByChiefResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ApproveByChief(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` предзапрос для действия "Вернуть Эксперту АЦ/Отклонить утверждение цены"
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::PreDeclineByChiefResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_pre_decline_by_chief(
        &self,
        dto: price_analysis::PreDeclineByChiefReq,
    ) -> ProcessingRes<price_analysis::PreDeclineByChiefResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaPreDeclineByChief(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` действие "Вернуть Эксперту АЦ/Отклонить утверждение цены"
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::DeclineByChiefResponseData`]) - Список ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_decline_by_chief(
        &self,
        dto: price_analysis::DeclineByChiefReq,
    ) -> ProcessingRes<price_analysis::DeclineByChiefResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaDeclineByChief(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn pa_review_progress(
        &self,
        dto: price_analysis::ReviewProgressReq,
    ) -> ProcessingRes<price_analysis::ReviewProgressResp> {
        let response = self
            .service_request(
                ProcessingRequest::PaReviewProgress(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;

        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` действие "Получить последнюю не удаленную и не исключенную повестку по ППЗ/ДС"
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::GetAgendaByPlanRes`]) - Найденная не удаленная и не исключенная повестка
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn get_plans_with_last_agenda_items(
        &self,
        dto: price_analysis::GetPlansWithLastAgendaItemsReq,
    ) -> ProcessingRes<price_analysis::GetPlansWithLastAgendaItemsRes> {
        let response = self
            .service_request(
                ProcessingRequest::GetPlansWithLastAgendaItems(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn pa_update_plan(
        &self,
        dto: UserIdWrapper<price_analysis::UpdatePlanReq>,
    ) -> ProcessingRes<price_analysis::UpdatePlanResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaUpdatePlan(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn pa_update_contract_amendment(
        &self,
        dto: UserIdWrapper<price_analysis::UpdateContractAmendmentReq>,
    ) -> ProcessingRes<price_analysis::UpdateContractAmendmentResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PaUpdateContractAmendment(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn pa_get_sections_count(
        &self,
        dto: price_analysis::GetSectionsCountRequest,
    ) -> ProcessingRes<price_analysis::GetSectionsCountResponse> {
        let response = self
            .service_request(
                ProcessingRequest::PaGetSectionsCount(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn ec_get_sections_count(
        &self,
        dto: UserIdWrapper<GetSectionsCountRequest>,
    ) -> ProcessingRes<GetSectionsCountResponse> {
        let response = self
            .service_request(
                ProcessingRequest::EcGetSectionsCount(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` действие "Получение данных для отображения в форме Результат определения цены"
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::PricingResultReq`]) - ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_pricing_result(
        &self,
        dto: price_analysis::PricingResultReq,
    ) -> ProcessingRes<price_analysis::PricingResultResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PricingResult(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `Processing` действие "Завершить лотирование"
    ///
    /// # Возвращает
    /// * Ok(ProcessingRes<ObjectIdentifierList>)  
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_complete_lotting(
        &self,
        dto: price_analysis::CompleteLottingRequest,
    ) -> ProcessingResAdv<CompleteLottingData, ()> {
        let response = self
            .service_request(
                ProcessingRequest::PaCompleteLotting(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Получение всех записей по пользователю АЦ
    ///
    /// # Возвращает
    /// * Ok([`price_analysis::PricingResultReq`]) - ППЗ/ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn get_price_analysis_user(
        &self,
        dto: price_analysis::GetPriceAnalysisUsersReq,
    ) -> ProcessingRes<price_analysis::GetPriceAnalysisUsersResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::PricingUser(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Экспорт данных
    ///
    /// # Возвращает
    /// * Ok([`DataRecords`]) - Полный набор данных для вывода в отчет: Заголовок колонок + Строки данных
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn request_export_data(
        &self,
        dto: ExportReq,
    ) -> ProcessingRes<DataRecords> {
        let response = self
            .service_request(
                ProcessingRequest::ExportData(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Экспорт спецификации
    ///
    /// # Возвращает
    /// * Ok([`DataRecords`]) - Позиции спецификации
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn request_export_specification(
        &self,
        dto: ExportSpecificationReq,
    ) -> ProcessingRes<DataRecords> {
        let response = self
            .service_request(
                ProcessingRequest::ExportSpecification(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }
    /// # Описание
    ///
    /// Импорт спецификации
    ///
    /// # Возвращает
    /// * Ok([`ImportSpecificationResponseData`]) - Позиции спецификации
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn request_import_specification(
        &self,
        dto: ImportReq,
    ) -> ProcessingRes<ImportSpecificationResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ImportSpecification(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Импорт списка ППЗ/ДС в повестку, протокол
    ///
    /// # Возвращает
    /// * Ok([`ImportItemListSpecificResponseData`]) - Обновлённый список ППЗ, ДС
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn request_import_ec_item_list_specific(
        &self,
        dto: ImportReq,
    ) -> ProcessingRes<ImportItemListSpecificResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::ImportItemListSpecific(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn request_pricing_report_common(
        &self,
        dto: PricingReportRequest,
    ) -> ProcessingRes<PricingReportResData> {
        let response = self
            .service_request(
                ProcessingRequest::PricingReportCommon(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn request_pricing_report_savings(
        &self,
        dto: PricingReportRequest,
    ) -> ProcessingRes<PricingReportResData> {
        let response = self
            .service_request(
                ProcessingRequest::PricingReportSavings(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    pub async fn request_pricing_report_commission(
        &self,
        dto: PricingReportRequest,
    ) -> ProcessingRes<PricingReportResData> {
        let response = self
            .service_request(
                ProcessingRequest::PricingReportCommission(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Запрос на получение данных ретроспективных закупок на этапе Определения цены
    ///
    /// # Возвращает
    /// * Ok(ApiResponse<[`GetRetrospectiveResponseData`], ()>) - Cписок ретроспективных закупок
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или при процессинге в `processing`
    pub async fn get_retrospective(
        &self,
        dto: GetRetrospectiveReq,
    ) -> ProcessingRes<GetRetrospectiveResponseData> {
        let response = self
            .service_request(
                ProcessingRequest::GetRetrospective(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Processing,
                Duration::from_millis(ProcessingService::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ProcessingError::from)?;
        response.content
    }
}

from_request!(ProcessingService);
