use asez2_shared_db::db_item::AsezTimestamp;
use shared_essential::presentation::dto::{
    general::{DataRecords, TaggedValue},
    processing::price_analysis::ImportReq,
    response_request::{EntityKind, MessageKind, Status},
};

use super::*;
use crate::app_process;

const IMPORT_ITEM_LIST_SPECIFIC_MIGS: &[&str] =
    &["estimated_commission/import_item_list_specific.sql"];

#[tokio::test]
async fn import_item_list_specific() {
    //import existing agenda item. is_registered_by_d647 = false
    let import_request1 = ImportReq {
        object_identifier: ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-4444-0000-0000-000000000001").unwrap(),
            EntityKind::Agenda,
        ),
        file_name: "file_name1".to_owned(),
        token: "token1".to_owned(),
        user_id: 0,
        data_records: DataRecords {
            data: vec![vec![
                TaggedValue::String("1".to_owned()),
                TaggedValue::String("1".to_owned()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::CValue(11111.11.into()),
                TaggedValue::CValue(22222.22.into()),
                TaggedValue::CValue(33333.33.into()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::String("19:19".to_owned()),
            ]],
            ..Default::default()
        },
        is_registered_by_d647: Some(false),
    };
    //import existing agenda item. is_registered_by_d647 = true
    let import_request2 = ImportReq {
        object_identifier: ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-4444-0000-0000-000000000001").unwrap(),
            EntityKind::Agenda,
        ),
        file_name: "file_name1".to_owned(),
        token: "token1".to_owned(),
        user_id: 0,
        data_records: DataRecords {
            data: vec![vec![
                TaggedValue::String("1".to_owned()),
                TaggedValue::String("2".to_owned()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::CValue(44444.44.into()),
                TaggedValue::Float(55555.55),
            ]],
            ..Default::default()
        },
        is_registered_by_d647: Some(true),
    };

    //import existing protocol item. is_registered_by_d647 = false
    let import_request3 = ImportReq {
        object_identifier: ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-3333-0000-0000-000000000001").unwrap(),
            EntityKind::Protocol,
        ),
        file_name: "file_name1".to_owned(),
        token: "token1".to_owned(),
        user_id: 0,
        data_records: DataRecords {
            data: vec![vec![
                TaggedValue::String("1".to_owned()),
                TaggedValue::String("1".to_owned()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Int(11111),
                TaggedValue::CValue(22222.22.into()),
                TaggedValue::Float(33333.33),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::String("19:19".to_owned()),
            ]],
            ..Default::default()
        },
        is_registered_by_d647: Some(false),
    };

    //import existing protocol item. is_registered_by_d647 = true
    let import_request4 = ImportReq {
        object_identifier: ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-3333-0000-0000-000000000001").unwrap(),
            EntityKind::Protocol,
        ),
        file_name: "file_name1".to_owned(),
        token: "token1".to_owned(),
        user_id: 0,
        data_records: DataRecords {
            data: vec![vec![
                TaggedValue::String("1".to_owned()),
                TaggedValue::String("2".to_owned()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Float(44444.44),
                TaggedValue::Float(55555.55),
            ]],
            ..Default::default()
        },
        is_registered_by_d647: Some(true),
    };

    //import new agenda item. is_registered_by_d647 = false, Добавление в Повестку запрещено. ППЗ/ДС 3 включена в Протокол 2
    let import_request5 = ImportReq {
        object_identifier: ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-4444-0000-0000-000000000001").unwrap(),
            EntityKind::Agenda,
        ),
        file_name: "file_name1".to_owned(),
        token: "token1".to_owned(),
        user_id: 0,
        data_records: DataRecords {
            data: vec![vec![
                TaggedValue::String("1".to_owned()),
                TaggedValue::String("3".to_owned()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Float(11111.11),
                TaggedValue::Float(22222.22),
                TaggedValue::Float(33333.33),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::String("19:19".to_owned()),
            ]],
            ..Default::default()
        },
        is_registered_by_d647: Some(false),
    };

    //import new agenda item. is_registered_by_d647 = false, Добавление в Повестку разрешено
    let import_request6 = ImportReq {
        object_identifier: ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-4444-0000-0000-000000000001").unwrap(),
            EntityKind::Agenda,
        ),
        file_name: "file_name1".to_owned(),
        token: "token1".to_owned(),
        user_id: 0,
        data_records: DataRecords {
            data: vec![vec![
                TaggedValue::String("1".to_owned()),
                TaggedValue::String("5".to_owned()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Float(11111.11),
                TaggedValue::Float(22222.22),
                TaggedValue::Float(33333.33),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::String("19:19".to_owned()),
            ]],
            ..Default::default()
        },
        is_registered_by_d647: Some(false),
    };

    //import existing protocol item. is_registered_by_d647 = false, ппз status_id=150
    let import_request7 = ImportReq {
        object_identifier: ObjectIdentifier::new_with_type(
            2,
            Uuid::parse_str("00000000-3333-0000-0000-000000000002").unwrap(),
            EntityKind::Protocol,
        ),
        file_name: "file_name1".to_owned(),
        token: "token1".to_owned(),
        user_id: 0,
        data_records: DataRecords {
            data: vec![vec![
                TaggedValue::String("66".to_owned()),
                TaggedValue::String("66".to_owned()),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Int(11111),
                TaggedValue::CValue(22222.22.into()),
                TaggedValue::Float(33333.33),
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::Null,
                TaggedValue::String("19:19".to_owned()),
            ]],
            ..Default::default()
        },
        is_registered_by_d647: Some(false),
    };

    run_db_test(IMPORT_ITEM_LIST_SPECIFIC_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let _pool: &sqlx::Pool<sqlx::Postgres> = &pctx.db_pool;

        let api_response =
            app_process::import_item_list_specific(import_request1, pctx.clone())
                .await
                .unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.messages.kind, MessageKind::Success);
        assert_eq!(api_response.messages.messages.len(), 0);
        match &api_response.data.item_list {
            MergedAgendaOrProtocolItem::AgendaItems(vec) => {
                assert_eq!(vec.len(), 1);
                let filter_v: Vec<MergedAgendaItem> = vec
                    .iter()
                    .filter(|item| item.plan.plan_id().unwrap() == 1)
                    .cloned()
                    .collect();
                assert_eq!(filter_v.len(), 1);
                let agenda_item: &EcAgendaItemRep =
                    &filter_v.get(0).unwrap().agenda_item;
                assert_eq!(
                    agenda_item.sum_excluded_vat,
                    Some(Some(11111.11.into()))
                );
                assert_eq!(
                    agenda_item.pricing_sum_excluded_vat,
                    Some(Some(22222.22.into()))
                );

                let reviewed_at: AsezTimestamp =
                    agenda_item.reviewed_at.unwrap().unwrap();
                assert_eq!(reviewed_at.0.time().hour(), 16);
                assert_eq!(reviewed_at.0.time().minute(), 19);
            }
            MergedAgendaOrProtocolItem::ProtocolItems(_vec) => {
                panic!("Protocol items must not be returned")
            }
        }

        let api_response =
            app_process::import_item_list_specific(import_request2, pctx.clone())
                .await
                .unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.messages.kind, MessageKind::Success);
        assert_eq!(api_response.messages.messages.len(), 0);
        match &api_response.data.item_list {
            MergedAgendaOrProtocolItem::AgendaItems(vec) => {
                assert_eq!(vec.len(), 1);
                let filter_v: Vec<MergedAgendaItem> = vec
                    .iter()
                    .filter(|item| item.plan.plan_id().unwrap() == 2)
                    .cloned()
                    .collect();
                assert_eq!(filter_v.len(), 1);
                let agenda_item: &EcAgendaItemRep =
                    &filter_v.get(0).unwrap().agenda_item;
                assert_eq!(
                    agenda_item.sum_excluded_vat,
                    Some(Some(55555.55.into()))
                );
                assert_eq!(
                    agenda_item.pricing_sum_excluded_vat,
                    Some(Some(44444.44.into()))
                );
            }
            MergedAgendaOrProtocolItem::ProtocolItems(_vec) => {
                panic!("Protocol items must not be returned")
            }
        }

        let api_response =
            app_process::import_item_list_specific(import_request3, pctx.clone())
                .await
                .unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.messages.kind, MessageKind::Success);
        assert_eq!(api_response.messages.messages.len(), 0);
        match &api_response.data.item_list {
            MergedAgendaOrProtocolItem::AgendaItems(_vec) => {
                panic!("Agenda items must not be returned")
            }
            MergedAgendaOrProtocolItem::ProtocolItems(vec) => {
                assert_eq!(vec.len(), 1);
                let filter_v: Vec<ProtocolDetailsItem> = vec
                    .iter()
                    .filter(|item| item.plan.plan_id().unwrap() == 1)
                    .cloned()
                    .collect();
                assert_eq!(filter_v.len(), 1);
                let protocol_item: &Calculated<EcProtocolItemRep> =
                    &filter_v.get(0).unwrap().protocol_item;

                assert_eq!(
                    protocol_item.item.sum_excluded_vat,
                    Some(Some(11111.00.into()))
                );
                assert_eq!(
                    protocol_item.item.pricing_sum_excluded_vat,
                    Some(Some(22222.22.into()))
                );
            }
        }

        let api_response =
            app_process::import_item_list_specific(import_request4, pctx.clone())
                .await
                .unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.messages.kind, MessageKind::Success);
        assert_eq!(api_response.messages.messages.len(), 0);
        match &api_response.data.item_list {
            MergedAgendaOrProtocolItem::AgendaItems(_vec) => {
                panic!("Agenda items must not be returned")
            }
            MergedAgendaOrProtocolItem::ProtocolItems(vec) => {
                assert_eq!(vec.len(), 1);
                let filter_v: Vec<ProtocolDetailsItem> = vec
                    .iter()
                    .filter(|item| item.plan.plan_id().unwrap() == 2)
                    .cloned()
                    .collect();
                assert_eq!(filter_v.len(), 1);
                let protocol_item: &Calculated<EcProtocolItemRep> =
                    &filter_v.get(0).unwrap().protocol_item;

                assert_eq!(
                    protocol_item.item.sum_excluded_vat,
                    Some(Some(55555.55.into()))
                );
                assert_eq!(
                    protocol_item.item.pricing_sum_excluded_vat,
                    Some(Some(44444.44.into()))
                );
            }
        }

        let api_response =
            app_process::import_item_list_specific(import_request5, pctx.clone())
                .await
                .unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.messages.kind, MessageKind::Error);
        assert_eq!(api_response.messages.messages.len(), 1);
        assert!(api_response.messages.messages[0]
            .text
            .contains("Добавление в Повестку запрещено"));

        let api_response =
            app_process::import_item_list_specific(import_request6, pctx.clone())
                .await
                .unwrap();

        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.messages.kind, MessageKind::Success);
        assert_eq!(api_response.messages.messages.len(), 0);
        match &api_response.data.item_list {
            MergedAgendaOrProtocolItem::AgendaItems(vec) => {
                assert_eq!(vec.len(), 1);
                let filter_v: Vec<MergedAgendaItem> = vec
                    .iter()
                    .filter(|item| item.plan.plan_id().unwrap() == 5)
                    .cloned()
                    .collect();
                assert_eq!(filter_v.len(), 1);
                let agenda_item: &EcAgendaItemRep =
                    &filter_v.get(0).unwrap().agenda_item;
                assert_eq!(
                    agenda_item.sum_excluded_vat,
                    Some(Some(11111.11.into()))
                );
                assert_eq!(
                    agenda_item.pricing_sum_excluded_vat,
                    Some(Some(22222.22.into()))
                );

                let reviewed_at: AsezTimestamp =
                    agenda_item.reviewed_at.unwrap().unwrap();
                assert_eq!(reviewed_at.0.time().hour(), 16);
                assert_eq!(reviewed_at.0.time().minute(), 19);
            }
            MergedAgendaOrProtocolItem::ProtocolItems(_vec) => {
                panic!("Protocol items must not be returned")
            }
        }

        let api_response =
            app_process::import_item_list_specific(import_request7, pctx.clone())
                .await
                .unwrap();

        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.messages.kind, MessageKind::Success);
        assert_eq!(api_response.messages.messages.len(), 0);
        match &api_response.data.item_list {
            MergedAgendaOrProtocolItem::AgendaItems(_vec) => {
                panic!("Agenda items must not be returned")
            }
            MergedAgendaOrProtocolItem::ProtocolItems(vec) => {
                assert_eq!(vec.len(), 1);
                let filter_v: Vec<ProtocolDetailsItem> = vec
                    .iter()
                    .filter(|item| item.plan.plan_id().unwrap() == 66)
                    .cloned()
                    .collect();
                assert_eq!(filter_v.len(), 1);
                let protocol_item: &Calculated<EcProtocolItemRep> =
                    &filter_v.get(0).unwrap().protocol_item;
                assert_eq!(protocol_item.item.result_id, Some(ResultId::Cancel));
                assert_eq!(
                    protocol_item.item.sum_excluded_vat,
                    Some(Some(11111.00.into()))
                );
                assert_eq!(
                    protocol_item.item.pricing_sum_excluded_vat,
                    Some(Some(22222.22.into()))
                );
            }
        }
    })
    .await
}
