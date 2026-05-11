use asez2_shared_db::{
    db_item::{AsezDate, AsezTimestamp},
    uuid,
};
use dto::estimated_commission::{
    UpdateAgendaHeader, UpdateAgendaItem, UpdateAgendaReqWithUser,
};
use dto::response_request::MessageKind;
use shared_essential::{
    domain::{
        JoinedEcAgendaEcAgendaItem as AgendaWithItems,
        JoinedEcAgendaEcAgendaItemSelector as AgendaWithItemsSelector,
    },
    presentation::dto::{self, response_request::BusinessMessage},
};

use super::*;
use crate::{
    app_process::{self, calls::items_common::ItemsError, UpdateAgendaError},
    common::ProcessingError,
    presentation::business_messages::agenda::AgendaUpdateMessage,
};

const UPDATE_AGENDA_EXTRA_MIGS: &[&str] =
    &["estimated_commission/update_agenda.sql"];

fn agenda_header(uuid: Uuid, id: i64) -> UpdateAgendaHeader {
    UpdateAgendaHeader {
        id,
        uuid,
        meeting_date: Some(Default::default()),
        pricing_organization_unit_id: None,
    }
}

#[tokio::test]
async fn test_update_agenda_success() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");
    let item_uuid3 = uuid!("00000000-0000-0000-0000-000000000003");
    let attachment_uuid = uuid!("00000000-0000-0000-0000-000000000003");
    let attachment_new_uuid = uuid!("00000000-0000-0000-0000-000000000099");
    let partner_uuid = uuid!("00000000-0000-0000-0000-000000000006");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: UpdateAgendaHeader {
            id: 1,
            uuid: agenda_uuid,
            pricing_organization_unit_id: Some(PricingUnitId::D646),
            meeting_date: AsezDate::try_from("01.02.1999").ok(),
        },
        items: vec![
            UpdateAgendaItem {
                uuid: Some(item_uuid1),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                is_excluded: false,
                sum_excluded_vat: Some(1000.into()),
                pricing_sum_excluded_vat: Some(1000.into()),
                is_removed: Some(true),
                reviewed_at: None,
            },
            UpdateAgendaItem {
                uuid: Some(item_uuid3),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                is_excluded: false,
                sum_excluded_vat: Some(100.into()),
                pricing_sum_excluded_vat: Some(100.into()),
                is_removed: Some(false),
                reviewed_at: None,
            },
        ],
        items_d647: vec![UpdateAgendaItem {
            uuid: Some(item_uuid2),
            source_uuid: uuid!("00000000-0000-0000-0000-000000000011"),
            is_excluded: true,
            sum_excluded_vat: Some(100.into()),
            pricing_sum_excluded_vat: Some(100.into()),
            is_removed: Some(false),
            reviewed_at: None,
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
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;
        // This is a precheck that we will check against.
        let fh = FieldChange::select(&Default::default(), pool).await.unwrap();
        assert!(fh.is_empty());

        let s = Select::default().eq(EcAgenda::uuid, agenda_uuid);
        let old_agenda = EcAgenda::select(&s, pool).await.unwrap().pop().unwrap();
        assert_eq!(old_agenda.status_id, EcAgendaStatus::Formed);
        assert_eq!(old_agenda.pricing_organization_unit_id, PricingUnitId::D647);
        assert_eq!(
            old_agenda.meeting_date,
            AsezDate::try_from("2000-01-01").unwrap()
        );

        let s = Select::default()
            .add_replace_order_desc(EcAgendaItem::uuid)
            .in_any(EcAgendaItem::uuid, [item_uuid1, item_uuid2]);
        let old_items = EcAgendaItem::select(&s, pool).await.unwrap();
        assert_eq!(old_items.len(), 2);
        assert_ne!(old_items[0].sum_excluded_vat, Some(1000.into()));
        assert!(!old_items[0].is_removed);
        assert_ne!(old_items[1].sum_excluded_vat, Some(100.into()));

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

        let r_ok = app_process::update_agenda(req, pctx.clone()).await;

        let res = r_ok.unwrap();
        // This is a precheck that we will check against.
        let h = StatusHistory::select(&Default::default(), pool).await.unwrap();
        let fh = FieldChange::select(&Default::default(), pool).await.unwrap();
        assert!(h.is_empty(), "{:?}", h);
        assert!(!fh.is_empty(), "{:?}", fh);

        let s = Select::default().eq(EcAgenda::uuid, agenda_uuid);
        let old_agenda = EcAgenda::select(&s, pool).await.unwrap().pop().unwrap();
        assert_eq!(old_agenda.status_id, EcAgendaStatus::Formed);

        let s = Select::default()
            .add_replace_order_asc(EcAgendaItem::uuid)
            .in_any(EcAgendaItem::uuid, [item_uuid1, item_uuid2]);
        let items = EcAgendaItem::select(&s, pool).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].sum_excluded_vat, Some(1000.into()));
        assert!(items[0].is_removed);
        assert_eq!(items[1].sum_excluded_vat, Some(100.into()));
        assert!(items[1].is_excluded);

        let s = Select::default().eq(EcPartner::uuid, partner_uuid);
        let old_partner = EcPartner::select(&s, pool).await.unwrap().pop().unwrap();
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
        assert_eq!(
            res.messages.messages[0].text,
            "Повестка № 1 на 01.02.1999 сохранена"
        );
        assert_eq!(res.data.agenda.id, Some(1));
        // Должно стать 1, так как один был удален
        assert_eq!(res.data.items.len(), 1);
        // Должен быть один, так как был обновлен один элемент на is_registered_by_d647=true
        assert_eq!(res.data.d647_items.len(), 1);

        assert_eq!(res.data.partner_list.len(), 1);
        assert_eq!(res.data.attachment_list.len(), 2);
    })
    .await
}

/// Тестирование кейса, когда пользователь через update/agenda пытается добавить
/// новый элемент в Повестку
#[tokio::test]
async fn test_update_agenda_with_insert() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000003");

    let req_ok = UpdateAgendaReqWithUser {
        user: 666,
        header: UpdateAgendaHeader {
            id: 3,
            uuid: agenda_uuid,
            meeting_date: Some(Default::default()),
            pricing_organization_unit_id: None,
        },
        items: vec![
            // existing previously removed
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0003-000000000006")),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                is_excluded: true,
                sum_excluded_vat: Some(0.01.into()),
                pricing_sum_excluded_vat: Some(0.01.into()),
                is_removed: None,
                reviewed_at: None,
            },
            // new position
            UpdateAgendaItem {
                uuid: None,
                source_uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                is_excluded: false,
                sum_excluded_vat: Some(0.02.into()),
                pricing_sum_excluded_vat: Some(0.02.into()),
                is_removed: None,
                reviewed_at: None,
            },
            // remove position
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0003-000000000005")),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000014"),
                is_excluded: false,
                sum_excluded_vat: Some(0.00.into()),
                pricing_sum_excluded_vat: Some(0.00.into()),
                is_removed: Some(true),
                reviewed_at: None,
            },
        ],
        items_d647: vec![
            // new position
            UpdateAgendaItem {
                uuid: None,
                source_uuid: uuid!("00000000-0000-0000-0000-000000000013"),
                is_excluded: false,
                sum_excluded_vat: Some(0.03.into()),
                pricing_sum_excluded_vat: Some(0.03.into()),
                is_removed: None,
                reviewed_at: None,
            },
            // existing removed
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0003-000000000008")),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000011"),
                is_excluded: true,
                sum_excluded_vat: Some(0.04.into()),
                pricing_sum_excluded_vat: Some(0.04.into()),
                is_removed: None,
                reviewed_at: None,
            },
            // remove position
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0003-000000000007")),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000015"),
                is_excluded: false,
                sum_excluded_vat: Some(0.00.into()),
                pricing_sum_excluded_vat: Some(0.00.into()),
                is_removed: Some(true),
                reviewed_at: None,
            },
        ],
        partner_list: vec![],
        attachment_list: vec![AttachmentRep {
            object_uuid: Some(uuid!("01111000-0000-0000-0000-000000000001")),
            size: Some(123_314),
            mime_id: Some(35),
            ..Default::default()
        }],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let res_ok =
            app_process::update_agenda(req_ok, pctx.clone()).await.unwrap();

        assert_eq!(res_ok.data.items.len(), 2);
        assert_eq!(res_ok.data.d647_items.len(), 2);

        let AgendaWithItems {
            agenda: _,
            agenda_items,
        } = AgendaWithItemsSelector::new(
            Select::full::<EcAgenda>().eq(EcAgenda::id, 3),
        )
        .get(pool)
        .await
        .unwrap()
        .pop()
        .unwrap();
        assert_eq!(agenda_items.len(), 6);

        let verify_agenda_item = |source_uuid: &str,
                                  number: i64,
                                  sum_excluded_vat: CurrencyValue,
                                  is_registered_by_d647: bool,
                                  is_removed: bool|
         -> bool {
            agenda_items
                .iter()
                .find(|i| i.source_uuid.to_string() == source_uuid)
                .map(|i| {
                    i.agenda_uuid == agenda_uuid
                        && i.sum_excluded_vat.unwrap() == sum_excluded_vat
                        && i.number == number
                        && i.is_registered_by_d647 == is_registered_by_d647
                        && i.is_removed == is_removed
                })
                .unwrap()
        };

        // restored from previously removed position
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000001",
            1,
            0.01.into(),
            false,
            false,
        ));
        // newly added position
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000004",
            2,
            0.02.into(),
            false,
            false,
        ));
        // removed position
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000014",
            0,
            0.into(),
            false,
            true,
        ));

        // newly added position
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000013",
            1,
            0.03.into(),
            true,
            false
        ));
        // restored from previously removed position
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000011",
            2,
            0.04.into(),
            true,
            false,
        ));
        // removed position
        assert!(verify_agenda_item(
            "00000000-0000-0000-0000-000000000015",
            0,
            0.into(),
            true,
            true
        ));
    })
    .await
}

/// Сохраняется порядок позиций повестки, пришедший из FE.
#[tokio::test]
async fn test_update_agenda_reorder() {
    let req_dud = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(uuid!("00000000-0000-0000-0000-000000000004"), 4),
        items: vec![
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0004-000000000001")),
                source_uuid: uuid!("00000000-0000-0000-0004-000000000001"),
                is_excluded: false,
                sum_excluded_vat: Some(1.into()),
                pricing_sum_excluded_vat: Some(1.into()),
                reviewed_at: None,
                is_removed: None,
            },
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0004-000000000002")),
                source_uuid: uuid!("00000000-0000-0000-0004-000000000002"),
                is_excluded: false,
                sum_excluded_vat: Some(2.into()),
                pricing_sum_excluded_vat: Some(2.into()),
                reviewed_at: None,
                is_removed: None,
            },
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0004-000000000003")),
                source_uuid: uuid!("00000000-0000-0000-0004-000000000003"),
                is_excluded: false,
                sum_excluded_vat: Some(3.into()),
                pricing_sum_excluded_vat: Some(3.into()),
                reviewed_at: None,
                is_removed: Some(true),
            },
        ],
        items_d647: vec![
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0004-000000000004")),
                source_uuid: uuid!("00000000-0000-0000-0004-000000000004"),
                is_excluded: false,
                sum_excluded_vat: Some(4.into()),
                pricing_sum_excluded_vat: Some(4.into()),
                reviewed_at: None,
                is_removed: None,
            },
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0004-000000000005")),
                source_uuid: uuid!("00000000-0000-0000-0004-000000000005"),
                is_excluded: false,
                sum_excluded_vat: Some(5.into()),
                pricing_sum_excluded_vat: Some(5.into()),
                reviewed_at: None,
                is_removed: None,
            },
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0004-000000000006")),
                source_uuid: uuid!("00000000-0000-0000-0004-000000000006"),
                is_excluded: false,
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
                is_removed: Some(true),
            },
        ],
        partner_list: vec![],
        attachment_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let r = app_process::update_agenda(req_dud, pctx.clone()).await;
        assert!(r.is_ok(), "should be successfull, got {r:?}");

        let AgendaWithItems {
            agenda: _,
            agenda_items,
        } = AgendaWithItemsSelector::new(
            Select::full::<EcAgenda>().eq(EcAgenda::id, 4),
        )
        .get(pool)
        .await
        .unwrap()
        .pop()
        .unwrap();

        let assert_agenda_item =
            |source_uuid: &str,
             number: i64,
             sum_excluded_vat: CurrencyValue,
             is_registered_by_d647: bool,
             is_excluded: bool,
             is_removed: bool| {
                agenda_items
                    .iter()
                    .find(|i| i.source_uuid.to_string() == source_uuid)
                    .map(|i| {
                        assert_eq!(
                            i.sum_excluded_vat.unwrap(),
                            sum_excluded_vat,
                            "{source_uuid}: sum_excluded_vat"
                        );
                        assert_eq!(i.number, number, "{source_uuid}: number");
                        assert_eq!(
                            i.is_registered_by_d647, is_registered_by_d647,
                            "{source_uuid}: is_registered_by_d647"
                        );
                        assert_eq!(
                            i.is_excluded, is_excluded,
                            "{source_uuid}: is_excluded"
                        );
                        assert_eq!(
                            i.is_removed, is_removed,
                            "{source_uuid}: is_removed"
                        );
                    })
                    .expect("should exist")
            };

        assert_agenda_item(
            "00000000-0000-0000-0004-000000000001",
            1,
            1.into(),
            false,
            false,
            false,
        );
        assert_agenda_item(
            "00000000-0000-0000-0004-000000000002",
            2,
            2.into(),
            false,
            false,
            false,
        );
        assert_agenda_item(
            "00000000-0000-0000-0004-000000000004",
            1,
            4.into(),
            true,
            false,
            false,
        );
        assert_agenda_item(
            "00000000-0000-0000-0004-000000000005",
            2,
            5.into(),
            true,
            false,
            false,
        );
    })
    .await
}

/// Попытка передать Повестку СК без Даты заседания СК
#[tokio::test]
async fn test_missing_meeting_date() {
    let req = UpdateAgendaReqWithUser {
        user: 1,
        header: UpdateAgendaHeader {
            id: 1,
            uuid: uuid!("00000000-0000-0000-0000-000000000009"),
            meeting_date: None,
            pricing_organization_unit_id: None,
        },
        items: vec![],
        items_d647: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let err = app_process::update_agenda(req, pctx.clone()).await.unwrap_err();
        assert!(
            matches!(
                err,
                ProcessingError::UpdateAgenda(
                    UpdateAgendaError::MissingMeetingDate
                )
            ),
            "{:?}",
            err
        );
    })
    .await
}

/// Попытка перевести позицию Повестки с is_excluded=true на is_excluded=false
#[tokio::test]
async fn test_include_check_failure() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000009");
    let item_uuid1 = uuid!("00000000-0000-0000-0009-000000000001");

    let req = UpdateAgendaReqWithUser {
        user: 1,
        header: UpdateAgendaHeader {
            id: 4,
            uuid: agenda_uuid,
            meeting_date: Some(Default::default()),
            pricing_organization_unit_id: None,
        },
        items: vec![UpdateAgendaItem {
            uuid: Some(item_uuid1),
            source_uuid: uuid!("00000000-0000-0000-0009-000000000001"),
            is_removed: None,
            is_excluded: false,
            sum_excluded_vat: None,
            pricing_sum_excluded_vat: None,
            reviewed_at: None,
        }],
        items_d647: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let pool = &*pctx.db_pool;

        let err = app_process::update_agenda(req, pctx.clone()).await.unwrap_err();

        match err {
            ProcessingError::UpdateAgenda(UpdateAgendaError::Messages(
                messages,
            )) => {
                let expected_messages =
                    vec![AgendaUpdateMessage::ExclusionAlreadyInAgenda(
                        &EcAgenda {
                            id: 10,
                            meeting_date: AsezDate::try_from("01.01.2001").unwrap(),
                            ..Default::default()
                        },
                    )
                    .singular(&PlanOrAmendment::Plan(Plan {
                        id: 15,
                        ..Default::default()
                    }))];
                assert_eq!(messages.messages, expected_messages);
            }
            err => panic!("Не та ошибка: {:?}", err),
        }

        // Update the DB since sqlx sometimes derps.
        pool.begin().await.unwrap().commit().await.unwrap();
    })
    .await
}

#[tokio::test]
async fn test_update_not_found_agenda_failure() {
    let req_dud = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(uuid!("00000000-9999-aaaa-bbbb-000000000001"), 123),
        items: vec![],
        items_d647: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let r_err = app_process::update_agenda_inner(req_dud, &pctx).await;
        assert!(matches!(
            r_err,
            Err(ProcessingError::UpdateAgenda(UpdateAgendaError::NoAgenda(123)))
        ));
    })
    .await
}

#[tokio::test]
async fn test_update_agenda_protocol_failure() {
    let agenda_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    let item_uuid2 =
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let item_uuid4 =
        Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 123),
        items: vec![
            UpdateAgendaItem {
                uuid: Some(item_uuid2),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000013"),
                is_excluded: false,
                sum_excluded_vat: Some(0.01.into()),
                pricing_sum_excluded_vat: Some(0.01.into()),
                reviewed_at: None,
                is_removed: None,
            },
            UpdateAgendaItem {
                uuid: Some(item_uuid4),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000012"),
                is_excluded: false,
                sum_excluded_vat: Some(0.00.into()),
                pricing_sum_excluded_vat: Some(0.00.into()),
                reviewed_at: None,
                is_removed: Some(true),
            },
        ],
        items_d647: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let r_err = app_process::update_agenda(req, pctx.clone()).await;
        let err = r_err.unwrap_err();
        match err {
            ProcessingError::UpdateAgenda(UpdateAgendaError::Messages(msg)) => {
                assert_eq!(msg.kind, MessageKind::Error);
                assert_eq!(msg.messages.len(), 1);
                assert_eq!(
                    &msg.messages[0].text,
                    "1 ППЗ/ДС включена(ы) в Протокол № 2 от 01.01.2001. Удаление выполнить невозможно."
                );
            }
            x => panic!("Wrong error: {:?}", x),
        }
    })
    .await
}

#[tokio::test]
async fn test_update_removed_agenda_failure() {
    let agenda_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000008").unwrap();

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 123),
        items: vec![],
        items_d647: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        // Should do nothing, since the agenda in question does not exist.
        let r_err = app_process::update_agenda(req, pctx.clone()).await;
        assert!(matches!(
            r_err,
            Err(ProcessingError::UpdateAgenda(UpdateAgendaError::RemovedAgenda(
                123
            )))
        ));
    })
    .await
}

#[tokio::test]
async fn test_update_partner_with_insert() {
    let agenda_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 2),
        items: vec![UpdateAgendaItem {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000004")),
            source_uuid: uuid!("00000000-0000-0000-0000-000000000012"),
            is_excluded: false,
            sum_excluded_vat: Some(0.01.into()),
            pricing_sum_excluded_vat: Some(0.01.into()),
            is_removed: None,
            reviewed_at: None,
        }],
        items_d647: vec![],
        partner_list: vec![
            EcPartnerRep {
                // Уже существует
                uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000008")
                    .unwrap()
                    .into(),
                user_id: 2.into(),
                e_mail: Some(Some(String::from("hoho@mail.ru"))),
                ..Default::default()
            },
            EcPartnerRep {
                // Уже существует, но is_removed=true, создается новый
                user_id: 666.into(),
                ..Default::default()
            },
            EcPartnerRep {
                // Не существует
                user_id: 777.into(),
                ..Default::default()
            },
        ],
        attachment_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        // Should do nothing, since the agenda in question does not exist.
        let res = app_process::update_agenda(req, pctx.clone()).await.unwrap();

        let partners = res.data.partner_list;

        // Возвращаются только неудаленные 3 партнера
        assert_eq!(partners.len(), 3, "{:#?}", partners);

        let partner_select = Select::full::<EcPartner>()
            .eq(EcPartner::protocol_agenda_uuid, agenda_uuid);
        let expected_partners =
            EcPartner::select(&partner_select, &*pctx.db_pool).await.unwrap();

        assert_eq!(expected_partners.len(), 4);
        // Должно обновить старого
        assert!(expected_partners
            .iter()
            .find(|p| p.user_id == 2)
            .map(|p| p.e_mail.as_ref().unwrap() == "hoho@mail.ru")
            .unwrap());
        // Один удаленный, один новый
        assert_eq!(
            expected_partners.iter().filter(|p| p.user_id == 666).count(),
            2
        );
        // Должен появиться новый
        assert!(expected_partners.iter().any(|p| p.user_id == 777));
    })
    .await
}

/// Новая позиция с признаком ``is_removed = true` не должна добавляться в БД.
#[tokio::test]
async fn test_update_agenda_removed_item() {
    let added_removed_item_uuid = uuid!("00000000-0000-0000-0000-000000000002");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(uuid!("00000000-0000-0000-0000-000000000003"), 3),
        items: vec![
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0003-000000000005")),
                source_uuid: uuid!("00000000-0000-0000-0000-000000000014"),
                is_excluded: false,
                sum_excluded_vat: Some(0.01.into()),
                pricing_sum_excluded_vat: Some(0.02.into()),
                is_removed: Some(false),
                reviewed_at: None,
            },
            UpdateAgendaItem {
                uuid: None,
                source_uuid: added_removed_item_uuid,
                is_excluded: false,
                sum_excluded_vat: Some(0.01.into()),
                pricing_sum_excluded_vat: Some(0.01.into()),
                is_removed: Some(true),
                reviewed_at: None,
            },
        ],
        items_d647: vec![UpdateAgendaItem {
            uuid: Some(uuid!("00000000-0000-0000-0003-000000000007")),
            source_uuid: uuid!("00000000-0000-0000-0000-000000000015"),
            is_excluded: false,
            sum_excluded_vat: Some(0.01.into()),
            pricing_sum_excluded_vat: Some(0.01.into()),
            is_removed: Some(false),
            reviewed_at: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    };
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let _res = app_process::update_agenda(req, pctx.clone()).await.unwrap();

        let AgendaWithItems {
            agenda: _,
            agenda_items,
        } = AgendaWithItemsSelector::new(
            Select::full::<EcAgenda>().eq(EcAgenda::id, 3),
        )
        .get(&*pctx.db_pool)
        .await
        .expect("success")
        .pop()
        .expect("should exist");

        assert!(
            !agenda_items
                .iter()
                .any(|item| item.source_uuid == added_removed_item_uuid),
            "should not add new removed position"
        );
    })
    .await;
}

/// Использование одного и того же ППЗ/ДС в списке и реестре.
#[tokio::test]
async fn test_update_agenda_list_d647_failure() {
    let plan_uuid = uuid!("00000000-0000-0000-0004-000000000006");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(uuid!("00000000-0000-0000-0000-000000000003"), 123),
        items: vec![UpdateAgendaItem {
            uuid: None,
            source_uuid: plan_uuid,
            is_excluded: false,
            sum_excluded_vat: Some(0.01.into()),
            pricing_sum_excluded_vat: Some(0.01.into()),
            is_removed: None,
            reviewed_at: None,
        }],
        items_d647: vec![UpdateAgendaItem {
            uuid: None,
            source_uuid: plan_uuid,
            is_excluded: false,
            sum_excluded_vat: Some(0.01.into()),
            pricing_sum_excluded_vat: Some(0.01.into()),
            is_removed: None,
            reviewed_at: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    };
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        let res = app_process::update_agenda(req, pctx.clone()).await;

        assert!(
            res.is_err(),
            "нельзя добавлять одну и ту же ППЗ/ДС и в список, и в реестр"
        );
    })
    .await;
}

/// Использование одного и того же ППЗ/ДС в списке и реестре.
#[tokio::test]
async fn test_update_agenda_list_d647_duplicate() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000005");
    let plan_uuid = uuid!("00000000-0000-0000-0005-000000000001");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 5),
        items: vec![],
        items_d647: vec![
            UpdateAgendaItem {
                uuid: None,
                source_uuid: plan_uuid,
                is_excluded: false,
                sum_excluded_vat: Some(0.01.into()),
                pricing_sum_excluded_vat: Some(0.01.into()),
                is_removed: Some(false),
                reviewed_at: None,
            },
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0005-000000000001")),
                source_uuid: plan_uuid,
                is_excluded: true,
                sum_excluded_vat: Some(0.01.into()),
                pricing_sum_excluded_vat: Some(0.01.into()),
                is_removed: Some(false),
                reviewed_at: None,
            },
        ],
        attachment_list: vec![],
        partner_list: vec![],
    };
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        // ППЗ включено в список, исключено из реестра
        let res = app_process::update_agenda(req, pctx.clone()).await;
        assert!(
            matches!(
                res,
                Err(ProcessingError::UpdateAgenda(UpdateAgendaError::Items(
                    ItemsError::DupItem(..)
                )))
            ),
            "expected DupItem(..), got {:?}",
            res
        );
    })
    .await;
}

/// Восстановление удаленного ранее ППЗ/ДС, присутствовавшего и в списке и реестре.
#[tokio::test]
async fn test_update_agenda_list_d647_undelete() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000006");
    let plan_uuid = uuid!("00000000-0000-0000-0006-000000000001");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 6),
        items: vec![],
        items_d647: vec![UpdateAgendaItem {
            uuid: None,
            source_uuid: plan_uuid,
            is_excluded: false,
            is_removed: Some(false),
            sum_excluded_vat: Some(6.into()),
            pricing_sum_excluded_vat: Some(6.into()),
            reviewed_at: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    };
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        assert!(app_process::update_agenda(req, pctx.clone()).await.is_ok());

        let agenda_items = fetch_agenda_items(agenda_uuid, &pctx).await;

        assert_eq!(agenda_items.len(), 2);

        // позиция в списке осталась удаленной
        assert!(agenda_items.iter().any(|item| item.uuid
            == uuid!("00000000-0000-0000-0006-000000000001")
            && !item.is_registered_by_d647
            && item.is_removed));

        // позиция в реестре теперь не удалена
        assert!(agenda_items.iter().any(|item| item.uuid
            == uuid!("00000000-0000-0000-0006-000000000002")
            && item.is_registered_by_d647
            && !item.is_removed));
    })
    .await;
}

/// Добавление ППЗ/ДС в список/реестр, которая исключена из реестра/списка
#[tokio::test]
async fn test_update_agenda_add_excluded() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000011");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 11),
        items: vec![
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0011-000000000001")),
                source_uuid: uuid!("00000000-0000-0000-0011-000000000001"),
                is_excluded: true,
                is_removed: None,
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
            UpdateAgendaItem {
                uuid: None,
                source_uuid: uuid!("00000000-0000-0000-0011-000000000002"),
                is_excluded: false,
                is_removed: Some(false),
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
        ],
        items_d647: vec![
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0011-000000000002")),
                source_uuid: uuid!("00000000-0000-0000-0011-000000000002"),
                is_excluded: true,
                is_removed: None,
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
            UpdateAgendaItem {
                uuid: None,
                source_uuid: uuid!("00000000-0000-0000-0011-000000000001"),
                is_excluded: false,
                is_removed: Some(false),
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
        ],
        attachment_list: vec![],
        partner_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        assert!(app_process::update_agenda(req, pctx.clone()).await.is_ok());

        let agenda_items = fetch_agenda_items(agenda_uuid, &pctx).await;

        assert_eq!(agenda_items.len(), 4);

        // позиция в списке осталась исключена
        assert!(agenda_items.iter().any(|item| item.uuid
            == uuid!("00000000-0000-0000-0011-000000000001")
            && !item.is_registered_by_d647
            && item.is_excluded));

        // добавленная позиция в реестр (исключена в списке)
        assert!(agenda_items.iter().any(|item| item.source_uuid
            == uuid!("00000000-0000-0000-0011-000000000001")
            && item.is_registered_by_d647
            && !item.is_removed
            && !item.is_excluded));

        // позиция в реестре исключена
        assert!(agenda_items.iter().any(|item| item.uuid
            == uuid!("00000000-0000-0000-0011-000000000002")
            && item.is_registered_by_d647
            && item.is_excluded));

        // добавленная позиция в список (исключена в реестре)
        assert!(agenda_items.iter().any(|item| item.source_uuid
            == uuid!("00000000-0000-0000-0011-000000000002")
            && !item.is_registered_by_d647
            && !item.is_removed
            && !item.is_excluded));
    })
    .await;
}

/// Добавление ППЗ/ДС в список/реестр, которая удалена из реестра/списка
#[tokio::test]
async fn test_update_agenda_add_removed() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000012");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 12),
        items: vec![
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0012-000000000001")),
                source_uuid: uuid!("00000000-0000-0000-0012-000000000001"),
                is_excluded: false,
                is_removed: Some(true),
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
            UpdateAgendaItem {
                uuid: None,
                source_uuid: uuid!("00000000-0000-0000-0012-000000000002"),
                is_excluded: false,
                is_removed: Some(false),
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
        ],
        items_d647: vec![
            UpdateAgendaItem {
                uuid: Some(uuid!("00000000-0000-0000-0012-000000000002")),
                source_uuid: uuid!("00000000-0000-0000-0012-000000000002"),
                is_excluded: false,
                is_removed: Some(true),
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
            UpdateAgendaItem {
                uuid: None,
                source_uuid: uuid!("00000000-0000-0000-0012-000000000001"),
                is_excluded: false,
                is_removed: Some(false),
                sum_excluded_vat: Some(6.into()),
                pricing_sum_excluded_vat: Some(6.into()),
                reviewed_at: None,
            },
        ],
        attachment_list: vec![],
        partner_list: vec![],
    };
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        assert!(app_process::update_agenda(req, pctx.clone()).await.is_ok());

        let agenda_items = fetch_agenda_items(agenda_uuid, &pctx).await;

        assert_eq!(agenda_items.len(), 4);

        // позиция в списке осталась удалена
        assert!(agenda_items.iter().any(|item| item.uuid
            == uuid!("00000000-0000-0000-0012-000000000001")
            && !item.is_registered_by_d647
            && item.is_removed));

        // добавленная позиция в реестр (удалена в списке)
        assert!(agenda_items.iter().any(|item| item.source_uuid
            == uuid!("00000000-0000-0000-0012-000000000001")
            && item.is_registered_by_d647
            && !item.is_removed
            && !item.is_excluded));

        // позиция в реестре удалена
        assert!(agenda_items.iter().any(|item| item.uuid
            == uuid!("00000000-0000-0000-0012-000000000002")
            && item.is_registered_by_d647
            && item.is_removed));

        // добавленная позиция в список (удалена в реестре)
        assert!(agenda_items.iter().any(|item| item.source_uuid
            == uuid!("00000000-0000-0000-0012-000000000002")
            && !item.is_registered_by_d647
            && !item.is_removed
            && !item.is_excluded));
    })
    .await;
}

#[tokio::test]
async fn test_update_agenda_plan_already_included() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000005");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 5),
        items: vec![UpdateAgendaItem {
            uuid: None,
            source_uuid: uuid!("00000000-0000-0000-0007-000000000001"),
            is_excluded: false,
            is_removed: Some(false),
            sum_excluded_vat: Some(6.into()),
            pricing_sum_excluded_vat: Some(6.into()),
            reviewed_at: None,
        }],
        items_d647: vec![UpdateAgendaItem {
            uuid: None,
            source_uuid: uuid!("00000000-0000-0000-0005-000000000001"),
            is_excluded: false,
            is_removed: Some(false),
            sum_excluded_vat: Some(6.into()),
            pricing_sum_excluded_vat: Some(6.into()),
            reviewed_at: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    };

    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;
        let res = app_process::update_agenda(req, pctx.clone()).await.unwrap_err();

        match res {
            ProcessingError::UpdateAgenda(UpdateAgendaError::Messages(
                messages,
            )) => {
                let expected_messages =
                    vec![AgendaUpdateMessage::AlreadyInAgenda(&EcAgenda {
                        id: 7,
                        meeting_date: AsezDate::try_from("01.01.2001").unwrap(),
                        ..Default::default()
                    })
                    .singular(&PlanOrAmendment::Plan(Plan {
                        id: 13,
                        ..Default::default()
                    }))];
                assert_eq!(messages.messages, expected_messages);
            }
            err => panic!("Не та ошибка: {:?}", err),
        }
    })
    .await;
}

/// Определение наличия изменений в позициях ППЗ/ДС
#[tokio::test]
async fn test_update_agenda_items_changed() {
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        // нет изменений
        assert_agenda_item_changed(pctx.clone(), false, false, |_| {}).await;
        // is_excluded, reviewed_at
        assert_agenda_item_changed(pctx.clone(), true, true, |req| {
            req.items[0].is_excluded = true;
            req.items_d647[0].reviewed_at = Some(AsezTimestamp::now());
        })
        .await;

        // sum, pricing_sum
        assert_agenda_item_changed(pctx, true, true, |req| {
            if let Some(ref mut x) = req.items[0].sum_excluded_vat {
                *x += 100.into();
            }
            if let Some(ref mut x) = req.items_d647[0].pricing_sum_excluded_vat {
                *x += 100.into();
            }
        })
        .await;
    })
    .await;
}

async fn assert_agenda_item_changed(
    proc_ctx: ProcessingCtx,
    item1_changed: bool,
    item2_changed: bool,
    f: impl FnOnce(&mut UpdateAgendaReqWithUser),
) {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000007");
    let item1_uuid = uuid!("00000000-0000-0000-0007-000000000001");
    let item2_uuid = uuid!("00000000-0000-0000-0007-000000000002");

    let mut req = req_from_db(666, agenda_uuid, &proc_ctx).await;

    f(&mut req);

    let ch1_before = item_changed(item1_uuid, &proc_ctx).await;
    let ch2_before = item_changed(item2_uuid, &proc_ctx).await;

    let _res = app_process::update_agenda(req, proc_ctx.clone()).await.unwrap();

    let ch1_after = item_changed(item1_uuid, &proc_ctx).await;
    let ch2_after = item_changed(item2_uuid, &proc_ctx).await;

    assert_eq!(ch1_before != ch1_after, item1_changed);
    assert_eq!(ch2_before != ch2_after, item2_changed);
}

async fn req_from_db(
    user: i32,
    agenda_uuid: Uuid,
    proc_ctx: &ProcessingCtx,
) -> UpdateAgendaReqWithUser {
    let AgendaWithItems {
        agenda,
        agenda_items,
    } = AgendaWithItemsSelector::new(
        Select::full::<EcAgenda>().eq(EcAgenda::uuid, agenda_uuid),
    )
    .get(&*proc_ctx.db_pool)
    .await
    .expect("success")
    .pop()
    .expect("should exist");

    let header = UpdateAgendaHeader {
        id: agenda.id,
        uuid: agenda_uuid,
        meeting_date: Some(agenda.meeting_date),
        pricing_organization_unit_id: Some(agenda.pricing_organization_unit_id),
    };

    let (mut items_d647, mut items): (Vec<_>, Vec<_>) = agenda_items
        .into_iter()
        .map(|item| {
            (
                UpdateAgendaItem {
                    uuid: Some(item.uuid),
                    source_uuid: item.source_uuid,
                    is_excluded: item.is_excluded,
                    sum_excluded_vat: item.sum_excluded_vat,
                    pricing_sum_excluded_vat: item.pricing_sum_excluded_vat,
                    reviewed_at: item.reviewed_at,
                    is_removed: Some(item.is_removed),
                },
                item.is_registered_by_d647,
            )
        })
        .partition(|(_, is_d647)| *is_d647);

    assert_eq!(items.len(), 1, "Повестка {} имеет не один элемент", agenda.id);
    assert_eq!(
        items_d647.len(),
        1,
        "Повестка {} имеет не один элемент д647",
        agenda.id
    );

    UpdateAgendaReqWithUser {
        user,
        header,
        items: vec![items.pop().unwrap().0],
        items_d647: vec![items_d647.pop().unwrap().0],
        partner_list: vec![],
        attachment_list: vec![],
    }
}

async fn item_changed(
    uuid: Uuid,
    proc_ctx: &ProcessingCtx,
) -> (AsezTimestamp, i32) {
    let items = EcAgendaItem::select(
        &Select::with_fields([EcAgendaItem::changed_at, EcAgendaItem::changed_by])
            .eq(EcAgendaItem::uuid, uuid),
        &*proc_ctx.db_pool,
    )
    .await
    .expect("success");
    assert_eq!(items.len(), 1);
    (items[0].changed_at, items[0].changed_by)
}

async fn fetch_agenda_items(
    agenda_uuid: Uuid,
    proc_ctx: &ProcessingCtx,
) -> Vec<EcAgendaItem> {
    let AgendaWithItems {
        agenda: _,
        agenda_items,
    } = AgendaWithItemsSelector::new(
        Select::full::<EcAgenda>().eq(EcAgenda::uuid, agenda_uuid),
    )
    .get(&*proc_ctx.db_pool)
    .await
    .expect("success")
    .pop()
    .expect("should exist");

    agenda_items
}

/// Добавление существующего ППЗ как нового.
#[tokio::test]
async fn test_update_agenda_existing_item_failure() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000005");
    let agenda_item_uuid = uuid!("00000000-0000-0000-0005-000000000001");
    let plan_uuid = uuid!("00000000-0000-0000-0005-000000000001");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 5),
        items: vec![UpdateAgendaItem {
            uuid: None,
            source_uuid: plan_uuid,
            is_excluded: false,
            is_removed: Some(false),
            sum_excluded_vat: Some(0.01.into()),
            pricing_sum_excluded_vat: Some(0.01.into()),
            reviewed_at: None,
        }],
        items_d647: vec![UpdateAgendaItem {
            uuid: Some(agenda_item_uuid),
            source_uuid: plan_uuid,
            is_excluded: false,
            is_removed: Some(false),
            sum_excluded_vat: Some(0.01.into()),
            pricing_sum_excluded_vat: Some(0.01.into()),
            reviewed_at: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    };
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        // ППЗ включено в список, исключено из реестра
        let res = app_process::update_agenda(req, pctx.clone()).await;
        assert!(
            matches!(
                res,
                Err(ProcessingError::UpdateAgenda(UpdateAgendaError::Items(
                    ItemsError::DupItemId(..)
                )))
            ),
            "expected DupItemId(..), got {:?}",
            res
        );
    })
    .await;
}

/// Обновление существующей позиции с другим ППЗ.
#[tokio::test]
async fn test_update_agenda_wrong_source_failure() {
    let agenda_uuid = uuid!("00000000-0000-0000-0000-000000000005");
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid = uuid!("00000000-0000-0000-0005-000000000001");

    let req = UpdateAgendaReqWithUser {
        user: 666,
        header: agenda_header(agenda_uuid, 5),
        items: vec![],
        items_d647: vec![UpdateAgendaItem {
            uuid: Some(item_uuid),
            source_uuid: plan_uuid,
            is_excluded: false,
            sum_excluded_vat: Some(0.01.into()),
            pricing_sum_excluded_vat: Some(0.01.into()),
            is_removed: Some(false),
            reviewed_at: None,
        }],
        attachment_list: vec![],
        partner_list: vec![],
    };
    run_db_test(UPDATE_AGENDA_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool).await;

        // ППЗ включено в список, исключено из реестра
        let res = app_process::update_agenda(req, pctx.clone()).await;
        assert!(
            matches!(
                res,
                Err(ProcessingError::UpdateAgenda(UpdateAgendaError::Items(
                    ItemsError::WrongSource(..)
                )))
            ),
            "expected WrongSource(..), got {:?}",
            res
        );
    })
    .await;
}
