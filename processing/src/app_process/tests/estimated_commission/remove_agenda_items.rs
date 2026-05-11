use super::*;
use crate::app_process;
use crate::presentation::business_messages::agenda::AgendaRemoveItemsMessage;

use asez2_shared_db::db_item::AsezDate;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind,
};

const PRE_REMOVE_AGENDA_ITEMS_EXTRA_MIGS: &[&str] =
    &["estimated_commission/remove_agenda_items.sql"];

#[tokio::test]
async fn test_pre_remove_agenda_items1() {
    run_db_test(PRE_REMOVE_AGENDA_ITEMS_EXTRA_MIGS, |pool| async move {
        let req_ok = PreRemoveAgendaItemsReq {
            agenda_uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                .unwrap(),
            agenda_id: 2,
            item_list: vec![
                // We take uuids of agenda items not in a protocol and the ids of their plans.
                new_remove_item(
                    4,
                    "00000000-0000-0000-0000-000000000004",
                    Some("00000000-0000-0000-0000-000000000005"),
                    EntityKind::Plan,
                ),
                new_remove_item(
                    5,
                    "00000000-0000-0000-0000-000000000005",
                    Some("00000000-0000-0000-0000-000000000006"),
                    EntityKind::Plan,
                ),
                new_remove_item(
                    6,
                    "00000000-0000-0000-0000-000000000006",
                    Some("00000000-0000-0000-0000-000000000007"),
                    EntityKind::Plan,
                ),
                new_remove_item(
                    7,
                    "00000000-0000-0000-0000-000000000007",
                    None,
                    EntityKind::Plan,
                ),
                new_remove_item(
                    8,
                    "00000000-0000-0000-0000-000000000008",
                    None,
                    EntityKind::Plan,
                ),
                new_remove_item(
                    12,
                    "00000000-0000-0000-0000-000000000012",
                    Some("00000000-0000-0000-0000-000000000008"),
                    EntityKind::ContractAmendment,
                ),
            ],
        };
        let req_oh_no = PreRemoveAgendaItemsReq {
            agenda_uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                .unwrap(),
            agenda_id: 2,
            item_list: vec![
                // All these agenda_items/plans/amendments are attached to a protocol.
                // They should all fail.
                new_remove_item(
                    1,
                    "00000000-0000-0000-0000-000000000001",
                    Some("00000000-0000-0000-0000-000000000001"),
                    EntityKind::Plan,
                ),
                new_remove_item(
                    2,
                    "00000000-0000-0000-0000-000000000002",
                    Some("00000000-0000-0000-0000-000000000003"),
                    EntityKind::Plan,
                ),
                new_remove_item(
                    3,
                    "00000000-0000-0000-0000-000000000003",
                    Some("00000000-0000-0000-0000-000000000004"),
                    EntityKind::Plan,
                ),
                new_remove_item(
                    11,
                    "00000000-0000-0000-0000-000000000011",
                    Some("00000000-0000-0000-0000-000000000002"),
                    EntityKind::ContractAmendment,
                ),
            ],
        };
        let res_oh_no =
            app_process::pre_remove_agenda_items(req_oh_no, pool.clone()).await;
        let res_ok = app_process::pre_remove_agenda_items(req_ok, pool).await;

        {
            let res = res_ok.unwrap();

            assert!(res.messages.is_empty(), "{:#?}", res.messages);
            assert_eq!(res.data.item_list.len(), 6);
        }
        {
            let res = res_oh_no.unwrap();
            assert!(res.data.is_empty());

            let protocol = EcProtocol {
                id: 2,
                protocol_date: AsezDate::try_from("01.01.1910").unwrap(),
                ..Default::default()
            };
            let expected_messages =
                vec![
                    AgendaRemoveItemsMessage::AlreadyInProtocol(&protocol)
                        .singular(&PlanOrAmendment::Plan(Plan {
                            id: 1,
                            ..Default::default()
                        })),
                    AgendaRemoveItemsMessage::AlreadyInProtocol(&protocol)
                        .singular(&PlanOrAmendment::Amendment(ContractAmendment {
                            id: 11,
                            ..Default::default()
                        })),
                    AgendaRemoveItemsMessage::AlreadyInProtocol(&protocol)
                        .singular(&PlanOrAmendment::Plan(Plan {
                            id: 2,
                            ..Default::default()
                        })),
                    AgendaRemoveItemsMessage::AlreadyInProtocol(&protocol)
                        .singular(&PlanOrAmendment::Plan(Plan {
                            id: 3,
                            ..Default::default()
                        })),
                ];

            assert_eq!(res.messages.messages, expected_messages,);
        }
    })
    .await
}

fn new_remove_item(
    id: i64,
    source_uuid: &str,
    uuid: Option<&str>,
    object_type: EntityKind,
) -> PreRemoveAgendaItem {
    PreRemoveAgendaItem {
        id,
        source_uuid: Uuid::parse_str(source_uuid).unwrap(),
        uuid: uuid.map(Uuid::parse_str).transpose().unwrap(),
        object_type,
    }
}
