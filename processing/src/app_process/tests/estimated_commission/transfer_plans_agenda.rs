//! Тестирование процесса `get_agenda_items_by_id_range`
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::{db_item::AsezDate, uuid};
use shared_essential::presentation::dto::response_request::MessageKind;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, Messages,
};

use super::*;
use crate::app_process::{pre_transfer_plans_agenda, transfer_plans_agenda};
use crate::presentation::business_messages::agenda::AgendaTransferPlansMessage;

const TRANSFER_PLANS_AGENDA_EXTRA_MIGS: &[&str] =
    &["estimated_commission/transfer_plans_agenda.sql"];

// Тестирование кейса, когда пользователь напоролся на ошибку
// по статусу ППЗ/ДС
#[tokio::test]
async fn pre_transfer_plans_status_error() {
    run_db_test(TRANSFER_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                1,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                11,
                Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap(),
                EntityKind::ContractAmendment,
            ),
        ];

        let result = pre_transfer_plans_agenda(dto, pool).await;

        let res = result.unwrap();
        assert!(res.data.item_list.is_empty());

        let expected_messages = vec![AgendaTransferPlansMessage::InvalidPlanStatus
            .plural(&[
                PlanOrAmendment::Plan(Plan {
                    id: 1,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 11,
                    ..Default::default()
                }),
            ])];
        let (check_res, msg) = res.messages.eq_unordered(&Messages {
            kind: MessageKind::Error,
            messages: expected_messages,
        });
        assert!(check_res, "{:?}", msg);
    })
    .await;
}

/// Тестирование кейса, когда пользователь напоролся на ошибку
/// по наличию Повестки по ППЗ/ДС
#[tokio::test]
async fn pre_transfer_plans_protocol_error() {
    run_db_test(TRANSFER_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                2,
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                12,
                Uuid::parse_str("00000000-0000-0000-0000-000000000012").unwrap(),
                EntityKind::ContractAmendment,
            ),
        ];

        let result = pre_transfer_plans_agenda(dto, pool).await;

        let res = result.unwrap();
        assert!(res.data.item_list.is_empty());

        let expected_messages = vec![
            AgendaTransferPlansMessage::AlreadyInProtocol(
                &EcProtocol {
                    id: 1,
                    protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                    ..Default::default()
                },
                &EcProtocolItem {
                    result_id: ResultId::AgreedWithPriceCorrection,
                    ..Default::default()
                },
            )
            .singular(&PlanOrAmendment::Plan(Plan {
                id: 2,
                ..Default::default()
            })),
            AgendaTransferPlansMessage::AlreadyInProtocol(
                &EcProtocol {
                    id: 1,
                    protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                    ..Default::default()
                },
                &EcProtocolItem {
                    result_id: ResultId::AgreedWithPriceCorrection,
                    ..Default::default()
                },
            )
            .singular(&PlanOrAmendment::Amendment(ContractAmendment {
                id: 12,
                ..Default::default()
            })),
        ];
        let (check_res, msg) = res.messages.eq_unordered(&Messages {
            kind: MessageKind::Error,
            messages: expected_messages,
        });
        assert!(check_res, "{:?}: {:#?}", msg, res.messages);
    })
    .await;
}

/// Тестирование кейса, когда пользователь напоролся на ошибку
/// по отсутствию Повестки по ППЗ/ДС
#[tokio::test]
async fn pre_transfer_plans_agenda_error() {
    run_db_test(TRANSFER_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            // По этой ППЗ позиция удалена
            ObjectIdentifier::new_with_type(
                5,
                Uuid::parse_str("00000000-0000-0000-0000-000000000005").unwrap(),
                EntityKind::Plan,
            ),
            // По этой ДС нет в принципе позиции
            ObjectIdentifier::new_with_type(
                15,
                Uuid::parse_str("00000000-0000-0000-0000-000000000015").unwrap(),
                EntityKind::ContractAmendment,
            ),
        ];

        let result = pre_transfer_plans_agenda(dto, pool).await;

        let res = result.unwrap();
        assert!(res.data.item_list.is_empty());

        let expected_messages =
            vec![AgendaTransferPlansMessage::NotIncludedInAgenda.plural(&[
                PlanOrAmendment::Plan(Plan {
                    id: 5,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 15,
                    ..Default::default()
                }),
            ])];
        let (check_res, msg) = res.messages.eq_unordered(&Messages {
            kind: MessageKind::Error,
            messages: expected_messages,
        });
        assert!(check_res, "{:?}", msg);
    })
    .await;
}

/// Тестирование предупреждения о несоответствии департаментов
#[tokio::test]
async fn pre_transfer_plans_departament_warning() {
    run_db_test(TRANSFER_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                3,
                Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                4,
                Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                EntityKind::Plan,
            ),
        ];

        let result = pre_transfer_plans_agenda(dto, pool).await;
        let res = result.unwrap();

        let expected_messages =
            vec![AgendaTransferPlansMessage::different_department()];
        assert_eq!(res.messages.messages, expected_messages);

        assert_eq!(res.data.item_list.len(), 2);
    })
    .await;
}

/// Тестирование кейса все элементы прошли проверки
#[tokio::test]
async fn pre_transfer_plans_full_success() {
    run_db_test(TRANSFER_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                3,
                Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                13,
                Uuid::parse_str("00000000-0000-0000-0000-000000000013").unwrap(),
                EntityKind::ContractAmendment,
            ),
        ];

        let result = pre_transfer_plans_agenda(dto, pool).await;
        let res = result.unwrap();

        assert!(res.messages.messages.is_empty(), "{:?}", res.messages.messages);
        assert_eq!(res.data.item_list.len(), 2);
    })
    .await;
}

/// Проверка кейса, когда пользователь пытается переместить ППЗ/ДС в Повестку с невалидным статусом
#[tokio::test(flavor = "multi_thread")]
async fn transfer_plans_agenda_status_error() {
    run_db_test(TRANSFER_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let dto = TransferPlansAgendaReq {
            agenda_id: 3,
            user_id: 123,
            is_force: false,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    13,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000013")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let result = transfer_plans_agenda(dto, pctx).await;
        let res = result.unwrap();

        assert_eq!(res.data.item_list.len(), 0);
        let expected_messages =
            vec![AgendaTransferPlansMessage::invalid_agenda_status(&EcAgenda {
                id: 3,
                ..Default::default()
            })];
        assert_eq!(res.messages.messages, expected_messages);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn transfer_plans_agenda_success() {
    run_db_test(TRANSFER_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let ids = vec![
            ObjectIdentifier::new_with_type(
                3,
                Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                13,
                Uuid::parse_str("00000000-0000-0000-0000-000000000013").unwrap(),
                EntityKind::ContractAmendment,
            ),
        ];
        let agenda_id = 4;
        let user_id = 123;

        let dto = TransferPlansAgendaReq {
            agenda_id,
            user_id,
            is_force: true,
            item_list: ids.clone(),
        };

        let res = transfer_plans_agenda(dto, pctx.clone()).await.unwrap();

        assert_eq!(res.messages.kind, MessageKind::Success);
        let expected_messages =
            vec![AgendaTransferPlansMessage::Success(&EcAgenda {
                id: 4,
                ..Default::default()
            })
            .plural(&vec![
                PlanOrAmendment::Plan(Plan {
                    id: 3,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 13,
                    ..Default::default()
                }),
            ])];
        assert_eq!(res.messages.messages, expected_messages);

        assert_eq!(res.data.item_list, vec![3, 13]);

        let new_agenda_items = EcAgendaItem::select(
            &Select::full::<EcAgendaItem>().eq(
                EcAgendaItem::agenda_uuid,
                uuid!("00000000-0000-0000-0000-000000000004"),
            ),
            &*pctx.db_pool,
        )
        .await
        .unwrap();
        let old_agenda_items = EcAgendaItem::select(
            &Select::full::<EcAgendaItem>().eq(
                EcAgendaItem::agenda_uuid,
                uuid!("00000000-0000-0000-0000-000000000002"),
            ),
            &*pctx.db_pool,
        )
        .await
        .unwrap();
        let updated_agenda = EcAgenda::select(
            &Select::full::<EcAgenda>().eq(EcAgenda::id, agenda_id),
            &*pctx.db_pool,
        )
        .await
        .unwrap()
        .pop()
        .unwrap();

        assert_eq!(updated_agenda.changed_by, user_id);
        assert_eq!(new_agenda_items.len(), 3);
        verify_agenda_item(
            &new_agenda_items,
            "00000000-0000-0000-0000-000000000014",
            1,
            false,
            false,
            false,
            1,
            2,
        );
        verify_agenda_item(
            &new_agenda_items,
            "00000000-0000-0000-0000-000000000003",
            2,
            false,
            false,
            false,
            1,
            2,
        );
        verify_agenda_item(
            &new_agenda_items,
            "00000000-0000-0000-0000-000000000013",
            3,
            false,
            false,
            false,
            2,
            3,
        );

        assert_eq!(old_agenda_items.len(), 4);
        verify_agenda_item(
            &old_agenda_items,
            "00000000-0000-0000-0000-000000000003",
            1,
            true,
            true,
            false,
            1,
            2,
        );
        verify_agenda_item(
            &old_agenda_items,
            "00000000-0000-0000-0000-000000000013",
            2,
            true,
            true,
            false,
            0,
            0,
        );
        verify_agenda_item(
            &old_agenda_items,
            "00000000-0000-0000-0000-000000000004",
            3,
            false,
            false,
            false,
            2,
            3,
        );
        verify_agenda_item(
            &old_agenda_items,
            "00000000-0000-0000-0000-000000000005",
            4,
            false,
            true,
            false,
            2,
            3,
        );

        let plans_select = Select::with_fields([Plan::commission_date])
            .in_any(Plan::id, vec![3, 13]);
        let plans =
            PlanOrAmendment::select(&plans_select, &pctx.db_pool).await.unwrap();

        assert!(plans.into_iter().all(|p| p.commission_date().unwrap()
            == AsezDate::try_from("1900-01-03").unwrap()))
    })
    .await;
}

#[allow(clippy::too_many_arguments)]
fn verify_agenda_item<T: Into<CurrencyValue>>(
    agenda_items: &[EcAgendaItem],
    source_uuid: &str,
    number: i64,
    is_registered_by_d647: bool,
    is_removed: bool,
    is_excluded: bool,
    sum_excluded_vat: T,
    pricing_sum_excluded_vat: T,
) {
    let item = agenda_items
        .iter()
        .find(|item| item.source_uuid == Uuid::parse_str(source_uuid).unwrap())
        .unwrap();
    assert!(
        item.number == number
            && item.is_registered_by_d647 == is_registered_by_d647
            && item.is_removed == is_removed
            && item.is_excluded == is_excluded
            && item.sum_excluded_vat.unwrap() == sum_excluded_vat.into()
            && item.pricing_sum_excluded_vat.unwrap()
                == pricing_sum_excluded_vat.into(),
        "{:#?}",
        item
    )
}
