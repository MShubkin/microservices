#[cfg(test)]
pub(crate) mod tests {
    use crate::application::external::common::{
        make_reqwest_client, query_data, LookupData, LookupDataFieldMap,
        LookupDataIdMap, LookupRecordData, LookupRecordId, MasterDataCfg,
        RequestType,
    };
    use crate::application::external::enrichment::{
        enrich_data_records, find_positions, replace_lookup_data, Id, SimpleExtData,
    };
    use crate::presentation::dto::general::{DataRecords, TaggedValue};
    use crate::presentation::dto::response_request::EntityKind;
    use ahash::AHashMap;
    use env_setup::{MDSCfg, MonolithCfg};

    const USER_ID: i32 = 658;
    const MONOLITH_TOKEN: &str = "monolith-token";

    fn make_sample_data_records() -> DataRecords {
        DataRecords {
            captions: vec![
                "Идентификатор".to_string(),
                "Клиент".to_string(),
                "Статус повестки".to_string(),
                "Изменерение".to_string(),
                "Сумма".to_string(),
                "Курс".to_string(),
                "Количество".to_string(),
            ],
            field_list: vec![
                "id".to_string(),
                "customer_id".to_string(),
                "agenda_status_id".to_string(),
                "some_float".to_string(),
                "some_sum".to_string(),
                "some_rate".to_string(),
                "quantity".to_string(),
            ],
            data: vec![
                vec![
                    TaggedValue::Int(11),
                    TaggedValue::Int(1),
                    TaggedValue::Int(100),
                    TaggedValue::Float(100.0),
                    TaggedValue::CValue(123.45.into()),
                    TaggedValue::CRate(100.into()),
                    TaggedValue::Quantity(4.234.into()),
                ],
                vec![
                    TaggedValue::Int(12),
                    TaggedValue::Int(2),
                    TaggedValue::Int(200),
                    TaggedValue::Float(200.0),
                    TaggedValue::CValue(10_000_000.into()),
                    TaggedValue::CRate(100.into()),
                    TaggedValue::Quantity(1.into()),
                ],
                vec![
                    TaggedValue::Int(13),
                    TaggedValue::Int(3),
                    TaggedValue::Int(300),
                    TaggedValue::Float(300.0),
                    TaggedValue::CValue(0.23.into()),
                    TaggedValue::CRate(0.0343.into()),
                    TaggedValue::Quantity(10_000_000_000.into()),
                ],
            ],
            entity_kind: vec![EntityKind::Plan; 3],
        }
    }

    fn make_enriched_sample_data_records() -> DataRecords {
        DataRecords {
            captions: vec![
                "Идентификатор".to_string(),
                "Клиент".to_string(),
                "Статус повестки".to_string(),
                "Изменерение".to_string(),
                "Сумма".to_string(),
                "Курс".to_string(),
                "Количество".to_string(),
            ],
            field_list: vec![
                "id".to_string(),
                "customer_id".to_string(),
                "agenda_status_id".to_string(),
                "some_float".to_string(),
                "some_sum".to_string(),
                "some_rate".to_string(),
                "quantity".to_string(),
            ],
            data: vec![
                vec![
                    TaggedValue::Int(11),
                    TaggedValue::String("Customer1".to_string()),
                    TaggedValue::String("Status100".to_string()),
                    TaggedValue::Float(100.0),
                    TaggedValue::CValue(123.45.into()),
                    TaggedValue::CRate(100.into()),
                    TaggedValue::Quantity(4.234.into()),
                ],
                vec![
                    TaggedValue::Int(12),
                    TaggedValue::Int(2),
                    TaggedValue::Int(200),
                    TaggedValue::Float(200.0),
                    TaggedValue::CValue(10_000_000.into()),
                    TaggedValue::CRate(100.into()),
                    TaggedValue::Quantity(1.into()),
                ],
                vec![
                    TaggedValue::Int(13),
                    TaggedValue::String("Customer3".to_string()),
                    TaggedValue::String("Status300".to_string()),
                    TaggedValue::Float(300.0),
                    TaggedValue::CValue(0.23.into()),
                    TaggedValue::CRate(0.03429.into()),
                    TaggedValue::Quantity(10_000_000_000.into()),
                ],
            ],
            entity_kind: vec![EntityKind::Plan; 3],
        }
    }

    fn make_lookup_data() -> LookupDataFieldMap {
        let mut lookup_data: LookupDataFieldMap = AHashMap::new();
        let customer_map: LookupDataIdMap = AHashMap::from_iter(vec![
            (
                LookupRecordId::with_id(1),
                LookupRecordData {
                    id: 1,
                    parent_id: 0,
                    text: "Customer1".to_string(),
                    ..Default::default()
                },
            ),
            (
                LookupRecordId::with_id(-2),
                LookupRecordData {
                    id: -2,
                    parent_id: 0,
                    text: "Customer2".to_string(),
                    ..Default::default()
                },
            ),
            (
                LookupRecordId::with_id(3),
                LookupRecordData {
                    id: 3,
                    parent_id: 0,
                    text: "Customer3".to_string(),
                    ..Default::default()
                },
            ),
        ]);
        let agenda_status_map: LookupDataIdMap = AHashMap::from_iter(vec![
            (
                LookupRecordId::with_id(100),
                LookupRecordData {
                    id: 100,
                    parent_id: 0,
                    text: "Status100".to_string(),
                    ..Default::default()
                },
            ),
            (
                LookupRecordId::with_id(-200),
                LookupRecordData {
                    id: -200,
                    parent_id: 0,
                    text: "Status200".to_string(),
                    ..Default::default()
                },
            ),
            (
                LookupRecordId::with_id(300),
                LookupRecordData {
                    id: 300,
                    parent_id: 0,
                    text: "Status300".to_string(),
                    ..Default::default()
                },
            ),
        ]);
        lookup_data.insert("customer_id".to_string(), customer_map);
        lookup_data.insert("agenda_status_id".to_string(), agenda_status_map);
        lookup_data
    }

    #[test]
    fn find_positions_test() {
        let data_records = DataRecords {
            captions: vec![
                "Один".to_string(),
                "Два".to_string(),
                "Три".to_string(),
            ],
            field_list: vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
            ],
            data: vec![],
            entity_kind: vec![],
        };
        let fields = vec![
            "four".to_string(),
            "three".to_string(),
            "six".to_string(),
            "two".to_string(),
        ];
        let hash_map = find_positions(&data_records, &fields);
        assert_eq!(hash_map.len(), 2);
    }

    #[tokio::test]
    #[ignore = "Тест по какой-то причине не работает и нуждается в исправлении в следующем МР"]
    async fn query_https_data_test() {
        let client = make_reqwest_client().unwrap();
        let base = MonolithCfg::from_env().unwrap().url;
        let query_path = "/api/json/organization/search_by_id/";
        let query_url = base.join(query_path).unwrap();

        let data = vec![Id::int(1), Id::int(2), Id::int(3)];
        let res = query_data::<SimpleExtData>(
            &client,
            query_url,
            RequestType::SearchById(data),
            USER_ID,
            MONOLITH_TOKEN,
        )
        .await
        .unwrap();

        assert!(!res.value.is_empty());
    }

    #[tokio::test]
    #[ignore = "Тест по какой-то причине не работает и нуждается в исправлении в следующем МР"]
    async fn query_mds_http_data_test() {
        let client = make_reqwest_client().unwrap();
        let base = MDSCfg::from_env().unwrap().url;
        let query_path = "/rest/dictionary/v1/agenda_status/search_by_id/";
        let query_url = base.join(query_path).unwrap();

        let data = vec![Id::int(100), Id::int(200), Id::int(300)];
        let res = query_data::<SimpleExtData>(
            &client,
            query_url,
            RequestType::SearchById(data),
            USER_ID,
            MONOLITH_TOKEN,
        )
        .await
        .unwrap();

        assert_eq!(res.value.len(), 3);
    }

    #[test]
    fn replace_data_test() {
        let mut sample_data: DataRecords = make_sample_data_records();
        let master_data_cfg = MasterDataCfg::default();
        let lookup_data = LookupData {
            id_lookup_data: make_lookup_data(),
            common_lookup_data: Default::default(),
            enum_lookup_data: Default::default(),
        };
        replace_lookup_data(&mut sample_data, &master_data_cfg, &lookup_data);

        assert_eq!(sample_data, make_enriched_sample_data_records())
    }

    #[tokio::test]
    #[ignore = "Тест обращается к сервисам Монолита и НСИ"]
    async fn enrich_data_records_test() {
        let res = enrich_data_records(
            make_sample_data_records(),
            MasterDataCfg::default(),
            USER_ID,
            MONOLITH_TOKEN,
        )
        .await
        .unwrap();

        assert_eq!(res.data.len(), 3);
    }
}
