use crate::processing::protocol_item::ResultId;

use super::*;

const CREATE_TABLE: &str = "(
    uuid uuid NOT NULL PRIMARY KEY,
    protocol_uuid uuid NOT NULL,
    source_uuid uuid NOT NULL,
    number BIGINT NOT NULL,
    is_registered_by_d647 BOOLEAN NOT NULL DEFAULT false,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    is_excluded BOOLEAN NOT NULL DEFAULT false,
    result_id SMALLINT DEFAULT 0,
    sum_excluded_vat BIGINT,
    pricing_sum_excluded_vat BIGINT,
    commission_sum_excluded_vat BIGINT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
  )";

// FIXME: Кажется, что этот тест не актуальный, так как при активации полей теперь не генерируется UUID
//
// #[tokio::test]
// async fn test_conversion() {
//     let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
//     // This structure contains the minimum fields that we should encounter.
//     let rep = EcProtocolItemRep {
//         protocol_uuid: Some(one),
//         source_uuid: Some(one),
//         number: Some(1000002345),
//         is_registered_by_d647: Some(false),
//         created_by: Some(USER1),
//         ..Default::default()
//     };
//     let expected_uninserted = EcProtocolItem {
//         protocol_uuid: one,
//         source_uuid: one,
//         number: 1000002345,
//         is_registered_by_d647: false,
//         created_by: USER1,
//         ..Default::default()
//     };
//     let uninserted = rep.into_item().unwrap();

//     assert_eq!(uninserted, expected_uninserted);

//     let mut activated = uninserted;
//     activated.activate_fields();

//     // If our uuid generator gives us all 0s we have a problem.
//     assert_ne!(
//         activated.uuid,
//         Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
//     );

//     let exp_activated = EcProtocolItem {
//         uuid: activated.uuid,
//         protocol_uuid: one,
//         source_uuid: one,
//         number: 1000002345,
//         is_registered_by_d647: false,
//         is_removed: false,
//         is_excluded: false,
//         result_id: ResultId::Undefined,
//         sum_excluded_vat: None,
//         pricing_sum_excluded_vat: None,
//         commission_sum_excluded_vat: None,
//         created_at: activated.created_at,
//         changed_at: activated.created_at,
//         created_by: USER1,
//         changed_by: USER1,
//     };
//     assert_eq!(activated, exp_activated);
// }

#[tokio::test]
async fn test_insert() {
    run_db_test(EcProtocolItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        // This structure contains the minimum fields that we should encounter.
        let mut rep = EcProtocolItemRep {
            protocol_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let res = rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        let got = sqlx::query("SELECT * FROM protocol_item;")
            .map(|x| {
                EcProtocolItemRep::from_item::<&str>(
                    EcProtocolItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].protocol_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(got[0].is_registered_by_d647, Some(false));
        assert_eq!(got[0].result_id, Some(ResultId::Undefined));
        assert_eq!(got[0].commission_sum_excluded_vat, Some(None));
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_at, got[0].created_at);
        assert!(got[0].created_at.is_some());
    })
    .await
}

#[tokio::test]
async fn test_insert_vec() {
    run_db_test(EcProtocolItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let two = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        // This structure contains the minimum fields that we should encounter.
        let mut rep = EcProtocolItemRep {
            protocol_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        rep.uuid = Uuid::new_v4();

        let mut rep2 = EcProtocolItemRep {
            protocol_uuid: Some(two),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        rep2.uuid = Uuid::new_v4();

        let mut items = vec![rep, rep2];

        let res = EcProtocolItem::insert_vec(&mut items, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM protocol_item;")
            .map(|x| {
                EcProtocolItemRep::from_item::<&str>(
                    EcProtocolItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].protocol_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(got[0].is_registered_by_d647, Some(false));
        assert_eq!(got[0].result_id, Some(ResultId::Undefined));
        assert_eq!(got[0].commission_sum_excluded_vat, Some(None));
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_at, got[0].created_at);
        assert!(got[0].created_at.is_some());

        assert!(got[1].uuid.is_some());
        assert_eq!(got[1].protocol_uuid, Some(two));
        assert_eq!(got[1].number, Some(0));
        assert_eq!(got[1].is_removed, Some(false));
        assert_eq!(got[1].is_registered_by_d647, Some(false));
        assert_eq!(got[1].result_id, Some(ResultId::Undefined));
        assert_eq!(got[1].commission_sum_excluded_vat, Some(None));
        assert_eq!(got[1].created_by, Some(USER1));
        assert_eq!(got[1].changed_at, got[1].created_at);
        assert!(got[1].created_at.is_some());
    })
    .await
}

#[tokio::test]
async fn test_update() {
    run_db_test(EcProtocolItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        // This structure contains the minimum fields that we should encounter.
        let mut rep = EcProtocolItemRep {
            protocol_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let res = rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        let got = sqlx::query("SELECT * FROM protocol_item;")
            .map(|x| {
                EcProtocolItemRep::from_item::<&str>(
                    EcProtocolItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].protocol_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].result_id, Some(ResultId::Undefined));
        assert_eq!(got[0].commission_sum_excluded_vat, Some(None));
        assert_eq!(got[0].created_by, Some(USER1));

        let rep_u = EcProtocolItemRep {
            uuid: got[0].uuid,
            changed_by: Some(USER1),
            result_id: Some(ResultId::Cancel),
            commission_sum_excluded_vat: Some(Some(99.99.into())),
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

        let got = sqlx::query("SELECT * FROM protocol_item;")
            .map(|x| {
                EcProtocolItemRep::from_item::<&str>(
                    EcProtocolItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].protocol_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].result_id, Some(ResultId::Cancel));
        assert_eq!(got[0].commission_sum_excluded_vat, Some(Some(99.99.into())));
        assert_eq!(got[0].created_by, Some(USER1));
    })
    .await
}

#[tokio::test]
async fn test_update_vec() {
    run_db_test(EcProtocolItem::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let one = Uuid::new_v4();
        let two = Uuid::new_v4();
        // This structure contains the minimum fields that we should encounter.
        let mut rep = EcProtocolItemRep {
            protocol_uuid: Some(one),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        rep.uuid = Uuid::new_v4();

        let mut rep2 = EcProtocolItemRep {
            protocol_uuid: Some(two),
            is_registered_by_d647: Some(false),
            created_by: Some(USER1),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        rep2.uuid = Uuid::new_v4();

        let mut items = vec![rep, rep2];

        let res = EcProtocolItem::insert_vec(&mut items, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM protocol_item;")
            .map(|x| {
                EcProtocolItemRep::from_item::<&str>(
                    EcProtocolItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert_eq!(got[0].protocol_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].result_id, Some(ResultId::Undefined));
        assert_eq!(got[0].commission_sum_excluded_vat, Some(None));

        assert_eq!(got[1].protocol_uuid, Some(two));
        assert_eq!(got[1].number, Some(0));
        assert_eq!(got[1].result_id, Some(ResultId::Undefined));
        assert_eq!(got[1].commission_sum_excluded_vat, Some(None));

        let rep_u = EcProtocolItemRep {
            uuid: got[0].uuid,
            changed_by: Some(USER1),
            result_id: Some(ResultId::Cancel),
            commission_sum_excluded_vat: Some(Some(99.99.into())),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let rep_u2 = EcProtocolItemRep {
            uuid: got[1].uuid,
            changed_by: Some(USER1),
            result_id: Some(ResultId::Cancel),
            commission_sum_excluded_vat: Some(Some(999.99.into())),
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
        let reps = vec![rep_u, rep_u2];

        let res =
            EcProtocolItem::update_vec(&reps, Some(&update_fields), &mut pool)
                .await
                .unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM protocol_item;")
            .map(|x| {
                EcProtocolItemRep::from_item::<&str>(
                    EcProtocolItem::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert_eq!(got[0].protocol_uuid, Some(one));
        assert_eq!(got[0].number, Some(0));
        assert_eq!(got[0].result_id, Some(ResultId::Cancel));
        assert_eq!(got[0].commission_sum_excluded_vat, Some(Some(99.99.into())));

        assert_eq!(got[1].protocol_uuid, Some(two));
        assert_eq!(got[1].number, Some(0));
        assert_eq!(got[1].result_id, Some(ResultId::Cancel));
        assert_eq!(got[1].commission_sum_excluded_vat, Some(Some(999.99.into())));
    })
    .await
}
