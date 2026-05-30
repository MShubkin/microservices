//! Клиент RabbitMQ для сервиса TCP (технико-коммерческое предложение).
use std::{sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;

use crate::callbacks::AsezCallback;
use shared_essential::presentation::dto::{
    integration::commercial_offer::{
        request_confirmation::CommercialOfferRequestConfirmationData,
        response::CommercialOfferResponseData,
    },
    technical_commercial_proposal::{TcpDataAction, TcpError},
    AsezResult, Source,
};

use super::{AsezRabbitProperties, AsezRabbitRouting, AsezRabbitService};

/// # Описание
///
/// Сервис ТКП
#[derive(Debug, Clone)]
pub struct TcpService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for TcpService {
    const SERVICE: Source = Source::TechnicalCommercialProposal;

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

impl TcpService {
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

    /// # Описание
    ///
    /// Передача подтверждения о доставке ЗЦИ и пакета документов по закупке
    ///
    /// # Принимает
    /// * `dto` - Подтверждение о доставке ЗЦИ
    ///
    /// # Возвращает
    /// * Ok(()) - Успешная отправка
    /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса
    pub async fn send_commercial_offer_request_confirmation(
        &self,
        dto: CommercialOfferRequestConfirmationData,
    ) -> AsezResult<()> {
        let response = self
            .service_request(
                TcpDataAction::CommercialOfferRequestConfirmation(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::TcpAction,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(TcpError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Передача ТКП из ЭТП ПГБ
    ///
    /// # Принимает
    /// * `dto` - ТКП
    ///
    /// # Возвращает
    /// * Ok(()) - Успешная отправка
    /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса
    pub async fn send_commercial_offer_response(
        &self,
        dto: CommercialOfferResponseData,
    ) -> AsezResult<()> {
        let response = self
            .service_request(
                TcpDataAction::CommercialOfferResponse(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::TcpAction,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(TcpError::from)?;
        response.content
    }

    /// # Описание
    ///
    /// Поиск ТКП по tcp_id
    /// Участвует в передаче доп документов из ЭТП ГПБ
    ///
    /// # Принимает
    /// * `dto` - id ТКП
    ///
    /// # Возвращает
    /// * Ok(()) - Успешная отправка
    /// * Err([`AsezError`]) - Ошибка при обращении к RabbitMQ или ошибка при процессинге запроса
    pub async fn send_commercial_offer_add_doc_response(
        &self,
        dto: i32,
    ) -> AsezResult<uuid::Uuid> {
        let response = self
            .service_request(
                TcpDataAction::CommercialOfferAddDocResponse(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::TcpAction,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(TcpError::from)?;
        response.content
    }
}

from_request!(TcpService);
