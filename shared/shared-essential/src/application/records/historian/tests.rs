use std::sync::Arc;

use super::super::*;
use super::*;

use crate::presentation::dto::response_request::{Message, Messages};
use asez2_shared_db::db_item::{AsezDate, DbItemExt, DbUpsert};
use asez2_shared_db::{DbAdaptor, DbItem, Value};

use sqlx::types::time::Date;
use sqlx::types::Json as SqlxJ;
use sqlx::FromRow;
use uuid::Uuid;

const USER3: i32 = 1000034569;

const RECORDS_EXTRA_MIGS: &[&str] = &["records.sql"];

mod hedgehog_history {
    use std::convert::Infallible;

    use super::*;
    use asez2_tables::traits::{HasId, HasPlanStatusId, HasUuid};
    use serde::{Deserialize, Serialize};

    #[derive(DbItem, DbItemExt, DbAdaptor, Debug, Clone, PartialEq)]
    #[adaptor_derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    #[item_table = "hedgehogs"]
    #[item_aggr_insert]
    struct Hedgehog {
        #[item_field_pkey]
        uuid: uuid::Uuid,
        favourite_food: String,
        born_on: AsezDate,
        died_on: Option<AsezDate>,
        changed_by: i32,
    }
    impl DbUpsert for Hedgehog {}

    // following 3 impls are just dummy ones

    impl HasId for Hedgehog {
        fn id(&self) -> i64 {
            0
        }

        fn set_id(&mut self, _: i64) {}
    }

    impl HasUuid for Hedgehog {
        fn uuid(&self) -> Uuid {
            Default::default()
        }

        fn set_uuid(&mut self, _: Uuid) {}
    }

    impl HasPlanStatusId for Hedgehog {
        fn plan_status(&self) -> asez2_tables::PlanStatus {
            Default::default()
        }

        fn set_plan_status(&mut self, _: asez2_tables::PlanStatus) {}
    }

    impl RulesLawyer for Hedgehog {}

    // The default function is fine.
    impl ProcessUpsert for Hedgehog {
        const CTX_UPDATE_FIELDS: &'static [&'static str] = &[Hedgehog::changed_by];

        fn apply_update_ctx(&mut self, ctx: &UpdateCtx) {
            self.changed_by = ctx.user_id;
        }

        fn apply_insert_ctx(&mut self, ctx: &UpdateCtx) {
            self.changed_by = ctx.user_id;
        }

        fn generate_uuid_if_needed(&mut self) {
            if self.uuid.is_nil() {
                self.uuid = Uuid::new_v4();
            }
        }
    }

    struct Dummy;

    #[async_trait::async_trait]
    impl StatusHandler for Dummy {
        type Error = Infallible;

        async fn check_insert<T: RulesLawyer>(
            &self,
            _new: &[T],
            _messages: &mut Messages,
        ) -> std::result::Result<bool, Self::Error> {
            unimplemented!()
        }

        async fn check_update<T: RulesLawyer>(
            &self,
            _fields_to_update: &[&str],
            _new: &[T],
            _old: &[T],
            messages: &mut Messages,
        ) -> std::result::Result<bool, Self::Error> {
            messages.add_prepared_message(Message::info(
                "I checked the hedgehogs very carefully.".to_string(),
            ));
            Ok(true)
        }

        async fn check_upsert<T: RulesLawyer>(
            &self,
            _fields_to_update: &[&str],
            _new: &[T],
            _old: &[T],
            _messages: &mut Messages,
        ) -> std::result::Result<bool, Self::Error> {
            unimplemented!()
        }
    }

    const TRANSIENT_TABLES: &[&str] = &[Hedgehog::TABLE];

    async fn run_db_test<F, FutFn>(
        extra_migs_files: &'static [&'static str],
        run: FutFn,
    ) where
        F: futures::Future<Output = ()>,
        FutFn: FnOnce(Arc<PgPool>) -> F + 'static,
    {
        testing::BaseMigPath::Other("../../processing/migrations")
            .run_test_with_migrations(
                "src/application/records/tests/extra_migrations",
                extra_migs_files,
                TRANSIENT_TABLES,
                run,
            )
            .await
    }

    fn original_hedgehogs() -> Vec<Hedgehog> {
        vec![
            Hedgehog {
                uuid: Uuid::parse_str("12345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Caterpillars".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 1, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
            Hedgehog {
                uuid: Uuid::parse_str("22345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Apples".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 2, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
            Hedgehog {
                uuid: Uuid::parse_str("32345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Snakes".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 3, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
            Hedgehog {
                uuid: Uuid::parse_str("42345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Engine oil".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 4, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
        ]
    }

    fn new_hedgehogs(changed_by: i32) -> Vec<Hedgehog> {
        vec![
            Hedgehog {
                uuid: Uuid::parse_str("12345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Caterpillars".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 1, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
            Hedgehog {
                uuid: Uuid::parse_str("22345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Caterpillars".to_string(), // changed
                born_on: AsezDate(Date::try_from_ymd(2000, 2, 1).unwrap()),
                died_on: None,
                changed_by,
            },
            Hedgehog {
                uuid: Uuid::parse_str("32345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Bananas".to_string(), // changed
                born_on: AsezDate(Date::try_from_ymd(2000, 3, 1).unwrap()),
                died_on: None,
                changed_by,
            },
            Hedgehog {
                uuid: Uuid::parse_str("42345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Engine oil".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 4, 1).unwrap()),
                // changed.
                died_on: Some(AsezDate(Date::try_from_ymd(2001, 4, 1).unwrap())),
                changed_by,
            },
        ]
    }

    #[test]
    fn test_hedgehog_crosscheck_same() {
        let old_hogs = original_hedgehogs();
        let new_hogs = old_hogs.clone();

        let CrossCheckResult {
            changes,
            new_uuids: orphans,
            ..
        } = crosscheck(old_hogs, new_hogs, Hedgehog::FIELDS, &UpdateCtx::new(0));
        assert!(changes.is_empty());
        assert!(orphans.is_empty());
    }

    #[test]
    fn test_hedgehog_crosscheck1() {
        let old_hogs = original_hedgehogs();
        let new_hogs = new_hedgehogs(0);

        assert_ne!(new_hogs, old_hogs);

        let CrossCheckResult {
            changes,
            new_uuids: orphans,
            ..
        } = crosscheck(old_hogs, new_hogs, Hedgehog::FIELDS, &UpdateCtx::new(0));
        assert!(orphans.is_empty());
        assert_eq!(changes.len(), 3, "{:#?}", changes);
        assert_eq!(
            changes,
            vec![
                NewFieldChange {
                    record_uuid: Uuid::parse_str(
                        "22345678-1234-1234-1234-123412341234"
                    )
                    .unwrap(),
                    table_name: "hedgehogs",
                    field_name: "favourite_food",
                    field_value: Some(Value::from("Caterpillars")),
                },
                NewFieldChange {
                    record_uuid: Uuid::parse_str(
                        "32345678-1234-1234-1234-123412341234"
                    )
                    .unwrap(),
                    table_name: "hedgehogs",
                    field_name: "favourite_food",
                    field_value: Some(Value::from("Bananas")),
                },
                NewFieldChange {
                    record_uuid: Uuid::parse_str(
                        "42345678-1234-1234-1234-123412341234"
                    )
                    .unwrap(),
                    table_name: "hedgehogs",
                    field_name: "died_on",
                    field_value: Some(Value::from(AsezDate(
                        Date::try_from_ymd(2001, 4, 1).unwrap()
                    ),)),
                },
            ]
        );
    }

    #[test]
    fn test_hedgehog_crosscheck2() {
        let old_hogs = vec![];
        let new_hogs = vec![
            Hedgehog {
                uuid: Uuid::parse_str("12345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Caterpillars".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 1, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
            Hedgehog {
                uuid: Uuid::parse_str("22345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Caterpillars".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 2, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
            Hedgehog {
                uuid: Uuid::parse_str("32345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Bananas".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 3, 1).unwrap()),
                died_on: None,
                changed_by: 0,
            },
            Hedgehog {
                uuid: Uuid::parse_str("42345678-1234-1234-1234-123412341234")
                    .unwrap(),
                favourite_food: "Engine Oil".to_string(),
                born_on: AsezDate(Date::try_from_ymd(2000, 4, 1).unwrap()),
                died_on: Some(AsezDate(Date::try_from_ymd(2001, 4, 1).unwrap())),
                changed_by: 0,
            },
        ];
        let CrossCheckResult {
            changes,
            new_uuids: orphans,
            ..
        } = crosscheck(old_hogs, new_hogs, Hedgehog::FIELDS, &UpdateCtx::new(0));
        // All of them have changed.
        assert_eq!(orphans.len(), 4);
        assert_eq!(changes.len(), 16, "{changes:#?}");
    }

    #[tokio::test]
    async fn test_insert_hedgehog_histories() {
        run_db_test(RECORDS_EXTRA_MIGS, |pool| async move {
            let old_hogs = original_hedgehogs();
            let mut t = pool.begin().await.unwrap();
            let mut historian = Historian::new(
                old_hogs.clone(),
                Hedgehog::FIELDS,
                HistorianMode::Insert,
            );

            historian
                .pre_update(
                    &mut Default::default(),
                    &mut t,
                    &pool,
                    &Default::default(),
                    &UpdateCtx::new(USER3),
                )
                .await
                .expect("pre_update");

            assert_eq!(historian.mode, HistorianMode::Insert);
            assert_eq!(historian.field_changes.len(), 16);

            let inserted_hogs =
                sqlx::query("select * from hedgehogs ORDER BY uuid ASC;")
                    .map(|x| Hedgehog::from_row(&x).unwrap())
                    .fetch_all(&*pool)
                    .await
                    .unwrap();

            // No update until historian is complete.
            assert!(inserted_hogs.is_empty());

            let mut inserted_histories =
                sqlx::query("select * from field_history ORDER BY created_at ASC;")
                    .map(|x| FieldChange::from_row(&x).unwrap())
                    .fetch_all(&*pool)
                    .await
                    .unwrap();

            assert_eq!(inserted_histories.len(), historian.field_changes.len());

            for (i, (a, b)) in inserted_histories
                .iter_mut()
                .zip(historian.field_changes.iter())
                .enumerate()
            {
                // timestamps will not match: Check this.
                a.created_at = b.created_at;
                assert_eq!(a, b, "{}-th", i);
            }
            t.commit().await.unwrap();
        })
        .await
    }

    #[tokio::test]
    async fn test_update_hedgehog_histories() {
        run_db_test(RECORDS_EXTRA_MIGS, |pool| async move {
            let mut elder_hogs = original_hedgehogs();
            let old_hogs = new_hedgehogs(0);

            assert_ne!(elder_hogs, old_hogs);

            // Initial setup of hogs.
            let mut t = pool.begin().await.unwrap();
            Hedgehog::insert_vec(&mut elder_hogs, &mut t)
                .await
                .expect("Could not insert hogs.");
            t.commit().await.unwrap();
            let mut t = pool.begin().await.unwrap();
            let ctx = UpdateCtx::new(USER3);
            // Update the older hogs.
            let mut historian = Historian::new(
                old_hogs.clone(),
                Hedgehog::FIELDS,
                HistorianMode::Update,
            );

            historian
                .pre_update(
                    &mut Default::default(),
                    &mut t,
                    &pool,
                    &Default::default(),
                    &ctx,
                )
                .await
                .expect("Historian failed on formation.");

            assert_eq!(historian.mode, HistorianMode::Update);

            assert_eq!(
                historian.field_changes,
                vec![
                    FieldChange {
                        id: historian.field_changes[0].id,
                        record_uuid: Uuid::parse_str(
                            "22345678-1234-1234-1234-123412341234"
                        )
                        .unwrap(),
                        table_name: "hedgehogs".to_string(),
                        field_name: "favourite_food".to_string(),
                        field_value: Some(SqlxJ(Value::from("Caterpillars"))),
                        record_status: HistoryStatus::Proposed,
                        created_by: ctx.user_id,
                        created_at: ctx.timestamp,
                    },
                    FieldChange {
                        id: historian.field_changes[1].id,
                        record_uuid: Uuid::parse_str(
                            "32345678-1234-1234-1234-123412341234"
                        )
                        .unwrap(),
                        table_name: "hedgehogs".to_string(),
                        field_name: "favourite_food".to_string(),
                        field_value: Some(SqlxJ(Value::from("Bananas"))),
                        record_status: HistoryStatus::Proposed,
                        created_by: ctx.user_id,
                        created_at: ctx.timestamp,
                    },
                    FieldChange {
                        id: historian.field_changes[2].id,
                        record_uuid: Uuid::parse_str(
                            "42345678-1234-1234-1234-123412341234"
                        )
                        .unwrap(),
                        table_name: "hedgehogs".to_string(),
                        field_name: "died_on".to_string(),
                        field_value: Some(SqlxJ(Value::from(AsezDate(
                            Date::try_from_ymd(2001, 4, 1).unwrap()
                        )))),
                        record_status: HistoryStatus::Proposed,
                        created_by: ctx.user_id,
                        created_at: ctx.timestamp,
                    },
                ]
            );

            let updated_hogs =
                sqlx::query("select * from hedgehogs ORDER BY uuid ASC;")
                    .map(|x| Hedgehog::from_row(&x).unwrap())
                    .fetch_all(&*pool)
                    .await
                    .unwrap();

            // No update until historian is complete. The historian carries new data,
            // but does not write it to the DB until the process is done.
            assert_eq!(updated_hogs, elder_hogs);

            let mut inserted_histories =
                sqlx::query("select * from field_history ORDER BY created_at ASC;")
                    .map(|x| FieldChange::from_row(&x).unwrap())
                    .fetch_all(&*pool)
                    .await
                    .unwrap();

            for (i, (a, b)) in inserted_histories
                .iter_mut()
                .zip(historian.field_changes.iter())
                .enumerate()
            {
                // timestamps will not match: Check this.
                a.created_at = b.created_at;
                assert_eq!(a, b, "{}-th", i);
            }
            t.commit().await.unwrap();
        })
        .await
    }

    #[tokio::test]
    async fn test_process_update() {
        run_db_test(RECORDS_EXTRA_MIGS, |pool| async move {
            let mut elder_hogs = original_hedgehogs();
            let old_hogs = new_hedgehogs(USER3);

            assert_ne!(elder_hogs, old_hogs);

            // Initial setup of hogs.
            let mut t = pool.begin().await.unwrap();
            Hedgehog::insert_vec(&mut elder_hogs, &mut t)
                .await
                .expect("Could not insert hogs.");
            t.commit().await.unwrap();

            let mut r = RecordCtx::new(USER3, pool.clone()).begin().await.expect("recorder");
            let ctx = r.ctx();
            // THIS IS WHERE THE WHOLE UPDATE PROCESS WITH HISTORY INSERTION
            // TAKES PLACE.
            let mut messages = Messages::default();
            r.process_update(
                old_hogs.clone(),
                Hedgehog::FIELDS,
                &mut messages,
            )
            .await
            .expect("ProcessUpsert for Hedgehog could not run.");
            r.commit().await.unwrap();

            // Here we check whether the updated hogs are what we expect.
            let updated_hogs =
                sqlx::query("select * from hedgehogs ORDER BY uuid ASC;")
                    .map(|x| Hedgehog::from_row(&x).unwrap())
                    .fetch_all(&*pool)
                    .await
                    .unwrap();

            assert_eq!(updated_hogs.len(), old_hogs.len());
            assert_eq!(updated_hogs.len(), 4);
            for (i, (a, b)) in updated_hogs.iter().zip(old_hogs.iter()).enumerate()
            {
                assert_eq!(a, b, "unequal {}-th hog", i);
            }

            // Here we check whether the inserted field histories are what we expect.
            let inserted_histories = sqlx::query(
                "select * from field_history ORDER BY record_uuid ASC, field_name ASC;",
            )
            .map(|x| FieldChange::from_row(&x).unwrap())
            .fetch_all(&*pool)
            .await
            .unwrap();

            let expected_histories = vec![
                FieldChange {
                    id: inserted_histories[0].id,
                    record_uuid: Uuid::parse_str(
                        "22345678-1234-1234-1234-123412341234",
                    )
                    .unwrap(),
                    table_name: "hedgehogs".to_string(),
                    field_name: "favourite_food".to_string(),
                    field_value: Some(SqlxJ(Value::from("Caterpillars"))),
                    record_status: HistoryStatus::Finished,
                    created_by: ctx.user_id,
                    created_at: ctx.timestamp,
                },
                FieldChange {
                    id: inserted_histories[1].id,
                    record_uuid: Uuid::parse_str(
                        "32345678-1234-1234-1234-123412341234",
                    )
                    .unwrap(),
                    table_name: "hedgehogs".to_string(),
                    field_name: "favourite_food".to_string(),
                    field_value: Some(SqlxJ(Value::from("Bananas"))),
                    record_status: HistoryStatus::Finished,
                    created_by: ctx.user_id,
                    created_at: ctx.timestamp,
                },
                FieldChange {
                    id: inserted_histories[2].id,
                    record_uuid: Uuid::parse_str(
                        "42345678-1234-1234-1234-123412341234",
                    )
                    .unwrap(),
                    table_name: "hedgehogs".to_string(),
                    field_name: "died_on".to_string(),
                    field_value: Some(SqlxJ(Value::from(
                        AsezDate(Date::try_from_ymd(2001, 4, 1).unwrap())
                    ))),
                    record_status: HistoryStatus::Finished,
                    created_by: ctx.user_id,
                    created_at: ctx.timestamp,
                },
            ];
            assert_eq!(inserted_histories.len(), 3);
            for (i, (a, b)) in
                inserted_histories.iter().zip(expected_histories.iter()).enumerate()
            {
                assert_eq!(a, b, "{}-th", i);
            }

            assert!(messages.messages.is_empty());

            let mut r = RecordCtx::new(USER3, pool.clone()).begin().await.expect("recorder");
            let mut messages = Messages::default();

            r.process_update_checked(
                original_hedgehogs(),
                Hedgehog::FIELDS,
                Dummy,
                &mut messages,
            )
            .await
            .expect("ProcessUpsert for Hedgehog could not run.");
            r.commit().await.unwrap();

            let exp_messages = vec![
                Message::info(
                    "I checked the hedgehogs very carefully.".to_string(),
                ),
            ];
            assert_eq!(messages.messages, exp_messages);
        })
        .await
    }

    #[tokio::test]
    async fn test_process_update_food_only_with_ctx() {
        run_db_test(RECORDS_EXTRA_MIGS, |pool| async move {
            let mut elder_hogs = original_hedgehogs();
            let old_hogs = new_hedgehogs(USER3);

            assert_ne!(elder_hogs, old_hogs);

            // Initial setup of hogs.
            let mut t = pool.begin().await.unwrap();
            Hedgehog::insert_vec(&mut elder_hogs, &mut t)
                .await
                .expect("Could not insert hogs.");
            t.commit().await.unwrap();

            // THIS IS WHERE THE WHOLE UPDATE PROCESS WITH HISTORY INSERTION
            // TAKES PLACE.
            let mut messages = Messages::default();
            let mut t = RecordCtx::new(USER3, pool.clone()).begin().await.expect("recorder");
            let ctx = t.ctx();

            t.process_update(
                old_hogs.clone(),
                &["favourite_food"],
                &mut messages,
            )
            .await
            .expect("ProcessUpsert for Hedgehog could not run.");
            t.commit().await.unwrap();

            // Here we check whether the updated hogs are what we expect.
            let updated_hogs =
                sqlx::query("select * from hedgehogs ORDER BY uuid ASC;")
                    .map(|x| Hedgehog::from_row(&x).unwrap())
                    .fetch_all(&*pool)
                    .await
                    .unwrap();
            assert_ne!(updated_hogs, old_hogs);
            for (old, new) in old_hogs.iter().zip(updated_hogs.iter()) {
                assert_eq!(old.uuid, new.uuid);
                assert_eq!(old.favourite_food, new.favourite_food);
            }

            // Here we check whether the inserted field histories are what we expect.
            let inserted_histories = sqlx::query(
                "select * from field_history ORDER BY record_uuid ASC, field_name ASC;",
            )
            .map(|x| FieldChange::from_row(&x).unwrap())
            .fetch_all(&*pool)
            .await
            .unwrap();

            let expected_histories = vec![
                FieldChange {
                    id: inserted_histories[0].id,
                    record_uuid: Uuid::parse_str(
                        "22345678-1234-1234-1234-123412341234",
                    )
                    .unwrap(),
                    table_name: "hedgehogs".to_string(),
                    field_name: "favourite_food".to_string(),
                    field_value: Some(SqlxJ(Value::from("Caterpillars"))),
                    record_status: HistoryStatus::Finished,
                    created_by: ctx.user_id,
                    created_at: ctx.timestamp,
                },
                FieldChange {
                    id: inserted_histories[1].id,
                    record_uuid: Uuid::parse_str(
                        "32345678-1234-1234-1234-123412341234",
                    )
                    .unwrap(),
                    table_name: "hedgehogs".to_string(),
                    field_name: "favourite_food".to_string(),
                    field_value: Some(SqlxJ(Value::from("Bananas"))),
                    record_status: HistoryStatus::Finished,
                    created_by: ctx.user_id,
                    created_at: ctx.timestamp,
                },
            ];
            println!("{:?}", inserted_histories);
            assert_eq!(inserted_histories.len(), 2);
            for (i, (a, b)) in
                inserted_histories.iter().zip(expected_histories.iter()).enumerate()
            {
                assert_eq!(a, b, "{}-th", i);
            }
        })
        .await
    }

    #[tokio::test]
    #[ignore = "Perf. test. Run manually when needed."]
    async fn test_perf() {
        use rand::distributions::{Alphanumeric, DistString};
        use rand::prelude::*;

        run_db_test(RECORDS_EXTRA_MIGS, |pool| async move {
            let mut hogs = Vec::with_capacity(400_000);
            let mut rng = thread_rng();
            println!("About to generate hedgehogs");
            for _ in 0..400_000 {
                let favourite_food = Alphanumeric.sample_string(&mut rng, 10);
                hogs.push(Hedgehog {
                    uuid: Uuid::new_v4(),
                    favourite_food,
                    born_on: AsezDate::today(),
                    died_on: None,
                    changed_by: 1,
                });
            }
            println!("hedgehogs generated");
            assert_eq!(hogs.len(), 400_000);
            // Initial setup of hogs.

            // THIS IS WHERE THE WHOLE UPDATE PROCESS WITH HISTORY INSERTION
            // TAKES PLACE.
            let mut messages = Messages::default();
            println!("Inserting");
            let mut t = RecordCtx::new(USER3, pool.clone())
                .begin()
                .await
                .expect("recorder");
            let start = std::time::Instant::now();
            hogs = t
                .process_insert(hogs, &mut messages)
                .await
                .expect("ProcessUpsert for Hedgehog could not run.");
            println!("Inserted");
            let elapsed = start.elapsed();
            t.commit().await.unwrap();

            let history_count: i64 =
                sqlx::query("select count(*) from field_history")
                    .try_map(|x| <(i64,)>::from_row(&x))
                    .fetch_one(&*pool)
                    .await
                    .unwrap()
                    .0;
            assert_eq!(history_count, 5 * 400_000);
            println!("elapsed: {}ms", elapsed.as_millis());
            assert!(100 > elapsed.as_secs(), "elapsed: {}ms", elapsed.as_millis());

            for x in hogs.iter_mut() {
                // guaranteed change because length is different.
                x.favourite_food = Alphanumeric.sample_string(&mut rng, 11);
            }

            let mut t = RecordCtx::new(USER3, pool.clone())
                .begin()
                .await
                .expect("recorder");
            let start = std::time::Instant::now();
            println!("updating");
            t.process_update(hogs, &[Hedgehog::favourite_food], &mut messages)
                .await
                .expect("ProcessUpsert for Hedgehog could not run.");
            println!("Updated");
            let elapsed = start.elapsed();
            t.commit().await.unwrap();

            let history_count: i64 =
                sqlx::query("select count(*) from field_history")
                    .try_map(|x| <(i64,)>::from_row(&x))
                    .fetch_one(&*pool)
                    .await
                    .unwrap()
                    .0;
            assert_eq!(history_count, 6 * 400_000);
            println!("elapsed: {}ms", elapsed.as_millis());
            assert!(100 > elapsed.as_secs(), "elapsed: {}ms", elapsed.as_millis());

            sqlx::query("delete from hedgehogs").execute(&*pool).await.unwrap();
        })
        .await
    }
}
