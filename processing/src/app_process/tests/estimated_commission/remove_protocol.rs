use super::*;
use crate::app_process;
use crate::presentation::business_messages::protocol::ProtocolRemoveMessage;

use ahash::AHashSet;
use asez2_shared_db::uuid;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind,
};

const PRE_REMOVE_PROTOCOL_EXTRA_MIGS: &[&str] =
    &["estimated_commission/remove_protocol.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn test_pre_remove_protocol() {
    run_db_test(PRE_REMOVE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let ok_req = PreRemoveProtocolReq {
            user_id: 9999,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    1,
                    uuid!("00000000-0000-0000-0000-000000000001"),
                    EntityKind::Protocol,
                ),
                ObjectIdentifier::new_with_type(
                    2,
                    uuid!("00000000-0000-0000-0000-000000000002"),
                    EntityKind::Protocol,
                ),
            ],
        };
        let warn_req = PreRemoveProtocolReq {
            user_id: 9999,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    3,
                    uuid!("00000000-0000-0000-0000-000000000003"),
                    EntityKind::Protocol,
                ),
                ObjectIdentifier::new_with_type(
                    4,
                    uuid!("00000000-0000-0000-0000-000000000004"),
                    EntityKind::Protocol,
                ),
            ],
        };
        let error_req = PreRemoveProtocolReq {
            user_id: 9999,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
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
                // Проверить, что предупреждения не будут возвращены при наличии ошибок
                ObjectIdentifier::new_with_type(
                    3,
                    uuid!("00000000-0000-0000-0000-000000000003"),
                    EntityKind::Protocol,
                ),
            ],
        };

        let ok_res = app_process::pre_remove_protocol(ok_req, pool.clone()).await;
        let warn_res =
            app_process::pre_remove_protocol(warn_req, pool.clone()).await;
        let error_res = app_process::pre_remove_protocol(error_req, pool).await;

        {
            let res = ok_res.unwrap();
            assert!(res.messages.messages.is_empty());
            assert_eq!(res.data.item_list.len(), 2);
        }
        {
            let res = warn_res.unwrap();

            assert_eq!(res.data.item_list.len(), 2);

            assert_eq!(res.messages.kind, MessageKind::Warning);
            let expected_messages = vec![
                ProtocolRemoveMessage::ProtocolStatusWarn.singular(&EcProtocol {
                    id: 3,
                    status_id: EcProtocolStatus::AgreementPending,
                    ..Default::default()
                }),
                ProtocolRemoveMessage::ProtocolStatusWarn.singular(&EcProtocol {
                    id: 4,
                    status_id: EcProtocolStatus::SignaturePending,
                    ..Default::default()
                }),
            ];
            assert_eq!(res.messages.messages, expected_messages);
        }
        {
            let res = error_res.unwrap();

            assert_eq!(res.data.item_list.len(), 0);

            assert_eq!(res.messages.kind, MessageKind::Error);
            assert_eq!(res.messages.messages.len(), 2);

            let expected_messages = vec![
                ProtocolRemoveMessage::InvalidProtocolStatus.singular(
                    &EcProtocol {
                        id: 5,
                        status_id: EcProtocolStatus::Confirmed,
                        ..Default::default()
                    },
                ),
                ProtocolRemoveMessage::InvalidProtocolStatus.singular(
                    &EcProtocol {
                        id: 6,
                        status_id: EcProtocolStatus::Deleted,
                        ..Default::default()
                    },
                ),
            ];
            assert_eq!(res.messages.messages, expected_messages);
        }
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_remove_protocol_correspondence() {
    run_db_test(PRE_REMOVE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req = RemoveProtocolReq {
            user_id: 9999,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                ObjectIdentifierWithStatusNote::new(
                    7,
                    uuid!("00000000-0000-0000-0000-000000000007"),
                    "123".to_owned(),
                ),
                ObjectIdentifierWithStatusNote::new(
                    8,
                    uuid!("00000000-0000-0000-0000-000000000008"),
                    "123".to_owned(),
                ),
            ],
        };
        let pctx = super::mock_processing_context(pool).await;

        let res = app_process::remove_protocol(req, pctx.clone()).await.unwrap();

        assert_eq!(res.messages.messages.len(), 1);
        assert_eq!(res.messages.kind, MessageKind::Success);

        let expected_messages = vec![ProtocolRemoveMessage::Success(
            ProtocolType::CorrespondenceMeeting,
        )
        .plural(&[
            EcProtocol {
                id: 7,
                ..Default::default()
            },
            EcProtocol {
                id: 8,
                ..Default::default()
            },
        ])];
        assert_eq!(res.messages.messages, expected_messages);

        assert_eq!(res.data.item_list.len(), 2);

        let protocol_select =
            Select::full::<EcProtocol>().in_any(EcProtocol::id, vec![7, 8]);
        let protocols =
            EcProtocol::select(&protocol_select, &*pctx.db_pool).await.unwrap();

        verify_protocol(&protocols, 7);
        verify_protocol(&protocols, 8);

        let status_history = StatusHistory::select(
            &Select::full::<StatusHistory>().eq(StatusHistory::comment, "123"),
            &*pctx.db_pool,
        )
        .await
        .unwrap();

        assert_eq!(2, status_history.len());
        let uuids: AHashSet<_> =
            status_history.into_iter().map(|x| x.object_uuid).collect();
        assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000007")));
        assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000008")));
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_remove_protocol_in_person() {
    run_db_test(PRE_REMOVE_PROTOCOL_EXTRA_MIGS, |pool| async move {
        let req = RemoveProtocolReq {
            user_id: 9999,
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                ObjectIdentifierWithStatusNote::new(
                    1,
                    uuid!("00000000-0000-0000-0000-000000000001"),
                    "123".to_owned(),
                ),
                ObjectIdentifierWithStatusNote::new(
                    2,
                    uuid!("00000000-0000-0000-0000-000000000002"),
                    "123".to_owned(),
                ),
            ],
        };
        let pctx = super::mock_processing_context(pool).await;

        let res = app_process::remove_protocol(req, pctx.clone()).await.unwrap();

        assert_eq!(res.messages.messages.len(), 1);
        assert_eq!(res.messages.kind, MessageKind::Success);

        let expected_messages =
            vec![ProtocolRemoveMessage::Success(ProtocolType::InPersonMeeting)
                .plural(&[
                    EcProtocol {
                        id: 1,
                        ..Default::default()
                    },
                    EcProtocol {
                        id: 2,
                        ..Default::default()
                    },
                ])];
        assert_eq!(res.messages.messages, expected_messages);

        assert_eq!(res.data.item_list.len(), 2);

        let protocol_select =
            Select::full::<EcProtocol>().in_any(EcProtocol::id, vec![1, 2]);
        let protocols =
            EcProtocol::select(&protocol_select, &*pctx.db_pool).await.unwrap();

        verify_protocol(&protocols, 1);
        verify_protocol(&protocols, 2);

        let agenda_select =
            Select::full::<EcAgenda>().in_any(EcAgenda::id, vec![1, 2]);
        let agendas =
            EcAgenda::select(&agenda_select, &*pctx.db_pool).await.unwrap();

        verify_agenda(&agendas, 1, EcAgendaStatus::Sent);
        verify_agenda(&agendas, 2, EcAgendaStatus::ProtocolFormed);

        let rels = RelAgendaProtocol::select_all(&*pctx.db_pool).await.unwrap();
        let item_rels =
            RelAgendaProtocolItem::select_all(&*pctx.db_pool).await.unwrap();

        assert!(rels.is_empty());
        assert!(item_rels.is_empty());

        let status_history = StatusHistory::select(
            &Select::full::<StatusHistory>().eq(StatusHistory::comment, "123"),
            &*pctx.db_pool,
        )
        .await
        .unwrap();

        assert_eq!(2, status_history.len());
        let uuids: AHashSet<_> =
            status_history.into_iter().map(|x| x.object_uuid).collect();
        assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000001")));
        assert!(uuids.contains(&uuid!("00000000-0000-0000-0000-000000000002")));
    })
    .await
}

fn verify_protocol(protocols: &[EcProtocol], id: i64) {
    let p = protocols.iter().find(|p| p.id == id).cloned().unwrap();
    assert!(p.is_removed && p.status_id == EcProtocolStatus::Deleted, "{:#?}", p);
}

fn verify_agenda(agendas: &[EcAgenda], id: i64, status_id: EcAgendaStatus) {
    let p = agendas.iter().find(|p| p.id == id).cloned().unwrap();
    assert!(p.status_id == status_id, "{:#?}", p);
}
