//! Тестирование процесса `pre_create_protocol`/`create_protocol`
use super::*;
use crate::app_process::{create_protocol, pre_create_protocol};
use crate::presentation::business_messages::protocol::ProtocolCreateMessage;
use asez2_shared_db::db_item::AsezDate;
use shared_essential::domain::processing::status_history::StatusHistory;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};

const CREATE_PROTOCOL_EXTRA_MIGS: &[&str] =
    &["estimated_commission/create_protocol.sql"];

/// Тестирование кейса с успешным получением повесток СК с доп данными
/// по agenda_item
#[tokio::test]
async fn test_pre_create_protocol_in_person() {
    run_db_test(CREATE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req_multiple = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    5,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000005")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    7,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000007")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
            ],
        };
        let req_yellow = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![ObjectIdentifier::new_with_type(
                2,
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                EntityKind::Agenda,
            )],
        };
        let req_green = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![ObjectIdentifier::new_with_type(
                1,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                EntityKind::Agenda,
            )],
        };
        let req_empty = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![ObjectIdentifier::new_with_type(
                6,
                Uuid::parse_str("00000000-0000-0000-0000-000000000006").unwrap(),
                EntityKind::Agenda,
            )],
        };
        let req_fail = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
            ],
        };

        let res_multiple =
            pre_create_protocol(req_multiple, pool.clone()).await.unwrap();
        {
            assert!(res_multiple.data.plans.is_none());
            let agenda_list = res_multiple.data.agenda_list.as_ref().unwrap();

            assert_eq!(agenda_list.len(), 2);
            assert!(res_multiple.messages.is_empty());
            assert_eq!(
                agenda_list[0].agenda_item_quantity_threshold,
                Some(ColorThreshold {
                    value: 1,
                    color_scheme_id: ColorScheme::Green,
                })
            );
            assert_eq!(
                agenda_list[1].agenda_item_quantity_threshold,
                Some(ColorThreshold {
                    value: 2,
                    color_scheme_id: ColorScheme::Red,
                })
            );
        }

        let res_yellow =
            pre_create_protocol(req_yellow, pool.clone()).await.unwrap();
        {
            assert!(res_yellow.data.plans.is_none());
            let agenda_list = res_yellow.data.agenda_list.as_ref().unwrap();
            assert_eq!(agenda_list.len(), 1);
            assert!(res_yellow.messages.is_empty());
            assert_eq!(
                agenda_list[0].agenda_item_quantity_threshold,
                Some(ColorThreshold {
                    value: 3,
                    color_scheme_id: ColorScheme::Yellow,
                })
            );
        }

        let res_green = pre_create_protocol(req_green, pool.clone()).await.unwrap();
        {
            assert!(res_green.data.plans.is_none());
            let agenda_list = res_green.data.agenda_list.as_ref().unwrap();
            assert_eq!(agenda_list.len(), 1);
            assert!(res_green.messages.is_empty());
            assert_eq!(
                agenda_list[0].agenda_item_quantity_threshold,
                Some(ColorThreshold {
                    // Потому что один элемент is_excluded=true
                    value: 1,
                    color_scheme_id: ColorScheme::Green,
                })
            );
        }

        let res_fail = pre_create_protocol(req_fail, pool.clone()).await.unwrap();
        {
            assert!(res_fail.data.plans.is_none());
            assert!(res_fail.data.agenda_list.is_none());

            let expected_messages = vec![
                ProtocolCreateMessage::invalid_agenda_status(&EcAgenda {
                    id: 3,
                    status_id: EcAgendaStatus::ProtocolFormed,
                    ..Default::default()
                }),
                ProtocolCreateMessage::invalid_agenda_status(&EcAgenda {
                    id: 4,
                    status_id: EcAgendaStatus::Deleted,
                    ..Default::default()
                }),
            ];
            assert_eq!(res_fail.messages.messages, expected_messages);
        }

        let res_empty =
            pre_create_protocol(req_empty, pool.clone()).await.unwrap_err();
        {
            match res_empty {
                crate::common::ProcessingError::CreateProtocol(msg) => {
                    assert_eq!(
                        msg.as_str(),
                        "Повестки СК с идентификаторами 6 не найдены"
                    )
                }
                err => panic!("Была возвращена не та ошибка {:?}", err),
            }
        }
    })
    .await;
}

/// Тестирование кейса с успешным получением повесток СК с доп данными
/// по ППЗ/ДС
#[tokio::test]
async fn test_pre_create_protocol_correspondence() {
    run_db_test(CREATE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req_fail_plan_status = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    101,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };
        let req_fail_in_protocol = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    5,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000005")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    102,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };
        let req_success = PreCreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    5,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000005")
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
        };

        let res_fail_plan_status =
            pre_create_protocol(req_fail_plan_status, pool.clone()).await.unwrap();
        {
            assert!(res_fail_plan_status.data.agenda_list.is_none());
            assert!(res_fail_plan_status.data.plans.is_none());

            let messages = res_fail_plan_status.messages;
            assert_eq!(messages.kind, MessageKind::Error);

            let expected_messages = vec![ProtocolCreateMessage::InvalidPlanStatus
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 1,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 101,
                        ..Default::default()
                    }),
                ])];
            assert_eq!(messages.messages, expected_messages);
        }

        let res_fail_in_protocol =
            pre_create_protocol(req_fail_in_protocol, pool.clone()).await.unwrap();
        {
            assert!(res_fail_plan_status.data.agenda_list.is_none());
            assert!(res_fail_plan_status.data.plans.is_none());
            let messages = res_fail_in_protocol.messages;
            assert_eq!(messages.kind, MessageKind::Error);
            // Сообщение одно, так как один из protocol_item имеет is_removed=true и is_excluded=true
            let expected_messages =
                vec![ProtocolCreateMessage::AlreadyInProtocol(&EcProtocol {
                    id: 1,
                    protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 102,
                        ..Default::default()
                    },
                ))];
            assert_eq!(messages.messages, expected_messages);
        }

        let res_success =
            pre_create_protocol(req_success, pool.clone()).await.unwrap();
        {
            assert!(res_success.data.agenda_list.is_none());
            let plans = res_success.data.plans.as_ref().unwrap();

            let messages = res_success.messages;
            assert_eq!(messages.messages.len(), 0);

            assert_eq!(plans.len(), 2);
        }
    })
    .await;
}

#[tokio::test]
async fn test_create_protocol_in_person() {
    run_db_test(CREATE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req_fail = CreateProtocolReq {
            user_id: 99,
            protocol_type_id: ProtocolType::InPersonMeeting,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            item_list: vec![
                // Take all items from first agenda.
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        1,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                            .unwrap(),
                        EntityKind::Agenda,
                    ),
                    all_items: Some(true),
                    item_list: Some(Vec::new()),
                },
                // Use an agenda with a bad status. This will fail
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        3,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                            .unwrap(),
                        EntityKind::Agenda,
                    ),
                    all_items: Some(true),
                    item_list: Some(Vec::new()),
                },
                // Повестка не имеет элементов для создания Протокола
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        9,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000009")
                            .unwrap(),
                        EntityKind::Agenda,
                    ),
                    all_items: Some(true),
                    item_list: Some(Vec::new()),
                },
            ],
        };
        // Mixed protocol
        let req1 = CreateProtocolReq {
            user_id: 99,
            protocol_type_id: ProtocolType::InPersonMeeting,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            item_list: vec![
                // Take all items from first agenda.
                // However, our DB has already included #4 in a protocol, so it should not
                // show not be included anywhere.
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        1,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                            .unwrap(),
                        EntityKind::Agenda,
                    ),
                    all_items: Some(true),
                    item_list: Some(Vec::new()),
                },
                // Take a couple of items from the second agenda.
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        2,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                            .unwrap(),
                        EntityKind::Agenda,
                    ),
                    all_items: Some(false),
                    item_list: Some(vec![
                        ObjectIdentifier::new_with_type(
                            6,
                            Uuid::parse_str("00000000-0000-0000-0000-000000000006")
                                .unwrap(),
                            EntityKind::Plan,
                        ),
                        ObjectIdentifier::new_with_type(
                            7,
                            Uuid::parse_str("00000000-0000-0000-0000-000000000007")
                                .unwrap(),
                            EntityKind::Plan,
                        ),
                    ]),
                },
                // Take all items from third agenda
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        7,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000007")
                            .unwrap(),
                        EntityKind::Agenda,
                    ),
                    all_items: Some(true),
                    item_list: Some(Vec::new()),
                },
            ],
        };
        let pctx = super::mock_processing_context(pool).await;

        let res_fail = create_protocol(req_fail, pctx.clone()).await;
        {
            let res_fail = res_fail.unwrap();

            assert_eq!(res_fail.messages.messages.len(), 2);
            assert_eq!(res_fail.messages.kind, MessageKind::Error);

            let messages = vec![
                ProtocolCreateMessage::invalid_agenda_status(&EcAgenda {
                    id: 3,
                    status_id: EcAgendaStatus::ProtocolFormed,
                    ..Default::default()
                }),
                ProtocolCreateMessage::empty_agenda(&EcAgenda {
                    id: 9,
                    meeting_date: AsezDate::try_from("1900-01-02").unwrap(),
                    ..Default::default()
                }),
            ];
            assert_eq!(res_fail.messages.messages, messages);
        }
        let pool = &*pctx.db_pool;
        // For the success case, we first check that nothing was there before, and then we
        // check that the correct items have been created in the DB.
        let rel_items_before =
            RelAgendaProtocolItem::select_all(pool).await.unwrap();
        let protocol_items_before = EcProtocolItem::select_all(pool).await.unwrap();
        let rels_before = RelAgendaProtocol::select_all(pool).await.unwrap();
        let partners_before = EcPartner::select_all(pool).await.unwrap();
        let protocols_before = EcProtocol::select_all(pool).await.unwrap();
        let status_histories_before =
            StatusHistory::select_all(pool).await.unwrap();

        assert_eq!(rels_before.len(), 0);
        assert_eq!(partners_before.len(), 4);
        assert_eq!(protocols_before.len(), 1);
        assert_eq!(status_histories_before.len(), 0);
        assert_eq!(rel_items_before.len(), 1);
        assert_eq!(protocol_items_before.len(), 2);

        let res_ok = create_protocol(req1, pctx.clone()).await;
        {
            let res_ok = res_ok.unwrap();

            assert_eq!(res_ok.messages.kind, MessageKind::Success);
            assert_eq!(
                res_ok.messages.messages,
                vec![ProtocolCreateMessage::success(&EcProtocol {
                    id: 8900000000,
                    protocol_type_id: ProtocolType::InPersonMeeting,
                    protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
                    ..Default::default()
                })]
            );

            let partner_select = Select::full::<EcPartner>()
                .add_replace_order_asc(EcPartner::created_at)
                .add_replace_order_asc(EcPartner::role_id);

            let rel_items = RelAgendaProtocolItem::select_all(pool).await.unwrap();
            let rels = RelAgendaProtocol::select_all(pool).await.unwrap();
            let partners = EcPartner::select(&partner_select, pool).await.unwrap();
            let protocols = EcProtocol::select_all(pool).await.unwrap();
            let protocol_items = EcProtocolItem::select_all(pool).await.unwrap();
            let status_histories = StatusHistory::select_all(pool).await.unwrap();

            // 3 из первой Повестки (#4 is excluded),
            // 3 из Повестки 2 (00000000-0000-0000-0000-000000000005 элемент тоже добавляется),
            // 3 из Повестки 3.
            assert_eq!(rel_items.len(), 10, "{:#?}", rel_items);
            // Technically our initial database is "broken", but that's OK.
            // The counts should be the same as for rel_items.
            assert_eq!(protocol_items.len(), 11, "{:#?}", protocol_items);
            // We should have one relationship for each agenda (3 in total).
            assert_eq!(rels.len(), 3, "{:#?}", rels);
            // We should have one status change for each agenda where all valid items have
            // been added to a protocol. (#1 & #7 -> #2 only half the items are added.)
            assert_eq!(status_histories.len(), 3, "{:#?}", status_histories);
            // Должен появиться еще один партнер
            assert_eq!(partners.len(), 6, "{:#?}", partners);
            assert_eq!(partners[4].role_id, 8);
            assert_eq!(partners[5].role_id, 9);
            assert_eq!(protocols.len(), 2, "{:#?}", protocols);

            // Проверяется меньшее количество элементов, так как у некоторых agenda_item один и тот же
            // source_uuid. Главное просто проверить что данные заполняются
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000001",
                0.01,
                0.02,
                None,
            );
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000002",
                0.01,
                0.02,
                None,
            );
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000003",
                0.01,
                0.02,
                None,
            );
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000005",
                0.01,
                0.02,
                None,
            );
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000006",
                0.01,
                0.02,
                None,
            );
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000007",
                0.01,
                0.02,
                None,
            );
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000008",
                0.01,
                0.02,
                None,
            );
        }
    })
    .await;
}

#[tokio::test]
async fn test_create_protocol_in_person_with_all_d647_items() {
    run_db_test(CREATE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req = CreateProtocolReq {
            user_id: 99,
            protocol_type_id: ProtocolType::InPersonMeeting,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            item_list: vec![CreateProtocolItem {
                id: ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000008")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                all_items: Some(false),
                item_list: Some(vec![ObjectIdentifier::new_with_type(
                    109,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000019")
                        .unwrap(),
                    EntityKind::Plan,
                )]),
            }],
        };
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let res = create_protocol(req, pctx.clone()).await.unwrap();

        assert_eq!(res.messages.kind, MessageKind::Success);
        assert_eq!(
            res.messages.messages,
            vec![ProtocolCreateMessage::success(&EcProtocol {
                id: 8900000000,
                protocol_type_id: ProtocolType::InPersonMeeting,
                protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
                ..Default::default()
            })]
        );

        let rel_items = RelAgendaProtocolItem::select_all(pool).await.unwrap();
        let rels = RelAgendaProtocol::select_all(pool).await.unwrap();
        let partners = EcPartner::select_all(pool).await.unwrap();
        let protocols = EcProtocol::select_all(pool).await.unwrap();
        let protocol_items = EcProtocolItem::select_all(pool).await.unwrap();
        let status_histories = StatusHistory::select_all(pool).await.unwrap();

        // Две новые связи
        assert_eq!(rel_items.len(), 3, "{:#?}", rel_items);
        // +2 новых элемента
        assert_eq!(protocol_items.len(), 4, "{:#?}", protocol_items);
        // Новая связь с Повесткой
        assert_eq!(rels.len(), 1, "{:#?}", rels);
        // Перевод статуса Повестки и Протокола
        assert_eq!(status_histories.len(), 2, "{:#?}", status_histories);
        // Партнер не должен появиться
        assert_eq!(partners.len(), 4, "{:#?}", partners);
        assert_eq!(protocols.len(), 2, "{:#?}", protocols);

        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0007-000000000000",
            0.01,
            0.02,
            None,
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0008-000000000000",
            0.01,
            0.02,
            None,
        );
    })
    .await;
}

/// Тестирование кейса с успешным получением повесток СК с доп данными
/// по ППЗ/ДС
#[tokio::test]
async fn test_create_protocol_correspondence() {
    run_db_test(CREATE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req_fail_plan_status = CreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            item_list: vec![
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        1,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                            .unwrap(),
                        EntityKind::Plan,
                    ),
                    all_items: None,
                    item_list: None,
                },
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        101,
                        Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                            .unwrap(),
                        EntityKind::ContractAmendment,
                    ),
                    all_items: None,
                    item_list: None,
                },
            ],
        };
        let req_fail_in_protocol = CreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            item_list: vec![
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        5,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000005")
                            .unwrap(),
                        EntityKind::Plan,
                    ),
                    all_items: None,
                    item_list: None,
                },
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        102,
                        Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                            .unwrap(),
                        EntityKind::ContractAmendment,
                    ),
                    all_items: None,
                    item_list: None,
                },
            ],
        };
        let req_success = CreateProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            item_list: vec![
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        5,
                        Uuid::parse_str("00000000-0000-0000-0000-000000000005")
                            .unwrap(),
                        EntityKind::Plan,
                    ),
                    all_items: None,
                    item_list: None,
                },
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        103,
                        Uuid::parse_str("00000000-0000-0000-0003-000000000000")
                            .unwrap(),
                        EntityKind::ContractAmendment,
                    ),
                    all_items: None,
                    item_list: None,
                },
            ],
        };

        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let res_fail_plan_status =
            create_protocol(req_fail_plan_status, pctx.clone()).await.unwrap();
        {
            let messages = res_fail_plan_status.messages;
            assert_eq!(messages.kind, MessageKind::Error);
            let expected_messages = vec![ProtocolCreateMessage::InvalidPlanStatus
                .plural(&[
                    PlanOrAmendment::Plan(Plan {
                        id: 1,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 101,
                        ..Default::default()
                    }),
                ])];
            assert_eq!(messages.messages, expected_messages);
        }

        let res_fail_in_protocol =
            create_protocol(req_fail_in_protocol, pctx.clone()).await.unwrap();
        {
            let messages = res_fail_in_protocol.messages;
            assert_eq!(messages.kind, MessageKind::Error);
            let expected_messages =
                vec![ProtocolCreateMessage::AlreadyInProtocol(&EcProtocol {
                    id: 1,
                    protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 102,
                        ..Default::default()
                    },
                ))];
            // Сообщение одно, так как один из protocol_item имеет is_removed=true и is_excluded=true
            assert_eq!(messages.messages, expected_messages);
        }

        let res_success = create_protocol(req_success, pctx.clone()).await.unwrap();
        {
            assert_eq!(res_success.messages.kind, MessageKind::Success);
            assert_eq!(
                res_success.messages.messages,
                vec![ProtocolCreateMessage::success(&EcProtocol {
                    id: 8900000000,
                    protocol_type_id: ProtocolType::CorrespondenceMeeting,
                    protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
                    ..Default::default()
                })]
            );

            let rel_items = RelAgendaProtocolItem::select_all(pool).await.unwrap();
            let rels = RelAgendaProtocol::select_all(pool).await.unwrap();
            let partners = EcPartner::select_all(pool).await.unwrap();
            let protocols = EcProtocol::select_all(pool).await.unwrap();
            let protocol_items = EcProtocolItem::select_all(pool).await.unwrap();
            let status_histories = StatusHistory::select_all(pool).await.unwrap();

            // Должно появиться два новых элемента Протокола СК
            assert_eq!(protocol_items.len(), 4, "{:#?}", protocol_items);
            // Никаких отношений не появляется при заочной СК
            assert_eq!(rels.len(), 0, "{:#?}", rels);
            assert_eq!(rel_items.len(), 1, "{:#?}", rel_items);
            // При заочной СК только добавляется история по Протоколу
            assert_eq!(status_histories.len(), 1, "{:#?}", status_histories);
            // Должен появиться новый партнер
            assert_eq!(partners.len(), 5, "{:#?}", partners);
            // Должен появиться новый протокол
            assert_eq!(protocols.len(), 2, "{:#?}", protocols);

            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0000-000000000005",
                0.04,
                0.05,
                Some(0.05),
            );
            verify_protocol_item(
                &protocol_items,
                "00000000-0000-0000-0003-000000000000",
                0.03,
                0.04,
                Some(0.04),
            );
        }
    })
    .await;
}

fn verify_protocol_item<T: Into<CurrencyValue>>(
    protocol_items: &[EcProtocolItem],
    source_uuid: &str,
    sum_excluded_vat: T,
    pricing_sum_excluded_vat: T,
    commission_sum_excluded_vat: Option<T>,
) {
    let protocol_item = protocol_items
        .iter()
        .find(|p| p.source_uuid.to_string() == source_uuid && !p.is_removed)
        .unwrap_or_else(|| {
            panic!("Не найден protocol_item с source_uuid {}", source_uuid)
        });

    assert!(
        protocol_item.sum_excluded_vat == Some(sum_excluded_vat.into())
            && protocol_item.pricing_sum_excluded_vat
                == Some(pricing_sum_excluded_vat.into())
            && protocol_item.commission_sum_excluded_vat
                == commission_sum_excluded_vat.map(Into::into),
        "{:#?}",
        protocol_item
    );
}
