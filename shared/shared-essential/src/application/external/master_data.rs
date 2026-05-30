// TODO: Возможно, что нужно вынести в `monolith-service`

use actix_web::{cookie::Cookie, http};
use ahash::AHashMap;
use env_setup::PlanningRestCfg;
use serde::Deserialize;
use url::Url;

use crate::application::external::IntegrationError;

use super::{
    planning_masterdata::{
        Currency, Customer, Document, PurchasingMethod, Status, Unit, User, Vat,
    },
    IntegrationResult,
};

// Основная структура MasterData
#[derive(Debug, Default, Clone)]
// Некоторые поля действительно пока не используются, но могут быть в будущем
#[allow(dead_code)]
pub struct MasterData {
    pub units: AHashMap<i64, Unit>,
    pub currencies: AHashMap<i64, Currency>,
    pub vats: AHashMap<i64, Vat>,
    pub users: AHashMap<i64, User>,
    pub customers: AHashMap<i64, Customer>,
    pub purchasing_methods: AHashMap<i64, PurchasingMethod>,
    pub statuses: AHashMap<i64, Status>,
    pub documents: AHashMap<i64, Document>,
}

// Вспомогательная структура для обработки полного JSON-ответа
#[derive(Deserialize, Clone, Debug)]
struct ApiMasterData {
    pub units: Vec<Unit>,
    pub currencies: Vec<Currency>,
    pub vats: Vec<Vat>,
    pub users: Vec<User>,
    pub customers: Vec<Customer>,
    pub purchasing_methods: Vec<PurchasingMethod>,
    pub statuses: Vec<Status>,
    pub documents: Vec<Document>,
}

#[derive(Deserialize)]
struct ApiResponse {
    pub data: ApiMasterData,
}

// Общий трейт для преобразования в HashMap
trait IntoMap<K, V> {
    fn into_map(self, key_extractor: impl Fn(&V) -> K) -> AHashMap<K, V>;
}

impl<K, V> IntoMap<K, V> for Vec<V>
where
    K: std::hash::Hash + Eq,
{
    fn into_map(self, key_extractor: impl Fn(&V) -> K) -> AHashMap<K, V> {
        self.into_iter().map(|item| (key_extractor(&item), item)).collect()
    }
}

// Макрос для генерации преобразований полей
macro_rules! convert_fields {
    ($data:expr, $($field:ident),+) => {
        MasterData {
            $(
                $field: $data.$field.into_map(|item| item.id as i64),
            )+
        }
    };
}

// Получение master_data/get_multiple
pub async fn get_multiple_master_data(
    user_id: i32,
    token: &str,
) -> IntegrationResult<MasterData> {
    let cfg = PlanningRestCfg::from_env()?;
    let url = Url::parse(cfg.url.as_str())?;
    let request_url = url.join("/api/json/master_data/get_multiple/")?;

    let response = reqwest::Client::new()
        .post(request_url.clone())
        .query(&[("user_id", user_id.to_string())])
        .header(
            http::header::COOKIE,
            Cookie::new("id", token.to_string()).to_string(),
        )
        .send()
        .await?
        .text()
        .await?;

    let response: ApiResponse =
        serde_json::from_str(response.as_str()).map_err(|err| {
            tracing::error!(kind = "integration", error = %err, "ошибка десериализации ответа master-data");
            IntegrationError::Format(
                "Can't deserialize a get_multiple response".to_string(),
            )
        })?;

    // Преобразуем в MasterData, что по факту поля из Vec в AHashMap
    Ok(convert_fields!(
        response.data,
        units,
        currencies,
        vats,
        users,
        customers,
        purchasing_methods,
        statuses,
        documents
    ))
}

// Тесты
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request() {
        let master_data = get_multiple_master_data(657, "token").await.unwrap();
        assert!(!master_data.currencies.is_empty());
        assert!(!master_data.documents.is_empty());
        assert!(!master_data.units.is_empty());
        assert!(!master_data.vats.is_empty());
        // На момент добавления теста этих полей не было в моке монолита
        // assert!(!master_data.users.is_empty());
        // assert!(!master_data.purchasing_methods.is_empty());
        // assert!(!master_data.statuses.is_empty());
        // assert!(!master_data.customers.is_empty());
    }
}
