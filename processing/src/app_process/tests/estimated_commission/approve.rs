use super::*;
use crate::app_process;
use crate::common::monolith_sender::{self, ObjectInner};
use crate::presentation::legacy_interaction::*;

use env_setup::RabbitCfg;
use shared_essential::application::records::RecordCtx;
use shared_essential::presentation::dto::response_request::*;

const APPROVE_EXTRA_MIGS: &[&str] = &["estimated_commission/approve.sql"];

#[tokio::test]
async fn pre_approve() {
    run_db_test(APPROVE_EXTRA_MIGS, |pool| async move {
        let request_ok = PreApprovePlansReq {
            item_list: vec![
                fixtures::plan_oid_1(),
                fixtures::plan_oid_7(),
                fixtures::plan_oid_8(),
                fixtures::contract_oid_3(),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let request_query = PreApprovePlansReq {
            item_list: vec![
                fixtures::plan_oid_1(),
                fixtures::plan_oid_7(),
                fixtures::plan_oid_8(),
                fixtures::contract_oid_3(),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };

        let request_fail_status = PreApprovePlansReq {
            item_list: vec![
                fixtures::plan_oid_1(),
                fixtures::plan_oid_2(),
                fixtures::plan_oid_3(),
                fixtures::contract_oid_4(),
            ],
            section_id: Section::EstimatedCommissionInPerson,
        };


        let result_ok = app_process::pre_approve(request_ok, pool.clone()).await;
        let result_query = app_process::pre_approve(request_query, pool.clone()).await;
        let result_failure =
            app_process::pre_approve(request_fail_status, pool).await;

        {
            let result = result_ok.unwrap();

            assert_eq!(result.data.item_list.len(), 4);
            assert!(result.messages.messages.is_empty());
        }
        {
            let result = result_query.unwrap();

            assert_eq!(result.data.item_list.len(), 4);
            assert!(result.messages.messages.is_empty());
        }
        {
            let result = result_failure.unwrap();
            assert!(result.data.is_empty());

            assert_eq!(result.messages.messages.len(), 1);

            assert_eq!(result.messages.messages[0].kind, MessageKind::Error);
            assert_eq!(result.messages.messages[0].text, String::from("Выполнить утверждение невозможно. 3 ППЗ/ДС находятся не на статусах СК"))
        }
    })
    .await
}

/// This test only tests whether the ok_request is ok and that plans are sent ok.
#[tokio::test(flavor = "multi_thread")]
async fn approve_send() {
    // Test only works reliably with test threads = 1.
    if !std::env::args().any(|x| x == "--test-threads=1") {
        return;
    };

    let request_ok = ApprovePlansReq {
        item_list: vec![
            fixtures::plan_oids_1(),
            fixtures::plan_oids_7(),
            fixtures::plan_oids_8(),
            fixtures::contract_oids_3(),
        ],
        section_id: Section::EstimatedCommissionInPerson,
        user_id: 666,
        is_force: true,
    };

    let mut note_hunter = [
        fixtures::plan_oids_1(),
        fixtures::plan_oids_7(),
        fixtures::plan_oids_8(),
        fixtures::contract_oids_3(),
    ]
    .into_iter()
    .map(|x| (x.uuid, x.status_note))
    .collect::<ahash::AHashMap<_, _>>();

    // Here we get plans and amendments to compare to what we are sending to
    // the legacy system upon status update.
    let plan_uuids = [
        fixtures::plan_oids_1(),
        fixtures::plan_oids_7(),
        fixtures::plan_oids_8(),
    ]
    .into_iter()
    .map(|x| Value::from(x.uuid));

    let amendment_uuids =
        [fixtures::contract_oids_3()].into_iter().map(|x| Value::from(x.uuid));

    run_db_test(APPROVE_EXTRA_MIGS, |pool| async move {
        let mut recorder =
            RecordCtx::new(0, pool.clone()).begin().await.expect("ok");

        let amendments = monolith_sender::amendments_for_monolith(
            amendment_uuids,
            &mut recorder,
        )
        .await
        .unwrap()
        .expect("some");
        let plans = monolith_sender::plans_for_monolith(plan_uuids, &mut recorder)
            .await
            .unwrap()
            .expect("some");

        let plans = if let ObjectInner::UpdatePlans(mut p) = plans {
            for plan in p.iter_mut() {
                let h = &mut plan.header;
                assert_eq!(h.commission_kind_id, Some(CommissionKind::InPerson));
                assert!(h.pricing_expert_id.is_some());
                assert!(h.pricing_method_id.is_some());
                assert!(h.pricing_resume.is_some());

                h.status_id = Some(PlanStatus::PriceConfirmed);
                h.changed_by = Some(666);
                h.changed_at = None;
                h.status_note = note_hunter.remove(&h.uuid.unwrap());
            }
            ProcessingToLegacyReq::UpdatePlans(p)
        } else {
            panic!("Impossible, we have plans.");
        };
        let amendments = if let ObjectInner::UpdateAmendments(mut a) = amendments {
            for amendment in a.iter_mut() {
                let h = &mut amendment.header;
                assert_eq!(h.commission_kind_id, Some(CommissionKind::InPerson));
                assert!(h.pricing_expert_id.is_some());
                assert!(h.pricing_method_id.is_some());
                assert!(h.pricing_resume.is_some());

                h.status_id = Some(PlanStatus::PriceConfirmed);
                h.changed_by = Some(666);
                h.changed_at = None;
                h.created_at = None;
                h.status_note = note_hunter.remove(&h.uuid.unwrap());
            }
            ProcessingToLegacyReq::UpdateAmendments(a)
        } else {
            panic!("Impossible, we have amendments.");
        };

        let pctx = mock_processing_context_with_vhost(pool, "send_success").await;
        let comparison_result =
            super::launch_monolith_listener(&pctx, vec![plans, amendments]).await;

        let result_ok = app_process::action_approve(request_ok, pctx).await;

        {
            let result = result_ok.unwrap();
            println!("{result:#?}");

            assert!(result.data.item_list.is_empty());
            assert_eq!(result.messages.messages.len(), 1);
            assert_eq!(
                result.messages.messages[0],
                Message {
                    kind: MessageKind::Success,
                    text: "Вы утвердили 4 ППЗ/ДС".to_string(),
                    parameters: Params {
                        description: "".to_string(),
                        item_list: [
                            ParamItem::from_id(1).with_type(EntityKind::Plan),
                            ParamItem::from_id(7).with_type(EntityKind::Plan),
                            ParamItem::from_id(8).with_type(EntityKind::Plan),
                            ParamItem::from_id(103).with_type(EntityKind::Plan),
                        ]
                        .to_vec(),
                    },
                    fields: Default::default()
                }
            );
        }
        comparison_result.await.unwrap();
    })
    .await
}

/// This test only tests whether the ok_request is ok and that plans are sent ok.
#[tokio::test(flavor = "multi_thread")]
async fn approve_send_fail_revert2() {
    // Test only works reliably with test threads = 1.
    if !std::env::args().any(|x| x == "--test-threads=1") {
        return;
    };

    let request_ok = ApprovePlansReq {
        item_list: vec![
            fixtures::plan_oids_1(),
            fixtures::plan_oids_7(),
            fixtures::plan_oids_8(),
            fixtures::contract_oids_3(),
        ],
        section_id: Section::EstimatedCommissionInPerson,
        user_id: 666,
        is_force: true,
    };

    // Here we get plans and amendments to compare to what we are sending to
    // the legacy system upon status update.
    let plan_uuids = [
        fixtures::plan_oids_1(),
        fixtures::plan_oids_7(),
        fixtures::plan_oids_8(),
    ]
    .into_iter()
    .map(|x| Value::from(x.uuid));

    let amendment_uuids =
        [fixtures::contract_oids_3()].into_iter().map(|x| Value::from(x.uuid));

    run_db_test(APPROVE_EXTRA_MIGS, |pool| async move {
        let old_histories = StatusHistory::select_all(&*pool).await.unwrap();
        assert_eq!(old_histories.len(), 10);

        let pctx = mock_processing_context_with_vhost(pool.clone(), "revert2").await;

        // We do not need the listener for this task
        // (since we want the messages to be lost in space)
        // but we do need the sender, since you must try to then fail.
        let monolith_sender =
            crate::common::MonolithSender::new(&RabbitCfg::from_env().unwrap().into(), &pool).await.unwrap();
        monolith_sender.run();
        let result_ok = app_process::action_approve(request_ok, pctx).await;

        {
            let result = result_ok.unwrap();

            assert!(result.data.item_list.is_empty());
            assert_eq!(result.messages.messages.len(), 1);
            assert_eq!(&result.messages.messages[0].text, "Вы утвердили 4 ППЗ/ДС",);
        }

        let newer_histories = StatusHistory::select_all(&*pool).await.unwrap();
        assert_eq!(newer_histories.len(), 14);
        // We wait a bit for the reversion, just in case we outrun our own train.
        // The timeout on the sender is 12 seconds, so we need to outwait it unfortunately.
        tokio::time::sleep(std::time::Duration::from_secs(14)).await;

        let new_histories = StatusHistory::select_all(&*pool).await.unwrap();

        assert_eq!(new_histories.len(), 18);
        let new_histories = new_histories
            .into_iter()
            .filter(|newest| !newer_histories.iter().any(|new| new.uuid==newest.uuid))
            .collect::<Vec<_>>();

        let select = Select::with_fields(["uuid", "status_id"])
            .in_any(Plan::uuid, plan_uuids)
            .add_replace_order_asc("id");
        let select2 = Select::with_fields(["uuid", "status_id"])
            .in_any(Plan::uuid, amendment_uuids)
            .add_replace_order_asc("id");

        let plans = Plan::select(&select, &*pool).await.unwrap();
        let amendments = ContractAmendment::select(&select2, &*pool).await.unwrap();

        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].status_id as i16, 223);
        assert_eq!(plans[1].status_id as i16, 343);
        assert_eq!(plans[2].status_id as i16, 223);
        assert_eq!(amendments.len(), 1);
        assert_eq!(amendments[0].status_id as i16, 353);

        let history = new_histories
            .iter()
            .find(|x| x.object_uuid==plans[0].uuid)
            .unwrap();

        assert_eq!(history.status_id, 223);
        assert_eq!(
            &history.comment,
            "Автоматический откат статуса системой: Broker error: Слишком долгое ожидание сообщения"
        );
    })
    .await
}

/// This test only tests whether the ok_request is ok and that plans are sent ok.
/// NB: This test is quite flaky when run in conjunction with other tests, even
/// if we're using a single test thread. Probably some rabbit interactions
/// with the other revert test.
#[tokio::test(flavor = "multi_thread")]
async fn approve_send_fail_revert1() {
    // Test only works reliably with test threads = 1.
    if !std::env::args().any(|x| x == "--test-threads=1") {
        return;
    };

    let request_ok = ApprovePlansReq {
        item_list: vec![
            fixtures::plan_oids_1(),
            fixtures::plan_oids_7(),
            fixtures::plan_oids_8(),
            fixtures::contract_oids_3(),
        ],
        section_id: Section::EstimatedCommissionInPerson,
        user_id: 666,
        is_force: true,
    };

    // Here we get plans and amendments to compare to what we are sending to
    // the legacy system upon status update.
    let plan_uuids = [
        fixtures::plan_oids_1(),
        fixtures::plan_oids_7(),
        fixtures::plan_oids_8(),
    ]
    .into_iter()
    .map(|x| Value::from(x.uuid));

    let amendment_uuids =
        [fixtures::contract_oids_3()].into_iter().map(|x| Value::from(x.uuid));

    run_db_test(APPROVE_EXTRA_MIGS, |pool| async move {
        let old_histories = StatusHistory::select_all(&*pool).await.unwrap();
        assert_eq!(old_histories.len(), 10);

        let pctx =
            mock_processing_context_with_vhost(pool.clone(), "revert1").await;
        let result_ok = app_process::action_approve(request_ok, pctx.clone()).await;
        // A little bit risky in terms of ordering, but should get there before
        // any rabbit operations.
        let newer_histories = StatusHistory::select_all(&*pool).await.unwrap();
        assert_eq!(newer_histories.len(), 14);

        // Sender can be activated after the the operation.
        let shutdown_handle =
            super::launch_monolith_listener_return_error(&pctx, 2).await;
        {
            let result = result_ok.unwrap();

            assert!(result.data.item_list.is_empty());
            assert_eq!(result.messages.messages.len(), 1);
            assert_eq!(&result.messages.messages[0].text, "Вы утвердили 4 ППЗ/ДС");
        }
        // We need to wait for rabbit to do its job after it finishes.
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        shutdown_handle.await.unwrap();

        let new_histories = StatusHistory::select_all(&*pool).await.unwrap();

        assert_eq!(new_histories.len(), 18);
        let new_histories = new_histories
            .into_iter()
            .filter(|newest| {
                !newer_histories.iter().any(|new| new.uuid == newest.uuid)
            })
            .collect::<Vec<_>>();

        let select = Select::with_fields(["uuid", "status_id"])
            .in_any(Plan::uuid, plan_uuids)
            .add_replace_order_asc("id");
        let select2 = Select::with_fields(["uuid", "status_id"])
            .in_any(Plan::uuid, amendment_uuids)
            .add_replace_order_asc("id");

        let plans = Plan::select(&select, &*pool).await.unwrap();
        let amendments = ContractAmendment::select(&select2, &*pool).await.unwrap();

        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].status_id as i16, 223);
        assert_eq!(plans[1].status_id as i16, 343);
        assert_eq!(plans[2].status_id as i16, 223);
        assert_eq!(amendments.len(), 1);
        assert_eq!(amendments[0].status_id as i16, 353);

        let history =
            new_histories.iter().find(|x| x.object_uuid == plans[0].uuid).unwrap();

        assert_eq!(history.status_id, 223);
        assert_eq!(
            &history.comment,
            "Автоматический откат статуса системой: Oh no"
        );
    })
    .await
}

#[tokio::test]
async fn empty() {
    run_db_test(APPROVE_EXTRA_MIGS, |pool| async move {
        let mut recorder =
            RecordCtx::new(0, pool.clone()).begin().await.expect("ok");

        let amendments = monolith_sender::amendments_for_monolith(
            std::iter::empty(),
            &mut recorder,
        )
        .await
        .unwrap();
        let plans =
            monolith_sender::plans_for_monolith(std::iter::empty(), &mut recorder)
                .await
                .unwrap();

        assert!(amendments.is_none());
        assert!(plans.is_none());
    })
    .await;
}

mod fixtures {
    use shared_essential::presentation::dto::general::{
        ObjectIdentifier, ObjectIdentifierWithStatusNote,
    };
    use shared_essential::presentation::dto::response_request::EntityKind;
    use uuid::Uuid;

    pub fn plan_oids_1() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            1,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            String::from("note3"),
        )
    }

    pub fn plan_oids_7() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            7,
            Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap(),
            String::from("note7"),
        )
    }

    pub fn plan_oids_8() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            8,
            Uuid::parse_str("00000000-0000-0000-0000-000000000008").unwrap(),
            String::from("note8"),
        )
    }

    pub fn contract_oids_3() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            103,
            Uuid::parse_str("00000000-0000-0000-0003-000000000000").unwrap(),
            String::from("notec3"),
        )
    }

    pub fn plan_oid_1() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn plan_oid_2() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            2,
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn plan_oid_3() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            3,
            Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn plan_oid_7() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            7,
            Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn plan_oid_8() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            8,
            Uuid::parse_str("00000000-0000-0000-0000-000000000008").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn contract_oid_3() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            103,
            Uuid::parse_str("00000000-0000-0000-0003-000000000000").unwrap(),
            EntityKind::ContractAmendment,
        )
    }

    pub fn contract_oid_4() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            104,
            Uuid::parse_str("00000000-0000-0000-0004-000000000000").unwrap(),
            EntityKind::ContractAmendment,
        )
    }
}
