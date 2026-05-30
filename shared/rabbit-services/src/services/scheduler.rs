//! Клиент RabbitMQ для модуля календаря Планировщика (Scheduler).
use std::{sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;
use shared_essential::presentation::dto::{
    scheduler::{
        CalendarReq, DateAfterWorkdaysReq, DateAfterWorkdaysRes,
        ProductionDirectoryRequest, ProductionDirectoryResponse, SchedulerError,
        WorkdaysBetweenDatesReq, WorkdaysBetweenDatesRes,
    },
    AsezResult, Source,
};

use super::{AsezRabbitProperties, AsezRabbitRouting, AsezRabbitService};
use crate::callbacks::AsezCallback;
/// # Описание
///
/// Сервис календарного планирования
///
/// # API
/// 1. [`Scheduler::update_catalog`] - Создание документа
#[derive(Debug, Clone)]
pub struct SchedulerService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for SchedulerService {
    const SERVICE: Source = Source::Scheduler;

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

impl SchedulerService {
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
    /// Хранение информации о событиях и специфических датах
    ///
    /// # Возвращает
    /// * Ok([`Response`]) - Успешное обновление каталога дат
    /// * Err([`AsezError`]) - Ошибка при взаимодействии с RabbitMQ
    pub async fn update_catalog(
        &self,
        dto: ProductionDirectoryRequest,
    ) -> AsezResult<ProductionDirectoryResponse> {
        let response = self
            .service_request(
                dto,
                self.rabbit_properties.clone(),
                AsezRabbitRouting::RequestDictionaries,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(SchedulerError::from)?;
        response.content
    }

    /// Получение количества рабочих дней между датами с учетом
    /// производственного календаря
    pub async fn workdays_between_dates(
        &self,
        dto: WorkdaysBetweenDatesReq,
    ) -> AsezResult<WorkdaysBetweenDatesRes> {
        let response = self
            .service_request(
                CalendarReq::WorkdaysBetweenDates(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::SchedulerCalendar,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(SchedulerError::from)?;
        response.content
    }

    /// Получение даты после определенного количества
    /// рабочих дней с учетом производственного календаря
    pub async fn date_after_workdays(
        &self,
        dto: DateAfterWorkdaysReq,
    ) -> AsezResult<DateAfterWorkdaysRes> {
        let response = self
            .service_request(
                CalendarReq::DateAfterWorkdays(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::SchedulerCalendar,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(SchedulerError::from)?;
        response.content
    }
}

from_request!(SchedulerService);
