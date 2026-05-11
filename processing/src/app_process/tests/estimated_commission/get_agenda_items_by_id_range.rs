//! Тестирование процесса `get_agenda_items_by_id_range`
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::db_item::AsezDate;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, Messages,
};
use shared_essential::{
    domain::PlanOrAmendmentRep, presentation::dto::response_request::MessageKind,
};
use tokio::test;

use super::*;
use crate::app_process::get_agenda_items_by_id_range;
use crate::common::ProcessingError;
use crate::presentation::business_messages::agenda::AgendaGetItemsMessage;

const GET_AGENDA_ITEMS_BY_ID_RANGE_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_agenda_items_by_id_range.sql"];

/// Тестирование кейса, когда пользователь передал список
/// несуществующих ППЗ/ДС
#[test]
async fn not_found_plans() {
    run_db_test(GET_AGENDA_ITEMS_BY_ID_RANGE_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaItemsByIdRangeReq {
            agenda_id: 22,
            is_registered_by_d647: false,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            item_list: vec![vec![777], vec![666, 667]],
        };

        let result = get_agenda_items_by_id_range(dto, pool.clone()).await;
        if let Err(ProcessingError::GetItemList(err)) = result {
            let msg = String::from(
                "Записи ППЗ/ДС c идентификаторами 666, 667, 777 не найдены",
            );
            assert_eq!(msg, err);
        } else {
            panic!("{:?}", result);
        }
    })
    .await;
}

/// Тестирование кейса, когда пользователь напоролся на все ошибки
/// по проверке данных
#[test]
async fn catch_all_errors_with_no_output() {
    run_db_test(GET_AGENDA_ITEMS_BY_ID_RANGE_EXTRA_MIGS, |pool| async move {
        // ППЗ
        // 21 - заочная СК
        // 22 - включено в протокол
        // 23 - включено в другую повестку, на рассмотрении
        // 24 - включено в данную повестку, данный раздел, на рассмотрении
        // 25 - включено в данную повестку, данный раздел, снято с рассмотрения
        // 26 - включено в данную повестку, другой раздел, на рассмотрении
        // ДС
        // 31 - заочная СК
        // 32 - включено в протокол
        // 33 - включено в другую повестку, на рассмотрении
        // 34 - включено в данную повестку, данный раздел, на рассмотрении
        // 35 - включено в данную повестку, данный раздел, снято с рассмотрения
        // 36 - включено в данную повестку, другой раздел, на рассмотрении
        let dto = GetAgendaItemsByIdRangeReq {
            agenda_id: 22,
            is_registered_by_d647: false,
            uuid: Uuid::parse_str("00000000-0000-0000-0002-000000000002").unwrap(),
            item_list: vec![vec![21, 26], vec![31, 36]],
        };

        let res = get_agenda_items_by_id_range(dto, pool.clone()).await.unwrap();

        assert!(res.data.item_list.is_empty());

        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![
                AgendaGetItemsMessage::InvalidCommissionKind.plural(&vec![
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 31,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Plan(Plan {
                        id: 21,
                        ..Default::default()
                    }),
                ]),
                AgendaGetItemsMessage::AlreadyInProtocol(
                    &EcProtocol {
                        id: 21,
                        protocol_date: AsezDate::try_from("1910-01-01").unwrap(),
                        ..Default::default()
                    },
                    &EcProtocolItem {
                        result_id: ResultId::AgreedWithPriceCorrection,
                        ..Default::default()
                    },
                )
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 22,
                    ..Default::default()
                })),
                AgendaGetItemsMessage::AlreadyInProtocol(
                    &EcProtocol {
                        id: 21,
                        protocol_date: AsezDate::try_from("1910-01-01").unwrap(),
                        ..Default::default()
                    },
                    &EcProtocolItem {
                        result_id: ResultId::AgreedWithPriceCorrection,
                        ..Default::default()
                    },
                )
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 32,
                        ..Default::default()
                    },
                )),
                AgendaGetItemsMessage::AlreadyInAgenda(&EcAgenda {
                    id: 21,
                    meeting_date: AsezDate::try_from("1900-02-02").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 23,
                    ..Default::default()
                })),
                AgendaGetItemsMessage::AlreadyInCurrentAgenda(&EcAgenda {
                    id: 22,
                    meeting_date: AsezDate::try_from("1900-02-02").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 24,
                    ..Default::default()
                })),
                AgendaGetItemsMessage::AlreadyInCurrentAgenda(&EcAgenda {
                    id: 22,
                    meeting_date: AsezDate::try_from("1900-02-02").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 26,
                    ..Default::default()
                })),
                AgendaGetItemsMessage::AlreadyInAgenda(&EcAgenda {
                    id: 21,
                    meeting_date: AsezDate::try_from("1900-02-02").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 33,
                        ..Default::default()
                    },
                )),
                AgendaGetItemsMessage::AlreadyInCurrentAgenda(&EcAgenda {
                    id: 22,
                    meeting_date: AsezDate::try_from("1900-02-02").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 34,
                        ..Default::default()
                    },
                )),
                AgendaGetItemsMessage::AlreadyInCurrentAgenda(&EcAgenda {
                    id: 22,
                    meeting_date: AsezDate::try_from("1900-02-02").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 36,
                        ..Default::default()
                    },
                )),
            ],
        };

        let (eq, err) = res.messages.eq_unordered(&expected_messages);
        if !(eq || err.is_none()) {
            panic!(
                "Messages not equal: {} \n {:?}",
                err.expect("No error text provided"),
                res.messages
            );
        }
    })
    .await;
}

/// Тестирование кейса все элементы прошли проверки
#[test]
async fn full_success() {
    run_db_test(GET_AGENDA_ITEMS_BY_ID_RANGE_EXTRA_MIGS, |pool| async move {
        // ППЗ
        // 1 - нет ни в протоколе, ни в повестке
        // 2 - есть в протоколе с признаком is_removed
        // 3 - есть в удаленном протоколе
        // 4 - есть в повестке с признаком is_removed
        // 5 - есть в повестке с признаком is_excluded
        // 6 - есть в удаленной повестке
        // 7 - есть в данной повестке с признаком is_removed
        // 8 - есть в данной повестке в другом разделе с признаком is_excluded
        // ДС
        // 11 - нет ни в протоколе, ни в повестке
        // 12 - есть в протоколе с признаком is_removed
        // 13 - есть в удаленном протоколе
        // 14 - есть в повестке с признаком is_removed
        // 15 - есть в повестке с признаком is_excluded
        // 16 - есть в удаленной повестке
        // 17 - есть в данной повестке с признаком is_removed
        // 18 - есть в данной повестке в другом разделе с признаком is_excluded

        let dto = GetAgendaItemsByIdRangeReq {
            agenda_id: 14,
            is_registered_by_d647: false,
            uuid: Uuid::parse_str("00000000-0000-0000-0001-000000000004").unwrap(),
            item_list: vec![vec![1, 8], vec![11, 18]],
        };

        let res = get_agenda_items_by_id_range(dto, pool.clone()).await.unwrap();

        assert_eq!(res.messages.kind, MessageKind::Success);
        let expected_messages = vec![AgendaGetItemsMessage::Success(&EcAgenda {
            id: 14,
            meeting_date: AsezDate::try_from("01.01.1900").unwrap(),
            ..Default::default()
        })
        .plural(
            &(1..=8)
                .map(|id| {
                    PlanOrAmendment::Plan(Plan {
                        id,
                        ..Default::default()
                    })
                })
                .chain((11..=18).map(|id| {
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id,
                        ..Default::default()
                    })
                }))
                .collect::<Vec<_>>(),
        )];
        assert_eq!(res.messages.messages, expected_messages);

        assert_eq!(res.data.item_list.len(), 16);
        assert!(verify_plans(&res.data.item_list));
    })
    .await;
}

/// Тестирование кейса все элементы прошли проверки
#[test]
async fn warnings() {
    run_db_test(GET_AGENDA_ITEMS_BY_ID_RANGE_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaItemsByIdRangeReq {
            agenda_id: 31,
            is_registered_by_d647: false,
            uuid: Uuid::parse_str("00000000-0000-0000-0003-000000000001").unwrap(),
            item_list: vec![vec![41, 44]],
        };

        let res = get_agenda_items_by_id_range(dto, pool.clone()).await.unwrap();
        assert_eq!(res.data.item_list.len(), 4);

        let expected_messages = Messages {
            kind: MessageKind::Warning,
            messages: vec![
                AgendaGetItemsMessage::DifferentDepartment(&EcAgenda {
                    id: 31,
                    meeting_date: AsezDate::try_from("03.03.1900").unwrap(),
                    ..Default::default()
                })
                .plural(&vec![
                    PlanOrAmendment::Plan(Plan {
                        id: 42,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 44,
                        ..Default::default()
                    }),
                ]),
                AgendaGetItemsMessage::Success(&EcAgenda {
                    id: 31,
                    meeting_date: AsezDate::try_from("03.03.1900").unwrap(),
                    ..Default::default()
                })
                .plural(&vec![
                    PlanOrAmendment::Plan(Plan {
                        id: 41,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Plan(Plan {
                        id: 42,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 43,
                        ..Default::default()
                    }),
                    PlanOrAmendment::Amendment(ContractAmendment {
                        id: 44,
                        ..Default::default()
                    }),
                ]),
            ],
        };

        let (eq, err) = res.messages.eq_unordered(&expected_messages);
        if !(eq || err.is_none()) {
            panic!("Messages not equal: {}", err.expect("No error text provided"));
        }

        assert!(verify_plans(&res.data.item_list));
    })
    .await;
}

/// Проверить, что были возвращены все нужные поля
fn verify_plans(plans: &[PlanOrAmendmentRep]) -> bool {
    plans.iter().all(|p| {
        p.uuid().is_some()
            && p.plan_id().is_some()
            && p.customer_id().is_some()
            && p.contract_subject().is_some()
            && p.pricing_expert_id().is_some()
            && p.pricing_resume().is_some()
            && p.supplier_id().is_some()
            && p.sum_excluded_vat().is_some()
            && p.currency_id().is_some()
            && p.section_id().is_some()
            && p.status_id().is_some()
            && p.pricing_sum_excluded_vat().is_some()
    })
}
