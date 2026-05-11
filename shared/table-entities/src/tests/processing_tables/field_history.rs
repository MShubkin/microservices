use crate::processing::field_histories::*;
use asez2_shared_db::test_setup::run_db_test;
use asez2_shared_db::{DbItem, Value};
use sqlx::FromRow;

const CREATE_TABLE: &str = "(
    id BIGSERIAL PRIMARY KEY NOT NULL UNIQUE,
    record_uuid uuid NOT NULL,
    table_name TEXT NOT NULL,
    field_name TEXT NOT NULL,
    field_value jsonb,
    record_status SMALLINT NOT NULL,
    created_at timestamp NOT NULL,
    created_by INTEGER NOT NULL
)";

#[tokio::test]
async fn field_history_status_update() {
    run_db_test(FieldChange::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let mut new = vec![
            FieldChange {
                record_uuid: "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11"
                    .parse()
                    .unwrap(),
                table_name: "plan".to_string(),
                field_name: "status_id".to_string(),
                field_value: Some(sqlx::types::Json(Value::Int(5))),
                ..Default::default()
            },
            FieldChange {
                record_uuid: "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22"
                    .parse()
                    .unwrap(),
                table_name: "plan".to_string(),
                field_name: "status_id".to_string(),
                field_value: Some(sqlx::types::Json(Value::Int(6))),
                ..Default::default()
            },
            FieldChange {
                record_uuid: "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33"
                    .parse()
                    .unwrap(),
                table_name: "plan".to_string(),
                field_name: "status_id".to_string(),
                field_value: Some(sqlx::types::Json(Value::Int(7))),
                ..Default::default()
            },
        ];

        let r =
            FieldChange::insert_vec_returning(&mut new, &mut pool).await.unwrap();

        assert_eq!(r.len(), 3);
        assert!(
            r.iter().all(|x| matches!(x.record_status, HistoryStatus::Proposed)),
            "History status not proposed {:?}",
            r
        );

        FieldChange::mass_update_status(&r, HistoryStatus::Finished, &mut pool)
            .await
            .unwrap();

        let r = sqlx::query(" select * from field_history;")
            .map(|x| FieldChange::from_row(&x).unwrap())
            .fetch_all(&mut pool)
            .await
            .unwrap();

        assert_eq!(r.len(), 3);
        assert!(
            r.iter().all(|x| matches!(x.record_status, HistoryStatus::Finished)),
            "History status not finished {:?}",
            r
        );
    })
    .await
}
