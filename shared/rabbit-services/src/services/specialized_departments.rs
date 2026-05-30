//! Клиент RabbitMQ для модуля Специализированных отделов.
use std::{sync::Arc, time::Duration};

use broker::rabbit::RabbitAdapter;
use shared_essential::presentation::dto::{
    response_request::ApiResponse,
    specialized_departments::{
        request::{GetApproversForPlansReq, SpecDepsAction},
        response::GetApproversForPlansResData,
        SpecDepsError,
    },
    AsezResult, Source,
};

use crate::{
    callbacks::AsezCallback, properties::AsezRabbitProperties, AsezRabbitRouting,
    AsezRabbitService,
};

#[derive(Clone, Debug)]
pub struct SpecializedDepartmentsService {
    rabbit_adapter: Arc<RabbitAdapter>,
    rabbit_properties: AsezRabbitProperties,
    service_caller: Source,
    callbacks: Vec<AsezCallback>,
}

impl AsezRabbitService for SpecializedDepartmentsService {
    const SERVICE: Source = Source::SpecializedDepartments;

    fn service_caller(&self) -> Source {
        self.service_caller
    }

    fn adapter(&self) -> &RabbitAdapter {
        &self.rabbit_adapter
    }

    fn callbacks(&self) -> &[AsezCallback] {
        &self.callbacks
    }

    fn with_callback(mut self, callback: AsezCallback) -> Self {
        self.callbacks.push(callback);
        self
    }
}

impl SpecializedDepartmentsService {
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

    pub async fn get_approvers_for_plans(
        &self,
        dto: GetApproversForPlansReq,
    ) -> AsezResult<ApiResponse<GetApproversForPlansResData, ()>> {
        let response = self
            .service_request(
                SpecDepsAction::GetApproversForPlans(dto),
                self.rabbit_properties.clone(),
                AsezRabbitRouting::SpecDepsRPC,
                Duration::from_millis(Self::DEFAULT_TIMEOUT),
            )
            .await
            .map_err(SpecDepsError::from)?;
        response.content
    }
}
