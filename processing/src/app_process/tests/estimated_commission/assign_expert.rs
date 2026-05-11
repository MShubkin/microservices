use super::*;
use crate::app_process;

use shared_essential::presentation::dto::response_request::{
    EntityKind, MessageKind,
};

const CANCEL_PLANS_EXTRA_MIGS: &[&str] =
    &["estimated_commission/assign_expert.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn test_assign_expert() {
    run_db_test(CANCEL_PLANS_EXTRA_MIGS, |pool| async move {
        let ok_req = AssignExpertReq {
            user_id: 666,
            ids: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
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
                    102,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        let fail_req = AssignExpertReq {
            user_id: 666,
            ids: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    EntityKind::Plan,
                ),
                ObjectIdentifier::new_with_type(
                    6,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000006")
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
                    102,
                    Uuid::parse_str("00000000-0000-0000-0002-000000000000")
                        .unwrap(),
                    EntityKind::ContractAmendment,
                ),
            ],
        };

        {
            let histories =
                StatusHistory::select(&Select::default(), &*pool).await.unwrap();
            assert!(histories.is_empty());
        }

        let pctx = super::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let ok = app_process::assign_expert(ok_req, pctx.clone()).await;

        let fail = app_process::assign_expert(fail_req, pctx).await;
        {
            let err = fail.unwrap();
            assert!(err.messages.is_error());
        }

        {
            let r1 = ok.unwrap();

            assert_eq!(r1.messages.messages.len(), 1);
            assert_eq!(
                r1.messages.kind,
                MessageKind::Success,
                "Сообщения: {:?}",
                r1.messages
            );
            assert_eq!(
                &r1.messages.messages[0].text,
                "Вы отправили Эксперту АЦ 4 ППЗ/ДС"
            );

            let histories =
                StatusHistory::select(&Select::default(), &*pool).await.unwrap();
            assert_eq!(r1.data.len(), 4);
            assert_eq!(histories.len(), 4);
        }
    })
    .await
}
