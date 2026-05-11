//! Тестирование процесса `pre_request/add_plans_protocol`
use super::*;
use crate::app_process::estimated_commission::add_plans_protocol::action::fetch_protocol_with_items;
use crate::app_process::{add_plans_protocol, pre_add_plans_protocol};
use crate::presentation::business_messages::protocol::ProtocolAddPlansMessage;
use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::uuid;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};

/// Для `protocol_type_id: ProtocolType::InPersonMeeting` полностью совпадает с `pre/create_protocol`
const PRE_ADD_PLANS_PROTOCOL_EXTRA_MIGS: &[&str] =
    &["estimated_commission/add_plans_protocol.sql"];

/// Тестирование кейса с успешным получением повесток СК с доп данными
/// по agenda_item
#[tokio::test]
async fn test_pre_add_plans_protocol_in_person() {
    run_db_test(PRE_ADD_PLANS_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req_multiple = PreAddPlansProtocolReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            user_id: 666,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    5,
                    uuid!("00000000-0000-0000-0000-000000000005"),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    7,
                    uuid!("00000000-0000-0000-0000-000000000007"),
                    EntityKind::Agenda,
                ),
            ],
        };
        let req_yellow = PreAddPlansProtocolReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            user_id: 666,
            item_list: vec![ObjectIdentifier::new_with_type(
                2,
                uuid!("00000000-0000-0000-0000-000000000002"),
                EntityKind::Agenda,
            )],
        };
        let req_green = PreAddPlansProtocolReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            user_id: 666,
            item_list: vec![ObjectIdentifier::new_with_type(
                1,
                uuid!("00000000-0000-0000-0000-000000000001"),
                EntityKind::Agenda,
            )],
        };
        let req_empty = PreAddPlansProtocolReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            user_id: 666,
            item_list: vec![ObjectIdentifier::new_with_type(
                6,
                uuid!("00000000-0000-0000-0000-000000000006"),
                EntityKind::Agenda,
            )],
        };
        let req_fail = PreAddPlansProtocolReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            user_id: 666,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    3,
                    uuid!("00000000-0000-0000-0000-000000000003"),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    4,
                    uuid!("00000000-0000-0000-0000-000000000004"),
                    EntityKind::Agenda,
                ),
            ],
        };
        let pctx = mock_processing_context(pool).await;

        let res_multiple =
            pre_add_plans_protocol(req_multiple, pctx.clone()).await.unwrap();
        {
            assert!(res_multiple.data.plans.is_none());

            let agenda_list = res_multiple.data.agenda_list.unwrap();
            assert_eq!(agenda_list.len(), 2);
            assert!(res_multiple.messages.is_empty());
            assert_eq!(
                agenda_list[0].agenda_item_quantity_threshold,
                Some(ColorThreshold {
                    value: 2,
                    color_scheme_id: ColorScheme::Red,
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
            pre_add_plans_protocol(req_yellow, pctx.clone()).await.unwrap();
        {
            assert!(res_multiple.data.plans.is_none());

            let agenda_list = res_yellow.data.agenda_list.unwrap();
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

        let res_green =
            pre_add_plans_protocol(req_green, pctx.clone()).await.unwrap();
        {
            assert!(res_multiple.data.plans.is_none());

            let agenda_list = res_green.data.agenda_list.unwrap();
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

        let res_fail =
            pre_add_plans_protocol(req_fail, pctx.clone()).await.unwrap();
        {
            assert!(res_fail.data.plans.is_none());
            assert!(res_fail.data.agenda_list.is_none());

            assert_eq!(res_fail.messages.messages.len(), 2);
            assert_eq!(res_fail.messages.kind, MessageKind::Error);

            let messages = vec![
                ProtocolAddPlansMessage::invalid_agenda_status(&EcAgenda {
                    status_id: EcAgendaStatus::ProtocolFormed,
                    id: 3,
                    ..Default::default()
                }),
                ProtocolAddPlansMessage::invalid_agenda_status(&EcAgenda {
                    status_id: EcAgendaStatus::Deleted,
                    id: 4,
                    ..Default::default()
                }),
            ];
            assert_eq!(res_fail.messages.messages, messages);
        }

        let res_empty =
            pre_add_plans_protocol(req_empty, pctx.clone()).await.unwrap_err();
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

/// Тестирование кейса с успешным получением ППЗ/ДС для дальнейшего добавления
/// в Протокол СК
#[tokio::test]
async fn test_pre_add_plans_protocol_correspondence() {
    run_db_test(PRE_ADD_PLANS_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req_fail_plan_status = PreAddPlansProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    uuid!("00000000-0000-0000-0000-000000000001"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    101,
                    uuid!("00000000-0000-0000-0001-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
        };
        let req_fail_in_protocol = PreAddPlansProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    108,
                    uuid!("00000000-0000-0000-0008-000000000000"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    109,
                    uuid!("00000000-0000-0000-0009-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
        };
        let req_success = PreAddPlansProtocolReq {
            user_id: 7,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    5,
                    uuid!("00000000-0000-0000-0000-000000000005"),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    103,
                    uuid!("00000000-0000-0000-0003-000000000000"),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let pctx = super::mock_processing_context(pool).await;

        let res_fail_plan_status =
            pre_add_plans_protocol(req_fail_plan_status, pctx.clone())
                .await
                .unwrap();
        {
            assert!(res_fail_plan_status.data.agenda_list.is_none());
            assert!(res_fail_plan_status.data.plans.is_none());

            let messages = res_fail_plan_status.messages;
            assert_eq!(messages.kind, MessageKind::Error);
            assert_eq!(messages.messages.len(), 1);

            let expected_messages =
                vec![ProtocolAddPlansMessage::InvalidPlanStatus.plural(&vec![
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
            pre_add_plans_protocol(req_fail_in_protocol, pctx.clone())
                .await
                .unwrap();
        {
            assert!(res_fail_plan_status.data.agenda_list.is_none());
            assert!(res_fail_plan_status.data.plans.is_none());

            let messages = res_fail_in_protocol.messages;
            assert_eq!(messages.kind, MessageKind::Error);
            // Сообщение одно, так как один из protocol_item имеет is_removed=true
            assert_eq!(messages.messages.len(), 1);

            let expected_messages =
                vec![ProtocolAddPlansMessage::AlreadyInProtocol(&EcProtocol {
                    id: 2,
                    protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 108,
                        ..Default::default()
                    },
                ))];
            assert_eq!(messages.messages, expected_messages);
        }

        let res_success =
            pre_add_plans_protocol(req_success, pctx.clone()).await.unwrap();
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

/// Тестирование кейса с успешным добавлением ППЗ/ДС в Протокол СК
#[tokio::test]
async fn test_add_plans_protocol_correspondence() {
    run_db_test(PRE_ADD_PLANS_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req = AddPlansProtocolReq {
            item_list: vec![
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        107,
                        uuid!("00000000-0000-0000-0007-000000000000"),
                        EntityKind::ContractAmendment,
                    ),
                    all_items: None,
                    item_list: None,
                },
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        9,
                        uuid!("00000000-0000-0000-0000-000000000009"),
                        EntityKind::Plan,
                    ),
                    all_items: None,
                    item_list: None,
                },
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        11,
                        uuid!("00000000-0000-0000-0000-000000000011"),
                        EntityKind::Plan,
                    ),
                    all_items: None,
                    item_list: None,
                },
            ],
            protocol_date: AsezDate::today(),
            protocol_id: 2,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            user_id: 666,
            uuid: uuid!("00000000-0000-0000-0000-000000000002"),
        };

        let pctx = super::mock_processing_context(pool.clone()).await;

        let res = add_plans_protocol(req, pctx).await.unwrap();

        assert_eq!(res.messages.kind, MessageKind::Success);
        let messages = vec![ProtocolAddPlansMessage::Success(&EcProtocol {
            id: 2,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            ..Default::default()
        })
        .plural(&vec![
            PlanOrAmendment::Amendment(ContractAmendment {
                id: 107,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                id: 9,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                id: 11,
                ..Default::default()
            }),
        ])];
        assert_eq!(res.messages.messages, messages);

        let protocol_with_items = fetch_protocol_with_items(
            2,
            uuid!("00000000-0000-0000-0000-000000000002"),
            &pool,
        )
        .await
        .unwrap();

        let protocol_items = protocol_with_items.protocol_items;
        assert_eq!(protocol_items.len(), 7);

        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0008-000000000000",
            1,
            false,
            0,
            0,
            Some(0),
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0007-000000000000",
            2,
            false,
            2,
            3,
            Some(3),
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000009",
            3,
            false,
            3,
            4,
            Some(4),
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0009-000000000000",
            4,
            true,
            0,
            0,
            Some(0),
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000010",
            5,
            false,
            0,
            0,
            Some(0),
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000011",
            6,
            false,
            3,
            4,
            Some(4),
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000012",
            7,
            true,
            0,
            0,
            Some(0),
        );
    })
    .await;
}

/// Тестирование кейса с успешным добавлением ППЗ/ДС в Протокол СК
#[tokio::test]
async fn test_add_plans_protocol_in_person() {
    run_db_test(PRE_ADD_PLANS_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let agenda_uuid1 = uuid!("00000000-0000-0000-0000-000000000005");
        let agenda_uuid2 = uuid!("00000000-0000-0000-0000-000000000007");

        let req = AddPlansProtocolReq {
            item_list: vec![
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        5,
                        agenda_uuid1,
                        EntityKind::Agenda,
                    ),
                    all_items: Some(true),
                    item_list: Some(Vec::new()),
                },
                CreateProtocolItem {
                    id: ObjectIdentifier::new_with_type(
                        7,
                        agenda_uuid2,
                        EntityKind::Agenda,
                    ),
                    all_items: Some(false),
                    item_list: Some(vec![
                        ObjectIdentifier {
                            uuid: uuid!("00000000-0000-0000-0000-000000000014"),
                            ..Default::default()
                        },
                        ObjectIdentifier {
                            uuid: uuid!("00000000-0000-0000-0000-000000000015"),
                            ..Default::default()
                        },
                    ]),
                },
            ],
            protocol_date: AsezDate::today(),
            protocol_id: 1,
            protocol_type_id: ProtocolType::InPersonMeeting,
            user_id: 666,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
        };

        let pctx = super::mock_processing_context(pool.clone()).await;

        let res = add_plans_protocol(req, pctx).await.unwrap();

        let protocol_with_items = fetch_protocol_with_items(
            1,
            uuid!("00000000-0000-0000-0000-000000000001"),
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(
            res.messages.messages,
            vec![ProtocolAddPlansMessage::Success(&EcProtocol {
                id: 1,
                protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
                protocol_type_id: ProtocolType::InPersonMeeting,
                ..Default::default()
            })
            .plural(&vec![
                PlanOrAmendment::Plan(Plan {
                    id: 10,
                    ..Default::default()
                }),
                PlanOrAmendment::Plan(Plan {
                    id: 5,
                    ..Default::default()
                }),
                PlanOrAmendment::Plan(Plan {
                    id: 6,
                    ..Default::default()
                })
            ])]
        );

        let protocol_items = protocol_with_items.protocol_items;
        assert_eq!(protocol_items.len(), 7);

        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0002-000000000000",
            1,
            false,
            0,
            0,
            None,
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000004",
            2,
            true,
            0,
            0,
            None,
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000007",
            3,
            false,
            0,
            0,
            None,
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000010",
            4,
            false,
            1,
            2,
            None,
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000005",
            5,
            false,
            1,
            2,
            None,
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000006",
            6,
            false,
            1,
            2,
            None,
        );
        verify_protocol_item(
            &protocol_items,
            "00000000-0000-0000-0000-000000000008",
            7,
            true,
            0,
            0,
            None,
        );

        let partners = EcPartner::select_all(&*pool).await.unwrap();
        let protocol_partner = partners
            .iter()
            .find(|i| {
                i.protocol_agenda_uuid
                    == uuid!("00000000-0000-0000-0000-000000000001")
            })
            .unwrap();
        assert!(!protocol_partner.is_removed);

        let item_rels = RelAgendaProtocolItem::select_all(&*pool).await.unwrap();
        // По 5 Повестке должно быть в общем 2 записи, так как одна относится к Протоколу 2 и не включается в Протокол 1
        // Вторая запись формируется из раннее удаленной позиции в 1 Протоколе
        assert_eq!(
            item_rels.iter().filter(|i| i.agenda_uuid == agenda_uuid1).count(),
            2
        );
        // По 7 Повестке должно быть 3 записи, так как 1 была раньше, 2 добавляются
        assert_eq!(
            item_rels.iter().filter(|i| i.agenda_uuid == agenda_uuid2).count(),
            3
        );

        let agenda_protocol_rels =
            RelAgendaProtocol::select_all(&*pool).await.unwrap();
        // Две записи уже существуют, где одна из них отношение Повестка 7 - Протокол 1
        // и Повестка 5 - Протокол 2
        // Это значит что нужно сформировать связь Повестка 5 - Протокол 1
        assert_eq!(agenda_protocol_rels.len(), 3, "{:?}", agenda_protocol_rels);

        let agendas = EcAgenda::select_all(&*pool).await.unwrap();
        let updated_agenda =
            agendas.iter().find(|i| i.uuid == agenda_uuid1).unwrap();
        let not_updated_agenda =
            agendas.iter().find(|i| i.uuid == agenda_uuid2).unwrap();
        assert_eq!(updated_agenda.status_id, EcAgendaStatus::ProtocolFormed);
        assert_eq!(not_updated_agenda.status_id, EcAgendaStatus::Sent);
    })
    .await;
}

fn verify_protocol_item<T: Into<CurrencyValue>>(
    protocol_items: &[EcProtocolItem],
    source_uuid: &str,
    number: i64,
    is_removed: bool,
    sum_excluded_vat: T,
    pricing_sum_excluded_vat: T,
    commission_sum_excluded_vat: Option<T>,
) {
    let item = protocol_items
        .iter()
        .find(|item| item.source_uuid == uuid!(source_uuid))
        .unwrap();

    assert_eq!(item.number, number, "{}", item.source_uuid);
    assert_eq!(item.is_removed, is_removed);
    assert_eq!(item.sum_excluded_vat.unwrap(), sum_excluded_vat.into());
    assert_eq!(
        item.pricing_sum_excluded_vat.unwrap(),
        pricing_sum_excluded_vat.into()
    );
    assert_eq!(
        item.commission_sum_excluded_vat,
        commission_sum_excluded_vat.map(Into::into)
    );
}
