use crate::OrganizationalUserAssignment;
use asez2_shared_db::test_setup::run_db_test;
use asez2_shared_db::DbItem;

// Cannot use pkeys in this kind of table creation.
const CREATE_TABLE: &str = "(
    uuid UUID NOT NULL PRIMARY KEY UNIQUE, -- ??? В МОНОЛИТЕ НЕТ
    user_id INTEGER NOT NULL,
    customer_id INTEGER,
    department_id INTEGER,
    position_id INTEGER,
    organizer_id INTEGER,
    purchasing_group_id INTEGER,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    created_by INTEGER NOT NULL DEFAULT 0,
    changed_by INTEGER NOT NULL DEFAULT 0
)";

fn new_org_user_assn(
    user_id: i32,
    department_id: i32,
    customer_id: Option<i32>,
    position_id: Option<i32>,
    organizer_id: Option<i32>,
    purchasing_group_id: Option<i32>,
) -> OrganizationalUserAssignment {
    OrganizationalUserAssignment {
        user_id,
        customer_id,
        department_id,
        position_id,
        organizer_id,
        purchasing_group_id,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_insert_returning() {
    run_db_test(
        OrganizationalUserAssignment::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let mut org_user_assn =
                new_org_user_assn(1, 2, Some(3), Some(4), Some(5), Some(6));

            let x = sqlx::query("SELECT uuid FROM organizational_user_assignment")
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert!(x.is_empty());

            let ret = org_user_assn
                .insert_returning(&mut pool)
                .await
                .expect("Error in DB");

            assert_ne!(ret.uuid, Default::default());

            assert_eq!(ret.user_id, 1);
            assert_eq!(ret.department_id, 2);
            assert_eq!(ret.customer_id, Some(3));
            assert_eq!(ret.position_id, Some(4));
            assert_eq!(ret.organizer_id, Some(5));
            assert_eq!(ret.purchasing_group_id, Some(6));
        },
    )
    .await
}
