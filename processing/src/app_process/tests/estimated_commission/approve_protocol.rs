use super::*;
use crate::app_process;
use crate::presentation::business_messages::protocol::ProtocolApproveMessage;
use ahash::AHashSet;
use asez2_shared_db::{db_item::SelectionKind, uuid};
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};
use sqlx::{Pool, Postgres};

const APPROVE_PROTOCOL_EXTRA_MIGS: &[&str] =
    &["estimated_commission/approve_protocol.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn test_pre_approve_protocol() {
    run_db_test(APPROVE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let ok_req = vec![
            ObjectIdentifier::new_with_type(
                1,
                uuid!("00000000-0000-0000-0000-000000000001"),
                EntityKind::Protocol,
            ),
            ObjectIdentifier::new_with_type(
                3,
                uuid!("00000000-0000-0000-0000-000000000003"),
                EntityKind::Protocol,
            ),
            ObjectIdentifier::new_with_type(
                5,
                uuid!("00000000-0000-0000-0000-000000000005"),
                EntityKind::Protocol,
            ),
            ObjectIdentifier::new_with_type(
                6,
                uuid!("00000000-0000-0000-0000-000000000006"),
                EntityKind::Protocol,
            ),
            ObjectIdentifier::new_with_type(
                8,
                uuid!("00000000-0000-0000-0000-000000000008"),
                EntityKind::Protocol,
            ),
            ObjectIdentifier::new_with_type(
                10,
                uuid!("00000000-0000-0000-0000-000000000010"),
                EntityKind::Protocol,
            ),
        ];
        let fail = vec![
            ObjectIdentifier::new_with_type(
                2,
                uuid!("00000000-0000-0000-0000-000000000002"),
                EntityKind::Protocol,
            ),
            ObjectIdentifier::new_with_type(
                9,
                uuid!("00000000-0000-0000-0000-000000000009"),
                EntityKind::Protocol,
            ),
        ];

        let r1 = app_process::pre_approve_protocol(ok_req, pool.clone()).await;
        let r2 = app_process::pre_approve_protocol(fail, pool).await;

        {
            let r1 = r1.unwrap();
            assert!(r1.messages.messages.is_empty());
            assert_eq!(r1.data.item_list.len(), 6);
        }
        {
            let r2 = r2.unwrap();
            let messages = vec![
                ProtocolApproveMessage::InvalidProtocolStatus.singular(
                    &EcProtocol {
                        id: 2,
                        status_id: EcProtocolStatus::AgreementPending,
                        ..Default::default()
                    },
                ),
                ProtocolApproveMessage::InvalidProtocolStatus.singular(
                    &EcProtocol {
                        id: 9,
                        status_id: EcProtocolStatus::Confirmed,
                        ..Default::default()
                    },
                ),
            ];

            assert_eq!(r2.messages.messages.len(), 2);
            assert_eq!(r2.messages.kind, MessageKind::Error);
            assert_eq!(r2.messages.messages, messages);
        }
    })
    .await
}

#[tokio::test]
#[ignore = "Тест отключен до обощения реализации Rabbit сервисов"]
async fn test_approve_protocol() {
    run_db_test(APPROVE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let ids = vec![
            ObjectIdentifierWithStatusNote::new(
                1,
                uuid!("00000000-0000-0000-0000-000000000001"),
                String::from("note1"),
            ),
            ObjectIdentifierWithStatusNote::new(
                3,
                uuid!("00000000-0000-0000-0000-000000000003"),
                String::from("note3"),
            ),
        ];
        let ok_req = ApproveProtocolReq {
            user_id: 9999,
            ids: ids.clone(),
            protocol_type_id: ProtocolType::InPersonMeeting,
        };
        let fail = ApproveProtocolReq {
            user_id: 9999,
            ids: vec![
                ObjectIdentifierWithStatusNote::new(
                    2,
                    uuid!("00000000-0000-0000-0000-000000000002"),
                    String::from("note1"),
                ),
                ObjectIdentifierWithStatusNote::new(
                    9,
                    uuid!("00000000-0000-0000-0000-000000000009"),
                    String::from("note9"),
                ),
            ],
            protocol_type_id: ProtocolType::InPersonMeeting,
        };
        let pctx = super::mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;
        let master_data_service = super::master_data_service(&pctx).await;

        // FAIL MUST COME BEFORE SUCCESS.
        let r2 = app_process::approve_protocol(fail, pctx.clone(),  master_data_service.clone()).await;
        let r1 = app_process::approve_protocol(ok_req, pctx.clone(), master_data_service.clone()).await;

        {
            let r1 = r1.unwrap();

            match cfg!(with_plan_db) {
                true => assert_eq!(r1.messages.messages.len(), 3),
                false => assert_eq!(r1.messages.messages.len(), 1),
            };
            assert!(r1.messages.kind == MessageKind::Success);

            let s = Select::default().add_expand_filter(
                "uuid",
                SelectionKind::In,
                ids.iter().map(|oid| oid.uuid),
            );
            let protocols = EcProtocol::select(&s, pctx.db_pool.as_ref())
                .await
                .expect("should succeed");
            assert!(protocols
                .iter()
                .all(|p| p.status_id == EcProtocolStatus::Confirmed));

            let pool = pctx.db_pool.clone();
            assert_plan_status(
                "00000000-0000-0000-0000-000000000001",
                PlanStatus::PriceConfirmed,
                false,
                pool.clone(),
            )
            .await;
            assert_plan_status(
                "00000000-0000-0000-0000-000000000002",
                PlanStatus::ExecutorAppointedD646,
                false,
                pool.clone(),
            )
            .await;
            assert_plan_status(
                "00000000-0000-0000-0000-000000000004",
                PlanStatus::PlanCancelled,
                true,
                pool.clone(),
            )
            .await;
            assert_plan_status(
                "00000000-0000-0000-0000-000000000006",
                PlanStatus::EstimatedCommissionInPerson,
                false,
                pool.clone(),
            )
            .await;
            assert_plan_status(
                "00000000-0000-0000-0000-000000000007",
                PlanStatus::EstimatedCommissionInPerson,
                false,
                pool.clone(),
            )
            .await;
        }
        {
            let r2 = r2.unwrap();
            let messages = vec![
                ProtocolApproveMessage::InvalidProtocolStatus.singular(
                    &EcProtocol {
                        id: 2,
                        status_id: EcProtocolStatus::AgreementPending,
                        ..Default::default()
                    },
                ),
                ProtocolApproveMessage::InvalidProtocolStatus.singular(
                    &EcProtocol {
                        id: 9,
                        status_id: EcProtocolStatus::Confirmed,
                        ..Default::default()
                    },
                ),
            ];

            assert_eq!(r2.messages.messages.len(), 2);
            assert_eq!(r2.messages.kind, MessageKind::Error);
            assert_eq!(r2.messages.messages, messages);
        }
        {
            let status_history = StatusHistory::select(
                &Select::full::<StatusHistory>()
                    .in_any(StatusHistory::comment, ["note1", "note3", "note9"]),
                &*pctx.db_pool,
            )
            .await
            .unwrap();

            assert_eq!(3, status_history.len());
            let uuids: AHashSet<_> =
                status_history.into_iter().map(|x| x.object_uuid).collect();
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000001")));
            assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000003")));

            assert!(!uuids.contains(&uuid!("00000000-0000-0000-0000-000000000002")));
            assert!(!uuids.contains(&uuid!("00000000-0000-0000-0000-000000000009")));
        }
    })
    .await
}

async fn assert_plan_status(
    uuid: &str,
    status: PlanStatus,
    commission_cleared: bool,
    pool: Arc<Pool<Postgres>>,
) {
    let uuid = uuid.parse::<Uuid>().expect("correct uuid");
    let plan = Plan::select(&Select::default().eq(Plan::uuid, uuid), pool.as_ref())
        .await
        .expect("should succeed")
        .pop()
        .expect("plan should exist");
    assert_eq!(plan.status_id, status, "plan {uuid}");
    assert_eq!(
        plan.commission_date.is_none(),
        commission_cleared,
        "commission date for plan {uuid}"
    );
    assert_eq!(
        plan.commission_kind_id == CommissionKind::Undefined,
        commission_cleared,
        "commission kind for plan {uuid}"
    );
}
