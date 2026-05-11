use super::*;
use crate::master_data::scheduler_calendar::scheduler_update_catalog_request::SchedulerRequestUpdateCatalog;
use asez2_shared_db::db_item::AsezTimestamp;
use asez2_shared_db::test_setup::run_db_test;
use sqlx::FromRow;
use tokio::test;

const CREATE_TABLE: &str = "(
    id SERIAL NOT NULL UNIQUE,
    event_name character varying(250) COLLATE pg_catalog.\"default\",
    event_date date NOT NULL,
    period_time SMALLINT NOT NULL,
    is_removed BOOLEAN DEFAULT false,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL)";

#[test]
async fn get_insert_update() {
    run_db_test(
        SchedulerRequestUpdateCatalog::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let mut new = SchedulerRequestUpdateCatalog {
                id: 1,
                event_name: "test_day".to_string(),
                event_date: AsezDate::today(),
                period_time: 2022,
                is_removed: false,
                created_at: AsezTimestamp::from_unix_timestamp(0),
                changed_at: AsezTimestamp::from_unix_timestamp(0),
                created_by: 1,
                changed_by: 1,
            };

            let for_update = SchedulerRequestUpdateCatalog {
                id: 1,
                event_name: "update_day".to_string(),
                event_date: AsezDate::today(),
                period_time: 2022,
                is_removed: false,
                created_at: AsezTimestamp::from_unix_timestamp(0),
                changed_at: AsezTimestamp::from_unix_timestamp(0),
                created_by: 1,
                changed_by: 1,
            };

            let r = new.insert(&mut pool).await.unwrap();

            assert_eq!(r, 1);

            let days = sqlx::query(" select * from scheduler_catalog;")
                .map(|x| SchedulerRequestUpdateCatalog::from_row(&x).unwrap())
                .fetch_all(&mut pool)
                .await
                .unwrap();

            assert_eq!(days[0], new);

            let x = for_update.update(None, &mut pool).await.unwrap();

            assert_eq!(x, 1);

            let days = sqlx::query(" select * from scheduler_catalog;")
                .map(|x| SchedulerRequestUpdateCatalog::from_row(&x).unwrap())
                .fetch_all(&mut pool)
                .await
                .unwrap();

            assert_eq!(days[0], for_update);
        },
    )
    .await
}
