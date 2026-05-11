use asez2_shared_db::db_item::AsezTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CreatePriceInformationRequest {
    pub plan_data: Vec<PlanUUIDs>,
    pub period_of_validity: AsezTimestamp,
    pub request_type: i16,
    pub suppliers: Option<Vec<SupplierFormData>>,
    pub technical_specification: FileFormData,
    pub draft_treaty: FileFormData,
    pub template_tkp: FileFormData,
    pub additional_documents: Option<Vec<FileFormData>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct SupplierFormData {
    pub supplier_id: i32,
    pub additional_email: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct PlanUUIDs {
    pub plan_uuid: String,
    pub plan_item_uuids: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct FileFormData {
    pub uuid: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use asez2_shared_db::db_item::AsezTimestamp;
    use crate::presentation::dto::technical_commercial_proposal::create_price_information_request::{CreatePriceInformationRequest, FileFormData, PlanUUIDs, SupplierFormData};

    #[test]
    fn test_structure_createpriceinformationrequest() {
        let plan1 = PlanUUIDs {
            plan_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            plan_item_uuids: vec![
                "550e8400-e29b-41d4-a716-446655440001".to_string(),
                "550e8400-e29b-41d4-a716-446655440002".to_string(),
            ],
        };

        let plan2 = PlanUUIDs {
            plan_uuid: "550e8400-e29b-41d4-a716-446655441000".to_string(),
            plan_item_uuids: vec![
                "550e8400-e29b-41d4-a716-146655440001".to_string(),
                "510e8400-e29b-41d4-a716-446655440002".to_string(),
            ],
        };

        let supplier1 = SupplierFormData {
            supplier_id: 1111,
            additional_email: Some("email1".to_string()),
        };
        let supplier2 = SupplierFormData {
            supplier_id: 2222,
            additional_email: Some("email2".to_string()),
        };

        let technical_specification = FileFormData {
            uuid: "550e8400-e29b-41d4-a717-446655441000".to_string(),
            name: "technical_specification.doc".to_string(),
        };

        let draft_treaty = FileFormData {
            uuid: "550e8400-e29b-41d4-a717-446655441100".to_string(),
            name: "draft_treaty.doc".to_string(),
        };

        let template_tkp = FileFormData {
            uuid: "550e8400-e29b-41d4-a717-446655441111".to_string(),
            name: "template_tkp.doc".to_string(),
        };

        let additional_document1 = FileFormData {
            uuid: "5522e8400-e29b-41d4-a717-446655441111".to_string(),
            name: "additional_document1.doc".to_string(),
        };

        let additional_document2 = FileFormData {
            uuid: "6522e8400-e29b-41d4-a717-446655441111".to_string(),
            name: "additional_document2.doc".to_string(),
        };

        let request1 = CreatePriceInformationRequest {
            plan_data: vec![plan1, plan2],
            //period_of_validity: Date(time::macros::date!(2024 - 01 - 01)),
            period_of_validity: AsezTimestamp::default(),
            request_type: 1,
            suppliers: Some(vec![supplier1, supplier2]),
            technical_specification,
            draft_treaty,
            template_tkp,
            additional_documents: Some(vec![
                additional_document1,
                additional_document2,
            ]),
        };

        let json = r#"
 {
   "plan_data":[
      {
         "plan_uuid":"550e8400-e29b-41d4-a716-446655440000",
         "plan_item_uuids":[
            "550e8400-e29b-41d4-a716-446655440001",
            "550e8400-e29b-41d4-a716-446655440002"
         ]
      },
      {
         "plan_uuid":"550e8400-e29b-41d4-a716-446655441000",
         "plan_item_uuids":[
            "550e8400-e29b-41d4-a716-146655440001",
            "510e8400-e29b-41d4-a716-446655440002"
         ]
      }
   ],
   "period_of_validity":"01-01-2024",
   "request_type":1,
   "suppliers":[
      {
         "supplier_id":1111,
         "additional_email":"email1"
      },
      {
         "supplier_id":2222,
         "additional_email":"email2"
      }
   ],
   "technical_specification":{
      "uuid":"550e8400-e29b-41d4-a717-446655441000",
      "name":"technical_specification.doc"
   },
   "draft_treaty":{
      "uuid":"550e8400-e29b-41d4-a717-446655441100",
      "name":"draft_treaty.doc"
   },
   "template_tkp":{
      "uuid":"550e8400-e29b-41d4-a717-446655441111",
      "name":"template_tkp.doc"
   },
   "additional_documents":[
      {
         "uuid":"5522e8400-e29b-41d4-a717-446655441111",
         "name":"additional_document1.doc"
      },
      {
         "uuid":"6522e8400-e29b-41d4-a717-446655441111",
         "name":"additional_document2.doc"
      }
   ]
}"#;

        let request2: CreatePriceInformationRequest =
            serde_json::from_str(json).unwrap();
        assert_eq!(request1, request2);
    }
}
