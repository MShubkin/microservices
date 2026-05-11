use ahash::AHashMap;
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
    processing::price_analysis::{DeclineByChiefReq, PreDeclineByChiefReq},
    response_request::EntityKind,
};

use crate::app_process::{
    price_analysis::{pa_decline_by_chief, pa_pre_decline_by_chief},
    tests::{mock_processing_context, run_db_test},
};

const DECLINE_BY_CHIEF_EXTRA_MIGS: &[&str] =
    &["price_analysis/decline_by_chief.sql"];
const USER_ID: i32 = 777;

#[tokio::test]
async fn test_pre_decline_by_chief() {
    run_db_test(DECLINE_BY_CHIEF_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let dto = PreDeclineByChiefReq {
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
            pa_pre_decline_by_chief(dto, pctx.db_pool.clone()).await.unwrap();
        assert_eq!(result.data.len(), 2);
    })
    .await;
}

#[tokio::test]
async fn test_decline_by_chief() {
    run_db_test(DECLINE_BY_CHIEF_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let request = DeclineByChiefReq {
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
                    2,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                    "comment4".to_string(),
                ),
            ],
        };

        let uuids = request.item_list.iter().map(|i| i.uuid).collect::<Vec<_>>();
        let r1 = pa_decline_by_chief(request, pctx.clone()).await;

        let plans_select = Select::with_fields(["uuid", "id", "status_id"])
            .add_expand_filter("uuid", SelectionKind::In, uuids);
        let plans =
            PlanOrAmendment::select(&plans_select, &pctx.db_pool).await.unwrap();

        let r1 = r1.unwrap();
        assert_eq!(r1.data.len(), 3);
        assert_eq!(plans.len(), 3);

        let plan_check = verify_items(
            &plans,
            vec![
                (
                    "00000000-0000-0000-0000-000000000001",
                    PlanStatus::ExecutorAppointedD646,
                ),
                (
                    "00000000-0000-0000-0000-000000000002",
                    PlanStatus::ExecutorAppointedD647,
                ),
                (
                    "00000000-0000-0000-0002-000000000000",
                    PlanStatus::ExecutorAppointedMTP,
                ),
            ],
        );
        assert!(plan_check);

        let histories = StatusHistory::select_all(&*pctx.db_pool).await.unwrap();
        assert_eq!(histories.len(), 3);

        let history_check = verify_history_items(
            &histories,
            vec![
                (
                    "00000000-0000-0000-0000-000000000001",
                    PlanStatus::ExecutorAppointedD646,
                    "comment1",
                ),
                (
                    "00000000-0000-0000-0000-000000000002",
                    PlanStatus::ExecutorAppointedD647,
                    "comment2",
                ),
                (
                    "00000000-0000-0000-0002-000000000000",
                    PlanStatus::ExecutorAppointedMTP,
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
    let plan_map: AHashMap<Uuid, PlanStatus> =
        plans.iter().map(|p| (*p.uuid(), *p.status_id())).collect();

    items_inner.into_iter().all(|(uuid_str, expected_status)| {
        let uuid = Uuid::parse_str(uuid_str).unwrap();
        plan_map.get(&uuid).map_or(false, |&status| status == expected_status)
    })
}

fn verify_history_items(
    histories: &[StatusHistory],
    items_inner: Vec<(&str, PlanStatus, &str)>,
) -> bool {
    let history_map: AHashMap<Uuid, (&str, i16, i32)> = histories
        .iter()
        .map(|s| (s.object_uuid, (s.comment.as_str(), s.status_id, s.created_by)))
        .collect();

    items_inner
        .into_iter()
        .all(|(uuid_str, expected_status, expected_comment)| {
            let uuid = Uuid::parse_str(uuid_str).unwrap();
            history_map.get(&uuid).map_or(
                false,
                |&(comment, status, created_by)| {
                    comment == expected_comment
                        && status == expected_status as i16
                        && created_by == USER_ID
                },
            )
        })
}
