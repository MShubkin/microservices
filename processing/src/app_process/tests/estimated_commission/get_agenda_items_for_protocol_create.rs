//! Тестирование процесса [`get_agenda_items_for_protocol_create`]
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::db_item::AsezDate;
use shared_essential::domain::PlanOrAmendmentRep;

use super::*;
use crate::app_process::get_agenda_items_for_protocol_create;
use crate::common::ProcessingError;

const GET_AGENDA_ITEMS_FOR_PROTOCOL_CREATE_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_agenda_items_for_protocol_create.sql"];

/// Тестирование кейса, когда пользователь передал
/// айди несуществующей повестки
#[tokio::test]
async fn not_found_agenda() {
    run_db_test(
        GET_AGENDA_ITEMS_FOR_PROTOCOL_CREATE_EXTRA_MIGS,
        |pool| async move {
            let dto = GetAgendaItemsForProtocolCreateReq {
                agenda_id: 123,
                uuid: Default::default(),
            };

            let result =
                get_agenda_items_for_protocol_create(dto, pool.clone()).await;
            if let Err(ProcessingError::GetItemList(err)) = result {
                let msg = String::from(
                    "Повестка СК c идентификатором 123 не была найдена",
                );
                assert_eq!(msg, err);
            } else {
                panic!("Failed with: {:?}", result.unwrap());
            }
        },
    )
    .await;
}

/// Тестирование кейса, когда пользователь пытается получить:
/// 1. ППЗ/ДС, которые не включены в Протокол
/// 2. ППЗ/ДС, которые включены в Протокол
/// 3. ППЗ/ДС, которые еще не включены в Повестку СК
#[tokio::test]
async fn full_coverage_request() {
    run_db_test(
        GET_AGENDA_ITEMS_FOR_PROTOCOL_CREATE_EXTRA_MIGS,
        |pool| async move {
            let dto = GetAgendaItemsForProtocolCreateReq {
                agenda_id: 1,
                uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                    .unwrap(),
            };

            let result = get_agenda_items_for_protocol_create(dto, pool.clone())
                .await
                .unwrap();
            let data: GetAgendaItemsForProtocolCreateResponseData = result.data;

            assert_eq!(data.item_list.len(), 3);

            assert_eq!(data.agenda_id, 1);
            assert_eq!(
                data.uuid,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
            );
            assert_eq!(
                data.meeting_date,
                AsezDate::try_from("2000-01-01").unwrap()
            );

            verify_item(
                &data.item_list,
                "00000000-0000-0000-0000-000000000001",
                true,
            );
            verify_item(
                &data.item_list,
                "00000000-0000-0000-0000-000000000011",
                true,
            );
            verify_item(
                &data.item_list,
                "00000000-0000-0000-0000-000000000012",
                false,
            );
        },
    )
    .await;
}

/// Проверка, что все поля содержатся в ответе
fn verify_item(
    items: &[GetAgendaItemsForProtocolCreateItem],
    uuid: &str,
    has_protocol: bool,
) {
    let uuid = Uuid::parse_str(uuid).unwrap();
    let item = items
        .iter()
        .find(|item| match &item.plan {
            PlanOrAmendmentRep::Plan(p) => p.uuid.unwrap() == uuid,
            PlanOrAmendmentRep::Amendment(p) => p.uuid.unwrap() == uuid,
        })
        .unwrap();

    let plan_check = item.plan.uuid().is_some()
        && item.plan.plan_id().is_some()
        && item.plan.customer_id().is_some()
        && item.plan.supplier_id().is_some()
        && item.plan.section_id().is_some()
        && item.plan.status_id().is_some()
        && item.plan.sum_excluded_vat().is_none();
    assert!(plan_check);

    assert!(
        item.agenda_item.sum_excluded_vat.is_some()
            && item.agenda_item.reviewed_at.is_some()
    );
    if has_protocol {
        assert!(item._meta.is_some());
        assert!(item
            ._meta
            .as_ref()
            .unwrap()
            .disabled_field_list
            .contains(&String::from("is_can_be_included_in_protocol")))
    }
}
