use super::common::{
    process_result, LookupDataDictionaryKindMap, LookupRecordData, LookupRecordId,
};
use super::enrichment::SimpleExtRecord;
use super::IntegrationResult;
use crate::common::maps::map_2;
use crate::presentation::dto::response_request::ApiResponseData;

use actix_web::cookie::Cookie;
use actix_web::http;
use ahash::AHashMap;
use env_setup::MonolithCfg;
use reqwest::{Client, Error, Response};
use serde::{
    de, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::value::RawValue;

/// Bulk load Monolith dictionaries from `/api/json/master_data/get_updates/{timestamp}/`

const GET_UPDATES_BASE_PATH: &str = "/api/json/master_data/get_updates/0/";

/// Внимание! В данных монолита встречаются позиции с отсутствующими полями "id" и "text".
/// Справочники с "кривыми" данными закомментированы.
#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Hash, Eq, Default,
)]
#[serde(rename_all = "PascalCase")]
pub enum DictionaryKind {
    Category,
    ContractAmendmentItemKind,
    ContractAmendmentKind,
    Country,
    Currency,
    Customer,
    Department,
    FundingSource,
    MasterSystem,
    NomenclatureGroup,
    Organization,
    PublicationType,
    PurchasingMethod,
    PurchasingPolicyItem,
    PurchasingTrend,
    // QualificationActivity,
    // QualificationPlanDefault,
    // QualificationPlanIrrelevant,
    RegulationDocument,
    Section,
    SmbException,
    Status,
    Unit,
    Vat,
    BudgetItem,
    ContractAmendmentApprovingDecision,
    // CountryPP2013,
    // Law2013Country,
    // Law2013Okdp2,
    // Law2013ViolationReason,
    // MtrGroup,
    // MtrReestr,
    // MtrReestrName,
    // MtrReestrProducerName,
    // Okpd2PP2013,
    PaymentBalanceItem,
    PurchasingPolicy,
    // Qualification,
    // RepairStage,
    // RfProductsReason,
    #[default]
    Unknown,
}
#[derive(Debug)]
pub(crate) struct DictionaryRecord {
    pub dictionary_kind: DictionaryKind,
    pub items: Vec<SimpleExtRecord>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct MonolithData {
    pub entities: Vec<DictionaryRecord>,
}

impl ApiResponseData for MonolithData {}

/// Загрузка данных из монолита ГПИ.
/// Требуется переменная окружения `MONOLITH_BASE_URL`
/// Получает обновления справочников из монолита и парсит в структуру для насыщения экспорта
pub async fn load_monolith_data(
    client: &Client,
    monolith_cfg: &MonolithCfg,
    timestamp: u32, // must be zero - format query path with timestamp
    user_id: i32,
    token: &str,
) -> IntegrationResult<LookupDataDictionaryKindMap> {
    let timed_path =
        GET_UPDATES_BASE_PATH.replace('0', timestamp.to_string().as_str());
    let monolith_query_url = monolith_cfg.url.join(timed_path.as_str())?;

    let request = client
        .post(monolith_query_url.clone())
        .query(&[("user_id", user_id.to_string())])
        .header(http::header::COOKIE, Cookie::new("id", token).to_string());

    let result: Result<Response, Error> = request.send().await;
    let monolith_data: MonolithData =
        process_result(monolith_query_url, result).await?;

    let mut lookup_data: LookupDataDictionaryKindMap = AHashMap::new();
    for entity in monolith_data.entities {
        let data = entity
            .items
            .iter()
            .map(|record| {
                let record_id = if entity.dictionary_kind == DictionaryKind::Status
                {
                    LookupRecordId::new(
                        record.id.as_int(),
                        record.scheme_id.as_int(),
                    )
                } else {
                    LookupRecordId::with_id(record.id.as_int())
                };
                (
                    record_id,
                    LookupRecordData {
                        id: record.id.as_int(),
                        parent_id: record.parent_id.as_int(),
                        text: record.text.clone(),
                        code: record.code.as_str(),
                        dictionary_kind: entity.dictionary_kind,
                    },
                )
            })
            .collect();
        lookup_data.insert(entity.dictionary_kind, data);
    }
    Ok(lookup_data)
}

/// Сериалайзер для DictionaryRecord. Особенность: `dictionary` используется как ключ,
/// а `items` как значения в структуре `json`-а. См. тесты.
impl Serialize for DictionaryRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(&self.dictionary_kind, {
            #[derive(Serialize, Debug)]
            #[serde(transparent)]
            struct Wrapper<'a> {
                items: &'a Vec<SimpleExtRecord>,
            }
            &Wrapper { items: &self.items }
        })?;
        map.end()
    }
}

/// Десериалайзер для DictionaryRecord. Особенность: `dictionary` используется как ключ,
/// а `items` как значения в структуре `json`-а. См. тесты.
impl<'de> Deserialize<'de> for DictionaryRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const CHANGED_AT: &str = "changed_at";
        const TYPE: &str = "type";

        struct DictionaryRecordVisitor;

        impl<'a> de::Visitor<'a> for DictionaryRecordVisitor {
            type Value = DictionaryRecord;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str("DictionaryRecord")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'a>,
            {
                let mut dictionary: Option<DictionaryKind> = None;
                let mut items: Option<Vec<SimpleExtRecord>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        TYPE => {
                            let _ = map.next_value::<&str>();
                            continue;
                        }
                        CHANGED_AT => {
                            let _ = map.next_value::<i64>();
                            continue;
                        }
                        maybe_dictionary_name => {
                            #[derive(Deserialize)]
                            #[serde(transparent)]
                            struct Wrapper {
                                items: Vec<SimpleExtRecord>,
                            }

                            if dictionary.is_none() {
                                let dictionary_str =
                                    format!("\"{}\"", maybe_dictionary_name);
                                match serde_json::from_str::<DictionaryKind>(
                                    dictionary_str.as_str(),
                                ) {
                                    Ok(dict) => dictionary = Some(dict),
                                    Err(_) => {
                                        tracing::error!(
                                            "Unexpected DictionaryKind value {}",
                                            key
                                        );
                                        let _ = map.next_value::<&RawValue>();
                                        continue;
                                    }
                                }
                                match map.next_value::<Wrapper>() {
                                    Ok(wrapper) => items = Some(wrapper.items),
                                    Err(e) => {
                                        tracing::error!(
                                            "Unexpected item value {}",
                                            e
                                        );
                                        items = Some(Vec::new());
                                        continue;
                                    }
                                }
                            } else {
                                tracing::error!("Duplicated DictionaryKey {}", key);
                                let _ = map.next_value::<&RawValue>();
                                continue;
                            }
                        }
                    }
                }

                map_2(dictionary, items, |dictionary_kind, items| {
                    Ok(Self::Value {
                        dictionary_kind,
                        items,
                    })
                })
                .unwrap_or(
                    // Err(de::Error::missing_field("[dictionary,items]"))
                    Ok(Self::Value {
                        dictionary_kind: DictionaryKind::Unknown,
                        items: Vec::new(),
                    }),
                )
            }
        }

        deserializer.deserialize_map(DictionaryRecordVisitor {})
    }
}
