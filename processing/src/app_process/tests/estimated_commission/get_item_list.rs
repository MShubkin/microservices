//! Тестирование процесса `get_agenda_list`
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::db_item::AsezDate;
use shared_essential::presentation::dto::response_request::Status;

use super::*;
use crate::app_process::get_item_list;
use crate::common::ProcessingError;

const GET_ITEM_LIST_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_item_list.sql"];

/// Тестирование кейса, когда не была найдена повестка
#[tokio::test]
async fn not_found_agenda() {
    run_db_test(GET_ITEM_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetItemListReq {
            id: 0,
            section_id: Section::EstimatedCommissionInPersonPreparation,
            is_registered_by_d647: Some(true),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        };

        let result = get_item_list(dto, pool.clone()).await;

        if let Err(ProcessingError::GetItemList(msg)) = result {
            assert_eq!(
                msg,
                String::from("Повестка СК c идентификатором 0 не была найдена")
            );
        } else {
            panic!("Expected Result::Err, but got: {:#?}", result)
        }
    })
    .await;
}

/// Тестирование кейса, когда не был найден протокол
#[tokio::test]
async fn not_found_protocol() {
    run_db_test(GET_ITEM_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetItemListReq {
            id: 0,
            section_id: Section::EstimatedCommissionSummingUpInPerson,
            is_registered_by_d647: Some(true),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        };

        let result = get_item_list(dto, pool.clone()).await;

        if let Err(ProcessingError::GetItemList(msg)) = result {
            assert_eq!(
                msg,
                String::from("Протокол СК c идентификатором 0 не был найден")
            );
        } else {
            panic!("Expected Result::Err, but got: {:#?}", result)
        }
    })
    .await;
}

/// Тестирование кейса, когда не был найден протокол
#[tokio::test]
async fn not_found_deleted_protocol() {
    run_db_test(GET_ITEM_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetItemListReq {
            id: 2,
            section_id: Section::EstimatedCommissionSummingUpInPerson,
            is_registered_by_d647: Some(true),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
        };

        let result = get_item_list(dto, pool.clone()).await;

        if let Err(ProcessingError::GetItemList(msg)) = result {
            assert_eq!(
                msg,
                String::from("Протокол СК c идентификатором 2 не был найден")
            );
        } else {
            panic!("Expected Result::Err, but got: {:#?}", result)
        }
    })
    .await;
}

#[tokio::test]
async fn get_item_list_in_person_preparation() {
    run_db_test(GET_ITEM_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetItemListReq {
            id: 1,
            section_id: Section::EstimatedCommissionInPersonPreparation,
            is_registered_by_d647: Some(true),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        };

        let res = get_item_list(dto, pool.clone()).await.unwrap();

        assert_eq!(res.status, Status::Ok);

        let data: GetItemListResponseData = res.data;
        assert_eq!(data.id, 1);
        assert!(data.protocol_date.is_none());
        assert_eq!(
            data.meeting_date.unwrap(),
            AsezDate::try_from("1911-11-11").unwrap()
        );
        // Приходит два элемента, так как признак is_registered_by_d647=true
        assert_eq!(data.item_list.len(), 2);

        let verify_closure =
            |protocol_item: Option<&EcProtocolItemRep>,
             agenda_item: Option<&EcAgendaItemRep>| {
                let agenda_item = agenda_item.unwrap();
                let reviewed_at = agenda_item.reviewed_at.unwrap().unwrap();
                assert_eq!(
                    (reviewed_at.year(), reviewed_at.month(), reviewed_at.day()),
                    (2000, 1, 1)
                );
                assert!(agenda_item.sum_excluded_vat.is_some());
                assert!(protocol_item.is_none());
            };
        verify_item(
            &data.item_list,
            true,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap(),
            verify_closure,
        );
        verify_item(
            &data.item_list,
            true,
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000021").unwrap(),
            verify_closure,
        );
    })
    .await;
}

#[tokio::test]
async fn get_item_list_summing_up_in_person() {
    run_db_test(GET_ITEM_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetItemListReq {
            id: 1,
            section_id: Section::EstimatedCommissionSummingUpInPerson,
            is_registered_by_d647: Some(true),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        };

        let res = get_item_list(dto, pool.clone()).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        let data: GetItemListResponseData = res.data;
        assert_eq!(data.id, 1);
        assert_eq!(
            data.protocol_date.unwrap(),
            AsezDate::try_from("1911-11-11").unwrap()
        );
        assert!(data.meeting_date.is_none());
        // Приходит только один элемент, так как только один protocol_item имеет
        // is_registered=false и is_removed=false
        assert_eq!(data.item_list.len(), 1);

        let verify_closure =
            |protocol_item: Option<&EcProtocolItemRep>,
             agenda_item: Option<&EcAgendaItemRep>| {
                let protocol_item = protocol_item.unwrap();
                assert!(protocol_item.commission_sum_excluded_vat.is_some());
                assert!(protocol_item.pricing_sum_excluded_vat.is_some());
                assert!(protocol_item.result_id.is_some());
                assert!(agenda_item.is_none());
            };
        verify_item(
            &data.item_list,
            false,
            Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000022").unwrap(),
            verify_closure,
        );
    })
    .await;
}

#[tokio::test]
async fn get_item_list_summing_up_correspondence() {
    run_db_test(GET_ITEM_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetItemListReq {
            id: 1,
            section_id: Section::EstimatedCommissionSummingUpCorrespondence,
            // Не имеет значения
            is_registered_by_d647: Some(true),
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        };

        let res = get_item_list(dto, pool.clone()).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        let data: GetItemListResponseData = res.data;

        assert_eq!(data.id, 1);
        assert_eq!(
            data.protocol_date.unwrap(),
            AsezDate::try_from("1911-11-11").unwrap()
        );
        assert!(data.meeting_date.is_none());
        assert_eq!(data.item_list.len(), 3);

        let verify_closure =
            |protocol_item: Option<&EcProtocolItemRep>,
             agenda_item: Option<&EcAgendaItemRep>| {
                let protocol_item = protocol_item.unwrap();
                assert!(protocol_item.commission_sum_excluded_vat.is_some());
                assert!(protocol_item.result_id.is_none());
                assert!(agenda_item.is_none());
            };
        verify_item(
            &data.item_list,
            false,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap(),
            verify_closure,
        );
        verify_item(
            &data.item_list,
            false,
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000021").unwrap(),
            verify_closure,
        );
        verify_item(
            &data.item_list,
            false,
            Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000022").unwrap(),
            verify_closure,
        );
    })
    .await;
}

/// Проверка того, что нужные поля были отданы
fn verify_item<F>(
    item_list: &[GetItemListItem],
    is_agenda_item: bool,
    agenda_or_protocol_item_uuid: Uuid,
    plan_uuid: Uuid,
    verify_protocol_and_agenda_item: F,
) where
    F: for<'a> FnOnce(Option<&'a EcProtocolItemRep>, Option<&'a EcAgendaItemRep>),
{
    item_list
        .iter()
        .find(|item| {
            if is_agenda_item {
                item.agenda_item.as_ref().unwrap().uuid.unwrap()
                    == agenda_or_protocol_item_uuid
            } else {
                item.protocol_item.as_ref().unwrap().uuid.unwrap()
                    == agenda_or_protocol_item_uuid
            }
        })
        .map(|item| {
            let GetItemListItem {
                protocol_item,
                plan,
                agenda_item,
            } = item;
            assert_eq!(plan.uuid().unwrap(), plan_uuid);
            assert!(plan.customer_id().is_some());
            assert!(plan.contract_subject().is_some());
            assert!(plan.pricing_expert_id().is_some());
            assert!(plan.pricing_resume().is_some());
            assert!(plan.supplier_id().is_some());
            assert!(plan.currency_id().is_some());
            assert!(plan.commission_date().is_some());
            assert!(plan.section_id().is_some());

            verify_protocol_and_agenda_item(
                protocol_item.as_ref(),
                agenda_item.as_ref(),
            )
        })
        .expect("Не был отдан нужный agenda_item");
}
