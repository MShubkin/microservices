#[cfg(test)]
pub(crate) mod tests {
    use crate::app_process::external::common::{
        make_reqwest_client, LookupRecordData, LookupRecordId,
    };
    use crate::app_process::external::enrichment::{SimpleExtId, SimpleExtRecord};
    use crate::app_process::external::monolith::{
        load_monolith_data, DictionaryKind, DictionaryRecord, MonolithData,
    };
    use ahash::AHashMap;
    use env_setup::MonolithCfg;
    use shared_essential::presentation::dto::response_request::{
        ApiResponse, Messages, Status,
    };

    const USER_ID: i32 = 658;
    const MONOLITH_TOKEN: &str = "test token";

    const TEST_JSON: &str = r#"
    {
      "status": "s",
      "data": {
        "changed_at": 1710363600,
        "entities": [
          {
            "UnknownItemKind_XXX": [
              { "id": 1, "code": "text_string" }
            ],
            "changed_at": 1710363601
          },
          {
            "UnknownItemKind_YYY": [
              { "text": "yyy_string" }
            ],
            "changed_at": 1710363601,
            "type": "1710363601"
          },
          {
            "changed_at": 1710363601,
            "PaymentBalanceItem": [
              { "pb_id": 3, "xxx_text": "text_string" }
            ]
          },
          {
            "ContractAmendmentItemKind": [
              {
                "id": 1,
                "text": "Item-Kind-1",
                "uuid": "081f6078-722e-440d-8d9b-f65e09ac36de",
                "is_removed": false,
                "changed_by": 25,
                "changed_at": 1710363600,
                "created_by": 25,
                "created_at": 1710363600
              }
            ],
            "changed_at": 1710363601,
            "unrecognized": "zzz-1710363601",
            "type": "xxx"
          }
        ]
      },
      "messages": {
        "kind": "Success",
        "messages": []
      }
    }
    "#;

    const EXPECTED_SERIALIZED_MONOLITH_DATA: &str = r#"
    {
      "status": "s",
      "data": {
        "entities": [
          {
            "ContractAmendmentItemKind": [
              { "id": 1, "text": "text_string", "parent_id": 0, "code": 0, "scheme_id": 0}
            ]
          }
        ]
      },
      "messages": {
        "messages": [],
        "kind": "Success"
      }
    }
    "#;

    #[test]
    fn test_serialize_monolith_data() {
        let monolith_data: ApiResponse<MonolithData, ()> = ApiResponse {
            status: Status::Ok,
            data: MonolithData {
                entities: vec![DictionaryRecord {
                    dictionary_kind: DictionaryKind::ContractAmendmentItemKind,
                    items: vec![SimpleExtRecord {
                        id: SimpleExtId::Int(1),
                        text: "text_string".to_string(),
                        ..Default::default()
                    }],
                }],
            },
            messages: Messages::default(),
            objects: vec![],
        };
        let string = serde_json::to_string(&monolith_data).unwrap();
        assert_eq!(
            string,
            EXPECTED_SERIALIZED_MONOLITH_DATA.replace(['\n', ' '], "")
        )
    }

    #[test]
    fn test_serialize_dictionary() {
        let result =
            serde_json::to_string(&DictionaryKind::ContractAmendmentItemKind)
                .unwrap();
        assert_eq!(result, "\"ContractAmendmentItemKind\"")
    }

    #[test]
    fn test_deserialize_dictionary() {
        let source_str = "\"ContractAmendmentItemKind\"";
        let result = serde_json::from_str::<DictionaryKind>(source_str).unwrap();
        assert_eq!(result, DictionaryKind::ContractAmendmentItemKind)
    }

    #[test]
    fn test_deserialize_monolith_data() {
        let result: ApiResponse<MonolithData, ()> =
            serde_json::from_str(TEST_JSON).unwrap();
        assert_eq!(result.data.entities.len(), 4);
        assert_eq!(
            result.data.entities[3].dictionary_kind,
            DictionaryKind::ContractAmendmentItemKind
        );
        assert_eq!(result.data.entities[3].items[0].id, SimpleExtId::Int(1));
        assert_eq!(result.data.entities[3].items[0].text, "Item-Kind-1");
    }

    #[tokio::test]
    #[ignore = "Тест обращается к монолиту и по умолчанию игнорируется"]
    async fn test_query_monolith_data() {
        let monolith_cfg = MonolithCfg::from_env().unwrap();
        let client = make_reqwest_client().unwrap();
        let result: AHashMap<
            DictionaryKind,
            AHashMap<LookupRecordId, LookupRecordData>,
        > = load_monolith_data(&client, &monolith_cfg, 0, USER_ID, MONOLITH_TOKEN)
            .await
            .unwrap();
        assert!(!result.is_empty())
    }
}
