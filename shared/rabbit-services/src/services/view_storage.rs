use shared_essential::domain::view_storage::notification::NotificationType;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

use super::{AsezRabbitProperties, AsezRabbitRouting, AsezRabbitService};
use crate::callbacks::AsezCallback;
use broker::rabbit::RabbitAdapter;
use shared_essential::presentation::dto::view_storage::{
    GetNotificationsByType, GetNotificationsByTypeResponse, NotificationStorageReq,
    PricingSectionUserResponse,
};
use shared_essential::presentation::dto::{
    view_storage::{
        AllowedFieldsRequest, AllowedFieldsResponse, AuthCheckReq, AuthCheckRes,
        CheckPermissionRequest, PricingSectionUserReq, ViewStorageError,
    },
    {AsezResult, Source},
};

/// # Описание
///
/// Сервис ракурсов
///
/// # API
/// 1. [`ViewStorageService::get_allowed_fields`] - Получение разрешенных полей для секции и воркплейса
/// 2. [`ViewStorageService::check_session`] - Проверка сессии пользователя
/// 3. [`ViewStorageService::check_permission`] - Проверка разрешения действия пользователем
#[derive(Clone, Debug)]
pub struct ViewStorageService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for ViewStorageService {
    const SERVICE: Source = Source::ViewStorage;

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

impl ViewStorageService {
    const DEFAULT_TIMEOUT: u64 = 10_000;
    pub const DEFAULT_EXPIRATION: u64 = 10_000;

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
    /// Обращение к `view-storage` сервиса для получения разрешенных полей для секции и воркплейса
    ///
    /// # Возвращает
    /// * Ok([`AllowedFieldsResponse`]) - Массив разрешенных полей для пользователя
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке
    /// внутри `view-storage`
    pub async fn get_allowed_fields(
        &self,
        dto: &AllowedFieldsRequest,
    ) -> AsezResult<AllowedFieldsResponse> {
        let response = self
            .service_request(
                dto,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::ViewsAllowedFields,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ViewStorageError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `view-storage` сервиса для для проверки сессии пользователя
    ///
    /// # Возвращает
    /// * Ok(['bool']) - Статус сессии (активна/неактивна)
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке
    /// внутри `view-storage`
    #[cfg(not(feature = "stub-view-storage"))]
    pub async fn check_session(&self, user_id: String) -> AsezResult<AuthCheckRes> {
        let auth_req = AuthCheckReq {
            user_id,
            token: None,
        };
        let response = self
            .service_request(
                auth_req,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::ViewsCheckSession,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ViewStorageError::from)?;
        response.content
    }

    #[cfg(feature = "stub-view-storage")]
    pub async fn check_session(&self, _user_id: String) -> AsezResult<bool> {
        Ok(true)
    }

    /// # Описание
    ///
    /// Обращение к `view-storage` сервиса для для проверки сессии пользователя
    ///
    /// # Возвращает
    /// * Ok(['bool']) - Статус сессии (активна/неактивна)
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке
    /// внутри `view-storage`
    pub async fn check_session_with_token(
        &self,
        user_id: String,
        token: Uuid,
    ) -> AsezResult<AuthCheckRes> {
        let auth_req = AuthCheckReq {
            user_id,
            token: Some(token),
        };
        let response = self
            .service_request(
                auth_req,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::ViewsCheckSession,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ViewStorageError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `view-storage` сервиса для проверки разрешения действия пользователем
    ///
    /// # Возвращает
    /// * Ok([`bool`]) - Индикатор разрешения действия
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке
    /// внутри `view-storage`
    pub async fn check_permission(
        &self,
        dto: CheckPermissionRequest,
    ) -> AsezResult<bool> {
        let response = self
            .service_request(
                dto,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::ViewsCheckPermission,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ViewStorageError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Обращение к `ViewStorage` для действия "Получение прав пользователя на операции и отображение секций" в АЦ
    ///
    /// # Возвращает
    /// * Ok([`PricingSectionUserResponseData`]) - Права и секции
    /// * Err([`AsezError`]) - Ошибка при неудачном обращении к RabbitMQ или ошибке внутри `processing`
    pub async fn pa_get_section_user(
        &self,
        dto: PricingSectionUserReq,
    ) -> AsezResult<PricingSectionUserResponse> {
        let response = self
            .service_request(
                dto,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::ViewsPricingSectionUser,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ViewStorageError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Получение информации по уведомлению по его типу(айди)
    pub async fn get_notifications_by_type(
        &self,
        types: Vec<NotificationType>,
        user_ids: Vec<i32>,
    ) -> AsezResult<GetNotificationsByTypeResponse> {
        let response = self
            .service_request(
                NotificationStorageReq::GetByType(GetNotificationsByType {
                    types,
                    user_ids,
                }),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::ViewsNotificationStorage,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(ViewStorageError::from)?;
        response.content
    }
}

from_request!(ViewStorageService);
