use super::*;
use crate::app_process;

use asez2_shared_db::db_item::AsezDate;
use shared_essential::presentation::dto::response_request::{
    EntityKind, MessageKind, Status,
};

const CHANGE_FORM_EXTRA_MIGS: &[&str] =
    &["estimated_commission/change_commission_date.sql"];

#[tokio::test]
async fn test_pre_change_commission_date() {
    run_db_test(CHANGE_FORM_EXTRA_MIGS, |pool| async move {
        let request_ok = PreChangeCommissionDateReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    7,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    8,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000008").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    102,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
            ],
        };

        let request_fail_status = PreChangeCommissionDateReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    105,
                    Uuid::parse_str("00000000-0000-0000-0005-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
            ],
        };

        let request_fail_protocol = PreChangeCommissionDateReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    106,
                    Uuid::parse_str("00000000-0000-0000-0006-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
            ],
        };


        let r1 = app_process::pre_change_commission_date(request_ok, pool.clone()).await;
        let r2 =
            app_process::pre_change_commission_date(request_fail_status, pool.clone())
                .await;
        let r3 =
            app_process::pre_change_commission_date(request_fail_protocol, pool)
                .await;

        {
            let mut r1 = r1.unwrap();
            assert_eq!(r1.data.item_list.len(), 4);
            assert_eq!(r1.status, Status::Ok);

            assert_eq!(r1.messages.kind, MessageKind::Warning);
            assert_eq!(r1.messages.messages.len(), 3);
            r1.messages.messages.sort();
            assert_eq!(r1.messages.messages[0].kind, MessageKind::Warning);
            assert_eq!(r1.messages.messages[1].kind, MessageKind::Warning);
            assert_eq!(r1.messages.messages[2].kind, MessageKind::Warning);

            assert_eq!(r1.messages.messages[2].text, String::from("ППЗ/ДС 8 включена в Повестку 2 на 02.01.1900. Вы подтверждаете изменение даты очной СК?"));
            assert_eq!(r1.messages.messages[1].text, String::from("ППЗ/ДС 7 включена в Повестку 2 на 02.01.1900. Вы подтверждаете изменение даты очной СК?"));
            assert_eq!(r1.messages.messages[0].text, String::from("ППЗ/ДС 102 включена в Повестку 3 на 03.01.1900. Вы подтверждаете изменение даты очной СК?"));
        }
        {
            let r2 = r2.unwrap();

            assert_eq!(r2.messages.messages.len(), 1);
            assert_eq!(r2.messages.messages[0].kind, MessageKind::Error);
            assert_eq!(
                &r2.messages.messages[0].text,
                "Выполнить изменение даты очной СК невозможно. 3 ППЗ/ДС находятся не на статусах СК"
            );
        }
        {
            let mut r3 = r3.unwrap();
            assert_eq!(r3.messages.messages.len(), 2);
            r3.messages.messages.sort();
            assert_eq!(r3.messages.messages[0].kind, MessageKind::Error);
            assert_eq!(r3.messages.messages[1].kind, MessageKind::Error);
            assert_eq!(
                &r3.messages.messages[1].text,
                "Выполнить изменение даты очной СК невозможно. ППЗ/ДС 4 включена в Протокол 2 от 01.01.1910"
            );
            assert_eq!(
                &r3.messages.messages[0].text,
                "Выполнить изменение даты очной СК невозможно. ППЗ/ДС 106 включена в Протокол 3 от 01.01.1910"
            );
        }
    })
    .await
}

/// Хоть ППЗ/ДС уже и включены в Повестку, но по этим agenda_item есть записи в item_agenda_protocol_relation
/// таблице
#[tokio::test]
async fn test_pre_change_agenda_protocol_item_relation_failure() {
    run_db_test(CHANGE_FORM_EXTRA_MIGS, |pool| async move {
        let req = PreChangeCommissionDateReq {
            item_list: vec![
                // Элемент имеет protocol_item с result_id=3 и ДВА связанных элемента Повестки СК
                ObjectIdentifier::new_with_type(
                    108,
                    Uuid::parse_str("00000000-0000-0000-0008-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
                // Элемент имеет protocol_item с result_id=3 и только ОДИН связанный элемент Повестки СК
                ObjectIdentifier::new_with_type(
                    109,
                    Uuid::parse_str("00000000-0000-0000-0009-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let res = app_process::pre_change_commission_date(req, pool).await.unwrap();

        assert_eq!(res.data.item_list.len(), 2);
        assert_eq!(res.status, Status::Ok);
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_change_commission_date() {
    run_db_test(CHANGE_FORM_EXTRA_MIGS, |pool| async move {

        let new_commission_date = AsezDate::try_from("2024-04-04").unwrap();
        let request_ok = ChangeCommissionDateReq {
            is_force: true,
            user_id: 123,
            item_list: vec![
                ChangeCommissionDateItem {
                    item: ObjectIdentifier::new_with_type(
                        1,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                        EntityKind::Plan
                    ),
                    commission_date: new_commission_date ,
                },
                ChangeCommissionDateItem {
                    item: ObjectIdentifier::new_with_type(
                        7,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap(),
                        EntityKind::Plan
                    ),
                    commission_date:new_commission_date,
                },
                ChangeCommissionDateItem {
                    item: ObjectIdentifier::new_with_type(
                        8,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000008").unwrap(),
                        EntityKind::Plan
                    ),
                    commission_date: new_commission_date,
                },
                ChangeCommissionDateItem {
                    item: ObjectIdentifier::new_with_type(
                        102,
                        Uuid::parse_str("00000000-0000-0000-0002-000000000000").unwrap(),
                        EntityKind::ContractAmendment
                    ),
                    commission_date: new_commission_date,
                },
            ],
        };

        let request_fail_status = ChangeCommissionDateReq {
            is_force: true,
            user_id: 123,
            item_list: vec![
                ChangeCommissionDateItem {
                    item: ObjectIdentifier::new_with_type(
                        2,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                        EntityKind::Plan
                    ),
                    commission_date: AsezDate::try_from("2021-01-01").unwrap(),
                },
                ChangeCommissionDateItem {
                    item: ObjectIdentifier::new_with_type(
                        3,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                        EntityKind::Plan
                    ),
                    commission_date: AsezDate::try_from("2022-01-07").unwrap(),
                },
                ChangeCommissionDateItem {
                    item: ObjectIdentifier::new_with_type(
                        105,
                        Uuid::parse_str("00000000-0000-0000-0005-000000000000").unwrap(),
                        EntityKind::ContractAmendment
                    ),
                    commission_date: AsezDate::try_from("2023-01-08").unwrap(),
                },
            ],
        };

        let request_fail_protocol = ChangeCommissionDateReq {
            is_force: true,
            user_id: 123,
            item_list: vec![ChangeCommissionDateItem {
                item:ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                    EntityKind::Plan
                ),
                commission_date: AsezDate::try_from("2021-01-01").unwrap(),
            }],
        };

        let proc_ctx = super::mock_processing_context(pool).await;
        let r1 = app_process::change_commission_date(request_ok, proc_ctx.clone()).await;
        let r2 =
            app_process::change_commission_date(request_fail_status, proc_ctx.clone()).await;
        let r3 =
            app_process::change_commission_date(request_fail_protocol, proc_ctx.clone())
                .await;

        {
            let r1 = r1.unwrap();

            assert_eq!(r1.status, Status::Ok);
            assert_eq!(r1.data.item_list.len(), 4);

            assert_eq!(r1.messages.messages.len(), 1);
            assert_eq!(r1.messages.kind, MessageKind::Success);
            assert_eq!(r1.messages.messages[0].text, String::from("Вы изменили дату очной СК по 4 ППЗ/ДС"));

            let have_new_commission_date = r1
                .data
                .item_list
                .iter()
                .all(|x| x.commission_date().unwrap().unwrap() == new_commission_date);
            assert!(have_new_commission_date);

            let agenda_items_select = Select::full_in::<_, EcAgendaItem>("source_uuid", r1.data.item_list.iter().map(|p| p.uuid().unwrap().into()));
            let agenda_items = EcAgendaItem::select(&agenda_items_select, &*proc_ctx.db_pool).await.unwrap();

            let verify_agenda_item  = |uuid: &str, is_removed: bool, is_excluded: bool, reviewed_at_is_none: bool| -> bool {
                agenda_items.iter().find(|i| i.uuid.to_string() == uuid).map(|i| {
                    i.is_excluded == is_excluded && i.is_removed == is_removed && i.reviewed_at.is_none() == reviewed_at_is_none
                }).unwrap_or_else(|| panic!("Не найден Элемент Повестки {}", uuid))
            };

            assert!(
                verify_agenda_item("00000000-0000-0000-0000-000000000001", false, false, false)
            );
            assert!(
                verify_agenda_item("00000000-0000-0000-0000-000000000001", false, false, false)
            );
            assert!(
                verify_agenda_item("00000000-0000-0000-0000-000000000003", true, false, false)
            );
            assert!(
                verify_agenda_item("00000000-0000-0000-0000-000000000004", true, false, false)
            );
            assert!(
                verify_agenda_item("00000000-0000-0000-0000-000000000001", false, false, false)
            );
            assert!(
                verify_agenda_item("00000000-0000-0000-0000-000000000006", false, true, true)
            );
        }

        {
            let r2 = r2.unwrap();
            assert_eq!(r2.messages.messages.len(), 1);
            assert_eq!(
                &r2.messages.messages[0].text,
                "Выполнить изменение даты очной СК невозможно. 3 ППЗ/ДС находятся не на статусах СК"
            );
        }
        {
            let r3 = r3.unwrap();
            assert_eq!(r3.messages.messages.len(), 1);
            assert_eq!(
                &r3.messages.messages[0].text,
                "Выполнить изменение даты очной СК невозможно. ППЗ/ДС 4 включена в Протокол 2 от 01.01.1910"
            );
        }
    })
    .await
}
