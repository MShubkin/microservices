use super::*;
use crate::app_process;
use crate::presentation::business_messages::plan::PlanCancelMessage;
use asez2_shared_db::db_item::AsezDate;
use shared_essential::domain::tables::legacy::plans::PlanStatus;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};

const CANCEL_PLANS_EXTRA_MIGS: &[&str] = &["estimated_commission/cancel_plans.sql"];

#[tokio::test]
async fn test_pre_cancel_plan_a() {
    run_db_test(CANCEL_PLANS_EXTRA_MIGS, |pool| async move {
        let cancel_ok = PreCancelPlansReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    7,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000007")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    8,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000008")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    103,
                    Uuid::parse_str("00000000-0000-0000-0003-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let cancel_fail_status = PreCancelPlansReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    104,
                    Uuid::parse_str("00000000-0000-0000-0004-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let cancel_fail_protocol = PreCancelPlansReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    7,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000007")
                        .unwrap(),
                    EntityKind::Plan,
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let pctx = super::mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let r1 =
            app_process::pre_cancel_plan(cancel_ok, pctx.db_pool.clone()).await;

        let r2 =
            app_process::pre_cancel_plan(cancel_fail_status, pctx.db_pool.clone())
                .await;

        let r3 = app_process::pre_cancel_plan(
            cancel_fail_protocol,
            pctx.db_pool.clone(),
        )
        .await;
        {
            let r1 = r1.unwrap();

            assert_eq!(r1.data.item_list.len(), 4);
            assert!(r1.messages.messages.is_empty());
        }
        {
            let r2 = r2.unwrap();
            assert!(r2.data.is_empty());

            let expected_messages = vec![PlanCancelMessage::InvalidPlanStatus
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 2,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Plan(Plan {
                        id: 3,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 104,
                        ..Default::default()
                    }),
                ])];

            assert_eq!(r2.messages.messages, expected_messages);
            assert_eq!(r2.messages.messages[0].kind, MessageKind::Error);
        }
        {
            let r3 = r3.unwrap();

            assert!(r3.data.is_empty());
            let expected_messages = vec![
                PlanCancelMessage::InvalidPlanStatus.singular(
                    &PlanOrAmendment::Plan(Plan {
                        id: 4,
                        ..Default::default()
                    }),
                ),
                PlanCancelMessage::AlreadyInProtocolErr(&EcProtocol {
                    id: 2,
                    protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                    status_id: EcProtocolStatus::Formed,
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 4,
                    ..Default::default()
                })),
            ];
            assert_eq!(r3.messages.messages, expected_messages);
        }
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Тест отключен до обощения реализации Rabbit сервисов"]
async fn test_cancel_plan_a() {
    run_db_test(CANCEL_PLANS_EXTRA_MIGS, |pool| async move {
        let cancel_ok = CancelPlansReq {
            item_list: vec![
                ObjectIdentifierWithStatusNote::new_with_reason_only(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                    String::from("hello"),
                    Some(1),
                ),
                ObjectIdentifierWithStatusNote::new_with_reason_only(
                    7,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000007")
                        .unwrap(),
                    EntityKind::Plan,
                    String::from("goodbye"),
                    Some(1),
                ),
                ObjectIdentifierWithStatusNote::new_with_reason(
                    8,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000008")
                        .unwrap(),
                    EntityKind::Plan,
                    String::from("yes"),
                    Some(2),
                    Some(5),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    102,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                    String::from("no"),
                ),
            ],
            user_id: 9999,
            section_id: Section::EstimatedCommissionInPerson,
        };

        let cancel_fail_status = CancelPlansReq {
            item_list: vec![
                ObjectIdentifierWithStatusNote::new_with_reason_only(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Plan,
                    String::default(),
                    Some(1),
                ),
                ObjectIdentifierWithStatusNote::new_with_reason_only(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Plan,
                    String::default(),
                    Some(1),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    105,
                    Uuid::parse_str("00000000-0000-0000-0005-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                    String::default(),
                ),
            ],
            user_id: 9999,
            section_id: Section::EstimatedCommissionInPerson,
        };

        let cancel_fail_protocol = CancelPlansReq {
            item_list: vec![ObjectIdentifierWithStatusNote::new(
                4,
                Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                String::default(),
            )],
            user_id: 9999,
            section_id: Section::EstimatedCommissionInPerson,
        };
        let pctx = super::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;
        let master_data_service = super::master_data_service(&pctx).await;

        let r1 = app_process::cancel_plan(
            cancel_ok,
            pctx.clone(),
            master_data_service.clone(),
        )
        .await;

        let r2 = app_process::cancel_plan(
            cancel_fail_status,
            pctx.clone(),
            master_data_service.clone(),
        )
        .await;

        let r3 = app_process::cancel_plan(
            cancel_fail_protocol,
            pctx.clone(),
            master_data_service.clone(),
        )
        .await;

        {
            let r1 = r1.unwrap();

            let expected_messages = vec![PlanCancelMessage::Success.plural(&[
                PlanOrAmendment::Plan(Plan {
                    id: 1,
                    ..Default::default()
                }),
                PlanOrAmendment::Plan(Plan {
                    id: 7,
                    ..Default::default()
                }),
                PlanOrAmendment::Plan(Plan {
                    id: 8,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 102,
                    ..Default::default()
                }),
            ])];
            assert_eq!(r1.messages.messages, expected_messages);

            let all_cancelled = r1.data.item_list.iter().all(|p| {
                p.status_id().unwrap() == PlanStatus::PlanCancelled
                    && p.commission_date().unwrap().is_none()
                    && p.commission_kind_id().unwrap() == CommissionKind::Undefined
            });
            assert_eq!(r1.data.item_list.len(), 4);
            assert!(all_cancelled);

            let agenda_items_select = Select::full_in::<_, EcAgendaItem>(
                "source_uuid",
                r1.data.item_list.iter().map(|p| p.uuid().unwrap().into()),
            );
            let agenda_items =
                EcAgendaItem::select(&agenda_items_select, &*pctx.db_pool)
                    .await
                    .unwrap();

            let verify_agenda_item = |uuid: &str,
                                      is_removed: bool,
                                      is_excluded: bool,
                                      reviewed_at_is_none: bool|
             -> bool {
                agenda_items
                    .iter()
                    .find(|i| i.uuid.to_string() == uuid)
                    .map(|i| {
                        i.is_excluded == is_excluded
                            && i.is_removed == is_removed
                            && i.reviewed_at.is_none() == reviewed_at_is_none
                    })
                    .unwrap_or_else(|| {
                        panic!("Не найден Элемент Повестки {}", uuid)
                    })
            };

            assert!(verify_agenda_item(
                "00000000-0000-0000-0000-000000000001",
                true,
                false,
                false
            ));
            assert!(verify_agenda_item(
                "00000000-0000-0000-0000-000000000002",
                false,
                true,
                true
            ));
            assert!(verify_agenda_item(
                "00000000-0000-0000-0000-000000000003",
                false,
                false,
                false,
            ));
            // Тут проверка на то, что обновляется именно самый новый элемент
            // Повестки СК. Позиция 4 и 5 обе идут к тому же плану, но 5 принадлежит
            // к более новый повестке. Так что он должен удалится.
            assert!(verify_agenda_item(
                "00000000-0000-0000-0000-000000000004",
                false,
                false,
                false
            ));
            assert!(verify_agenda_item(
                "00000000-0000-0000-0000-000000000005",
                false,
                true,
                true
            ));
        }
        {
            let r2 = r2.unwrap();

            assert!(r2.data.is_empty());

            let expected_messages = vec![PlanCancelMessage::InvalidPlanStatus
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 2,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Plan(Plan {
                        id: 3,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 105,
                        ..Default::default()
                    }),
                ])];
            assert_eq!(r2.messages.messages, expected_messages);

            let sel = Select::default().add_replace_order_asc("object_uuid");

            let histories = StatusHistory::select(&sel, &*pool).await.unwrap();
            assert_eq!(histories.len(), 4);
            assert_eq!(&histories[0].comment, "hello");
            assert_eq!(&histories[1].comment, "goodbye");
            assert_eq!(&histories[2].comment, "yes");
            assert_eq!(&histories[3].comment, "no");
        }
        {
            let r3 = r3.unwrap();

            assert!(r3.data.is_empty());

            let expected_messages = vec![
                PlanCancelMessage::InvalidPlanStatus.singular(
                    &PlanOrAmendment::Plan(Plan {
                        id: 4,
                        ..Default::default()
                    }),
                ),
                PlanCancelMessage::AlreadyInProtocolErr(&EcProtocol {
                    id: 2,
                    protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                    status_id: EcProtocolStatus::Formed,
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 4,
                    ..Default::default()
                })),
            ];
            assert_eq!(r3.messages.messages, expected_messages);
        }
    })
    .await
}
