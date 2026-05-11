use sqlx::types::Json;
use uuid::Uuid;

use super::*;
use crate::master_data::routes::{CritPredicate, CritValue, RouteCrit};
use asez2_shared_db::test_setup::run_db_test;

// Cannot use pkeys in this kind of table creation.
const CREATE_TABLE: &str = "(
    route_uuid UUID NOT NULL,
    field_name VARCHAR NOT NULL,
    predicate JSON NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_by INTEGER NOT NULL,
    PRIMARY KEY (route_uuid, field_name)
)";

fn new_route_crit(
    route_uuid: Uuid,
    name: &str,
    predicate: CritPredicate,
) -> RouteCrit {
    RouteCrit {
        route_uuid,
        field_name: name.to_string(),
        predicate: Json(predicate),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_insert_returning() {
    run_db_test(RouteCrit::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let uuid = Uuid::new_v4();
        let mut route = new_route_crit(
            uuid,
            "column",
            CritPredicate::Equal {
                value: CritValue::Int(100),
            },
        );

        let x = sqlx::query("SELECT * FROM route_crit")
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert!(x.is_empty());

        let ret = route.insert_returning(&mut pool).await.expect("Error in DB");

        assert_eq!(ret.route_uuid, uuid);
        assert_eq!(&ret.field_name, "column");
        assert!(matches!(
            ret.predicate.as_ref(),
            &CritPredicate::Equal {
                value: CritValue::Int(100)
            }
        ));

        assert!(!ret.is_removed);
    })
    .await
}
