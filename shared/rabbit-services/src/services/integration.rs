//! Клиент RabbitMQ для сервиса интеграции (SAP PI и хранение документов OpenText).
use std::{sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;

use shared_essential::presentation::dto::{
    integration::{
        commercial_offer::request::CommercialOfferData,
        documents::{
            DocumentRequest, GetDocumentReq, GetDocumentResponse, SaveDocumentReq,
            SaveDocumentResponse,
        },
        IntegrationRequest,
    },
    integration::{CommonResponse, IntegError},
    AsezResult, Source,
};

use crate::callbacks::AsezCallback;

use super::{AsezRabbitProperties, AsezRabbitRouting, AsezRabbitService};

/// # Описание
///
/// Сервис интеграции
///
/// # API
/// 1. [`IntegrationService::get_document`] - Получение документа
/// 2. [`IntegrationService::save_document`] - Сохранение документа
#[derive(Debug, Clone)]
pub struct IntegrationService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for IntegrationService {
    const SERVICE: Source = Source::Integration;

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

impl IntegrationService {
    const DEFAULT_TIMEOUT: u64 = 10_000;
    pub const DEFAULT_EXPIRATION: u64 = 60_000;

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
    /// Передача ЗЦИ и пакета документов по закупке
    ///
    /// # Принимает
    /// * `dto` - ЗЦИ
    ///
    /// # Возвращает
    /// * Ok(()) - Успешная отправка
    /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса
    pub async fn send_commercial_offer_request(
        &self,
        dto: CommercialOfferData,
    ) -> AsezResult<CommonResponse> {
        let response = self
            .service_request(
                IntegrationRequest::CommercialOfferRequestOut(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::IntegrationCallPi,
                Duration::from_millis(Self::DEFAULT_EXPIRATION),
            )
            .await
            .map_err(IntegError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Получение документа
    ///
    /// # Возвращает
    /// * Ok([`GetDocumentResponse`]) - Успешное получение документа
    /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса в `integration`
    pub async fn get_document(
        &self,
        dto: GetDocumentReq,
    ) -> AsezResult<GetDocumentResponse> {
        let response = self
            .service_request(
                DocumentRequest::Get(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::IntegrationOpenText,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(IntegError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Сохранение документа
    ///
    /// # Возвращает
    /// * Ok([`SaveDocumentResponse`]) - Успешное сохранение документа
    /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса в `integration`
    pub async fn save_document(
        &self,
        dto: SaveDocumentReq,
    ) -> AsezResult<SaveDocumentResponse> {
        let response = self
            .service_request(
                DocumentRequest::Save(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::IntegrationOpenText,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(IntegError::from)?;
        response.content
    }
}

from_request!(IntegrationService);
