use super::*;
use crate::RouteAddep;

// Cannot use pkeys in this kind of table creation.
const CREATE_TABLE: &str = "(
    uuid UUID NOT NULL PRIMARY KEY UNIQUE,
    route_id INTEGER NOT NULL,
    department_id INTEGER NOT NULL,
    division_id INTEGER NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    created_by INTEGER NOT NULL DEFAULT 0,
    changed_by INTEGER NOT NULL DEFAULT 0
)";

fn new_route(route_id: i32, department_id: i32, division_id: i32) -> RouteAddep {
    RouteAddep {
        route_id,
        department_id,
        division_id,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_insert_returning() {
    run_db_test(RouteAddep::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let mut route = new_route(1, 2, 3);

        let x = sqlx::query("SELECT uuid FROM route_addep")
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert!(x.is_empty());

        let ret = route.insert_returning(&mut pool).await.expect("Error in DB");

        assert_ne!(ret.uuid, Default::default());

        assert_eq!(ret.route_id, 1);
        assert_eq!(ret.department_id, 2);
        assert_eq!(ret.division_id, 3);

        assert!(!ret.is_removed);
    })
    .await
}
