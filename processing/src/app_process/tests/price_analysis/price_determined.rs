use asez2_shared_db::{db_item::Select, uuid};

use asez2_tables::ContractAmendment;
use shared_essential::{
    domain::{Plan, PlanOrAmendment, PlanStatus},
    presentation::dto::{
        general::ObjectIdentifier,
        processing::price_analysis::PriceDeterminedReq,
        response_request::{EntityKind, MessageKind, Messages},
    },
};
use uuid::Uuid;

use crate::app_process::{
    pa_price_determined,
    price_analysis::price_determined::PriceDeterminedMessage,
    tests::{mock_processing_context, run_db_test},
};

const PRICE_DETERMINED_EXTRA_MIGS: &[&str] =
    &["price_analysis/price_determined.sql"];
const USER_ID: i32 = 777;

/// Тестирование кейса, когда пользователь передал
/// невалидные ППЗ/ДС
#[tokio::test]
async fn price_determined_general_failure() {
    run_db_test(PRICE_DETERMINED_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let dto = PriceDeterminedReq {
            user_id: USER_ID,
            item_list: vec![
                // Невалидная ППЗ
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                // Валидная ДС
                ObjectIdentifier::new_with_type(
                    12,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let result = pa_price_determined(dto, pctx).await.unwrap();

        let invalid_plan = PlanOrAmendment::Plan(Plan {
            id: 1,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            ..Default::default()
        });
        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![
                PriceDeterminedMessage::missing_field(
                    "Решение Эксперта АЦ",
                    &invalid_plan,
                ),
                PriceDeterminedMessage::missing_field(
                    "Заключение Эксперта АЦ",
                    &invalid_plan,
                ),
                PriceDeterminedMessage::missing_field(
                    "Метод ценообразования",
                    &invalid_plan,
                ),
                PriceDeterminedMessage::missing_field("Эксперт АЦ", &invalid_plan),
            ],
        };

        assert!(result.data.is_empty());
        assert_eq!(result.messages, expected_messages);
    })
    .await;
}

/// Тестирование кейса, когда пользователь передал
/// невалидные ППЗ/ДС по Экономии
#[tokio::test]
async fn price_determined_savings_failure() {
    run_db_test(PRICE_DETERMINED_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let dto = PriceDeterminedReq {
            user_id: USER_ID,
            item_list: vec![
                // Невалидная ДС
                ObjectIdentifier::new_with_type(
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
                // Валидная ППЗ
                ObjectIdentifier::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Plan,
                ),
            ],
        };

        let result = pa_price_determined(dto, pctx).await.unwrap();

        let invalid_plan = PlanOrAmendment::Amendment(ContractAmendment {
            id: 11,
            uuid: uuid!("00000000-0000-0000-0001-000000000000"),
            ..Default::default()
        });
        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![PriceDeterminedMessage::missing_field(
                "\"Учитывать экономию\"",
                &invalid_plan,
            )],
        };

        assert!(result.data.is_empty());
        assert_eq!(result.messages, expected_messages);
    })
    .await;
}

/// Тестирование кейса, когда пользователь передал
/// ППЗ/ДС c невалидным заключением Эксперта АЦ
#[tokio::test]
async fn price_determined_conclusion_failure() {
    run_db_test(PRICE_DETERMINED_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let dto = PriceDeterminedReq {
            user_id: USER_ID,
            item_list: vec![ObjectIdentifier::new_with_type(
                3,
                Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                EntityKind::Plan,
            )],
        };

        let result = pa_price_determined(dto, pctx).await.unwrap();

        let invalid_plan = PlanOrAmendment::Plan(Plan {
            id: 3,
            uuid: uuid!("00000000-0000-0000-0000-000000000003"),
            ..Default::default()
        });
        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![PriceDeterminedMessage::on_documentation_conclusion(
                &invalid_plan,
            )],
        };

        assert!(result.data.is_empty());
        assert_eq!(result.messages, expected_messages);
    })
    .await;
}

/// Тестирование успешного кейса перевода на новый статус
#[tokio::test]
async fn price_determined_success() {
    run_db_test(PRICE_DETERMINED_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let dto = PriceDeterminedReq {
            user_id: USER_ID,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    12,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
                ObjectIdentifier::new_with_type(
                    13,
                    Uuid::parse_str("00000000-0000-0000-0003-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let result = pa_price_determined(dto.clone(), pctx.clone()).await.unwrap();
        assert_eq!(result.data.len(), 3, "{:?}", result);

        let updated_plans = PlanOrAmendment::select(
            &Select::full::<Plan>()
                .in_any(Plan::uuid, dto.item_list.iter().map(|i| i.uuid))
                .add_replace_order_asc(Plan::id),
            &pctx.db_pool,
        )
        .await
        .unwrap();

        let expected_messages = Messages {
            kind: MessageKind::Success,
            messages: vec![PriceDeterminedMessage::success(&updated_plans)],
        };

        [
            PlanStatus::AnalysisPerformedD646,
            PlanStatus::AnalysisPerformedD647,
            PlanStatus::AnalysisPerformedMTP,
        ]
        .into_iter()
        .zip(updated_plans)
        .for_each(|(new_status, poa)| {
            assert_eq!(
                new_status,
                *poa.status_id(),
                "Не сходится статус в {:?}",
                poa
            )
        });

        assert_eq!(result.messages, expected_messages);
    })
    .await;
}
