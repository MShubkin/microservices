use reqwest::{Client, Url};
use serde::Deserialize;

use super::{
    common::{make_reqwest_client, query_data, RequestType},
    IntegrationResult,
};
use crate::presentation::dto::response_request::ApiResponseData;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct PlanningRestConfig {
    pub url: String,
}
impl From<env_setup::PlanningRestCfg> for PlanningRestConfig {
    fn from(o: env_setup::PlanningRestCfg) -> Self {
        Self { url: o.url }
    }
}
impl PlanningRestConfig {
    pub fn get_client(&self) -> IntegrationResult<PlanningRestClient> {
        Ok(PlanningRestClient {
            service_url: Url::parse(self.url.clone().as_str())?,
            client: make_reqwest_client()?,
        })
    }
}
#[derive(Clone)]
pub(super) struct PlanningRestClient {
    pub service_url: Url,
    pub client: Client,
}

impl PlanningRestClient {
    pub(super) async fn query_data<R: ApiResponseData>(
        &self,
        function_url: &str,
        request_type: RequestType,
        user_id: i32,
        token: &str,
    ) -> IntegrationResult<R> {
        let full_url = self.service_url.join(function_url)?;
        query_data(&self.client, full_url, request_type, user_id, token).await
    }
}
