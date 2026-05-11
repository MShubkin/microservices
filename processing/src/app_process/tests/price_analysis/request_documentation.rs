use uuid::Uuid;

use asez2_tables::{
    legacy::plans::PlanStatus, processing::status_history::StatusHistory,
    PlanOrAmendment,
};

use asez2_shared_db::{
    db_item::{Select, SelectionKind},
    DbItem,
};

use shared_essential::presentation::dto::{
    general::{ObjectIdentifier, ObjectIdentifierWithStatusNote},
    processing::price_analysis::{
        PreRequestDocumentationReq, RequestDocumentationReq,
    },
    response_request::EntityKind,
};

use crate::app_process::{
    price_analysis::{pa_pre_request_documentation, pa_request_documentation},
    tests::{mock_processing_context, run_db_test},
};

const REQUEST_DOCUMENTATION_EXTRA_MIGS: &[&str] =
    &["price_analysis/request_documentation.sql"];
const USER_ID: i32 = 777;

#[tokio::test]
async fn test_pre_request_documentation() {
    run_db_test(REQUEST_DOCUMENTATION_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let dto = PreRequestDocumentationReq {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    12,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let result =
            pa_pre_request_documentation(dto, pctx.db_pool.clone()).await.unwrap();
        assert_eq!(result.data.len(), 2);
    })
    .await;
}

#[tokio::test]
async fn test_request_documentation() {
    run_db_test(REQUEST_DOCUMENTATION_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let request = RequestDocumentationReq {
            user_id: USER_ID,
            item_list: vec![
                ObjectIdentifierWithStatusNote::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                    "comment1".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    7,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Plan,
                    "comment2".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    8,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Plan,
                    "comment3".to_string(),
                ),
                ObjectIdentifierWithStatusNote::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                    "comment4".to_string(),
                ),
            ],
        };

        let uuids = request.item_list.iter().map(|i| i.uuid).collect::<Vec<_>>();
        let r1 = pa_request_documentation(request, pctx.clone()).await;

        let plans_select = Select::with_fields(["uuid", "id", "status_id"])
            .add_expand_filter("uuid", SelectionKind::In, uuids);
        let plans =
            PlanOrAmendment::select(&plans_select, &pctx.db_pool).await.unwrap();

        let r1 = r1.unwrap();
        assert_eq!(r1.data.len(), 4);
        assert_eq!(plans.len(), 4);

        let plan_check = verify_items(
            &plans,
            vec![
                (
                    "00000000-0000-0000-0000-000000000001",
                    PlanStatus::RequestClientDocumentation,
                ),
                (
                    "00000000-0000-0000-0000-000000000002",
                    PlanStatus::RequestClientDocumentation,
                ),
                (
                    "00000000-0000-0000-0000-000000000003",
                    PlanStatus::RequestClientDocumentation,
                ),
                (
                    "00000000-0000-0000-0002-000000000000",
                    PlanStatus::RequestClientDocumentation,
                ),
            ],
        );
        assert!(plan_check);

        let histories = StatusHistory::select_all(&*pctx.db_pool).await.unwrap();
        assert_eq!(histories.len(), 4);

        let history_check = verify_history_items(
            &histories,
            vec![
                (
                    "00000000-0000-0000-0000-000000000001",
                    PlanStatus::RequestClientDocumentation,
                    "comment1",
                ),
                (
                    "00000000-0000-0000-0000-000000000002",
                    PlanStatus::RequestClientDocumentation,
                    "comment2",
                ),
                (
                    "00000000-0000-0000-0000-000000000003",
                    PlanStatus::RequestClientDocumentation,
                    "comment3",
                ),
                (
                    "00000000-0000-0000-0002-000000000000",
                    PlanStatus::RequestClientDocumentation,
                    "comment4",
                ),
            ],
        );
        assert!(history_check);
    })
    .await
}

fn verify_items(
    plans: &[PlanOrAmendment],
    items_inner: Vec<(&str, PlanStatus)>,
) -> bool {
    items_inner.into_iter().all(|(uuid, status_id)| {
        plans
            .iter()
            .find(|p| *p.uuid() == Uuid::parse_str(uuid).unwrap())
            .map(|p| *p.status_id() == status_id)
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
