use super::common::{
    make_reqwest_client, query_data, LookupData, LookupDataDictionaryKindMap,
    LookupDataFieldMap, LookupDataIdMap, LookupRecordData, LookupRecordId,
    MasterDataCfg, RequestType,
};
use super::id_lookup_cfg::IdLookupCfg;
use super::{IntegrationError, IntegrationResult};

use actix_web::cookie::Cookie;
use actix_web::http;
use ahash::AHashMap;
use itertools::{izip, Itertools};
use tracing::instrument;
use url::Url;

use super::monolith::{load_monolith_data, DictionaryKind};
use super::planning_masterdata::{
    process_planning_multiple_request, MultipleRequest, StatusHistory,
};
use crate::presentation::dto::general::{DataRecords, TaggedValue};

use crate::presentation::dto::response_request::ApiResponseData;
use crate::presentation::dto::response_request::EntityKind;

use crate::common::export::convert_timestamp_fields;
use env_setup::{MonolithCfg, PlanningRestCfg};
use rayon::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const NON_LOOKUP_FIELDS: &[&str] = &[
    "items_number",
    "vote_iteraction_price",
    "protocol_item_quantity_threshold",
    "protocol_item_d647_quantity_threshold",
];
/// Идентификатор схемы статусов ППЗ в монолите
const PLAN_STATUS_SCHEME_ID: usize = 10;
/// Идентификатор схемы статусов ДС в монолите
const CONTRACT_AMENDMENT_STATUS_SCHEME_ID: usize = 4;

/// "Насыщение" исходных данных данными из справочников
pub async fn enrich_data_records(
    mut data_records: DataRecords,
    master_data_cfg: MasterDataCfg,
    user_id: i32,
    token: &str,
) -> IntegrationResult<DataRecords> {
    // Данные по ручке '/<dictionary>/search_by_id/`
    let id_lookup_data =
        get_id_lookup_data(&master_data_cfg, &mut data_records, user_id, token)
            .await?;
    // Данные по ручке '/master_data/get_updates/0/>'
    let common_lookup_data = get_common_lookup_data(user_id, token).await?;
    // Данные db-энумов
    let enum_lookup_data = master_data_cfg.enum_lookup_config.data.clone();
    // Все справочные данные
    let lookup_data = LookupData {
        id_lookup_data,
        common_lookup_data,
        enum_lookup_data,
    };
    // Замена идентификаторов на текстовые значения
    replace_lookup_data(&mut data_records, &master_data_cfg, &lookup_data);

    // Конвертация времени в gmt+3
    convert_timestamp_fields(&mut data_records);

    // Заполнение наименования стутуса
    fill_status_name(&mut data_records, &lookup_data).await?;
    // Заполнение истории изменения статусов
    fill_status_history(&mut data_records, &lookup_data, user_id, token).await?;

    Ok(data_records)
}

/// Получение данных по ручке '/<dictionary>/search_by_id/`
pub async fn get_id_lookup_data(
    master_data_cfg: &MasterDataCfg,
    data_records: &mut DataRecords,
    user_id: i32,
    token: &str,
) -> IntegrationResult<LookupDataFieldMap> {
    // Fields from search by id configuration
    let search_by_id_fields: Vec<String> =
        master_data_cfg.id_lookup_config.fields();

    // Data record fields for lookup/replace
    let intersected_fields: Vec<String> = data_records
        .field_list
        .iter()
        .filter(|field| search_by_id_fields.contains(*field))
        .map(|field| field.to_string())
        .collect();

    // Collect unique values from fields
    let extracted_values: HashMap<String, Vec<Id>> =
        extract_data_values(data_records, intersected_fields.as_slice());

    let client: Client = make_reqwest_client()?;

    // Data from external sources (<dictionary/search_by_id>)
    let id_lookup_data: LookupDataFieldMap = load_lookup_data(
        &client,
        &extracted_values,
        &master_data_cfg.id_lookup_config,
        user_id,
        token,
    )
    .await?;

    Ok(id_lookup_data)
}

/// Получение данных по ручке '/master_data/get_updates/0/>'
pub async fn get_common_lookup_data(
    user_id: i32,
    token: &str,
) -> IntegrationResult<LookupDataDictionaryKindMap> {
    let monolith_cfg = MonolithCfg::from_env()?;
    let client: Client = make_reqwest_client()?;
    let common_lookup_data: LookupDataDictionaryKindMap = load_monolith_data(
        &client,
        &monolith_cfg,
        0, // must be zero - format query path with timestamp
        user_id,
        token,
    )
    .await?;
    Ok(common_lookup_data)
}

pub async fn fill_status_name(
    data_records: &mut DataRecords,
    lookup_data: &LookupData,
) -> IntegrationResult<()> {
    let status_id_position =
        data_records.field_list.iter().position(|x| x.clone() == "status_id");

    let status_id_position = if let Some(position) = status_id_position {
        position
    } else {
        return Ok(());
    };

    let status_map = lookup_data
        .common_lookup_data
        .get(&DictionaryKind::Status)
        .ok_or(IntegrationError::NotFound("словарь статусов"))?;

    for (index, data) in data_records.data.iter_mut().enumerate() {
        let entity_kind = data_records
            .entity_kind
            .get(index)
            .ok_or(IntegrationError::NotFound("entity kind"))?;
        let scheme_id = get_status_scheme_id(*entity_kind);
        if let Some(value) = data.get_mut(status_id_position) {
            if let TaggedValue::Int(status_id) = value {
                let (status_name, parent_status_name) = get_status_name_with_parent(
                    status_map,
                    *status_id as i32,
                    scheme_id as i32,
                );
                *value = TaggedValue::String(concat_status_names(
                    None,
                    parent_status_name,
                    status_name,
                ));
            }
        }
    }

    Ok(())
}

/// Получение наименования статуса вместие с родительским
pub fn get_status_name_with_parent(
    status_map: &LookupDataIdMap,
    status_id: i32,
    scheme_id: i32,
) -> (Option<String>, Option<String>) {
    let record = match status_map.get(&LookupRecordId::new(status_id, scheme_id)) {
        Some(r) => r,
        None => return (None, None),
    };
    let parent_name = if record.parent_id > 0 {
        status_map
            .get(&LookupRecordId::new(record.parent_id, scheme_id))
            .map(|p| p.text.clone())
    } else {
        None
    };
    (Some(record.text.clone()), parent_name)
}
pub fn concat_status_names(
    timestamp: Option<String>,
    parent_status_name: Option<String>,
    status_name: Option<String>,
) -> String {
    let mut result = String::new();
    if let Some(ts) = timestamp {
        result.push_str(&ts);
        result.push(' ');
    }
    if let Some(parent) = parent_status_name {
        result.push_str(&parent);
        result.push_str(". ");
    }
    if let Some(status) = status_name {
        result.push_str(&status);
    }
    result
}

#[instrument(skip_all, fields(kind = "export_enrichment"))]
pub async fn fill_status_history(
    data_records: &mut DataRecords,
    lookup_data: &LookupData,
    user_id: i32,
    token: &str,
) -> IntegrationResult<()> {
    tracing::debug!(
        ?data_records,
        ?lookup_data,
        "Заполнение истории изменения статусов для ППЗ/ДС",
    );

    let status_history_position = data_records
        .field_list
        .iter()
        .position(|x| x.clone() == "status_history");

    let status_history_short_position = data_records
        .field_list
        .iter()
        .position(|x| x.clone() == "status_history_short");

    if status_history_position.is_some() || status_history_short_position.is_some()
    {
        if let Some(plan_id_index) =
            data_records.field_list.iter().position(|x| x == "plan_id")
        {
            let plan_ids = data_records
                .data
                .iter()
                .flat_map(|data_record| {
                    data_record
                        .get(plan_id_index)
                        .map(|item| item.get_string_value())
                })
                .collect_vec();
            let multiple_request = MultipleRequest {
                document_ids: plan_ids,
                ..Default::default()
            };

            tracing::debug!(
                "Получение истории изменения статуса(/api/json/master_data/get_multiple/): {multiple_request:#?}"
            );

            let response =
                process_planning_multiple_request(multiple_request, user_id, token)
                    .await?;

            tracing::debug!(
                "История изменения статусов(/api/json/master_data/get_multiple/): {:?}",
                response.documents
            );

            let response_map: AHashMap<String, Vec<StatusHistory>> = response
                .documents
                .iter()
                .map(|item| (item.id.to_string(), item.status_history.clone()))
                .collect();
            if let Some(status_map) =
                lookup_data.common_lookup_data.get(&DictionaryKind::Status)
            {
                tracing::debug!(
                    "Справочные данные по статусам для заполнения полей: {:?}",
                    status_map
                );

                for (index, data) in data_records.data.iter_mut().enumerate() {
                    if let Some(plan_value) = data.get(plan_id_index) {
                        if let Some(status_history_vec) =
                            response_map.get(&plan_value.get_string_value())
                        {
                            let entity_kind =
                                data_records.entity_kind.get(index).ok_or(
                                    IntegrationError::NotFound("entity kind"),
                                )?;
                            let scheme_id = get_status_scheme_id(*entity_kind);
                            let statuses_vec = status_history_vec
                                .iter()
                                .map(|status_history| {
                                    let timestamp = status_history
                                        .changed_at
                                        .to_moscow_timestamp()
                                        .to_string();
                                    let (status_name, parent_status_name) =
                                        get_status_name_with_parent(
                                            status_map,
                                            status_history.status_id as i32,
                                            scheme_id as i32,
                                        );
                                    concat_status_names(
                                        Some(timestamp),
                                        parent_status_name,
                                        status_name,
                                    )
                                })
                                .collect_vec();

                            tracing::debug!(
                                "Заполнение истории изменения статусов для ППЗ/ДС {plan_value}: {statuses_vec:?}"
                            );

                            if let Some(value) = status_history_position
                                .and_then(|x| data.get_mut(x))
                            {
                                *value =
                                    TaggedValue::String(statuses_vec.join("\n"));
                            }

                            if let Some(value) = status_history_short_position
                                .and_then(|x| data.get_mut(x))
                            {
                                *value =
                                    TaggedValue::String(statuses_vec.join("\n"));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Получение идентификатора статусной схемы
/// Из монолита мы получаем только статусы ППЗ(схема 4) и ДС(схема 10)
pub fn get_status_scheme_id(entity_kind: EntityKind) -> usize {
    match entity_kind {
        EntityKind::Plan => PLAN_STATUS_SCHEME_ID,
        EntityKind::ContractAmendment => CONTRACT_AMENDMENT_STATUS_SCHEME_ID,
        _ => usize::default(),
    }
}

/// Replace id with text data from lookup sources
pub(crate) fn replace_lookup_data(
    data_records: &mut DataRecords,
    master_data_csg: &MasterDataCfg,
    lookup_data: &LookupData,
) {
    // Подготовили данные из справочников в соответствии с `field_list`
    let common_lookup_map = master_data_csg.common_lookup_config.as_map();
    let id_lookups = data_records
        .field_list
        .iter()
        .map(|field_name| lookup_data.id_lookup_data.get(field_name))
        .collect::<Vec<_>>();

    let common_lookups = data_records
        .field_list
        .iter()
        .map(|field_name| {
            common_lookup_map.get(field_name).and_then(|dictionary_kind| {
                lookup_data.common_lookup_data.get(dictionary_kind)
            })
        })
        .collect::<Vec<_>>();
    let enum_lookups = data_records
        .field_list
        .iter()
        .map(|field_name| lookup_data.enum_lookup_data.get(field_name))
        .collect::<Vec<_>>();

    let non_lookup = data_records
        .field_list
        .iter()
        .map(|field_name| NON_LOOKUP_FIELDS.contains(&field_name.as_str()))
        .collect::<Vec<_>>();

    data_records.data.par_iter_mut().for_each(|data| {
        replace_data_record(
            &id_lookups,
            &common_lookups,
            &enum_lookups,
            data,
            &non_lookup,
        );
    });
}

fn replace_data_record(
    id_lookups: &[Option<&LookupDataIdMap>],
    common_lookups: &[Option<&LookupDataIdMap>],
    enum_lookups: &[Option<&LookupDataIdMap>],
    data: &mut Vec<TaggedValue>,
    non_lookups: &[bool],
) {
    let mut zip_iter = izip!(id_lookups, common_lookups, enum_lookups, non_lookups);
    for tagged_value in data {
        let next_iter = zip_iter.next();
        if let TaggedValue::Int(id) = tagged_value {
            if let Some((id_lookup, common_lookup, enum_lookups, non_lookup)) =
                next_iter
            {
                if *id == 0 && !non_lookup {
                    *tagged_value = TaggedValue::Null;
                    continue;
                }
                // Поиск по энумам
                if let Some(lookup) = enum_lookups {
                    *tagged_value = get_text_tagged_value(lookup, id);
                }
                // Поиск в справочниках(монолита и мастердата) по ручке /search_by_id/
                else if let Some(lookup) = id_lookup {
                    *tagged_value = get_text_tagged_value(lookup, id);
                }
                // Поиск в справочниках монолита по ручке /master_data/get_updates/
                else if let Some(lookup) = common_lookup {
                    *tagged_value = get_text_tagged_value(lookup, id);
                }
            }
        }
        if let TaggedValue::Vec64(array) = tagged_value {
            if let Some((_, Some(lookup), _, _)) = next_iter {
                *tagged_value = TaggedValue::String(
                    array
                        .0
                        .iter()
                        .fold(Vec::new(), |mut vec, id| {
                            if let Some(value) =
                                lookup.get(&(LookupRecordId::with_id(*id as i32)))
                            {
                                vec.push(value.text.clone())
                            }
                            vec
                        })
                        .join("\n"),
                );
            }
        }
    }
}
fn get_text_tagged_value(map: &LookupDataIdMap, id: &i64) -> TaggedValue {
    if let Some(new_value) = map.get(&(LookupRecordId::with_id(*id as i32))) {
        if new_value.dictionary_kind == DictionaryKind::PurchasingPolicyItem {
            TaggedValue::String(new_value.code.clone())
        } else {
            TaggedValue::String(new_value.text.clone())
        }
    } else {
        TaggedValue::Int(*id)
    }
}

async fn load_lookup_data(
    client: &Client,
    extracted_values: &HashMap<String, Vec<Id>>,
    lookup_config: &IdLookupCfg,
    user_id: i32,
    token: &str,
) -> IntegrationResult<LookupDataFieldMap> {
    let mut lookup_data: LookupDataFieldMap = AHashMap::new();
    for replace_record in lookup_config.records.iter() {
        let host = &replace_record.host;
        for record in replace_record.replacements.iter() {
            // `IntegrationError::Url` импортирован через `#[from]` — пропускаем ошибку
            // парсинга URL вверх вместо паники на потенциально невалидном конфиге.
            let query_url = host.join(&record.path)?;
            tracing::debug!(
                kind = "get",
                "query url: {} [{}][{}]",
                query_url,
                host,
                record.path
            );
            if let Some(query_values) = extracted_values.get(&record.replace_field)
            {
                // Для некоторых запросов мы должны предоставлять "id" как строку
                // (например organizations и units), а для других как целое
                let fixed_query_values: Vec<Id> = if replace_record.is_int {
                    query_values.to_vec()
                } else {
                    query_values
                        .iter()
                        .map(|value| Id::str(value.id.as_str()))
                        .collect::<Vec<Id>>()
                };

                let external_data = query_data::<SimpleExtData>(
                    client,
                    query_url.clone(),
                    RequestType::SearchById(fixed_query_values),
                    user_id,
                    token,
                )
                .await?;
                let data: LookupDataIdMap = external_data
                    .value
                    .iter()
                    .map(|record| {
                        (
                            LookupRecordId::with_id(record.id.as_int()),
                            LookupRecordData {
                                id: record.id.as_int(),
                                parent_id: record.parent_id.as_int(),
                                text: record.text.clone(),
                                code: record.code.as_str(),
                                ..Default::default()
                            },
                        )
                    })
                    .collect();
                lookup_data.insert(record.replace_field.clone(), data);
            }
        }
    }
    Ok(lookup_data)
}

pub(crate) fn find_positions(
    data: &DataRecords,
    fields: &[String],
) -> AHashMap<String, usize> {
    fields
        .iter()
        .filter_map(|field| {
            data.field_list
                .iter()
                .position(|data_field| data_field == field)
                .map(|index| (field.clone(), index))
        })
        .collect()
}

/// Собираем значения из полей требующих замены, чтобы не дергать все данные из всех справочников
fn extract_data_values(
    data_records: &DataRecords,
    fields: &[String],
) -> HashMap<String, Vec<Id>> {
    let positions = find_positions(data_records, fields);
    data_records
        .data
        .iter()
        .flat_map(|data_record| {
            positions.iter().flat_map(move |(field, index)| {
                data_record.get(*index).into_iter().flat_map(|value| {
                    Id::from_tagged(value).map(|id| (field.clone(), id))
                })
            })
        })
        .dedup()
        .into_group_map()
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Id {
    pub id: SimpleExtId,
}

impl Id {
    pub fn int(id: i32) -> Self {
        Self {
            id: SimpleExtId::Int(id),
        }
    }

    pub fn str(id: String) -> Self {
        Self {
            id: SimpleExtId::Str(id),
        }
    }

    pub fn from_tagged(tagged: &TaggedValue) -> Option<Self> {
        match tagged {
            TaggedValue::String(s) => s.parse::<i32>().ok().map(Id::int),
            TaggedValue::Int(i) => i32::try_from(*i).ok().map(Id::int),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub(crate) enum SimpleExtId {
    Str(String),
    Int(i32),
}

impl SimpleExtId {
    pub fn as_int(&self) -> i32 {
        match self {
            SimpleExtId::Str(s) => s.parse::<i32>().unwrap_or(0),
            SimpleExtId::Int(i) => *i,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            SimpleExtId::Str(s) => s.clone(),
            SimpleExtId::Int(i) => i.to_string(),
        }
    }
}

impl Default for SimpleExtId {
    fn default() -> Self {
        Self::Int(0)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct SimpleExtRecord {
    pub id: SimpleExtId,
    pub text: String,
    #[serde(default)]
    pub parent_id: SimpleExtId,
    #[serde(default)]
    pub code: SimpleExtId,
    #[serde(default)]
    pub scheme_id: SimpleExtId,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct SimpleExtData {
    #[serde(
        alias = "agenda_status",
        alias = "estimated_commission_agenda_status",
        alias = "object_type",
        alias = "pricing_organization_unit",
        alias = "pricing_unit",
        alias = "value",
        alias = "values",
        alias = "price_analysis_method"
    )]
    #[serde(default)]
    pub value: Vec<SimpleExtRecord>,
}

impl ApiResponseData for SimpleExtData {}

// Получение справочников по search_by_id
pub async fn search_by_id_in_dictionary(
    user_id: i32,
    dict_name: &str,
    ids: &[i64],
    token: &str,
) -> IntegrationResult<AHashMap<i64, String>> {
    #[derive(Debug, Deserialize)]
    struct ApiResponse {
        status: String,
        data: ApiData,
    }

    #[derive(Debug, Deserialize)]
    struct ApiData {
        #[serde(rename = "value")]
        entries: Vec<DictionaryEntry>,
    }

    #[derive(Debug, Deserialize)]
    struct DictionaryEntry {
        id: i64,
        text: String,
        // Остальные поля не используются, поэтому можно опустить
    }

    if ids.is_empty() {
        return Ok(AHashMap::new());
    }

    let cfg = PlanningRestCfg::from_env()?;
    let url = Url::parse(cfg.url.as_str())?;
    let request_url = url.join(&format!("/api/json/{dict_name}/search_by_id/"))?;

    // TODO: Логично что search_by_id должен искать по каким-то id, но я так понимаю что мок монолита этого не поддерживает, надо уточнить
    let response: ApiResponse = reqwest::Client::new()
        .post(request_url.clone())
        .query(&[("user_id", user_id.to_string())])
        .header(
            http::header::COOKIE,
            Cookie::new("id", token.to_string()).to_string(),
        )
        .send()
        .await?
        .json()
        .await?;

    // Проверяем статус ответа
    if response.status != "s" {
        return Err(super::IntegrationError::Status(
            "search_by_id API returned non-success status".to_owned(),
        ));
    }

    // Создаем HashSet для быстрого поиска ID
    let id_set: std::collections::HashSet<_> = ids.iter().cloned().collect();

    // Собираем результат
    let result = response
        .data
        .entries
        .into_iter()
        .filter(|entry| id_set.contains(&entry.id))
        .map(|entry| (entry.id, entry.text))
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use ahash::AHashMap;
    use tokio::test;

    use super::search_by_id_in_dictionary;

    #[test]
    async fn test_simple_search_by_id() {
        let okved_texts =
            search_by_id_in_dictionary(667, "okved2", &[647, 442], "token")
                .await
                .expect("getting OKVED failed");

        let expected_map: AHashMap<_, _> = [
            (
                442,
                "Производство рафинированного соевого масла и его фракций "
                    .to_string(),
            ),
            (
                647,
                "Нанесение рисунка на текстильные изделия и готовую одежду "
                    .to_string(),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(okved_texts, expected_map);

        let okato_texts =
            search_by_id_in_dictionary(667, "okato", &[9767, 161693], "token")
                .await
                .expect("getting OKATO failed");
        let expected_map: AHashMap<_, _> = [
            (9767, "д Никулинская".to_string()),
            (161693, "п Дмитриевка".to_string()),
        ]
        .into_iter()
        .collect();

        assert_eq!(okato_texts, expected_map);
    }
}
