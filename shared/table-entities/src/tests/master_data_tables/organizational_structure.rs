//! We use some unusual array types here, so it is important that we can test
//! whether we can insert retrieve them correctly.

use crate::tests::master_data_tables::*;
use crate::{DepartmentLevel, DepartmentType, OrganizationalStructure};
use asez2_shared_db::test_setup::run_db_test;

// Cannot use pkeys in this kind of table creation.
const CREATE_TABLE: &str = "(
    uuid UUID NOT NULL PRIMARY KEY UNIQUE,
    id INTEGER NOT NULL DEFAULT 0,
    code SMALLINT NOT NULL DEFAULT 0,
    text VARCHAR(50) NOT NULL DEFAULT '',
    text_short VARCHAR(20) NOT NULL DEFAULT '',
    level SMALLINT NOT NULL DEFAULT 1,
    parent_id INTEGER,
    type SMALLINT NOT NULL DEFAULT 1,
    is_specialized_department BOOLEAN NOT NULL DEFAULT false,
    sap_id INTEGER,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    created_by INTEGER NOT NULL DEFAULT 0,
    changed_by INTEGER NOT NULL DEFAULT 0
)";

fn new_dept(
    id: i32,
    level: DepartmentLevel,
    dep_type: DepartmentType,
) -> OrganizationalStructure {
    OrganizationalStructure {
        id,
        level,
        dep_type,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_insert_returning() {
    run_db_test(
        OrganizationalStructure::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let mut dept_1 =
                new_dept(1, DepartmentLevel::GP, DepartmentType::Section);

            let x = sqlx::query("SELECT uuid FROM organizational_structure")
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert!(x.is_empty());

            let ret =
                dept_1.insert_returning(&mut pool).await.expect("Error in DB");

            assert_ne!(ret.uuid, Default::default());

            assert_eq!(ret.id, 1);
            assert_eq!(ret.level, DepartmentLevel::GP);
            assert_eq!(ret.dep_type, DepartmentType::Section);
            // check that we're testing with non-default value for renamed field
            assert_ne!(ret.dep_type, DepartmentType::default());

            assert!(!ret.is_specialized_department);
            assert!(!ret.is_removed);
        },
    )
    .await
}

#[tokio::test]
async fn test_insert_vec_returning() {
    run_db_test(
        OrganizationalStructure::TABLE,
        CREATE_TABLE,
        None,
        |mut pool| async move {
            let dept_1 = new_dept(
                2,
                DepartmentLevel::Department,
                DepartmentType::Department,
            );
            let dept_2 =
                new_dept(3, DepartmentLevel::Division, DepartmentType::Division);
            let dept_3 =
                new_dept(4, DepartmentLevel::SubDivision, DepartmentType::Section);

            let mut multi = vec![dept_1, dept_2, dept_3];

            let x = sqlx::query("SELECT uuid FROM organizational_structure")
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert!(x.is_empty());

            // Non joined test.
            {
                let ret = OrganizationalStructure::insert_vec_returning(
                    &mut multi, &mut pool,
                )
                .await
                .expect("Error in DB");

                assert_eq!(ret.len(), 3);

                assert_ne!(ret[0].uuid, Default::default());

                assert_eq!(ret[0].id, 2);
                assert_eq!(ret[0].level, DepartmentLevel::Department);
                assert_eq!(ret[0].dep_type, DepartmentType::Department);

                assert_ne!(ret[1].uuid, Default::default());

                assert_eq!(ret[1].id, 3);
                assert_eq!(ret[1].level, DepartmentLevel::Division);
                assert_eq!(ret[1].dep_type, DepartmentType::Division);

                assert_ne!(ret[2].uuid, Default::default());

                assert_eq!(ret[2].id, 4);
                assert_eq!(ret[2].level, DepartmentLevel::SubDivision);
                assert_eq!(ret[2].dep_type, DepartmentType::Section);
            }
        },
    )
    .await
}
