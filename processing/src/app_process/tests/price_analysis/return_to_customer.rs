//! Тестирование процесса [`return_to_customer`]
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::{
    db_item::{Select, SelectionKind},
    DbItem,
};

use uuid::Uuid;

use asez2_tables::{
    legacy::plans::PlanStatus, processing::status_history::StatusHistory,
    ExpertConclusionId, PlanOrAmendment,
};
use shared_essential::presentation::dto::{
    general::ObjectIdentifierWithStatusNote,
    processing::price_analysis::ReturnToCustomerReq, response_request::EntityKind,
};

use crate::{
    app_process::{
        price_analysis::pa_return_to_customer,
        tests::{mock_processing_context, run_db_test},
    },
    common::ProcessingError,
};

const RETURN_TO_CUSTOMER_EXTRA_MIGS: &[&str] =
    &["price_analysis/return_to_customer.sql"];
const USER_ID: i32 = 777;

/// Тестирование кейса, когда пользователю передал
/// невалидные ППЗ/ДС для данного действия
#[tokio::test]
async fn pa_return_to_customer_wrong_identifier_list() {
    run_db_test(RETURN_TO_CUSTOMER_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let dto = ReturnToCustomerReq {
            user_id: USER_ID,
            item_list: vec![
                ObjectIdentifierWithStatusNote::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                    EntityKind::Plan,
                    "Something".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000").unwrap(),
                    EntityKind::ContractAmendment,
                    "Something".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                    EntityKind::Plan,
                    "Something".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    14,
                    Uuid::parse_str("00000000-0000-0000-0004-000000000000").unwrap(),
                    EntityKind::ContractAmendment,
                    "Something".to_string(),
                ),
            ],
        };

        let result = pa_return_to_customer(dto, pctx).await.unwrap_err();

        match result {
            ProcessingError::GetItemList(err) => assert_eq!(err, String::from("ППЗ/ДС с идентификаторами 4, 14 не были найдены для данного действия")),
            _ => panic!("Была возвращена не та ошибка")
        }
    })
    .await;
}

/// Тестирование успешного кейса перевода на новый статус
#[tokio::test]
async fn pa_return_to_customer_success() {
    run_db_test(RETURN_TO_CUSTOMER_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let dto = ReturnToCustomerReq {
            user_id: USER_ID,
            item_list: vec![
                ObjectIdentifierWithStatusNote::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                    "Something1".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    11,
                    Uuid::parse_str("00000000-0000-0000-0001-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                    "Something11".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Plan,
                    "Something2".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    12,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                    "Something12".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Plan,
                    "Something3".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    13,
                    Uuid::parse_str("00000000-0000-0000-0003-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                    "Something13".to_string(),
                ),
            ],
        };
        let uuids = dto.item_list.iter().map(|i| i.uuid).collect::<Vec<_>>();

        pa_return_to_customer(dto, pctx.clone()).await.unwrap();

        let plans_select = Select::with_fields([
            "uuid",
            "id",
            "status_id",
            "expert_conclusion_id",
        ])
        .add_expand_filter("uuid", SelectionKind::In, uuids);
        let plans =
            PlanOrAmendment::select(&plans_select, &pctx.db_pool).await.unwrap();
        assert_eq!(plans.len(), 6);

        let plan_check = verify_items(
            &plans,
            vec![
                (
                    "00000000-0000-0000-0000-000000000001",
                    PlanStatus::AnalysisPerformedD646,
                ),
                (
                    "00000000-0000-0000-0000-000000000002",
                    PlanStatus::AnalysisPerformedD647,
                ),
                (
                    "00000000-0000-0000-0000-000000000003",
                    PlanStatus::AnalysisPerformedMTP,
                ),
                (
                    "00000000-0000-0000-0001-000000000000",
                    PlanStatus::AnalysisPerformedD646,
                ),
                (
                    "00000000-0000-0000-0002-000000000000",
                    PlanStatus::AnalysisPerformedD647,
                ),
                (
                    "00000000-0000-0000-0003-000000000000",
                    PlanStatus::AnalysisPerformedMTP,
                ),
            ],
        );
        assert!(plan_check);

        let histories = StatusHistory::select_all(&*pctx.db_pool).await.unwrap();
        assert_eq!(histories.len(), 6);

        let history_check = verify_history_items(
            &histories,
            vec![
                (
                    "00000000-0000-0000-0000-000000000001",
                    PlanStatus::AnalysisPerformedD646,
                    "Something1",
                ),
                (
                    "00000000-0000-0000-0000-000000000002",
                    PlanStatus::AnalysisPerformedD647,
                    "Something2",
                ),
                (
                    "00000000-0000-0000-0000-000000000003",
                    PlanStatus::AnalysisPerformedMTP,
                    "Something3",
                ),
                (
                    "00000000-0000-0000-0001-000000000000",
                    PlanStatus::AnalysisPerformedD646,
                    "Something11",
                ),
                (
                    "00000000-0000-0000-0002-000000000000",
                    PlanStatus::AnalysisPerformedD647,
                    "Something12",
                ),
                (
                    "00000000-0000-0000-0003-000000000000",
                    PlanStatus::AnalysisPerformedMTP,
                    "Something13",
                ),
            ],
        );
        assert!(history_check);
    })
    .await;
}

fn verify_items(
    plans: &[PlanOrAmendment],
    items_inner: Vec<(&str, PlanStatus)>,
) -> bool {
    items_inner.into_iter().all(|(uuid, status_id)| {
        plans
            .iter()
            .find(|p| *p.uuid() == Uuid::parse_str(uuid).unwrap())
            .map(|p| {
                *p.status_id() == status_id
                    && p.expert_conclusion_id().unwrap()
                        == ExpertConclusionId::RefundToCustomer
            })
            .unwrap()
    })
}

fn verify_history_items(
    histories: &[StatusHistory],
    items_inner: Vec<(&str, PlanStatus, &str)>,
) -> bool {
    items_inner.into_iter().all(|(uuid, status_id, comment)| {
        histories
            .iter()
            .find(|s| s.object_uuid == Uuid::parse_str(uuid).unwrap())
            .map(|s| {
                s.comment == comment
                    && s.created_by == USER_ID
                    && s.status_id == status_id as i16
            })
            .unwrap()
    })
}
