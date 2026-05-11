use super::*;
use crate::app_process;
use crate::presentation::business_messages::agenda::AgendaSendMessage;

use asez2_shared_db::db_item::AsezDate;
use shared_essential::presentation::dto::general::ObjectIdentifierList;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind,
};

const AGENDA_SEND_EXTRA_MIGS: &[&str] = &["estimated_commission/agenda_send.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn test_pre_agenda_send() {
    run_db_test(AGENDA_SEND_EXTRA_MIGS, |pool| async move {
        let ok = ObjectIdentifierList {
            item_list: vec![ObjectIdentifier::new_with_type(
                1,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                EntityKind::Agenda,
            )],
        };
        let fail = ObjectIdentifierList {
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    5,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000005")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
            ],
        };

        let r1 = app_process::pre_agenda_send(ok, pool.clone()).await;
        let r2 = app_process::pre_agenda_send(fail, pool).await;
        {
            let r1 = r1.unwrap();
            assert!(r1.messages.messages.is_empty(), "{:#?}", r1.messages);
            assert_eq!(r1.data.item_list.len(), 1);
        }
        {
            let r2 = r2.unwrap();
            assert_eq!(r2.messages.messages.len(), 4);

            let messages = vec![
                AgendaSendMessage::InvalidAgendaStatus.singular(&EcAgenda {
                    id: 2,
                    status_id: EcAgendaStatus::Sent,
                    meeting_date: AsezDate::try_from("2024-07-02").unwrap(),
                    ..Default::default()
                }),
                AgendaSendMessage::InvalidAgendaStatus.singular(&EcAgenda {
                    id: 3,
                    status_id: EcAgendaStatus::ProtocolFormed,
                    meeting_date: AsezDate::try_from("2024-07-02").unwrap(),
                    ..Default::default()
                }),
                AgendaSendMessage::InvalidAgendaStatus.singular(&EcAgenda {
                    id: 4,
                    status_id: EcAgendaStatus::Deleted,
                    meeting_date: AsezDate::try_from("2024-07-04").unwrap(),
                    ..Default::default()
                }),
                AgendaSendMessage::EmptyAgenda.singular(&EcAgenda {
                    id: 5,
                    meeting_date: AsezDate::try_from("2024-07-04").unwrap(),
                    ..Default::default()
                }),
            ];

            assert_eq!(r2.messages.messages, messages);
        }
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_agenda_send() {
    run_db_test(AGENDA_SEND_EXTRA_MIGS, |pool| async move {
        let ok = AgendaSendReq {
            user_id: 108,
            item_list: vec![ObjectIdentifier::new_with_type(
                1,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                EntityKind::Agenda,
            )],
        };
        let fail = AgendaSendReq {
            user_id: 108,
            item_list: vec![
                ObjectIdentifier::new_with_type(
                    2,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    3,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
                ObjectIdentifier::new_with_type(
                    4,
                    Uuid::parse_str("00000000-0000-0000-0000-000000000004")
                        .unwrap(),
                    EntityKind::Agenda,
                ),
            ],
        };
        let pctx = super::mock_processing_context(pool).await;

        let r1 = app_process::agenda_send(ok, pctx.clone()).await;
        let r2 = app_process::agenda_send(fail, pctx.clone()).await;
        {
            let r1 = r1.unwrap();
            assert_eq!(
                r1.messages.messages,
                vec![AgendaSendMessage::Success.singular(&EcAgenda {
                    id: 1,
                    ..Default::default()
                })]
            );
        }
        {
            let r2 = r2.unwrap();

            let messages = vec![
                AgendaSendMessage::InvalidAgendaStatus.singular(&EcAgenda {
                    id: 2,
                    status_id: EcAgendaStatus::Sent,
                    meeting_date: AsezDate::try_from("2024-07-02").unwrap(),
                    ..Default::default()
                }),
                AgendaSendMessage::InvalidAgendaStatus.singular(&EcAgenda {
                    id: 3,
                    status_id: EcAgendaStatus::ProtocolFormed,
                    meeting_date: AsezDate::try_from("2024-07-02").unwrap(),
                    ..Default::default()
                }),
                AgendaSendMessage::InvalidAgendaStatus.singular(&EcAgenda {
                    id: 4,
                    status_id: EcAgendaStatus::Deleted,
                    meeting_date: AsezDate::try_from("2024-07-04").unwrap(),
                    ..Default::default()
                }),
            ];

            assert_eq!(r2.messages.messages, messages);
        }
    })
    .await
}
