use super::*;

// Cannot use pkeys in this kind of table creation.
const CREATE_TABLE: &str = "(
    id SMALLINT NOT NULL,
    value char(50) NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
)";
const HOT_WATER: &str = "Горячая вода? Она святая!                         ";
const X: &str = "x                                                 ";
const Y: &str = "y                                                 ";
const NUMBERS: &str = "1234567890                                        ";

#[tokio::test]
async fn test_conversion() {
    let rep = EsCommissionResultRep {
        id: None,
        value: Some("Горячая вода? Она святая!".to_string()),
        created_by: Some(USER1),
        ..Default::default()
    };
    let uninserted_exp = EsCommissionResult {
        value: "Горячая вода? Она святая!".to_string(),
        created_by: USER1,
        ..Default::default()
    };
    let uninserted = rep.into_item().unwrap();

    // Test if the experimental NewEsCommissionResultRep performs as expected.
    assert_eq!(uninserted_exp, uninserted);

    let mut activated = uninserted;
    activated.activate_fields();

    let exp_activated = EsCommissionResult {
        id: activated.id,
        value: "Горячая вода? Она святая!".to_string(),
        created_by: USER1,
        created_at: activated.created_at,
        changed_at: activated.created_at,
        changed_by: USER1,
    };
    assert_eq!(activated, exp_activated);
}

#[tokio::test]
async fn test_insert() {
    run_db_test(
        EsCommissionResult::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let new = EsCommissionResultRep {
                value: Some("Горячая вода? Она святая!".to_string()),
                created_by: Some(USER1),
                ..Default::default()
            };
            let mut agenda_rep = new.into_item().unwrap();

            let res = agenda_rep.insert(&mut pool).await.unwrap();
            assert_eq!(res, 1);

            let got = sqlx::query("SELECT * FROM estimated_commission_result;")
                .map(|x| {
                    EsCommissionResultRep::from_item::<&str>(
                        EsCommissionResult::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert_eq!(got.len(), 1);
            assert!(got[0].id.is_some());
            // 64 chars, not variable.
            assert_eq!(
                got[0].value,
                Some(
                    "Горячая вода? Она святая!                         "
                        .to_string()
                )
            );
            assert_eq!(got[0].created_by, Some(USER1));
            assert_eq!(got[0].changed_by, Some(USER1));
            assert!(got[0].created_at.is_some());
            assert!(got[0].changed_at.is_some());
        },
    )
    .await
}

#[tokio::test]
async fn test_insert_vec() {
    run_db_test(
        EsCommissionResult::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let new = EsCommissionResultRep {
                value: Some("Горячая вода? Она святая!".to_string()),
                created_by: Some(USER1),
                ..Default::default()
            };
            let new2 = EsCommissionResultRep {
                value: Some("1234567890".to_string()),
                created_by: Some(USER1),
                ..Default::default()
            };
            let mut reps =
                vec![new.into_item().unwrap(), new2.into_item().unwrap()];

            let res =
                EsCommissionResult::insert_vec(&mut reps, &mut pool).await.unwrap();
            assert_eq!(res, 2);

            let got = sqlx::query("SELECT * FROM estimated_commission_result;")
                .map(|x| {
                    EsCommissionResultRep::from_item::<&str>(
                        EsCommissionResult::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert_eq!(got.len(), 2);

            assert!(got[0].id.is_some());
            // 64 chars, not variable.
            assert_eq!(got[0].value, Some(HOT_WATER.to_string()));
            assert_eq!(got[0].created_by, Some(USER1));
            assert_eq!(got[0].changed_by, Some(USER1));
            assert!(got[0].created_at.is_some());
            assert!(got[0].changed_at.is_some());

            assert!(got[1].id.is_some());
            // 64 chars, not variable.
            assert_eq!(got[1].value, Some(NUMBERS.to_string()));
            assert_eq!(got[1].created_by, Some(USER1));
            assert_eq!(got[1].changed_by, Some(USER1));
            assert!(got[0].created_at.is_some());
            assert!(got[0].changed_at.is_some());
        },
    )
    .await
}

#[tokio::test]
async fn test_update() {
    run_db_test(
        EsCommissionResult::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let new = EsCommissionResultRep {
                value: Some("Горячая вода? Она святая!".to_string()),
                created_by: Some(USER1),
                ..Default::default()
            };
            let mut rep = new.into_item().unwrap();

            let res = rep.insert(&mut pool).await.unwrap();
            assert_eq!(res, 1);

            let got = sqlx::query("SELECT * FROM estimated_commission_result;")
                .map(|x| {
                    EsCommissionResultRep::from_item::<&str>(
                        EsCommissionResult::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert_eq!(got.len(), 1);

            assert!(got[0].id.is_some());
            // 64 chars, not variable.
            assert_eq!(got[0].value, Some(HOT_WATER.to_string()));
            assert_eq!(got[0].created_by, Some(USER1));
            assert_eq!(got[0].changed_by, Some(USER1));

            // Make update.
            let upd = EsCommissionResultRep {
                // We select what to update by id.
                id: got[0].id,
                value: Some("x".to_string()),
                changed_by: Some(USER2),
                ..Default::default()
            }
            .into_item()
            .unwrap();
            let update_fields = ["value", "changed_by"];
            let res = upd.update(Some(&update_fields), &mut pool).await.unwrap();
            assert_eq!(res, 1);

            // Check content after update.
            let got = sqlx::query("SELECT * FROM estimated_commission_result;")
                .map(|x| {
                    EsCommissionResultRep::from_item::<&str>(
                        EsCommissionResult::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert_eq!(got.len(), 1);
            assert!(got[0].id.is_some());
            // 64 chars, not variable.
            assert_eq!(got[0].value, Some(X.to_string()));
            assert_eq!(got[0].created_by, Some(USER1));
            assert_eq!(got[0].changed_by, Some(USER2));
        },
    )
    .await
}

#[tokio::test]
async fn test_update_vec() {
    use crate::processing::protocol_item::ResultId;
    run_db_test(
        EsCommissionResult::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let new = EsCommissionResultRep {
                id: Some(ResultId::Approved),
                value: Some("Горячая вода? Она святая!".to_string()),
                created_by: Some(USER1),
                ..Default::default()
            };
            let new2 = EsCommissionResultRep {
                id: Some(ResultId::Cancel),
                value: Some("1234567890".to_string()),
                created_by: Some(USER1),
                ..Default::default()
            };
            let mut reps =
                vec![new.into_item().unwrap(), new2.into_item().unwrap()];

            let res =
                EsCommissionResult::insert_vec(&mut reps, &mut pool).await.unwrap();
            assert_eq!(res, 2);

            let got = sqlx::query("SELECT * FROM estimated_commission_result;")
                .map(|x| {
                    EsCommissionResultRep::from_item::<&str>(
                        EsCommissionResult::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert_eq!(got.len(), 2);

            assert!(got[0].id.is_some());
            assert!(got[1].id.is_some());
            // 64 chars, not variable.
            assert_eq!(got[0].value, Some(HOT_WATER.to_string()));
            assert_eq!(got[1].value, Some(NUMBERS.to_string()));
            assert_eq!(got[0].created_by, Some(USER1));
            assert_eq!(got[0].changed_by, Some(USER1));
            assert_eq!(got[1].created_by, Some(USER1));
            assert_eq!(got[1].changed_by, Some(USER1));

            // Make update.
            let upd = EsCommissionResultRep {
                // We select what to update by id.
                id: got[0].id,
                value: Some("x".to_string()),
                changed_by: Some(USER2),
                ..Default::default()
            }
            .into_item()
            .unwrap();
            let upd1 = EsCommissionResultRep {
                // We select what to update by id.
                id: got[1].id,
                value: Some("y".to_string()),
                changed_by: Some(USER2),
                ..Default::default()
            }
            .into_item()
            .unwrap();
            let update_fields = ["value", "changed_by"];
            let res = EsCommissionResult::update_vec(
                &[upd, upd1][..],
                Some(&update_fields),
                &mut pool,
            )
            .await
            .unwrap();
            assert_eq!(res, 2);

            // Check content after update.
            let mut got = sqlx::query("SELECT * FROM estimated_commission_result;")
                .map(|x| {
                    EsCommissionResultRep::from_item::<&str>(
                        EsCommissionResult::from_row(&x).unwrap(),
                        None,
                    )
                })
                .fetch_all(&mut pool)
                .await
                .unwrap();
            got.sort_unstable_by(|a, b| a.value.cmp(&b.value));

            assert_eq!(got.len(), 2);
            assert!(got[0].id.is_some());
            // 64 chars, not variable.
            assert_eq!(got[0].value, Some(X.to_string()));
            assert_eq!(got[0].created_by, Some(USER1));
            assert_eq!(got[0].changed_by, Some(USER2));
            assert!(got[1].id.is_some());
            // 64 chars, not variable.
            assert_eq!(got[1].value, Some(Y.to_string()));
            assert_eq!(got[1].created_by, Some(USER1));
            assert_eq!(got[1].changed_by, Some(USER2));
        },
    )
    .await
}
