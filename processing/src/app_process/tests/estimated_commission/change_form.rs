use super::*;
use crate::app_process;

use shared_essential::presentation::dto::response_request::{
    EntityKind, MessageKind, Status,
};
use tables::legacy::plans::PlanStatus;
use tokio::test;

const CHANGE_FORM_EXTRA_MIGS: &[&str] = &["estimated_commission/change_form.sql"];

#[test]
async fn test_change_form() {
    let req = ChangeFormReq {
        item_list: vec![
            ObjectIdentifierWithStatusNote::new(
                6,
                Uuid::parse_str("00000000-0000-0000-0000-000000000006").unwrap(),
                String::from("note6"),
            ),
            ObjectIdentifierWithStatusNote::new(
                9,
                Uuid::parse_str("00000000-0000-0000-0000-000000000009").unwrap(),
                String::from("note9"),
            ),
        ],
        is_force: true,
        commission_kind_id: CommissionKind::Correspondence,
        user_id: 123,
        section_id: Section::EstimatedCommissionInPerson,
    };

    run_db_test(CHANGE_FORM_EXTRA_MIGS, |pool| async move {
        let pctx = super::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let res = app_process::change_form(req, pctx.clone()).await.unwrap();

        assert_eq!(res.data.item_list.len(), 2);

        assert_eq!(res.messages.messages.len(), 1);
        assert_eq!(res.messages.messages[0].kind, MessageKind::Success);
        assert_eq!(
            &res.messages.messages[0].text,
            "Вы перевели на заочную СК 2 ППЗ/ДС"
        );

        assert!(res.data.item_list.iter().all(|p| p.status_id().unwrap()
            == PlanStatus::EstimatedCommissionCorrespondence
            && p.commission_kind_id().unwrap() == CommissionKind::Correspondence));

        let agenda_items_select = Select::full_in::<_, EcAgendaItem>(
            "source_uuid",
            res.data.item_list.iter().map(|p| p.uuid().unwrap().into()),
        );
        let agenda_items =
            EcAgendaItem::select(&agenda_items_select, &*pctx.db_pool)
                .await
                .unwrap();

        let verify_agenda_item = |uuid: &str,
                                  is_removed: bool,
                                  is_excluded: bool|
         -> bool {
            agenda_items
                .iter()
                .find(|i| i.uuid.to_string() == uuid)
                .map(|i| i.is_excluded == is_excluded && i.is_removed == is_removed)
                .unwrap_or_else(|| panic!("Не найден Элемент Повестки {}", uuid))
        };

        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000001",
            true,
            false
        ));
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000002",
            true,
            false
        ));
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000005",
            false,
            false
        ));
    })
    .await
}

#[test(flavor = "multi_thread")]
async fn test_pre_change_form() {
    run_db_test(CHANGE_FORM_EXTRA_MIGS, |pool| async move {
        let everything_ok = PreChangeFormReq {
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
            ],
            section_id: Section::EstimatedCommissionInPerson
        };

        let fail_status = PreChangeFormReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    EntityKind::Plan
                ),
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
            ],
            section_id: Section::EstimatedCommissionInPerson
        };

        let fail_protocol = PreChangeFormReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    7,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap(),
                    EntityKind::Plan
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson
        };

        let fail_agenda = PreChangeFormReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    6,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000006").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    9,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000009").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    10,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000010").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    11,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap(),
                    EntityKind::Plan
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson
        };

        let r1 = app_process::pre_change_form(everything_ok, pool.clone()).await;
        let r2 = app_process::pre_change_form(fail_status, pool.clone()).await;
        let r3 = app_process::pre_change_form(fail_protocol, pool.clone()).await;
        let r4 = app_process::pre_change_form(fail_agenda, pool).await;

        {
            let r1 = r1.unwrap();

            assert_eq!(r1.data.item_list.len(), 3);
            assert!(r1.messages.messages.is_empty());
        }
        {
            let r2 = r2.unwrap();

            assert_eq!(r2.messages.messages.len(), 1);
            assert_eq!(r2.messages.messages[0].kind, MessageKind::Error);
            assert_eq!(
                &r2.messages.messages[0].text,
                "Выполнить изменение формы СК невозможно. 2 ППЗ/ДС находятся не на статусах СК."
            );
        }
        {
            let r3 = r3.unwrap();

            assert!(r3.data.is_empty());
            assert_eq!(r3.messages.messages.len(), 2);

            assert_eq!(r3.messages.messages[0].kind, MessageKind::Error);
            assert_eq!(r3.messages.messages[1].kind, MessageKind::Error);
            assert_eq!(
                &r3.messages.messages[0].text,
                "Выполнить изменение формы СК невозможно. ППЗ/ДС 4 находится не на статусах СК."
            );
            assert_eq!(
                &r3.messages.messages[1].text,
                "ППЗ/ДС 4 включена в Протокол 2 от 01.01.1910 с решением \"Согласовано с корректировкой стоимости\". Изменить форму СК невозможно"
            );
        }
        {
            let mut r4 = r4.unwrap();

            assert!(!r4.data.is_empty());
            assert_eq!(r4.messages.messages.len(), 2);
            r4.messages.messages.sort();
            assert_eq!(r4.messages.messages[0].kind, MessageKind::Warning);
            assert_eq!(
                &r4.messages.messages[1].text,
                "ППЗ/ДС 9 включена в Повестку 1 на 01.01.1900 в статусе \"Сформирована\". Вы хотите изменить форму СК. Подтвердить?"
            );
            assert_eq!(r4.messages.messages[1].kind, MessageKind::Warning);
            assert_eq!(
                &r4.messages.messages[0].text,
                "ППЗ/ДС 6 включена в Повестку 1 на 01.01.1900 в статусе \"Сформирована\". Вы хотите изменить форму СК. Подтвердить?"
            );
        }
    })
        .await
}

/// Хоть ППЗ/ДС уже и включены в Повестку, но по этим agenda_item есть записи в item_agenda_protocol_relation
/// таблице
#[test]
async fn test_pre_change_agenda_protocol_item_relation_success() {
    run_db_test(CHANGE_FORM_EXTRA_MIGS, |pool| async move {
        let req = PreChangeFormReq {
            item_list: vec![
                // Элемент имеет protocol_item с result_id=3 и ДВА связанных элемента Повестки СК
                ObjectIdentifier::new_with_type(
                    13,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000013")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                // Элемент имеет protocol_item с result_id=3 и только ОДИН связанный элемент Повестки СК
                ObjectIdentifier::new_with_type(
                    14,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000014")
                        .unwrap(),
                    EntityKind::Plan,
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let res = app_process::pre_change_form(req, pool).await.unwrap();

        assert_eq!(res.data.item_list.len(), 2);
        assert_eq!(res.status, Status::Ok);

        assert_eq!(res.messages.messages.len(), 0, "{:#?}", res.messages.messages);
    })
    .await
}
