mod common;
use common::setup_test;
use shared_essential::presentation::dto::master_data::updates::MasterDataUpdates;
use actix_web::{dev::ServiceResponse, test};
use actix_http::StatusCode;
use shared_essential::presentation::dto::response_request::ApiResponse;
use shared_essential::presentation::dto::master_data::updates::MasterDataUpdateEntity::*;

const GET_UPDATES_ALL: &str = "/v1/master_data/get_updates/0/";

/// Получение данных справочников с момента их первоначального наполнения
#[actix_web::test]
#[ignore = "support for web service tests should be implemented"]
async fn get_updates_all() {
    let app = setup_test().await;

    let req = test::TestRequest::get().uri(GET_UPDATES_ALL).to_request();

    let response: ServiceResponse = test::call_service(&app, req).await;
    let status = response.status();

    let bytes = test::read_body(response).await;

    let content: ApiResponse<MasterDataUpdates, ()> =
        serde_json::from_slice(&bytes)
            .expect("Не удалось десериализовать тело ответа");

    assert_eq!(status, StatusCode::OK);

    assert_eq!(content.messages.messages.len(), 0);

    let master_data_updates = content.data;

    assert!(!master_data_updates.entity_list.is_empty());

    master_data_updates.entity_list.into_iter().for_each(|value| {
        let master_data_entity = value.entity;

        match master_data_entity {
            AnalysisMethod(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            AssigningExecutorMethod(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            AttachmentType(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            CriticalTypeColorScheme(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            AgendaStatus(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            ProtocolStatus(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                    assert!((value.protocol_type_id as i16) != 0);
                });
            }

            ProtocolType(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id as i16 != 0);
                });
            }

            EstimatedCommissionResult(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id as i16 != 0);
                });
            }

            EstimatedCommissionRole(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id as i16 != 0);
                });
            }

            ExpertConclusionType(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id as i16 != 0);
                });
            }

            ObjectType(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            OutputForm(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id != 0);
                });
            }

            PaymentConditions(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id as i16 != 0);
                });
            }

            PpzType(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id as i16 != 0);
                });
            }

            PriceInformationRequestType(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!(value.id as i16 != 0);
                });
            }

            PricingMethod(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            PricingOrganizationUnit(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            DepartmentResponseStatus(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            SchedulerRequestUpdateCatalog(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            PlanReasonCancelCheckReason(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id) != 0);
                });
            }

            PlanReasonCancelFunctionality(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id) != 0);
                });
            }

            PlanReasonCancelImpactArea(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id) != 0);
                });
            }

            FavoriteDictionary(list) => {
                assert!(!list.is_empty());
                list.into_iter().for_each(|value| {
                    assert!((value.id as i16) != 0);
                });
            }

            Empty => panic!(),
        }
    });
}
