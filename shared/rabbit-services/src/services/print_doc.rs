//! Клиент RabbitMQ для сервиса генерации документов (PrintDoc).
use std::{sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;

use shared_essential::presentation::dto::{
    print_docs::{Content, PrintDocError, Response},
    AsezResult, Source,
};

use crate::callbacks::AsezCallback;

use super::{AsezRabbitProperties, AsezRabbitRouting, AsezRabbitService};

/// # Описание
///
/// Сервис печатных форм
///
/// # API
/// 1. [`PrintDocService::create_document`] - Создание документа
#[derive(Debug, Clone)]
pub struct PrintDocService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for PrintDocService {
    const SERVICE: Source = Source::PrintDocs;

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

impl PrintDocService {
    const DEFAULT_TIMEOUT: u64 = 5_000;

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
    /// Создание и сохранение документа
    ///
    /// # Возвращает
    /// * Ok([`Response`]) - Успешное создание документам
    /// * Err([`AsezError`]) - Ошибка при взаимодействии с RabbitMQ или ошибка при процессинге запроса в print-doc
    pub async fn create_document(&self, dto: &Content) -> AsezResult<Response> {
        let response = self
            .service_request(
                dto,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::RequestPrintDoc,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(PrintDocError::from)?;
        response.content
    }
}

from_request!(PrintDocService);
