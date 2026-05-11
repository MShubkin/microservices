use asez2_shared_db::{db_item::AsezDate, uuid};
use shared_essential::presentation::dto::estimated_commission::UpdateProtocolReqWithUser;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, MessageKind,
};

use super::*;
use crate::app_process::{
    self, calls::items_common::ItemsError, UpdateProtocolError,
    UpdateProtocolHeader, UpdateProtocolItem, UpdateProtocolReqInner,
};
use crate::presentation::business_messages::protocol::{
    ProtocolGetItemsMessage, ProtocolUpdateMessage,
};

const UPDATE_PROTOCOL_EXTRA_MIGS: &[&str] =
    &["estimated_commission/update_protocol.sql"];

#[tokio::test]
async fn test_update_protocol() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");
    let item_uuid3 = uuid!("00000000-0000-0000-0000-000000000003");
    let attachment_uuid = uuid!("00000000-0000-0000-0000-000000000003");
    let attachment_new_uuid = uuid!("00000000-0000-0000-0000-000000000099");
    let partner_uuid = uuid!("00000000-0000-0000-0000-000000000006");
    let plan_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let plan_uuid2 = uuid!("00000000-0000-0000-0000-000000000011");
    let plan_uuid3 = uuid!("00000000-0000-0000-0000-000000000002");

    let req = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 1,
            uuid: protocol_uuid,
            protocol_type_id: ProtocolType::InPersonMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: true,
        },
        items: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid2),
                source_uuid: plan_uuid2,
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: None,
            },
            UpdateProtocolItem {
                uuid: Some(item_uuid1),
                source_uuid: plan_uuid1,
                is_removed: true,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: Some(ResultId::Cancel),
            },
        ],
        items_d647: vec![UpdateProtocolItem {
            uuid: Some(item_uuid3),
            source_uuid: plan_uuid3,
            is_removed: false,
            is_excluded: false,
            sum_excluded_vat: Some(1000.into()),
            pricing_sum_excluded_vat: None,
            commission_sum_excluded_vat: None,
            result_id: Some(ResultId::Cancel),
        }],
        partner_list: vec![EcPartnerRep {
            uuid: Some(partner_uuid),
            protocol_agenda_uuid: Some(uuid!(
                "00000000-0000-0000-0000-000000000001"
            )),
            user_id: Some(1),
            e_mail: Some(Some("sunday@is.holy".to_string())),
            ..Default::default()
        }],
        attachment_list: vec![
            AttachmentRep {
                uuid: Some(attachment_uuid),
                object_uuid: Some(uuid!("01111000-0000-0000-0000-000000000001")),
                size: Some(123_312),
                mime_id: Some(34),
                ..Default::default()
            },
            AttachmentRep {
                uuid: Some(attachment_new_uuid),
                object_uuid: Some(uuid!("01111000-0000-0000-0000-000000000001")),
                size: Some(123_314),
                mime_id: Some(35),
                ..Default::default()
            },
        ],
    }
    .into();
    let req_dud = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 123,
            uuid: uuid!("00000000-9999-aaaa-bbbb-000000000001"),
            protocol_type_id: ProtocolType::InPersonMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: false,
        },
        items: vec![],
        items_d647: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        // Проверка что ничего не менялось заранее
        let h = StatusHistory::select(
            &Select::default().eq(StatusHistory::object_uuid, protocol_uuid),
            pool,
        )
        .await
        .unwrap();
        let fh = FieldChange::select(&Default::default(), pool).await.unwrap();
        assert_eq!(h.len(), 2);
        assert!(fh.is_empty());

        // Проверка что Протокол еще не был изменен
        let s = Select::default().eq(EcProtocol::uuid, protocol_uuid);
        let old_protocol =
            EcProtocol::select(&s, pool).await.unwrap().pop().unwrap();
        assert_ne!(old_protocol.status_id, EcProtocolStatus::AgreementPending);

        let s = Select::default()
            .add_replace_order_desc(EcProtocolItem::uuid)
            .eq(EcProtocolItem::protocol_uuid, protocol_uuid);
        let old_items = EcProtocolItem::select(&s, pool).await.unwrap();
        assert_eq!(old_items.len(), 3);

        verify_protocol_item(
            &old_items,
            "00000000-0000-0000-0000-000000000001",
            false,
            false,
            0.01.into(),
            1,
            ResultId::Approved,
        );
        verify_protocol_item(
            &old_items,
            "00000000-0000-0000-0000-000000000011",
            false,
            false,
            0.02.into(),
            2,
            ResultId::Approved,
        );
        verify_protocol_item(
            &old_items,
            "00000000-0000-0000-0000-000000000002",
            false,
            true,
            0.03.into(),
            3,
            ResultId::Approved,
        );

        let s = Select::default().eq(EcPartner::uuid, partner_uuid);
        let old_partner = EcPartner::select(&s, pool).await.unwrap().pop().unwrap();
        assert_eq!(old_partner.e_mail, None);
        assert_ne!(old_partner.user_id, 667);

        let s = Select::default().eq(Attachment::uuid, attachment_uuid);
        let old_attachment =
            Attachment::select(&s, pool).await.unwrap().pop().unwrap();
        assert_ne!(old_attachment.size, 123_312);
        assert_ne!(old_attachment.mime_id, 34);

        let s = Select::default().eq(Attachment::uuid, attachment_new_uuid);
        let empty = Attachment::select(&s, pool).await.unwrap();
        assert!(empty.is_empty());

        let r_ok = app_process::update_protocol(req, pctx.clone()).await;

        {
            let res = r_ok.unwrap();
            // This is a precheck tha we will check against.
            let h = StatusHistory::select(&Default::default(), pool).await.unwrap();
            let fh = FieldChange::select(&Default::default(), pool).await.unwrap();
            assert!(!h.is_empty(), "{:?}", h);
            assert!(!fh.is_empty(), "{:?}", fh);

            let s = Select::default().eq(EcProtocol::uuid, protocol_uuid);
            let new_protocol =
                EcProtocol::select(&s, pool).await.unwrap().pop().unwrap();
            assert_eq!(new_protocol.status_id, EcProtocolStatus::Formed);
            assert!(new_protocol.is_secret);

            let s = Select::default()
                .add_replace_order_asc(EcProtocolItem::uuid)
                .eq(EcProtocolItem::protocol_uuid, protocol_uuid);
            let new_items = EcProtocolItem::select(&s, pool).await.unwrap();
            assert_eq!(new_items.len(), 3);

            verify_protocol_item(
                &new_items,
                "00000000-0000-0000-0000-000000000011",
                false,
                false,
                1000.into(),
                1,
                ResultId::Undefined,
            );
            verify_protocol_item(
                &new_items,
                "00000000-0000-0000-0000-000000000001",
                true,
                false,
                1000.into(),
                0,
                ResultId::Cancel,
            );
            verify_protocol_item(
                &new_items,
                "00000000-0000-0000-0000-000000000002",
                false,
                true,
                1000.into(),
                1,
                ResultId::Cancel,
            );

            let s = Select::default().eq(EcPartner::uuid, partner_uuid);
            let old_partner =
                EcPartner::select(&s, pool).await.unwrap().pop().unwrap();
            assert_eq!(old_partner.e_mail, Some("sunday@is.holy".to_string()));
            assert_eq!(old_partner.user_id, 1);

            let s = Select::default().eq(Attachment::uuid, attachment_uuid);
            let old_attachment =
                Attachment::select(&s, pool).await.unwrap().pop().unwrap();
            assert_eq!(old_attachment.size, 123_312);
            assert_eq!(old_attachment.mime_id, 34);

            let s = Select::default().eq(Attachment::uuid, attachment_new_uuid);
            let new_attachments = Attachment::select(&s, pool).await.unwrap();
            assert_eq!(new_attachments.len(), 1);

            assert_eq!(res.messages.messages.len(), 1);
            let messages = vec![ProtocolUpdateMessage::success(&EcProtocol {
                id: 1,
                protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
                ..Default::default()
            })];
            assert_eq!(res.messages.messages, messages);

            // Update the DB since sqlx sometimes derps.
            pool.begin().await.unwrap().commit().await.unwrap();
        }
        // Should do nothing, since the protocol in question does not exist.
        let r_err = app_process::update_protocol(req_dud, pctx.clone()).await;
        {
            let _err = r_err.unwrap_err();
                match _err {
                    ProcessingError::UpdateFail(item, msg) => {
                        assert_eq!(&item, "protocol");
                        assert_eq!(msg.kind, MessageKind::Stop);
                        assert_eq!(msg.messages.len(), 1);
                        assert_eq!(
                            &msg.messages[0].text,
                            "Строки с UUID 00000000-9999-aaaa-bbbb-000000000001 не существует."
                        );
                    }
                    x => panic!("Wrong error: {}", x),
                }
        }
    })
    .await
}

#[tokio::test]
async fn test_upsert_check_failure() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");
    let item_uuid3 = uuid!("00000000-0000-0000-0000-000000000003");

    let new_plan1 = uuid!("00000000-0000-0000-0000-000000000005");
    let new_plan2 = uuid!("00000000-0000-0000-0000-000000000006");

    let req = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 1,
            uuid: protocol_uuid,
            protocol_type_id: ProtocolType::InPersonMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: true,
        },
        items: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid2),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000011"),
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
            UpdateProtocolItem {
                uuid: Some(item_uuid1),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                is_removed: true,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
            // Новый элемент
            UpdateProtocolItem {
                uuid: None,
                source_uuid: new_plan1,
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
        ],
        items_d647: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid3),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
            // Новый элемент
            UpdateProtocolItem {
                uuid: None,
                source_uuid: new_plan2,
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
        ],
        partner_list: vec![],
        attachment_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let res = app_process::update_protocol(req, pctx.clone()).await.unwrap();

        assert_eq!(res.messages.kind, MessageKind::Error);
        let messages = vec![
            ProtocolGetItemsMessage::AlreadyInProtocol(&EcProtocol {
                id: 3,
                protocol_date: AsezDate::try_from("2001-01-01").unwrap(),
                ..Default::default()
            })
            .singular(&PlanOrAmendment::Plan(Plan {
                id: 5,
                ..Default::default()
            })),
            ProtocolGetItemsMessage::InvalidInPersonCommissionKind.singular(
                &PlanOrAmendment::Plan(Plan {
                    id: 6,
                    ..Default::default()
                }),
            ),
        ];
        assert_eq!(res.messages.messages, messages);

        // Update the DB since sqlx sometimes derps.
        pool.begin().await.unwrap().commit().await.unwrap();
    })
    .await
}

/// Попытка перевести позицию Протокола с is_excluded=true на is_excluded=false
#[tokio::test]
async fn test_include_check_failure() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000004");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000006");

    let req = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 4,
            uuid: protocol_uuid,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: true,
        },
        items: vec![UpdateProtocolItem {
            uuid: Some(item_uuid1),
            source_uuid: uuid!("00000000-0000-0000-0000-000000000005"),
            is_removed: false,
            is_excluded: false,
            sum_excluded_vat: Some(1000.into()),
            pricing_sum_excluded_vat: Some(1000.into()),
            commission_sum_excluded_vat: Some(1000.into()),
            result_id: None,
        }],
        items_d647: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let res = app_process::update_protocol(req, pctx.clone()).await.unwrap();

        assert_eq!(res.messages.kind, MessageKind::Error);
        let messages = vec![ProtocolUpdateMessage::ExclusionInvalidCommissionKind
            .singular(&PlanOrAmendment::Plan(Plan {
                id: 5,
                commission_kind_id: CommissionKind::InPerson,
                ..Default::default()
            }))];
        assert_eq!(res.messages.messages, messages);

        // Update the DB since sqlx sometimes derps.
        pool.begin().await.unwrap().commit().await.unwrap();
    })
    .await
}

#[tokio::test]
async fn test_upsert_protocol_items_success_in_person() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");
    let item_uuid3 = uuid!("00000000-0000-0000-0000-000000000003");

    let new_plan1 = uuid!("00000000-0000-0000-0000-000000000003");
    let new_plan2 = uuid!("00000000-0000-0000-0000-000000000004");
    let new_amendment1 = uuid!("00000000-0000-0000-0000-000000000013");

    let partner_uuid = uuid!("00000000-0000-0000-0000-000000000006");

    let req = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 1,
            uuid: protocol_uuid,
            protocol_type_id: ProtocolType::InPersonMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: true,
        },
        items: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid2),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000011"),
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(0.02.into()),
                pricing_sum_excluded_vat: Some(0.02.into()),
                commission_sum_excluded_vat: Some(0.02.into()),
                result_id: Some(ResultId::Approved),
            },
            UpdateProtocolItem {
                uuid: Some(item_uuid1),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                is_removed: true,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
            // Новые элементы
            UpdateProtocolItem {
                uuid: None,
                source_uuid: new_plan1,
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: None,
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: None,
            },
            UpdateProtocolItem {
                uuid: None,
                source_uuid: new_amendment1,
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: None,
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: None,
            },
        ],
        items_d647: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid3),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
            // Новый элемент
            UpdateProtocolItem {
                uuid: None,
                source_uuid: new_plan2,
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
        ],
        partner_list: vec![EcPartnerRep {
            uuid: Some(partner_uuid),
            protocol_agenda_uuid: Some(uuid!(
                "00000000-0000-0000-0000-000000000001"
            )),
            user_id: Some(1),
            e_mail: Some(Some("sunday@is.holy".to_string())),
            ..Default::default()
        }],
        attachment_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let s = Select::default()
            .add_replace_order_desc(EcProtocolItem::uuid)
            .eq(EcProtocolItem::protocol_uuid, protocol_uuid);
        let old_items = EcProtocolItem::select(&s, pool).await.unwrap();
        assert_eq!(old_items.len(), 3);

        verify_protocol_item(
            &old_items,
            "00000000-0000-0000-0000-000000000001",
            false,
            false,
            0.01.into(),
            1,
            ResultId::Approved,
        );
        verify_protocol_item(
            &old_items,
            "00000000-0000-0000-0000-000000000011",
            false,
            false,
            0.02.into(),
            2,
            ResultId::Approved,
        );
        verify_protocol_item(
            &old_items,
            "00000000-0000-0000-0000-000000000002",
            false,
            true,
            0.03.into(),
            3,
            ResultId::Approved,
        );

        let res = app_process::update_protocol(req, pctx.clone()).await.unwrap();

        let protocol = EcProtocol::select(
            &Select::full::<EcProtocol>().eq(EcProtocol::uuid, protocol_uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .unwrap();
        let new_items = EcProtocolItem::select(
            &Select::full::<EcProtocolItem>()
                .add_replace_order_asc(EcProtocolItem::uuid)
                .eq(EcProtocolItem::protocol_uuid, protocol_uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(new_items.len(), 6);

        verify_protocol_item(
            &new_items,
            "00000000-0000-0000-0000-000000000011",
            false,
            false,
            0.02.into(),
            1,
            ResultId::Approved,
        );
        // Значение по суммовым полям должно взяться из позиции Повестки по этому ППЗ/ДС, так как пользователь
        // не заполнил эти поля
        verify_protocol_item(
            &new_items,
            new_plan1.to_string().as_str(),
            false,
            false,
            3000.into(),
            2,
            ResultId::Undefined,
        );
        verify_protocol_item(
            &new_items,
            new_amendment1.to_string().as_str(),
            false,
            false,
            0.01.into(),
            3,
            ResultId::Undefined,
        );
        verify_protocol_item(
            &new_items,
            "00000000-0000-0000-0000-000000000001",
            true,
            false,
            1000.into(),
            0,
            ResultId::Undefined,
        );
        verify_protocol_item(
            &new_items,
            "00000000-0000-0000-0000-000000000002",
            false,
            true,
            1000.into(),
            1,
            ResultId::Undefined,
        );
        verify_protocol_item(
            &new_items,
            new_plan2.to_string().as_str(),
            false,
            true,
            1000.into(),
            2,
            ResultId::Undefined,
        );

        assert_eq!(res.messages.messages.len(), 1);
        let messages = vec![ProtocolUpdateMessage::success(&EcProtocol {
            id: 1,
            protocol_date: AsezDate::try_from("01.01.2000").unwrap(),
            ..Default::default()
        })];
        assert_eq!(res.messages.messages, messages);

        // ППЗ/ДС по новым позициям должны быть обновлены
        let plans = PlanOrAmendment::select(
            &Select::default()
                .in_any(Plan::uuid, [new_plan1, new_plan2, new_amendment1]),
            pool,
        )
        .await
        .unwrap();
        assert!(plans
            .iter()
            .all(|p| *p.commission_kind_id() == CommissionKind::InPerson
                && p.commission_date().unwrap() == protocol.protocol_date));

        // Update the DB since sqlx sometimes derps.
        pool.begin().await.unwrap().commit().await.unwrap();
    })
    .await
}

/// Тестирование конкретно апсерта партнера
#[tokio::test]
async fn test_upsert_partner() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");
    let item_uuid3 = uuid!("00000000-0000-0000-0000-000000000003");

    let partner_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let partner_uuid2 = uuid!("00000000-0000-0000-0000-000000000006");

    let req = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 1,
            uuid: protocol_uuid,
            protocol_type_id: ProtocolType::InPersonMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: true,
        },
        items: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid1),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                is_removed: true,
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                commission_sum_excluded_vat: Some(1000.into()),
                result_id: None,
            },
            UpdateProtocolItem {
                uuid: Some(item_uuid2),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000011"),
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: Some(0.02.into()),
                pricing_sum_excluded_vat: Some(0.02.into()),
                commission_sum_excluded_vat: Some(0.02.into()),
                result_id: Some(ResultId::Approved),
            },
        ],
        items_d647: vec![UpdateProtocolItem {
            uuid: Some(item_uuid3),
            source_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
            is_removed: false,
            is_excluded: false,
            sum_excluded_vat: Some(1000.into()),
            pricing_sum_excluded_vat: Some(1000.into()),
            commission_sum_excluded_vat: Some(1000.into()),
            result_id: None,
        }],
        partner_list: vec![
            EcPartnerRep {
                uuid: Some(partner_uuid1),
                protocol_agenda_uuid: Some(protocol_uuid),
                user_id: Some(1),
                e_mail: Some(Some("sunday@is.holy".to_string())),
                ..Default::default()
            },
            EcPartnerRep {
                uuid: Some(partner_uuid2),
                protocol_agenda_uuid: Some(protocol_uuid),
                user_id: Some(1),
                e_mail: Some(Some("steins@gate".to_string())),
                ..Default::default()
            },
            // Новый
            EcPartnerRep {
                uuid: None,
                protocol_agenda_uuid: Some(protocol_uuid),
                user_id: Some(1),
                e_mail: Some(Some("makise@kurisu".to_string())),
                ..Default::default()
            },
        ],
        attachment_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let s =
            Select::default().eq(EcPartner::protocol_agenda_uuid, protocol_uuid);
        let old_partners = EcPartner::select(&s, pool).await.unwrap();
        assert_eq!(old_partners.len(), 2);

        verify_partner(&old_partners, partner_uuid1, protocol_uuid, None);
        verify_partner(&old_partners, partner_uuid2, protocol_uuid, None);

        let _res = app_process::update_protocol(req, pctx.clone()).await.unwrap();

        let s =
            Select::default().eq(EcPartner::protocol_agenda_uuid, protocol_uuid);
        let new_partners = EcPartner::select(&s, pool).await.unwrap();
        assert_eq!(new_partners.len(), 3);

        verify_partner(
            &new_partners,
            partner_uuid1,
            protocol_uuid,
            Some(String::from("sunday@is.holy")),
        );
        verify_partner(
            &new_partners,
            partner_uuid2,
            protocol_uuid,
            Some(String::from("steins@gate")),
        );
        let new_partner = new_partners
            .iter()
            .find(|i| i.uuid != partner_uuid1 && i.uuid != partner_uuid2)
            .map(|p| p.uuid)
            .unwrap();
        verify_partner(
            &new_partners,
            new_partner,
            protocol_uuid,
            Some(String::from("makise@kurisu")),
        );

        // Update the DB since sqlx sometimes derps.
        pool.begin().await.unwrap().commit().await.unwrap();
    })
    .await
}

/// Удаление одной позиции должно приводить к удалению элемента rel_items_agenda_protocol
/// при этом элемент таблицы agenda_protocol_relations должен остаться, а статус Повестки должен быть обновлен,
/// так как теперь не все элементы Повестки есть в Протоколе
#[tokio::test]
async fn remove_protocol_item_without_agenda_protocol_relation_removal() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2_remove = uuid!("00000000-0000-0000-0000-000000000002");
    let item_d647_uuid3 = uuid!("00000000-0000-0000-0000-000000000003");

    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let req = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 1,
            uuid: protocol_uuid,
            protocol_type_id: ProtocolType::InPersonMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: false,
        },
        items: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid1),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                is_removed: false,
                is_excluded: false,
                sum_excluded_vat: None,
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: None,
            },
            UpdateProtocolItem {
                uuid: Some(item_uuid2_remove),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000011"),
                is_removed: true,
                is_excluded: false,
                sum_excluded_vat: None,
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: None,
            },
        ],
        items_d647: vec![UpdateProtocolItem {
            uuid: Some(item_d647_uuid3),
            source_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
            is_removed: false,
            is_excluded: false,
            sum_excluded_vat: None,
            pricing_sum_excluded_vat: None,
            commission_sum_excluded_vat: None,
            result_id: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let res = app_process::update_protocol(req, pctx.clone()).await;
        assert!(res.is_ok(), "{:#?}", res);

        let item_rels = RelAgendaProtocolItem::select(
            &Select::default()
                .eq(RelAgendaProtocolItem::protocol_uuid, protocol_uuid),
            pool,
        )
        .await
        .unwrap();
        for uuid in [item_uuid1, item_d647_uuid3] {
            assert!(
                item_rels.iter().any(|rel| rel.protocol_item_uuid == uuid),
                "agenda/protocol item relation to protocol_item {} should be kept",
                uuid
            );
        }
        assert!(
            !item_rels
                .iter()
                .any(|rel| rel.protocol_item_uuid == item_uuid2_remove),
            "agenda/protocol item relation to protocol_item {} should be removed",
            item_uuid2_remove
        );

        let agenda_protocl_rels = RelAgendaProtocol::select(
            &Select::default().eq(RelAgendaProtocol::protocol_uuid, protocol_uuid),
            pool,
        )
        .await
        .unwrap();
        assert!(
            !agenda_protocl_rels.is_empty(),
            "agenda/protocol relation should be kept"
        );

        let agenda = EcAgenda::select(
            &Select::default().eq(EcAgenda::uuid, agenda_uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .unwrap();
        assert_eq!(
            agenda.status_id,
            EcAgendaStatus::Formed,
            "Статус Повестки должен стать прошлым из записи status_history"
        );
    })
    .await;
}

/// Удаление всех позиций, относящихся к повестке, должно приводить к удалению элементов
/// rel_items_agenda_protocol и элемента таблицы agenda_protocol_relations, так как ни один из
/// элементов Повестки не относится к Протоколу, и обновлению статуса Повестки
#[tokio::test]
async fn remove_protocol_item_with_agenda_protocol_relation_removal() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1_remove = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2_remove = uuid!("00000000-0000-0000-0000-000000000002");
    let item_d647_uuid3_remove = uuid!("00000000-0000-0000-0000-000000000003");

    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let req = UpdateProtocolReqInner {
        user_id: 666,
        header: UpdateProtocolHeader {
            id: 1,
            uuid: protocol_uuid,
            protocol_type_id: ProtocolType::InPersonMeeting,
            registration_number: None,
            protocol_date: AsezDate::try_from("2000-01-01").unwrap(),
            pricing_organization_unit_id: None,
            is_secret: false,
        },
        items: vec![
            UpdateProtocolItem {
                uuid: Some(item_uuid1_remove),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                is_removed: true,
                is_excluded: false,
                sum_excluded_vat: None,
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: None,
            },
            UpdateProtocolItem {
                uuid: Some(item_uuid2_remove),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000011"),
                is_removed: true,
                is_excluded: false,
                sum_excluded_vat: None,
                pricing_sum_excluded_vat: None,
                commission_sum_excluded_vat: None,
                result_id: None,
            },
        ],
        items_d647: vec![UpdateProtocolItem {
            uuid: Some(item_d647_uuid3_remove),
            source_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
            is_removed: true,
            is_excluded: false,
            sum_excluded_vat: None,
            pricing_sum_excluded_vat: None,
            commission_sum_excluded_vat: None,
            result_id: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let prev_agenda_protocol_rels = RelAgendaProtocol::select(&Select::default().eq(RelAgendaProtocol::protocol_uuid, protocol_uuid), pool).await.unwrap();
        assert!(!prev_agenda_protocol_rels.is_empty(), "agenda/protocol relation should be there");

        let res = app_process::update_protocol(req, pctx.clone()).await;
        assert!(res.is_ok(), "expected {res:?} to be Ok");

        let item_rels = RelAgendaProtocolItem::select(&Select::default().eq(RelAgendaProtocolItem::protocol_uuid, protocol_uuid), pool).await.unwrap();
        for uuid in [item_uuid1_remove, item_uuid2_remove, item_d647_uuid3_remove, ] {
            assert!(!item_rels.iter().any(|rel| rel.protocol_item_uuid == uuid), "agenda/protocol item relation to protocol_item {} should be removed", uuid);
        }

        let agenda_protocol_rels = RelAgendaProtocol::select(&Select::default().eq(RelAgendaProtocol::protocol_uuid, protocol_uuid), pool).await.unwrap();
        assert!(agenda_protocol_rels.is_empty(), "agenda/protocol relation should be removed");

        let agenda = EcAgenda::select(&Select::default().eq(EcAgenda::uuid, agenda_uuid), pool).await.unwrap().pop().unwrap();
        assert_eq!(agenda.status_id, EcAgendaStatus::Formed, "Статус Повестки должен стать прошлым из записи status_history");
    }).await;
}

/// Ошибка при добавлении нового ППЗ при наличии существующей позиции с ним.
#[tokio::test]
async fn missing_protocol_date() {
    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000002");

    let protocol_id = 2;
    let user_id = 666;
    let protocol_type_id = ProtocolType::InPersonMeeting;

    let req = UpdateProtocolReqWithUser {
        user_id,
        header: EcProtocolRep {
            id: Some(protocol_id),
            uuid: Some(protocol_uuid),
            protocol_type_id: Some(protocol_type_id),
            registration_number: None,
            protocol_date: None,
            pricing_organization_unit_id: None,
            is_secret: Some(false),
            ..Default::default()
        },
        items: vec![],
        items_d647: vec![],
        attachment_list: vec![],
        partner_list: vec![],
    };

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let res =
            app_process::update_protocol(req, pctx.clone()).await.unwrap_err();

        assert!(
            matches!(
                res,
                ProcessingError::UpdateProtocol(
                    UpdateProtocolError::MissingProtocolDate
                )
            ),
            "got {:?}",
            res
        );
    })
    .await;
}

/// Ошибка при добавлении нового ППЗ при наличии существующей позиции с ним.
#[tokio::test]
async fn existing_item_failure() {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000012");

    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000002");
    let protocol_item_uuid = uuid!("00000000-0000-0000-0000-000000000004");

    let protocol_id = 2;
    let user_id = 666;
    let protocol_type_id = ProtocolType::InPersonMeeting;

    let req = UpdateProtocolReqInner {
        user_id,
        header: UpdateProtocolHeader {
            id: protocol_id,
            uuid: protocol_uuid,
            protocol_type_id,
            registration_number: None,
            protocol_date: Default::default(),
            pricing_organization_unit_id: None,
            is_secret: false,
        },
        items: vec![UpdateProtocolItem {
            uuid: None,
            source_uuid: plan_uuid,
            is_removed: false,
            is_excluded: false,
            sum_excluded_vat: None,
            pricing_sum_excluded_vat: None,
            commission_sum_excluded_vat: None,
            result_id: None,
        }],
        items_d647: vec![UpdateProtocolItem {
            uuid: Some(protocol_item_uuid),
            source_uuid: plan_uuid,
            is_removed: false,
            is_excluded: false,
            sum_excluded_vat: None,
            pricing_sum_excluded_vat: None,
            commission_sum_excluded_vat: None,
            result_id: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let res = app_process::update_protocol(req, pctx.clone()).await;

        assert!(
            matches!(
                res,
                Err(ProcessingError::UpdateProtocol(UpdateProtocolError::Items(
                    ItemsError::DupItemId(..)
                )))
            ),
            "got {:?}",
            res
        );
    })
    .await;
}

/// Ошибка при обновлении позиции со сменой ППЗ.
#[tokio::test]
async fn wrong_source_failure() {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000011");
    let item_uuid = uuid!("00000000-0000-0000-0000-000000000004");

    let protocol_uuid = uuid!("00000000-0000-0000-0000-000000000002");
    let protocol_id = 2;
    let user_id = 666;
    let protocol_type_id = ProtocolType::InPersonMeeting;

    let req = UpdateProtocolReqInner {
        user_id,
        header: UpdateProtocolHeader {
            id: protocol_id,
            uuid: protocol_uuid,
            protocol_type_id,
            registration_number: None,
            protocol_date: Default::default(),
            pricing_organization_unit_id: None,
            is_secret: false,
        },
        items: vec![],
        items_d647: vec![UpdateProtocolItem {
            uuid: Some(item_uuid),
            source_uuid: plan_uuid,
            is_removed: false,
            is_excluded: false,
            sum_excluded_vat: None,
            pricing_sum_excluded_vat: None,
            commission_sum_excluded_vat: None,
            result_id: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    }
    .into();

    run_db_test(UPDATE_PROTOCOL_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let res = app_process::update_protocol(req, pctx.clone()).await;

        assert!(
            matches!(
                res,
                Err(ProcessingError::UpdateProtocol(UpdateProtocolError::Items(
                    ItemsError::WrongSource(..)
                )))
            ),
            "got {:?}",
            res
        );
    })
    .await;
}

fn verify_protocol_item(
    protocol_items: &[EcProtocolItem],
    source_uuid: &str,
    is_removed: bool,
    is_registered_by_d647: bool,
    sum_excluded_vat: CurrencyValue,
    number: i64,
    result_id: ResultId,
) {
    let protocol_item = protocol_items
        .iter()
        .find(|i| i.source_uuid.to_string() == source_uuid)
        .unwrap();

    assert_eq!(protocol_item.is_removed, is_removed);
    assert_eq!(protocol_item.is_registered_by_d647, is_registered_by_d647);
    assert_eq!(protocol_item.sum_excluded_vat.unwrap(), sum_excluded_vat);
    assert_eq!(protocol_item.number, number);
    assert_eq!(protocol_item.result_id, result_id);
}

fn verify_partner(
    partners: &[EcPartner],
    uuid: Uuid,
    protocol_agenda_uuid: Uuid,
    email: Option<String>,
) {
    let partner = partners.iter().find(|i| i.uuid == uuid).unwrap();

    assert_eq!(partner.protocol_agenda_uuid, protocol_agenda_uuid);
    assert_eq!(partner.e_mail, email);
}
