use super::*;
use crate::app_process;
use crate::presentation::business_messages::agenda::AgendaCreateMessage;

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::AsezDate;
use shared_essential::domain::JoinedEcAgendaEcAgendaItemSelector as JoinedAgendaSelector;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind,
};

use crate::common::ProcessingError;

const CREATE_AGENDA_EXTRA_MIGS: &[&str] =
    &["estimated_commission/create_agenda.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn test_create_agenda_a() {
    run_db_test(CREATE_AGENDA_EXTRA_MIGS, |pool| async move {
        let input = CreateAgendaReq {
            user_id: USER1,
            is_force: true,
            meeting_date: AsezDate::try_from("2026-01-01").unwrap(),
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
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };
        let pctx = super::mock_processing_context(pool.clone()).await;

        let r = app_process::create_agenda(input, pctx).await.unwrap();

        assert_eq!(r.data.item_list.len(), 3);

        let expected_messages = vec![AgendaCreateMessage::Success(&EcAgenda {
            id: 8800000000,
            meeting_date: AsezDate::try_from("01.01.2026").unwrap(),
            ..Default::default()
        })
        .plural(&[
            PlanOrAmendment::Plan(Plan {
                id: 1,
                ..Default::default()
            }),
            PlanOrAmendment::Plan(Plan {
                id: 7,
                ..Default::default()
            }),
            PlanOrAmendment::Amendment(ContractAmendment {
                id: 11,
                ..Default::default()
            }),
        ])];
        assert_eq!(r.messages.messages, expected_messages);

        let agenda_select = Select::full::<EcAgenda>().eq("id", 8800000000i64);
        let mut agenda = JoinedAgendaSelector::new(agenda_select)
            .set_agenda_items(
                EcAgendaItem::join_default()
                    .selecting(Select::full::<EcAgendaItem>()),
            )
            .get(&*pool)
            .await
            .unwrap();
        assert_eq!(agenda.len(), 1);

        let JoinedEcAgendaEcAgendaItem {
            agenda,
            agenda_items,
        } = agenda.remove(0);

        assert_eq!(agenda_items.len(), 3);

        // Test whether agenda is created more or less correctly.
        assert_eq!(agenda.pricing_organization_unit_id, PricingUnitId::D646);
        assert_eq!(agenda.created_by, USER1);
        assert_eq!(agenda.meeting_date, AsezDate::try_from("2026-01-01").unwrap());
        assert_eq!(agenda.status_id, EcAgendaStatus::Formed);
        assert!(agenda_items.iter().all(|x| x.agenda_uuid == agenda.uuid));

        verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000001",
            1,
            1,
            2,
        );
        verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000007",
            2,
            7,
            8,
        );
        verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0001-000000000000",
            3,
            1,
            2,
        );

        let plan_select = Select::with_fields(["uuid"])
            .eq("commission_date", agenda.meeting_date);
        let plans = PlanOrAmendment::select(&plan_select, &pool).await.unwrap();
        assert_eq!(plans.len(), 3);

        let ec_partners = EcPartner::select_all(&*pool).await.unwrap();
        assert_eq!(ec_partners.len(), 2);
        assert_eq!(ec_partners[0].role_id, 9);
        assert_eq!(ec_partners[1].role_id, 8);
        assert!(verify_ec_partner(&ec_partners, 111, agenda.uuid));
        assert!(verify_ec_partner(&ec_partners, 222, agenda.uuid));

        let mut status_histories = StatusHistory::select_all(&*pool).await.unwrap();
        assert_eq!(status_histories.len(), 1);
        let status_history = status_histories.remove(0);
        assert!(
            status_history.object_uuid == agenda.uuid
                && status_history.status_id == EcAgendaStatus::Formed as i16
                && status_history.created_at == agenda.created_at
                && status_history.created_by == agenda.created_by
        );
    })
    .await
}

#[tokio::test]
async fn test_create_agenda_fail_a() {
    run_db_test(CREATE_AGENDA_EXTRA_MIGS, |pool| async move {
        let input = CreateAgendaReq {
            user_id: USER1,
            is_force: true,
            meeting_date: AsezDate::try_from("2026-01-01").unwrap(),
            item_list: vec![ObjectIdentifier::new_with_type(
                20,
                Uuid::parse_str("00000000-0000-0000-0000-000000000020").unwrap(),
                EntityKind::Plan,
            )],
        };
        let pctx = super::mock_processing_context(pool).await;

        let r = app_process::create_agenda(input, pctx).await.unwrap_err();

        if let ProcessingError::GetItemList(msg) = r {
            assert_eq!(&msg, "Записи ППЗ/ДС c идентификаторами 20 не найдены");
        } else {
            panic!("Была возвращена не та ошибка: {r:?}")
        }
    })
    .await
}

#[tokio::test]
async fn test_create_agenda_fail_b() {
    run_db_test(&[], |pool| async move {
        let input = CreateAgendaReq {
            user_id: USER1,
            is_force: true,
            meeting_date: AsezDate::try_from("2026-01-01").unwrap(),
            item_list: vec![],
        };
        let pctx = super::mock_processing_context(pool).await;

        let r = app_process::create_agenda(input, pctx).await.unwrap_err();

        assert!(matches!(r, ProcessingError::GetItemList(_)));
    })
    .await
}

#[tokio::test]
async fn test_pre_create_agenda_a() {
    run_db_test(CREATE_AGENDA_EXTRA_MIGS, |pool| async move {
        let input = PreCreateAgendaReq {
            user_id: USER1,
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
            ],
        };

        let input2 = PreCreateAgendaReq {
            user_id: USER1,
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
            ],
        };

        {
            let r =
                app_process::pre_create_agenda(input, pool.clone()).await.unwrap();

            assert_eq!(r.data.item_list.len(), 3);

            let expected_messages =
                vec![AgendaCreateMessage::different_department()];
            assert_eq!(r.messages.messages, expected_messages);
        }
        {
            let r =
                app_process::pre_create_agenda(input2, pool.clone()).await.unwrap();

            assert_eq!(r.data.item_list.len(), 2);
            assert!(r.messages.messages.is_empty());
        }
    })
    .await
}

/// Можно создать повестку для ППЗ/ДС, которое упоминается в
/// удаленных/исключенных записях повесток, или в удаленной повестке.
#[tokio::test]
async fn test_pre_create_agenda_removed_exluded() {
    run_db_test(CREATE_AGENDA_EXTRA_MIGS, |pool| async move {
        let input = PreCreateAgendaReq {
            user_id: USER1,
            item_list: vec![ObjectIdentifier::new_with_type(
                11,
                Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap(),
                EntityKind::Plan,
            )],
        };

        {
            let r =
                app_process::pre_create_agenda(input, pool.clone()).await.unwrap();

            assert_eq!(r.data.item_list.len(), 1);
            assert_eq!(r.messages.messages.len(), 0);
        }
    })
    .await
}

/// We put three tests in one here because these tests have a high startup time.
#[tokio::test]
async fn test_pre_create_agenda_failures() {
    run_db_test(CREATE_AGENDA_EXTRA_MIGS, |pool| async move {
        let fail_status = PreCreateAgendaReq {
            user_id: USER1,
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
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let fail_protocol = PreCreateAgendaReq {
            user_id: USER1,
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
                ObjectIdentifier::new_with_type(
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let fail_agenda = PreCreateAgendaReq {
            user_id: USER1,
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
                    5,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000005")
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
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
                ObjectIdentifier::new_with_type(
                    12,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        {
            let r = app_process::pre_create_agenda(fail_status, pool.clone())
                .await
                .unwrap();

            assert_eq!(r.data.item_list.len(), 0);

            let expected_messages = vec![AgendaCreateMessage::InvalidPlanStatus
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 2,
                    ..Default::default()
                }))];
            assert_eq!(r.messages.messages, expected_messages);
        }
        {
            let r = app_process::pre_create_agenda(fail_protocol, pool.clone())
                .await
                .unwrap();

            assert_eq!(r.data.item_list.len(), 0);

            let expected_messages = vec![AgendaCreateMessage::AlreadyInProtocol(
                &EcProtocol {
                    id: 2,
                    protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                    ..Default::default()
                },
                &EcProtocolItem {
                    result_id: ResultId::AgreedWithPriceCorrection,
                    ..Default::default()
                },
            )
            .singular(&PlanOrAmendment::Plan(Plan {
                id: 4,
                ..Default::default()
            }))];
            assert_eq!(r.messages.messages, expected_messages);
        }
        {
            let r = app_process::pre_create_agenda(fail_agenda, pool.clone())
                .await
                .unwrap();

            assert_eq!(r.data.item_list.len(), 0);
            let expected_messages = vec![
                AgendaCreateMessage::AlreadyInProtocol(
                    &EcProtocol {
                        id: 2,
                        protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                        ..Default::default()
                    },
                    &EcProtocolItem {
                        result_id: ResultId::AgreedWithPriceCorrection,
                        ..Default::default()
                    },
                )
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 4,
                    ..Default::default()
                })),
                AgendaCreateMessage::AlreadyInProtocol(
                    &EcProtocol {
                        id: 2,
                        protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                        ..Default::default()
                    },
                    &EcProtocolItem {
                        result_id: ResultId::Cancel,
                        ..Default::default()
                    },
                )
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 12,
                        ..Default::default()
                    },
                )),
                AgendaCreateMessage::AlreadyInAgenda(&EcAgenda {
                    id: 1,
                    meeting_date: AsezDate::try_from("01.01.1900").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 5,
                    ..Default::default()
                })),
                AgendaCreateMessage::different_department(),
            ];
            assert_eq!(r.messages.messages, expected_messages);
        }
    })
    .await
}

/// Тестирование кейса, когда пользователь хочет создать элемент Повестки с ППЗ/ДС
/// по которой уже есть protocol_item с result_id=3 и agenda_item, но при этом существуют записи
/// item_relation_agenda_protocol которые пропускают все проверки по Повестке
#[tokio::test]
async fn test_pre_create_agenda_on_item_relation_fail() {
    run_db_test(CREATE_AGENDA_EXTRA_MIGS, |pool| async move {
        let req = PreCreateAgendaReq {
            user_id: USER1,
            item_list: vec![ObjectIdentifier::new_with_type(
                13,
                Uuid::parse_str("00000000-0000-0000-0003-000000000000").unwrap(),
                EntityKind::ContractAmendment,
            )],
        };

        let r = app_process::pre_create_agenda(req, pool.clone()).await.unwrap();

        assert_eq!(r.data.item_list.len(), 1);
        assert_eq!(r.messages.messages.len(), 0);
    })
    .await
}

fn verify_agenda_item<T: Into<CurrencyValue>>(
    agenda_items: &[EcAgendaItem],
    source_uuid: &str,
    number: i64,
    sum_excluded_vat: T,
    pricing_sum_excluded_vat: T,
) {
    let x = agenda_items
        .iter()
        .find(|x| x.source_uuid == Uuid::parse_str(source_uuid).unwrap())
        .unwrap_or_else(|| {
            panic!("agenda_item с source_uuid {} не найден", source_uuid)
        });

    assert!(
        x.number == number
            && !x.is_excluded
            && !x.is_removed
            && !x.is_registered_by_d647
            && x.created_by == USER1
            && x.changed_by == USER1
            && x.sum_excluded_vat.unwrap() == sum_excluded_vat.into()
            && x.pricing_sum_excluded_vat.unwrap()
                == pricing_sum_excluded_vat.into(),
        "{:?}",
        x
    );
}

fn verify_ec_partner(
    partners: &[EcPartner],
    user_id: i32,
    agenda_uuid: Uuid,
) -> bool {
    partners
        .iter()
        .find(|x| x.user_id == user_id)
        .map(|x| x.protocol_agenda_uuid == agenda_uuid)
        .unwrap()
}
