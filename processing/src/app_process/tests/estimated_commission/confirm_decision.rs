use super::*;
use crate::app_process;
use crate::presentation::business_messages::protocol::ConfirmDecisionMessage;
use asez2_shared_db::uuid;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};

const CONFIRM_DECISION_EXTRA_MIGS: &[&str] =
    &["estimated_commission/confirm_decision.sql"];

#[tokio::test]
async fn test_confirm_decision_d647() {
    run_db_test(CONFIRM_DECISION_EXTRA_MIGS, |pool| async move {
        let req = ConfirmDecisionReq {
            is_registered_by_d647: true,
            protocol_id: 1,
            protocol_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            user_id: 1,
            item_list: vec![
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                    source_uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                    plan_id: 3,
                    object_type: EntityKind::Plan,
                    result_id: ResultId::Approved,
                    status_note: String::new(),
                },
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                    source_uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                    plan_id: 4,
                    object_type: EntityKind::Plan,
                    result_id: ResultId::Approved,
                    status_note: String::new(),
                },
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000007"),
                    source_uuid: uuid!("00000000-0000-0000-0003-000000000000"),
                    plan_id: 103,
                    object_type: EntityKind::ContractAmendment,
                    result_id: ResultId::Approved,
                    status_note: String::new(),
                },
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000008"),
                    source_uuid: uuid!("00000000-0000-0000-0004-000000000000"),
                    plan_id: 104,
                    object_type: EntityKind::ContractAmendment,
                    result_id: ResultId::Approved,
                    status_note: String::new(),
                },
            ],
        };

        let pctx = super::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;
        let master_data_service = super::master_data_service(&pctx).await;

        let res = app_process::confirm_decision(
            req,
            pctx.clone(),
            master_data_service.clone(),
        )
        .await
        .unwrap();

        assert_eq!(res.messages.messages.len(), 1);

        assert!(res.messages.kind == MessageKind::Success);
        assert_eq!(
            res.messages.messages,
            vec![ConfirmDecisionMessage::Success.plural(&[
                PlanOrAmendment::Plan(Plan {
                    id: 3,
                    ..Default::default()
                }),
                PlanOrAmendment::Plan(Plan {
                    id: 4,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 103,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 104,
                    ..Default::default()
                }),
            ])]
        );

        let plans = PlanOrAmendment::select_dual(
            &Select::full::<Plan>(),
            &Select::full::<ContractAmendment>(),
            &pctx.db_pool,
        )
        .await
        .unwrap();

        assert_plan(
            &plans,
            "00000000-0000-0000-0000-000000000003",
            PlanStatus::PriceDetermined,
            false,
            false,
        );
        assert_plan(
            &plans,
            "00000000-0000-0000-0000-000000000004",
            PlanStatus::PriceConfirmed,
            false,
            false,
        );
        assert_plan(
            &plans,
            "00000000-0000-0000-0003-000000000000",
            PlanStatus::PriceConfirmed,
            false,
            false,
        );
        assert_plan(
            &plans,
            "00000000-0000-0000-0004-000000000000",
            PlanStatus::PriceConfirmed,
            false,
            false,
        );
    })
    .await
}

#[tokio::test]
#[ignore = "Тест отключен до обощения реализации Rabbit сервисов"]
async fn test_confirm_decision_not_d647() {
    run_db_test(CONFIRM_DECISION_EXTRA_MIGS, |pool| async move {
        let req = ConfirmDecisionReq {
            is_registered_by_d647: false,
            protocol_id: 1,
            protocol_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            user_id: 1,
            item_list: vec![
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                    source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                    plan_id: 1,
                    object_type: EntityKind::Plan,
                    result_id: ResultId::Approved,
                    status_note: String::new(),
                },
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                    source_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                    plan_id: 2,
                    object_type: EntityKind::Plan,
                    result_id: ResultId::AgreedWithPriceCorrection,
                    status_note: String::new(),
                },
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000005"),
                    source_uuid: uuid!("00000000-0000-0000-0001-000000000000"),
                    plan_id: 101,
                    object_type: EntityKind::ContractAmendment,
                    result_id: ResultId::NotAgreed,
                    status_note: String::new(),
                },
                ConfirmDecisionItem {
                    uuid: uuid!("00000000-0000-0000-0000-000000000006"),
                    source_uuid: uuid!("00000000-0000-0000-0002-000000000000"),
                    plan_id: 102,
                    object_type: EntityKind::ContractAmendment,
                    result_id: ResultId::Cancel,
                    status_note: String::new(),
                },
            ],
        };

        let pctx = super::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;
        let master_data_service = super::master_data_service(&pctx).await;

        let res = app_process::confirm_decision(
            req,
            pctx.clone(),
            master_data_service.clone(),
        )
        .await
        .unwrap();

        assert_eq!(res.messages.messages.len(), 1);

        assert!(res.messages.kind == MessageKind::Success);
        assert_eq!(
            res.messages.messages,
            vec![ConfirmDecisionMessage::Success.plural(&[
                PlanOrAmendment::Plan(Plan {
                    id: 1,
                    ..Default::default()
                }),
                PlanOrAmendment::Plan(Plan {
                    id: 2,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 101,
                    ..Default::default()
                }),
                PlanOrAmendment::Amendment(ContractAmendment {
                    id: 102,
                    ..Default::default()
                }),
            ])]
        );

        let plans = PlanOrAmendment::select_dual(
            &Select::full::<Plan>(),
            &Select::full::<ContractAmendment>(),
            &pctx.db_pool,
        )
        .await
        .unwrap();

        assert_plan(
            &plans,
            "00000000-0000-0000-0000-000000000001",
            PlanStatus::PriceConfirmed,
            false,
            false,
        );
        assert_plan(
            &plans,
            "00000000-0000-0000-0000-000000000002",
            PlanStatus::ExecutorAppointedD646,
            false,
            false,
        );
        assert_plan(
            &plans,
            "00000000-0000-0000-0001-000000000000",
            PlanStatus::ExecutorAppointedD647,
            true,
            true,
        );
        assert_plan(
            &plans,
            "00000000-0000-0000-0002-000000000000",
            PlanStatus::PlanCancelled,
            true,
            true,
        );
    })
    .await
}

#[tokio::test]
async fn test_status_note_written() {
    run_db_test(CONFIRM_DECISION_EXTRA_MIGS, |pool| async move {
        let req = ConfirmDecisionReq {
            is_registered_by_d647: false,
            protocol_id: 1,
            protocol_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            user_id: 1,
            item_list: vec![ConfirmDecisionItem {
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                plan_id: 1,
                object_type: EntityKind::Plan,
                result_id: ResultId::Approved,
                status_note: "hello!".to_string(),
            }],
        };

        let pctx = super::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;
        let master_data_service = super::master_data_service(&pctx).await;

        let res = app_process::confirm_decision(
            req,
            pctx.clone(),
            master_data_service.clone(),
        )
        .await
        .unwrap();

        assert_eq!(res.messages.messages.len(), 1);

        assert!(res.messages.kind == MessageKind::Success);

        let status_history = StatusHistory::select(
            &Select::full::<StatusHistory>().eq(StatusHistory::comment, "hello!"),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(1, status_history.len());
        assert_eq!(
            uuid!("00000000-0000-0000-0000-000000000001"),
            status_history[0].object_uuid
        );
    })
    .await;
}

fn assert_plan(
    plans: &[PlanOrAmendment],
    uuid: &str,
    status: PlanStatus,
    commission_kind_cleared: bool,
    commission_date_cleared: bool,
) {
    let plan = plans.iter().find(|p| p.uuid().to_string() == uuid).unwrap();

    assert_eq!(*plan.status_id(), status, "plan {uuid}");
    assert_eq!(
        plan.commission_date().is_none(),
        commission_date_cleared,
        "commission date for plan {uuid} - {:?}",
        plan.commission_date()
    );
    assert_eq!(
        *plan.commission_kind_id() == CommissionKind::Undefined,
        commission_kind_cleared,
        "commission kind for plan {uuid} - {}",
        plan.commission_kind_id()
    );
}
