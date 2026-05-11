//! We cannot currently update this kind of relationship because all we have to
//! work with is (protocol_uuid, agenda_uuid), and the current API does not allow
//! updating of keys.
//!
//! We will have to investigate the best path after discussion.
use super::*;

// Constraint is removed for this test.
const CREATE_TABLE: &str = "(
    agenda_item_uuid uuid NOT NULL,
    agenda_uuid uuid NOT NULL,
    protocol_uuid uuid NOT NULL,
    protocol_item_uuid uuid NOT NULL,
    created_at timestamp without time zone NOT NULL,
    created_by INTEGER NOT NULL,
    CONSTRAINT rels_agenda_protocol_items_pkey PRIMARY KEY (protocol_item_uuid, agenda_item_uuid)
)";

#[tokio::test]
async fn test_conversion() {
    let one = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let two = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    let rep = RelAgendaProtocolItemRep {
        protocol_item_uuid: Some(one),
        agenda_item_uuid: Some(two),
        protocol_uuid: Some(one),
        agenda_uuid: Some(two),
        created_by: Some(USER1),
        ..Default::default()
    };
    let uninserted_exp = RelAgendaProtocolItem {
        protocol_item_uuid: one,
        agenda_item_uuid: two,
        protocol_uuid: one,
        agenda_uuid: two,
        created_by: USER1,
        ..Default::default()
    };
    let uninserted = rep.into_item().unwrap();

    // Test if the experimental NewRelAgendaProtocolItemRep performs as expected.
    assert_eq!(uninserted_exp, uninserted);

    let mut activated = uninserted;
    activated.activate_fields();

    let exp_activated = RelAgendaProtocolItem {
        protocol_item_uuid: one,
        agenda_item_uuid: two,
        created_by: USER1,
        created_at: activated.created_at,
        protocol_uuid: one,
        agenda_uuid: two,
    };
    assert_eq!(activated, exp_activated);
}

#[tokio::test]
async fn test_insert() {
    run_db_test(
        RelAgendaProtocolItem::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let one =
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
            let two =
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

            let new = RelAgendaProtocolItemRep {
                protocol_item_uuid: Some(one),
                agenda_item_uuid: Some(two),
                created_by: Some(USER1),
                ..Default::default()
            };
            let mut rep = new.into_item().unwrap();

            let res = rep.insert(&mut pool).await.unwrap();
            assert_eq!(res, 1);

            let got = sqlx::query("SELECT * FROM item_relation_agenda_protocol;")
                .map(|x| {
                    RelAgendaProtocolItemRep::from_item::<&str>(
                        RelAgendaProtocolItem::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert_eq!(got.len(), 1);

            assert_eq!(got[0].created_by, Some(USER1));
            assert!(got[0].created_at.is_some());
            assert_eq!(got[0].protocol_item_uuid, Some(one));
            assert_eq!(got[0].agenda_item_uuid, Some(two));
        },
    )
    .await
}

#[tokio::test]
async fn test_insert_vec() {
    run_db_test(
        RelAgendaProtocolItem::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let one = Uuid::new_v4();
            let two = Uuid::new_v4();
            let three = Uuid::new_v4();
            let four = Uuid::new_v4();

            let new = RelAgendaProtocolItemRep {
                protocol_item_uuid: Some(one),
                agenda_item_uuid: Some(two),
                created_by: Some(USER1),
                ..Default::default()
            };

            let new2 = RelAgendaProtocolItemRep {
                protocol_item_uuid: Some(three),
                agenda_item_uuid: Some(four),
                created_by: Some(USER1),
                ..Default::default()
            };
            let mut reps =
                vec![new.into_item().unwrap(), new2.into_item().unwrap()];

            let res = RelAgendaProtocolItem::insert_vec(&mut reps, &mut pool)
                .await
                .unwrap();
            assert_eq!(res, 2);

            let got = sqlx::query("SELECT * FROM item_relation_agenda_protocol;")
                .map(|x| {
                    RelAgendaProtocolItemRep::from_item::<&str>(
                        RelAgendaProtocolItem::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert_eq!(got.len(), 2);

            assert_eq!(got[0].created_by, Some(USER1));
            assert!(got[0].created_at.is_some());
            assert_eq!(got[0].protocol_item_uuid, Some(one));
            assert_eq!(got[0].agenda_item_uuid, Some(two));

            assert_eq!(got[1].created_by, Some(USER1));
            assert!(got[1].created_at.is_some());
            assert_eq!(got[1].protocol_item_uuid, Some(three));
            assert_eq!(got[1].agenda_item_uuid, Some(four));
        },
    )
    .await
}
