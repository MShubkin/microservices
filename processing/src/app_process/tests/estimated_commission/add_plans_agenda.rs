//! Тестирование процесса `get_agenda_items_by_id_range`
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::db_item::AsezDate;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind,
};
use shared_essential::{
    domain::PlanOrAmendmentRep, presentation::dto::response_request::MessageKind,
};

use super::*;
use crate::app_process::estimated_commission::add_plans_agenda::fetch_agenda_with_all_items;
use crate::app_process::{add_plans_agenda, pre_add_plans_agenda};
use crate::common::ProcessingError;
use crate::presentation::business_messages::agenda::AgendaAddPlansMessage;

const ADD_PLANS_AGENDA_EXTRA_MIGS: &[&str] =
    &["estimated_commission/add_plans_agenda.sql"];

/// Тестирование кейса, когда пользователь передал список
/// несуществующих ППЗ/ДС
#[tokio::test]
async fn pre_add_plans_not_found_plans() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                777,
                Uuid::parse_str("00000000-0000-0000-0000-000000000777").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                666,
                Uuid::parse_str("00000000-0000-0000-0000-000000000666").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                667,
                Uuid::parse_str("00000000-0000-0000-0000-000000000667").unwrap(),
                EntityKind::Plan,
            ),
        ];

        let result = pre_add_plans_agenda(dto, pool).await;
        if let Err(ProcessingError::GetItemList(err)) = result {
            let msg = String::from(
                "Записи ППЗ/ДС c идентификаторами 777, 666, 667 не найдены",
            );
            assert_eq!(msg, err);
        } else {
            panic!("Была возвращена не та ошибка: {:?}", result)
        }
    })
    .await;
}

/// Тестирование кейса, когда пользователь хочет добавить элемент Повестки с ППЗ/ДС
/// по которой уже есть protocol_item с result_id=3 и agenda_item, но при этом существуют записи
/// item_relation_agenda_protocol которые пропускают все проверки по Повестке
#[tokio::test]
async fn pre_add_plans_relation_fail() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![ObjectIdentifier::new_with_type(
            9,
            Uuid::parse_str("00000000-0000-0000-0000-000000000009").unwrap(),
            EntityKind::Plan,
        )];

        let r = pre_add_plans_agenda(dto, pool).await.unwrap();

        assert_eq!(r.data.item_list.len(), 1);
        assert_eq!(r.messages.messages.len(), 0);
    })
    .await;
}

// Тестирование кейса, когда пользователь напоролся на ошибку
// по статусу ППЗ/ДС
#[tokio::test]
async fn pre_add_plans_plan_status_error() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                1,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                EntityKind::Plan
            ),
            ObjectIdentifier::new_with_type(
                11,
                Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap(),
                EntityKind::ContractAmendment
            ),
        ];

        let result = pre_add_plans_agenda(dto, pool).await;

        let res = result.unwrap();
        assert!(res.data.item_list.is_empty());

        let messages = res.messages.messages;

        assert_eq!(messages.len(), 1);

        assert_eq!(messages[0].kind, MessageKind::Error);
        assert_eq!(
            messages[0].text, String::from(
                "Добавление в Повестку запрещено. 2 ППЗ/ДС находятся не на статусах СК"
            )
        );
    })
    .await;
}

// Тестирование кейса, когда пользователь напоролся на ошибку
// по наличию Повестки по ППЗ/ДС
#[tokio::test]
async fn pre_add_plans_protocol_error() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![ObjectIdentifier::new_with_type(
            15,
            Uuid::parse_str("00000000-0000-0000-0000-000000000015").unwrap(),
            EntityKind::ContractAmendment,
        )];

        let result = pre_add_plans_agenda(dto, pool).await;

        let res = result.unwrap();
        assert!(res.data.item_list.is_empty());

        let messages = res.messages.messages;

        let expected_messages = vec![AgendaAddPlansMessage::AlreadyInProtocol(
            &EcProtocol {
                id: 3,
                protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                ..Default::default()
            },
            &EcProtocolItem {
                result_id: ResultId::AgreedWithPriceCorrection,
                ..Default::default()
            },
        )
        .singular(&PlanOrAmendment::Amendment(ContractAmendment {
            id: 15,
            ..Default::default()
        }))];

        assert_eq!(messages, expected_messages);
    })
    .await;
}

// Тестирование кейса, когда пользователь напоролся на ошибку
// по наличию Повестки по ППЗ/ДС
#[tokio::test]
async fn pre_add_plans_agenda_error() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                16,
                Uuid::parse_str("00000000-0000-0000-0000-000000000016").unwrap(),
                EntityKind::ContractAmendment
            )
        ];

        let result = pre_add_plans_agenda(dto, pool).await;

        let res = result.unwrap();
        assert!(res.data.item_list.is_empty());

        let messages = res.messages.messages;

        assert_eq!(messages.len(), 1);

        assert_eq!(messages[0].kind, MessageKind::Error);
        assert_eq!(
            messages[0].text, String::from(
                "Добавление в Повестку запрещено. ППЗ/ДС 16 включена в Повестку 4 на 03.01.1900"
            )
        );
    })
    .await;
}

/// Тестирование кейса все элементы прошли проверки
#[tokio::test]
async fn pre_add_plans_full_success() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = vec![
            ObjectIdentifier::new_with_type(
                2,
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                EntityKind::Plan,
            ),
            ObjectIdentifier::new_with_type(
                3,
                Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                EntityKind::Plan,
            ),
        ];

        let result = pre_add_plans_agenda(dto, pool).await;
        let res = result.unwrap();

        assert_eq!(res.data.item_list.len(), 2);
        assert!(verify_plans(&res.data.item_list))
    })
    .await;
}

#[tokio::test]
async fn add_plans_fail() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let not_found = AddPlansAgendaReq {
            is_force: false,
            user_id: USER1,
            item_list: vec![
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
            ],
            agenda_id: 777,
        };
        let wrong_status = AddPlansAgendaReq {
            is_force: true,
            user_id: USER1,
            item_list: vec![
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
            agenda_id: 5,
        };

        {
            let not_found_res = add_plans_agenda(not_found, pctx.clone()).await;
            if let Err(ProcessingError::AddPlansAgenda(msg)) = not_found_res {
                assert_eq!(
                    msg,
                    String::from("Повестка СК с идентификатором 777 не найдена")
                )
            } else {
                panic!("Failed!")
            }
        }
        {
            let wrong_status_res =
                add_plans_agenda(wrong_status, pctx).await.unwrap();

            assert!(wrong_status_res.messages.is_error());
            let messages =
                vec![AgendaAddPlansMessage::invalid_agenda_status(&EcAgenda {
                    id: 5,
                    meeting_date: AsezDate::try_from("1900-01-03").unwrap(),
                    status_id: EcAgendaStatus::ProtocolFormed,
                    ..Default::default()
                })];
            assert_eq!(wrong_status_res.messages.messages, messages);
        }
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn add_plans_success() {
    run_db_test(ADD_PLANS_AGENDA_EXTRA_MIGS, |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let dto = AddPlansAgendaReq {
            is_force: true,
            user_id: USER1,
            item_list: vec![
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
                    12,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000012")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
                // Уже в повестке с is_removed=true и is_excluded=true
                ObjectIdentifier::new_with_type(
                    5,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000005")
                        .unwrap(),
                    EntityKind::Plan,
                ),
            ],
            agenda_id: 3,
        };

        let res = add_plans_agenda(dto, pctx.clone()).await.unwrap();

        assert_eq!(res.messages.kind, MessageKind::Success);
        let expected_messages = vec![AgendaAddPlansMessage::Success(&EcAgenda {
            id: 3,
            meeting_date: AsezDate::try_from("1900-01-03").unwrap(),
            ..Default::default()
        })
        .plural(&vec![
            PlanOrAmendment::Plan(Plan {
                id: 5,
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
                id: 12,
                ..Default::default()
            }),
        ])];
        assert_eq!(res.messages.messages, expected_messages);

        assert_eq!(res.data.item_list, vec![5, 7, 8, 12]);

        let agenda_with_items =
            fetch_agenda_with_all_items(3, &pctx.db_pool).await.unwrap();
        let (agenda, agenda_items): (EcAgenda, Vec<EcAgendaItem>) =
            (agenda_with_items.agenda, agenda_with_items.agenda_items);

        assert_eq!(agenda.changed_by, USER1);
        assert_eq!(agenda_items.len(), 8);

        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000004",
            1,
            false,
            false,
            false,
            1,
            2
        ));
        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000005",
            2,
            false,
            false,
            false,
            1,
            2
        ));
        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000007",
            3,
            false,
            false,
            false,
            1,
            2
        ));
        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000008",
            4,
            false,
            false,
            false,
            1,
            2
        ));
        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000012",
            5,
            false,
            false,
            false,
            2,
            3
        ));
        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000014",
            6,
            false,
            true,
            false,
            2,
            3
        ));
        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000006",
            7,
            true,
            false,
            false,
            1,
            2
        ));
        assert!(verify_agenda_item(
            &agenda_items,
            "00000000-0000-0000-0000-000000000013",
            8,
            true,
            true,
            true,
            2,
            3
        ));

        let plans_select = Select::with_fields([Plan::commission_date])
            .in_any(Plan::id, vec![5, 7, 8, 12]);
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
) -> bool {
    agenda_items
        .iter()
        .find(|item| item.source_uuid == Uuid::parse_str(source_uuid).unwrap())
        .map(|item| {
            item.number == number
                && item.is_registered_by_d647 == is_registered_by_d647
                && item.is_removed == is_removed
                && item.is_excluded == is_excluded
                && item.sum_excluded_vat.unwrap() == sum_excluded_vat.into()
                && item.pricing_sum_excluded_vat.unwrap()
                    == pricing_sum_excluded_vat.into()
        })
        .unwrap()
}

macro_rules! verify_plan {
    ($p: expr) => {
        $p.plan_id.is_some()
            && $p.customer_id.is_some()
            && $p.contract_subject.is_some()
            && $p.pricing_expert_id.is_some()
            && $p.supplier_id.is_some()
            && $p.sum_excluded_vat.is_some()
            && $p.currency_id.is_some()
            && $p.pricing_organization_unit_id.is_some()
            && $p.commission_date.is_some()
            && $p.status_id.is_some()
    };
}

/// Проверить, что были возвращены все нужные поля
fn verify_plans(plans: &[PlanOrAmendmentRep]) -> bool {
    plans.iter().all(|p| match p {
        PlanOrAmendmentRep::Plan(p) => verify_plan!(p),
        PlanOrAmendmentRep::Amendment(a) => verify_plan!(a),
    })
}
