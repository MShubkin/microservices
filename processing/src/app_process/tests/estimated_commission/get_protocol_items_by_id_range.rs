use asez2_shared_db::db_item::AsezDate;
use shared_essential::{
    domain::{PlanOrAmendment, ProtocolType},
    presentation::dto::{
        processing::GetProtocolItemsByIdRangeReq,
        response_request::{BusinessMessage, MessageKind},
    },
};

use super::*;
use crate::{
    app_process::get_protocol_items_by_id_range,
    presentation::business_messages::protocol::ProtocolGetItemsMessage,
};

const GET_PROTOCOL_ITEMS_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_protocol_items_by_id_range.sql"];

/// Тестирование кейса с успешным получением списка ППЗ/ДС, включенных в Протокол по диапазону идентификаторов ППЗ/ДС
/// Заочной СК
#[tokio::test]
async fn test_get_protocol_items_by_id_range_correspondence() {
    run_db_test(GET_PROTOCOL_ITEMS_EXTRA_MIGS, |pool| async move {
        let req_fail_plan_status_commission_kind = GetProtocolItemsByIdRangeReq {
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![vec![1, 2], vec![101, 102]],
            is_registered_by_d647: Default::default(),
            protocol_id: 2,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
        };
        let req_fail_in_protocol = GetProtocolItemsByIdRangeReq {
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![vec![5]],
            is_registered_by_d647: Default::default(),
            protocol_id: 2,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
        };
        let req_success = GetProtocolItemsByIdRangeReq {
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![vec![3, 4], vec![103, 104]],
            protocol_id: 2,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            is_registered_by_d647: Default::default(),
        };

        let res_fail_plan_status_commission_kind = get_protocol_items_by_id_range(
            req_fail_plan_status_commission_kind,
            pool.clone(),
        )
        .await
        .unwrap();
        {
            let res = res_fail_plan_status_commission_kind;
            assert!(res.data.item_list.is_empty());

            assert_eq!(res.messages.kind, MessageKind::Error);

            let invalid_plans = vec![
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
            ];
            let messages =
                vec![ProtocolGetItemsMessage::InvalidCorrespondenceCommissionKind
                    .plural(&invalid_plans)];
            assert_eq!(res.messages.messages, messages);
        }

        let res_fail_in_protocol =
            get_protocol_items_by_id_range(req_fail_in_protocol, pool.clone())
                .await
                .unwrap();
        {
            assert!(res_fail_in_protocol.data.item_list.is_empty());

            assert_eq!(res_fail_in_protocol.messages.kind, MessageKind::Error);

            let messages =
                vec![ProtocolGetItemsMessage::AlreadyInProtocol(&EcProtocol {
                    id: 2,
                    protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Plan(Plan {
                    id: 5,
                    ..Default::default()
                }))];

            assert_eq!(res_fail_in_protocol.messages.messages, messages);
        }

        let res_success = get_protocol_items_by_id_range(req_success, pool.clone())
            .await
            .unwrap();
        {
            let plans = res_success.data.item_list;

            let messages = vec![ProtocolGetItemsMessage::Success(&EcProtocol {
                id: 2,
                protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
                protocol_type_id: ProtocolType::CorrespondenceMeeting,
                ..Default::default()
            })
            .plural(&[
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
            ])];
            assert_eq!(res_success.messages.messages, messages);

            assert_eq!(plans.len(), 4);

            assert_plan(&plans, 3, 0.04.into(), 0.05.into());
            assert_plan(&plans, 4, 0.04.into(), 0.05.into());
            assert_plan(&plans, 103, 0.03.into(), 0.04.into());
            assert_plan(&plans, 104, 0.03.into(), 0.04.into());
        }
    })
    .await;
}

/// Тестирование кейса с успешным получением списка ППЗ/ДС, включенных в Протокол по диапазону идентификаторов ППЗ/ДС
/// Очной СК
#[tokio::test]
async fn test_get_protocol_items_by_id_range_in_person() {
    run_db_test(GET_PROTOCOL_ITEMS_EXTRA_MIGS, |pool| async move {
        let req_fail_commission_kind = GetProtocolItemsByIdRangeReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![vec![3, 4], vec![103, 104]],
            protocol_id: 3,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            is_registered_by_d647: Default::default(),
        };
        let req_fail_in_protocol = GetProtocolItemsByIdRangeReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![vec![105]],
            protocol_id: 3,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            is_registered_by_d647: Default::default(),
        };
        let req_success = GetProtocolItemsByIdRangeReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![vec![1, 2], vec![101, 102]],
            protocol_id: 3,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            is_registered_by_d647: Default::default(),
        };

        let res_fail_commission_kind =
            get_protocol_items_by_id_range(req_fail_commission_kind, pool.clone())
                .await
                .unwrap();
        {
            assert!(res_fail_commission_kind.data.item_list.is_empty());

            assert_eq!(res_fail_commission_kind.messages.kind, MessageKind::Error);

            let messages =
                vec![ProtocolGetItemsMessage::InvalidInPersonCommissionKind
                    .plural(&vec![
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
                    ])];
            assert_eq!(res_fail_commission_kind.messages.messages, messages);
        }

        let res_fail_in_protocol =
            get_protocol_items_by_id_range(req_fail_in_protocol, pool.clone())
                .await
                .unwrap();
        {
            assert!(res_fail_in_protocol.data.item_list.is_empty());

            assert_eq!(res_fail_in_protocol.messages.kind, MessageKind::Error);

            let messages =
                vec![ProtocolGetItemsMessage::AlreadyInProtocol(&EcProtocol {
                    id: 3,
                    protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
                    ..Default::default()
                })
                .singular(&PlanOrAmendment::Amendment(
                    ContractAmendment {
                        id: 105,
                        ..Default::default()
                    },
                ))];

            assert_eq!(res_fail_in_protocol.messages.messages, messages);
        }

        let res_success = get_protocol_items_by_id_range(req_success, pool.clone())
            .await
            .unwrap();
        {
            let plans = res_success.data.item_list;

            let messages = vec![ProtocolGetItemsMessage::Success(&EcProtocol {
                id: 3,
                protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
                protocol_type_id: ProtocolType::InPersonMeeting,
                ..Default::default()
            })
            .plural(&[
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
            ])];
            assert_eq!(res_success.messages.messages, messages);

            assert_eq!(plans.len(), 4);

            assert_plan(&plans, 1, 0.04.into(), 0.05.into());
            assert_plan(&plans, 2, 0.04.into(), 0.05.into());
            assert_plan(&plans, 101, 0.03.into(), 0.04.into());
            assert_plan(&plans, 102, 0.03.into(), 0.04.into());
        }
    })
    .await;
}

fn assert_plan(
    plans: &[GetProtocolItemsByIdRangeItem],
    id: i64,
    sum_excluded_vat: CurrencyValue,
    pricing_sum_excluded_vat: CurrencyValue,
) {
    let item = plans.iter().find(|i| i.plan.plan_id().unwrap() == id).unwrap();

    assert!(
        item.plan.sum_excluded_vat().unwrap() == sum_excluded_vat
            && item.plan.pricing_sum_excluded_vat().unwrap()
                == pricing_sum_excluded_vat
            && item.actual_sum_excluded_vat.unwrap() == pricing_sum_excluded_vat,
        "{:#?}",
        item
    );
}
