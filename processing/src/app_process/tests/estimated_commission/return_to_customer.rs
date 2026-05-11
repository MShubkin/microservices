use super::*;
use crate::app_process;
use crate::presentation::business_messages::plan::PlanReturnToCustomerMessage;

use ahash::AHashSet;
use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::uuid;
use return_to_customer::return_to_expert::{
    assert_agenda_item, assert_plan, assert_protocol_item,
};
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};

const RETURN_TO_CUSTOMER_EXTRA_MIGS: &[&str] =
    &["estimated_commission/return_to_customer.sql"];

#[tokio::test]
async fn test_pre_return_to_customer_correspondence() {
    run_db_test(RETURN_TO_CUSTOMER_EXTRA_MIGS, |pool| async move {
        let request_ok = PreReturnToCustomerReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    6,
                    uuid!("00000000-0000-0000-0000-000000000006"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    105,
                    uuid!("00000000-0000-0000-0005-000000000000"),
                    EntityKind::ContractAmendment,
                ),
                ObjectIdentifier::new_with_type(
                    6,
                    uuid!("00000000-0000-0000-0006-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionCorrespondence,
        };

        let request_fail_status = PreReturnToCustomerReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    7,
                    uuid!("00000000-0000-0000-0000-000000000007"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    107,
                    uuid!("00000000-0000-0000-0007-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionCorrespondence,
        };

        let request_fail_protocol = PreReturnToCustomerReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    8,
                    uuid!("00000000-0000-0000-0000-000000000008"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    108,
                    uuid!("00000000-0000-0000-0008-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionCorrespondence,
        };

        let res_ok =
            app_process::pre_return_to_customer(request_ok, pool.clone()).await;
        let res_fail_status =
            app_process::pre_return_to_customer(request_fail_status, pool.clone())
                .await;
        let res_fail_protocol =
            app_process::pre_return_to_customer(request_fail_protocol, pool).await;

        {
            let res = res_ok.unwrap();
            assert_eq!(res.data.item_list.len(), 3);
            assert!(res.messages.messages.is_empty());
        }

        {
            let res = res_fail_status.unwrap();

            assert_eq!(res.messages.kind, MessageKind::Error);
            let messages = vec![PlanReturnToCustomerMessage::InvalidPlanStatus
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 7,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 107,
                        ..Default::default()
                    }),
                ])];

            assert_eq!(res.messages.messages, messages);
        }

        {
            let res = res_fail_protocol.unwrap();

            assert_eq!(res.messages.kind, MessageKind::Error);
            let messages = vec![
                PlanReturnToCustomerMessage::AlreadyInProtocolWarn(&EcProtocol {
                    id: 1,
                    protocol_date: AsezDate::try_from("1910-01-01").unwrap(),
                    status_id: EcProtocolStatus::Formed,
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 8,
                    ..Default::default()
                })),
                PlanReturnToCustomerMessage::AlreadyInProtocolErr(&EcProtocol {
                    id: 2,
                    protocol_date: AsezDate::try_from("1910-01-01").unwrap(),
                    status_id: EcProtocolStatus::SignaturePending,
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 108,
                        ..Default::default()
                    },
                )),
            ];

            assert_eq!(res.messages.messages, messages);
        }
    })
    .await
}

#[tokio::test]
async fn test_pre_return_to_customer_in_person() {
    run_db_test(RETURN_TO_CUSTOMER_EXTRA_MIGS, |pool| async move {
        let request_ok = PreReturnToCustomerReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    uuid!("00000000-0000-0000-0000-000000000001"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    2,
                    uuid!("00000000-0000-0000-0000-000000000002"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    101,
                    uuid!("00000000-0000-0000-0001-000000000000"),
                    EntityKind::ContractAmendment,
                ),
                ObjectIdentifier::new_with_type(
                    102,
                    uuid!("00000000-0000-0000-0002-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let request_fail_status = PreReturnToCustomerReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    3,
                    uuid!("00000000-0000-0000-0000-000000000003"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    103,
                    uuid!("00000000-0000-0000-0003-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };
        let request_fail_protocol = PreReturnToCustomerReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    4,
                    uuid!("00000000-0000-0000-0000-000000000004"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    104,
                    uuid!("00000000-0000-0000-0004-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let res_ok =
            app_process::pre_return_to_customer(request_ok, pool.clone()).await;
        let res_fail_status =
            app_process::pre_return_to_customer(request_fail_status, pool.clone())
                .await;
        let res_fail_protocol =
            app_process::pre_return_to_customer(request_fail_protocol, pool).await;

        {
            let res = res_ok.unwrap();
            assert_eq!(res.data.item_list.len(), 4);
            assert!(res.messages.messages.is_empty());
        }

        {
            let res = res_fail_status.unwrap();

            assert_eq!(res.messages.kind, MessageKind::Error);
            let messages = vec![PlanReturnToCustomerMessage::InvalidPlanStatus
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 3,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 103,
                        ..Default::default()
                    }),
                ])];

            assert_eq!(res.messages.messages, messages);
        }

        {
            let res = res_fail_protocol.unwrap();

            assert_eq!(res.messages.kind, MessageKind::Error);
            let messages = vec![PlanReturnToCustomerMessage::AlreadyInProtocolErr(
                &EcProtocol {
                    id: 3,
                    protocol_date: AsezDate::try_from("1910-01-01").unwrap(),
                    status_id: EcProtocolStatus::Formed,
                    ..Default::default()
                },
            )
            .singular(&PlanOrAmendment::Amendment(ContractAmendment {
                id: 104,
                ..Default::default()
            }))];

            assert_eq!(res.messages.messages, messages);
        }
    })
    .await
}

#[tokio::test]
async fn test_return_to_customer_in_person() {
    run_db_test(RETURN_TO_CUSTOMER_EXTRA_MIGS, |pool| async move {
        let request_ok = ReturnToCustomerReq {
            item_list: vec![
                ReturnToSomeoneItem {
                    id: 1,
                    uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                    status_note: String::from("1"),
                    object_type: EntityKind::Plan,
                    is_excluded: Some(true),
                },
                ReturnToSomeoneItem {
                    id: 2,
                    uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                    status_note: String::from("2"),
                    object_type: EntityKind::Plan,
                    is_excluded: Some(false),
                },
                ReturnToSomeoneItem {
                    id: 101,
                    uuid: uuid!("00000000-0000-0000-0001-000000000000"),
                    status_note: String::from("101"),
                    object_type: EntityKind::ContractAmendment,
                    is_excluded: Some(true),
                },
                ReturnToSomeoneItem {
                    id: 102,
                    uuid: uuid!("00000000-0000-0000-0002-000000000000"),
                    status_note: String::from("102"),
                    object_type: EntityKind::ContractAmendment,
                    is_excluded: Some(true),
                },
            ],
            action_type: ActionType::Revision,
            section_id: Section::EstimatedCommissionInPerson,
            is_force: true,
            user_id: 123,
        };

        let pctx = super::mock_processing_context(pool.clone()).await;
        let res = app_process::return_to_customer(request_ok, pctx.clone())
            .await
            .unwrap();

        assert_eq!(res.data.item_list.len(), 4);

        let messages =
            vec![PlanReturnToCustomerMessage::Success(ActionType::Revision)
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 1,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Plan(Plan {
                        id: 2,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 101,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 102,
                        ..Default::default()
                    }),
                ])];
        assert_eq!(res.messages.messages, messages);

        let plans = PlanOrAmendment::select(&Select::full::<Plan>(), &pctx.db_pool)
            .await
            .unwrap();
        let agenda_items = EcAgendaItem::select_all(&*pctx.db_pool).await.unwrap();

        assert_plan(&plans, 1, PlanStatus::ReturnToClientRework, false, false);
        assert_plan(&plans, 2, PlanStatus::ReturnToClientRework, true, true);
        assert_plan(&plans, 101, PlanStatus::ReturnToClientRework, false, false);
        assert_plan(&plans, 102, PlanStatus::ReturnToClientRework, false, false);

        assert_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000001",
            false,
            true,
            false,
        );
        assert_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000002",
            false,
            false,
            false,
        );
        assert_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000003",
            true,
            false,
            true,
        );

        {
            let status_history = StatusHistory::select(
                &Select::full::<StatusHistory>()
                    .in_any(StatusHistory::comment, ["1", "2", "101", "102"]),
                &*pctx.db_pool,
            )
            .await
            .unwrap();

            assert_eq!(4, status_history.len());
            let uuids: AHashSet<_> =
                status_history.into_iter().map(|x| x.object_uuid).collect();
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000001")));
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000001")));
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0001-000000000000")));
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0001-000000000000")));
        }
    })
    .await
}

#[tokio::test]
async fn test_return_to_customer_correspondence() {
    run_db_test(RETURN_TO_CUSTOMER_EXTRA_MIGS, |pool| async move {
        let request_ok = ReturnToCustomerReq {
            item_list: vec![
                ReturnToSomeoneItem {
                    id: 5,
                    uuid: uuid!("00000000-0000-0000-0000-000000000005"),
                    status_note: String::from("5"),
                    object_type: EntityKind::Plan,
                    is_excluded: Some(true),
                },
                ReturnToSomeoneItem {
                    id: 6,
                    uuid: uuid!("00000000-0000-0000-0000-000000000006"),
                    status_note: String::from("6"),
                    object_type: EntityKind::Plan,
                    is_excluded: Some(true),
                },
                ReturnToSomeoneItem {
                    id: 105,
                    uuid: uuid!("00000000-0000-0000-0005-000000000000"),
                    status_note: String::from("105"),
                    object_type: EntityKind::ContractAmendment,
                    is_excluded: Some(true),
                },
                ReturnToSomeoneItem {
                    id: 106,
                    uuid: uuid!("00000000-0000-0000-0006-000000000000"),
                    status_note: String::from("106"),
                    object_type: EntityKind::ContractAmendment,
                    is_excluded: Some(true),
                },
            ],
            action_type: ActionType::Revision,
            section_id: Section::EstimatedCommissionCorrespondence,
            is_force: true,
            user_id: 123,
        };

        let pctx = super::mock_processing_context(pool.clone()).await;
        let res = app_process::return_to_customer(request_ok, pctx.clone())
            .await
            .unwrap();

        assert_eq!(res.data.item_list.len(), 4);

        let messages =
            vec![PlanReturnToCustomerMessage::Success(ActionType::Revision)
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 5,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Plan(Plan {
                        id: 6,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 105,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 106,
                        ..Default::default()
                    }),
                ])];
        assert_eq!(res.messages.messages, messages);

        let plans = PlanOrAmendment::select(&Select::full::<Plan>(), &pctx.db_pool)
            .await
            .unwrap();
        let protocol_items =
            EcProtocolItem::select_all(&*pctx.db_pool).await.unwrap();

        assert_plan(&plans, 5, PlanStatus::ReturnToClientRework, true, false);
        assert_plan(&plans, 6, PlanStatus::ReturnToClientRework, true, false);
        assert_plan(&plans, 105, PlanStatus::ReturnToClientRework, true, false);
        assert_plan(&plans, 106, PlanStatus::ReturnToClientRework, true, false);

        assert_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000005",
            true,
        );

        {
            let status_history = StatusHistory::select(
                &Select::full::<StatusHistory>()
                    .in_any(StatusHistory::comment, ["5", "6", "105", "106"]),
                &*pctx.db_pool,
            )
            .await
            .unwrap();

            assert_eq!(4, status_history.len());
            let uuids: AHashSet<_> =
                status_history.into_iter().map(|x| x.object_uuid).collect();
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000005")));
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000006")));
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0005-000000000000")));
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0006-000000000000")));
        }
    })
    .await
}
