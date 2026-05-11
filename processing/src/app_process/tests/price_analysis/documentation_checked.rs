//! Тестирование процесса [`documentation_checked`]
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::db_item::AsezTimestamp;

use shared_essential::domain::{ContractAmendment, Plan, PlanOrAmendment};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier,
    processing::price_analysis::DocumentationCheckedReq,
    response_request::{BusinessMessage, EntityKind, Messages},
};
use uuid::Uuid;

use crate::{
    app_process::{
        price_analysis::{
            documentation_checked::DocumentationCheckedMessage,
            pa_documentation_checked,
        },
        tests::{mock_processing_context, run_db_test},
    },
    common::ProcessingError,
};

const DOCUMENTATION_CHECKED_EXTRA_MIGS: &[&str] =
    &["price_analysis/documentation_checked.sql"];
const USER_ID: i32 = 777;

/// Тестирование кейса, когда пользователю передал
/// невалидные ППЗ/ДС для данного действия
#[tokio::test]
async fn pa_documentation_checked_wrong_identifier_list() {
    run_db_test(DOCUMENTATION_CHECKED_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let dto = DocumentationCheckedReq {
            user_id: USER_ID,
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
                ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                    EntityKind::Plan
                ),
                ObjectIdentifier::new_with_type(
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
                ObjectIdentifier::new_with_type(
                    12,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
                ObjectIdentifier::new_with_type(
                    13,
                    Uuid::parse_str("00000000-0000-0000-0003-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
                ObjectIdentifier::new_with_type(
                    14,
                    Uuid::parse_str("00000000-0000-0000-0004-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
            ],
        };

        let result = pa_documentation_checked(dto, pctx).await.unwrap_err();

        match result {
            ProcessingError::GetItemList(err) => assert_eq!(err, String::from("ППЗ/ДС с идентификаторами 3, 4, 13, 14 не были найдены для данного действия")),
            _ => panic!("Была возвращена не та ошибка")
        }
    })
    .await;
}

/// По ППЗ/ДС не заполнены поля
#[tokio::test]
async fn missing_fields() {
    run_db_test(DOCUMENTATION_CHECKED_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let dto = DocumentationCheckedReq {
            user_id: USER_ID,
            item_list: vec![ObjectIdentifier::new_with_type(
                101,
                Uuid::parse_str("00000000-0000-0000-0000-000000000101").unwrap(),
                EntityKind::Plan,
            )],
        };

        let res = pa_documentation_checked(dto, pctx).await.unwrap();

        let expected_messages = Messages::default().with_messages(vec![
            DocumentationCheckedMessage::MissingField("Эксперт АЦ").singular(
                &PlanOrAmendment::Plan(Plan {
                    id: 101,
                    ..Default::default()
                }),
            ),
        ]);
        assert_eq!(expected_messages, res.messages);
    })
    .await;
}

/// Тестирование успешного кейса перевода на новый статус
#[tokio::test(flavor = "multi_thread")]
async fn pa_documentation_checked_success() {
    run_db_test(DOCUMENTATION_CHECKED_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let item_list = vec![
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
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
                ObjectIdentifier::new_with_type(
                    12,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000").unwrap(),
                    EntityKind::ContractAmendment
                ),
            ];
        let dto = DocumentationCheckedReq {
            user_id: USER_ID,
            item_list,
        };

        let response = pa_documentation_checked(dto, pctx).await.unwrap();
        let plans = response.data;

        let expected_messages = Messages::default().with_messages(vec![
            DocumentationCheckedMessage::Success.plural(&[
                PlanOrAmendment::Plan(Plan {
                    id: 1,
                    ..Default::default()
                }),
                PlanOrAmendment::Plan(Plan {
                    id: 2,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 11,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 12,
                    ..Default::default()
                })
            ])
        ]);
        assert_eq!(expected_messages, response.messages);

        assert_eq!(plans.len(), 4);

        let now = AsezTimestamp::now().unix_timestamp();
        let verified_plans = plans.into_iter().all(|p| {
            p.is_check_documentation().unwrap()
                // Такая проверка связана с тем, что обработка запроса тоже занимает какое то время
                // поэтому now будет в этом интервале
                && ((now - 10)..=now).contains(
                    &p.check_documentation_date().unwrap().unwrap().unix_timestamp(),
                )
        });
        assert!(verified_plans);
    })
    .await;
}
