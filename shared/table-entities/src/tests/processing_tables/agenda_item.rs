use super::*;

const CREATE_TABLE: &str = "(
    uuid uuid NOT NULL PRIMARY KEY,
    agenda_uuid uuid NOT NULL,
    source_uuid uuid NOT NULL,
    number BIGINT NOT NULL,
    is_registered_by_d647 BOOLEAN NOT NULL DEFAULT false,
    is_excluded BOOLEAN NOT NULL DEFAULT false,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    reviewed_at timestamp without time zone,
    sum_excluded_vat BIGINT,
    pricing_sum_excluded_vat BIGINT,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
  )";

// FIXME: Активации больше нет, удаляем?
//
// #[tokio::test]
// async fn test_conversion() {
//     let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
//     let eleven = Uuid::parse_str("00000000-0000-0000-0000-00000000000A").unwrap();
//     // Agenda items inherit the uuid of their PlanItemLegacy.
//     let uuid = Uuid::new_v4();
//     // This structure contains the minimum fields that we should encounter.
//     let rep = EcAgendaItemRep {
//         uuid: Some(uuid),
//         agenda_uuid: Some(one),
//         source_uuid: Some(eleven),
//         number: Some(1000002345),
//         is_registered_by_d647: Some(false),
//         created_by: Some(USER1),
//         ..Default::default()
//     };
//     let expected_uninserted = EcAgendaItem {
//         uuid,
//         agenda_uuid: one,
//         source_uuid: eleven,
//         number: 1000002345,
//         is_registered_by_d647: false,
//         created_by: USER1,
//         ..Default::default()
//     };
//     let uninserted = rep.into_item().unwrap();

//     assert_eq!(uninserted, expected_uninserted);

//     let mut activated = uninserted;
//     activated.activate_fields();

//     let exp_activated = EcAgendaItem {
//         uuid: activated.uuid,
//         agenda_uuid: one,
//         source_uuid: eleven,
//         number: 1000002345,
//         is_registered_by_d647: false,
//         is_removed: false,
//         is_excluded: false,
//         reviewed_at: Default::default(),
//         pricing_sum_excluded_vat: Default::default(),
//         sum_excluded_vat: Default::default(),
//         created_at: activated.created_at,
//         changed_at: activated.created_at,
//         created_by: USER1,
//         changed_by: 0,
//     };
//     assert_eq!(activated, exp_activated);
// }

#[tokio::test]
async fn test_insert() {
    run_db_test(EcAgendaItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        // This structure contains the minimum fields that we should encounter.
        let mut rep = EcAgendaItemRep {
            agenda_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let res = rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        let got = sqlx::query("SELECT * FROM agenda_item;")
            .map(|x| {
                EcAgendaItemRep::from_item::<&str>(
                    EcAgendaItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].agenda_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(got[0].is_registered_by_d647, Some(false));
        assert_eq!(got[0].created_by, Some(USER1));
        assert!(got[0].created_at.is_some());
    })
    .await
}

#[tokio::test]
async fn test_insert_vec() {
    run_db_test(EcAgendaItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let two = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        // This structure contains the minimum fields that we should encounter.
        let rep = EcAgendaItemRep {
            uuid: Some(Uuid::new_v4()),
            agenda_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let rep2 = EcAgendaItemRep {
            uuid: Some(Uuid::new_v4()),
            agenda_uuid: Some(two),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let mut items = vec![rep, rep2];

        let res = EcAgendaItem::insert_vec(&mut items, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM agenda_item;")
            .map(|x| {
                EcAgendaItemRep::from_item::<&str>(
                    EcAgendaItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].agenda_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(got[0].is_registered_by_d647, Some(false));
        assert_eq!(got[0].created_by, Some(USER1));
        assert!(got[0].created_at.is_some());

        assert!(got[1].uuid.is_some());
        assert_eq!(got[1].agenda_uuid, Some(two));
        assert_eq!(got[1].number, Some(0));
        assert_eq!(got[1].is_removed, Some(false));
        assert_eq!(got[1].is_registered_by_d647, Some(false));
        assert_eq!(got[1].created_by, Some(USER1));
        assert!(got[1].created_at.is_some());
    })
    .await
}

#[tokio::test]
async fn test_insert_vec_returning() {
    run_db_test(EcAgendaItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let two = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        // This structure contains the minimum fields that we should encounter.
        let rep = EcAgendaItemRep {
            uuid: Some(Uuid::new_v4()),
            agenda_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let rep2 = EcAgendaItemRep {
            uuid: Some(Uuid::new_v4()),
            agenda_uuid: Some(two),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let mut items = vec![rep, rep2];

        let res = EcAgendaItem::insert_vec_returning(&mut items, &mut pool)
            .await
            .unwrap();

        let got = sqlx::query("SELECT * FROM agenda_item;")
            .try_map(|x| EcAgendaItem::from_row(&x))
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got, res);

        let got = got
            .into_iter()
            .map(|x| EcAgendaItemRep::from_item::<&str>(x, None))
            .collect::<Vec<_>>();

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].agenda_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(got[0].is_registered_by_d647, Some(false));
        assert_eq!(got[0].created_by, Some(USER1));
        assert!(got[0].created_at.is_some());

        assert!(got[1].uuid.is_some());
        assert_eq!(got[1].agenda_uuid, Some(two));
        assert_eq!(got[1].number, Some(0));
        assert_eq!(got[1].is_removed, Some(false));
        assert_eq!(got[1].is_registered_by_d647, Some(false));
        assert_eq!(got[1].created_by, Some(USER1));
        assert!(got[1].created_at.is_some());
    })
    .await
}

#[tokio::test]
async fn test_update() {
    run_db_test(EcAgendaItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        // This structure contains the minimum fields that we should encounter.
        let mut rep = EcAgendaItemRep {
            agenda_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let res = rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        let got = sqlx::query("SELECT * FROM agenda_item;")
            .map(|x| {
                EcAgendaItemRep::from_item::<&str>(
                    EcAgendaItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].agenda_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].created_by, Some(USER1));

        let rep_u = EcAgendaItemRep {
            uuid: got[0].uuid,
            changed_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let update_fields = [
            "updated_by",
            "result_id",
            "commission_sum_excluded_vat",
            "pricing_sum_excluded_vat",
        ];
        let res = rep_u.update(Some(&update_fields), &mut pool).await.unwrap();
        assert_eq!(res, 1);

        let got = sqlx::query("SELECT * FROM agenda_item;")
            .map(|x| {
                EcAgendaItemRep::from_item::<&str>(
                    EcAgendaItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].agenda_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].created_by, Some(USER1));
    })
    .await
}

#[tokio::test]
async fn test_update_vec() {
    run_db_test(EcAgendaItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let two = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        // This structure contains the minimum fields that we should encounter.
        let rep = EcAgendaItemRep {
            uuid: Some(Uuid::new_v4()),
            agenda_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let rep2 = EcAgendaItemRep {
            uuid: Some(Uuid::new_v4()),
            agenda_uuid: Some(two),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let mut items = vec![rep, rep2];

        let res = EcAgendaItem::insert_vec(&mut items, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM agenda_item;")
            .map(|x| {
                EcAgendaItemRep::from_item::<&str>(
                    EcAgendaItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert_eq!(got[0].agenda_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));

        assert_eq!(got[1].agenda_uuid, Some(two));
        assert_eq!(got[1].number, Some(0));

        let rep_u = EcAgendaItemRep {
            uuid: got[0].uuid,
            changed_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let rep_u2 = EcAgendaItemRep {
            uuid: got[1].uuid,
            changed_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let update_fields = ["updated_by"];
        let reps = vec![rep_u, rep_u2];

        let res = EcAgendaItem::update_vec(&reps, Some(&update_fields), &mut pool)
            .await
            .unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM agenda_item;")
            .map(|x| {
                EcAgendaItemRep::from_item::<&str>(
                    EcAgendaItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert_eq!(got[0].agenda_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));

        assert_eq!(got[1].agenda_uuid, Some(two));
        assert_eq!(got[1].number, Some(0));
    })
    .await
}
