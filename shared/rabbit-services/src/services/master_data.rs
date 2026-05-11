use std::{fmt::Debug, sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;
use shared_essential::presentation::dto::master_data::plan_reasons_cancel::PlanReasonCancel;

use crate::callbacks::AsezCallback;
use serde::Serialize;
use shared_essential::presentation::dto::master_data::request::SearchPlanReasonsCancelRabbitReq;
use shared_essential::presentation::dto::master_data::request::{
    RouteCopyReq, SearchByDepartmentReq,
};
use shared_essential::presentation::dto::master_data::response::RouteCopyResponse;
use shared_essential::presentation::dto::{
    master_data::{
        error::MasterDataError,
        request::{
            MasterDataAction, MasterDataSearchRequest, RouteCreateReq,
            RouteDetailsReq, RouteFindReq, RouteListReq, RouteRemoveReq,
            RouteStartReq, RouteStopReq, RouteUpdateReq, SearchByIdReq,
        },
        response::{
            MasterDataSearchResponse, OrgUserAssignmentSearchResponse,
            RouteCreateResponse, RouteDetailsResponse, RouteFindResponse,
            RouteListResponse, RouteRemoveResponse, RouteStartResponse,
            RouteStopResponse, RouteUpdateResponse,
        },
    },
    response_request::ApiResponse,
    AsezResult, Source,
};

use super::{AsezRabbitProperties, AsezRabbitRouting, AsezRabbitService};

/// # Описание
///
/// Сервис НСИ - Reference Information(Master Data)
///
/// # API
/// 1. [`MasterDataService::get_dictionaries`] - Получение cправочников
#[derive(Debug, Clone)]
pub struct MasterDataService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for MasterDataService {
    const SERVICE: Source = Source::MasterData;

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

macro_rules! service_call {
    ($(#[$meta:meta])* $name:ident($req:ty) -> $res:ty;) => {
        $(#[$meta])*
        pub async fn $name(&self, dto: $req) -> AsezResult<$res> {
            let response = self
                .service_request(
                    dto,
                    self.rabbit_properties.clone(),
                    AsezRabbitRouting::RequestDictionaries,
                    Duration::from_millis(Self::DEFAULT_TIMEOUT),
                )
                .await
                .map_err(MasterDataError::from)?;
            response.content
        }
    };
    ($(#[$meta:meta])* $name:ident($req:ty as $selector:path) -> $res:ty;) => {
        $(#[$meta])*
        pub async fn $name(&self, dto: $req) -> AsezResult<$res> {
            let response = self
                .service_request(
                    $selector(dto),
                    self.rabbit_properties.clone(),
                    AsezRabbitRouting::MasterDataAction,
                    Duration::from_millis(Self::DEFAULT_TIMEOUT),
                )
                .await
                .map_err(MasterDataError::from)?;
            response.content
        }
    };
}

impl MasterDataService {
    const DEFAULT_TIMEOUT: u64 = 10_000;

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

    service_call!(
        /// Получение справочников от сервиса НСИ
        ///
        /// # Возвращает
        /// * Ok([`MasterDataSearchResponse`]) - Успешное получение справочников
        /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса в `master-data`
        get_dictionaries(MasterDataSearchRequest) -> MasterDataSearchResponse;
    );

    service_call!(
        /// Активация маршрутов
        route_start(RouteStartReq as MasterDataAction::RouteStart) -> ApiResponse<RouteStartResponse, ()>;
    );

    service_call!(
        /// Деактивация маршрутов
        route_stop(RouteStopReq as MasterDataAction::RouteStop) -> ApiResponse<RouteStopResponse, ()>;
    );

    service_call!(
        /// Удаление маршрутов
        route_remove(RouteRemoveReq as MasterDataAction::RouteRemove) -> ApiResponse<RouteRemoveResponse, ()>;
    );

    service_call!(
        /// Копирование маршрута
        route_copy(RouteCopyReq as MasterDataAction::RouteCopy) -> ApiResponse<RouteCopyResponse, ()>;
    );

    service_call!(
        /// Список маршрутов.
        get_route_list(RouteListReq as MasterDataAction::RouteList) -> ApiResponse<RouteListResponse, ()>;
    );

    service_call!(
        /// Детали маршрута.
        get_route_details(RouteDetailsReq as MasterDataAction::RouteDetails) -> ApiResponse<RouteDetailsResponse, ()>;
    );

    service_call!(
        /// Создание маршрута.
        route_create(RouteCreateReq as MasterDataAction::RouteCreate) -> ApiResponse<RouteCreateResponse, ()>;
    );

    service_call!(
        /// Изменение/создание маршрута.
        route_update(RouteUpdateReq as MasterDataAction::RouteUpdate ) -> ApiResponse<RouteUpdateResponse, ()>;
    );

    /// Поиск подходящего маршрута.
    ///
    /// Для каждого элемента `dto` возвращяется вектор из данных,
    /// ассоциированных с маршрутами, которые соответствуют этому элементу.
    pub async fn find_routes<T>(
        &self,
        dto: RouteFindReq<T>,
    ) -> AsezResult<ApiResponse<RouteFindResponse, ()>>
    where
        T: Serialize + Send + Sync,
    {
        let response = self
            .service_request(
                dto,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::Routing,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(MasterDataError::from)?;
        response.content
    }

    service_call!(
        org_user_assignment_search_by_id(
            SearchByIdReq as MasterDataAction::OrganizationUserAssignmentSearchById
        ) -> ApiResponse<OrgUserAssignmentSearchResponse, ()>;
    );

    service_call!(
        org_user_assignment_search_by_department(
            SearchByDepartmentReq as MasterDataAction::OrganizationUserAssignmentSearchByDepartment
        ) -> ApiResponse<OrgUserAssignmentSearchResponse, ()>;
    );

    service_call!(
        /// Детали причины аннулирования
        plan_reasons_cancel_search(SearchPlanReasonsCancelRabbitReq as MasterDataAction::SearchPlanReasonCancel) -> ApiResponse<Vec<PlanReasonCancel>, ()>;
    );
}

from_request!(MasterDataService);
