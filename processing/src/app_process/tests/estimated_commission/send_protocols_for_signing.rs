use super::*;
use crate::app_process;
use crate::presentation::business_messages::protocol::ProtocolSignMessage;
use shared_essential::presentation::dto::general::ObjectIdsWithUserAndComment;

use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};

const SEND_PROTOCOL_FOR_SIGNING_EXTRA_MIGS: &[&str] =
    &["estimated_commission/send_protocols_for_signing.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn test_send_protocol_for_signing() {
    run_db_test(SEND_PROTOCOL_FOR_SIGNING_EXTRA_MIGS, |pool| async move {
        let success_uuid =
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let ok_req = ObjectIdsWithUserAndComment {
            user_id: 9999,
            ids: vec![ObjectIdentifierWithStatusNote::new(
                1,
                success_uuid,
                "success comment".to_owned(),
            )],
        };
        let fail_req = ObjectIdsWithUserAndComment {
            user_id: 9999,
            ids: vec![
                ObjectIdentifierWithStatusNote::new(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    "fail comment 2".to_owned(),
                ),
                ObjectIdentifierWithStatusNote::new(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    "fail comment 3".to_owned(),
                ),
                ObjectIdentifierWithStatusNote::new(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004")
                        .unwrap(),
                    "fail comment 4".to_owned(),
                ),
                ObjectIdentifierWithStatusNote::new(
                    5,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000005")
                        .unwrap(),
                    "fail comment 5".to_owned(),
                ),
            ],
        };
        let pctx = super::mock_processing_context(pool).await;
        let r2 =
            app_process::send_protocol_for_signing(fail_req, pctx.clone()).await;
        let r1 = app_process::send_protocol_for_signing(ok_req, pctx.clone()).await;
        {
            let r1 = r1.unwrap();
            {
                let s = Select::default();
                let histories =
                    StatusHistory::select(&s, &*pctx.db_pool).await.unwrap();

                assert_eq!(histories.len(), 1);
                assert_eq!(histories[0].object_uuid, success_uuid);
                assert_eq!(
                    histories[0].status_id,
                    EcProtocolStatus::SignaturePending as i16
                );
                assert_eq!(&histories[0].comment, "success comment");
                assert_eq!(histories[0].created_by, 9999);
            }

            match cfg!(with_plan_db) {
                true => assert_eq!(r1.messages.messages.len(), 2),
                false => assert_eq!(r1.messages.messages.len(), 1),
            };

            assert_eq!(r1.messages.kind, MessageKind::Success);
            let messages =
                vec![ProtocolSignMessage::Success.singular(&EcProtocol {
                    id: 1,
                    ..Default::default()
                })];
            assert_eq!(r1.messages.messages, messages);

            assert_eq!(r1.data.item_list.len(), 1);
            assert_eq!(
                r1.data.item_list[0].protocol_status_id.unwrap(),
                EcProtocolStatus::SignaturePending
            );
        }
        {
            let r2 = r2.unwrap();
            let messages = vec![
                ProtocolSignMessage::InvalidProtocolStatus.singular(&EcProtocol {
                    id: 2,
                    status_id: EcProtocolStatus::SignaturePending,
                    ..Default::default()
                }),
                ProtocolSignMessage::InvalidProtocolStatus.singular(&EcProtocol {
                    id: 3,
                    status_id: EcProtocolStatus::Confirmed,
                    ..Default::default()
                }),
                ProtocolSignMessage::InvalidProtocolStatus.singular(&EcProtocol {
                    id: 4,
                    status_id: EcProtocolStatus::Deleted,
                    ..Default::default()
                }),
            ];

            assert_eq!(r2.messages.messages.len(), 3);
            assert_eq!(r2.messages.kind, MessageKind::Error);
            assert_eq!(r2.messages.messages, messages);
            assert!(r2.data.is_empty());
        }
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_pre_send_protocol_for_signing() {
    run_db_test(SEND_PROTOCOL_FOR_SIGNING_EXTRA_MIGS, |pool| async move {
        let ok_req = vec![ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            EntityKind::Plan,
        )];
        let fail_req = vec![
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
            ObjectIdentifier::new_with_type(
                4,
                Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                EntityKind::Plan,
            ),
        ];

        let r1 =
            app_process::pre_send_protocol_for_signing(ok_req, pool.clone()).await;
        let r2 = app_process::pre_send_protocol_for_signing(fail_req, pool).await;
        {
            let r1 = r1.unwrap();
            assert_eq!(r1.messages.messages.len(), 0);
            assert_eq!(r1.messages.kind, MessageKind::Success);
            assert_eq!(r1.data.item_list.len(), 1);
        }
        {
            let r2 = r2.unwrap();
            let messages = vec![
                ProtocolSignMessage::InvalidProtocolStatus.singular(&EcProtocol {
                    id: 2,
                    status_id: EcProtocolStatus::SignaturePending,
                    ..Default::default()
                }),
                ProtocolSignMessage::InvalidProtocolStatus.singular(&EcProtocol {
                    id: 3,
                    status_id: EcProtocolStatus::Confirmed,
                    ..Default::default()
                }),
                ProtocolSignMessage::InvalidProtocolStatus.singular(&EcProtocol {
                    id: 4,
                    status_id: EcProtocolStatus::Deleted,
                    ..Default::default()
                }),
            ];

            assert_eq!(r2.messages.messages.len(), 3);
            assert_eq!(r2.messages.kind, MessageKind::Error);
            assert_eq!(r2.messages.messages, messages);
            assert_eq!(r2.data.item_list.len(), 0);
        }
    })
    .await
}
