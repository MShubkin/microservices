use asez2_shared_db::{asez_date, db_item::int_array::AsezArray, uuid};

use super::*;
use crate::{ApprovalStatus, DocumentApprover};

const CREATE_TABLE: &str = r#"(
    uuid UUID NOT NULL PRIMARY KEY UNIQUE,
    "number" INTEGER NOT NULL,
    document_uuid UUID NOT NULL,
    plan_id BIGINT NOT NULL,
    department_id INTEGER NOT NULL,
    planned_date DATE NOT NULL,
    started_at TIMESTAMP WITHOUT TIME ZONE,
    division_id INTEGER,
    division_assigned_at TIMESTAMP WITHOUT TIME ZONE,
    expert_id INTEGER,
    responded_at TIMESTAMP WITHOUT TIME ZONE,
    response_id INTEGER,
    response_note VARCHAR(1024),
    total_when_decision BIGINT,
    status_appr SMALLINT NOT NULL DEFAULT 0,
    responsible_person_id INTEGER,
    is_auto BOOLEAN NOT NULL DEFAULT false,
    route_id BIGINT[] NOT NULL DEFAULT ARRAY[]::BIGINT[],
    send_date_1 TIMESTAMP WITHOUT TIME ZONE,
    send_users_1 INTEGER[] NOT NULL DEFAULT '{}',
    send_date_2 TIMESTAMP WITHOUT TIME ZONE,
    send_users_2 INTEGER[] NOT NULL DEFAULT '{}',
    is_preapproved BOOLEAN NOT NULL DEFAULT false,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    is_actual BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT '1900-01-01 00:00:00',
    created_by INTEGER NOT NULL DEFAULT 0,
    changed_by INTEGER NOT NULL DEFAULT 0
)"#;

fn new_document_approver(
    document_uuid: Uuid,
    plan_id: i64,
    department_id: i32,
    number: i32,
    planned_date: AsezDate,
    send_users_1: &[i32],
    send_users_2: &[i32],
) -> DocumentApprover {
    DocumentApprover {
        document_uuid,
        plan_id,
        department_id,
        number,
        planned_date,
        send_users_1: AsezArray(send_users_1.to_vec()),
        send_users_2: AsezArray(send_users_2.to_vec()),
        ..Default::default()
    }
}

#[tokio::test]
async fn insert_returning() {
    let plan_uuid = uuid!("33376446-01ca-4270-87cf-1731fcdef02c");
    let planned_date = asez_date!("2024-10-12");
    run_db_test(
        DocumentApprover::TABLE,
        CREATE_TABLE,
        None,
        move |mut pool| async move {
            let mut doc_appr = new_document_approver(
                plan_uuid,
                1000,
                10,
                3,
                planned_date,
                &[1, 2, 3],
                &[20, 30],
            );

            let x = sqlx::query("SELECT uuid FROM document_approver")
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert!(x.is_empty());

            let ret =
                doc_appr.insert_returning(&mut pool).await.expect("Error in DB");

            assert_ne!(ret.uuid, Default::default());

            assert_eq!(ret.document_uuid, plan_uuid);
            assert_eq!(ret.plan_id, 1000);
            assert_eq!(ret.department_id, 10);
            assert_eq!(ret.number, 3);
            assert_eq!(ret.planned_date, planned_date);
            assert_eq!(&ret.send_users_1.0, &[1, 2, 3]);
            assert_eq!(&ret.send_users_2.0, &[20, 30]);

            assert_eq!(ret.status_appr, ApprovalStatus::New);
            assert!(!ret.is_auto);
            assert!(!ret.is_preapproved);
            assert!(!ret.is_removed);
            assert!(!ret.is_actual);
        },
    )
    .await
}

#[tokio::test]
async fn insert_vec_returning() {
    let plan_uuid0 = uuid!("00000000-0000-0000-0000-000000000001");
    let plan_uuid1 = uuid!("00000000-0000-0000-0000-000000000002");
    let plan_uuid2 = uuid!("00000000-0000-0000-0000-000000000003");
    let planned_date0 = asez_date!("2024-10-12");
    let planned_date1 = asez_date!("2024-10-13");
    let planned_date2 = asez_date!("2024-10-14");
    let send_users: &[&[&[i32]]] =
        &[&[&[1, 2, 3], &[]], &[&[], &[20, 30]], &[&[4, 5, 6], &[100, 200]]];

    run_db_test(
        DocumentApprover::TABLE,
        CREATE_TABLE,
        None,
        move |mut pool| async move {
            let mut doc_apprs = vec![
                new_document_approver(
                    plan_uuid0,
                    1000,
                    10,
                    3,
                    planned_date0,
                    send_users[0][0],
                    send_users[0][1],
                ),
                new_document_approver(
                    plan_uuid1,
                    1001,
                    11,
                    4,
                    planned_date1,
                    send_users[1][0],
                    send_users[1][1],
                ),
                new_document_approver(
                    plan_uuid2,
                    1002,
                    12,
                    5,
                    planned_date2,
                    send_users[2][0],
                    send_users[2][1],
                ),
            ];

            let x = sqlx::query("SELECT uuid FROM document_approver")
                .fetch_all(&mut pool)
                .await
                .unwrap();
            assert!(x.is_empty());

            let ret =
                DocumentApprover::insert_vec_returning(&mut doc_apprs, &mut pool)
                    .await
                    .expect("Error in DB");
            assert_eq!(ret.len(), 3);

            assert_ne!(ret[0].uuid, Default::default());

            assert_eq!(ret[0].document_uuid, plan_uuid0);
            assert_eq!(ret[0].plan_id, 1000);
            assert_eq!(ret[0].department_id, 10);
            assert_eq!(ret[0].number, 3);
            assert_eq!(ret[0].planned_date, planned_date0);
            assert_eq!(&ret[0].send_users_1.0, send_users[0][0]);
            assert_eq!(&ret[0].send_users_2.0, send_users[0][1]);

            assert_eq!(ret[0].status_appr, ApprovalStatus::New);
            assert!(!ret[0].is_auto);
            assert!(!ret[0].is_preapproved);
            assert!(!ret[0].is_removed);

            assert_ne!(ret[1].uuid, Default::default());

            assert_eq!(ret[1].document_uuid, plan_uuid1);
            assert_eq!(ret[1].plan_id, 1001);
            assert_eq!(ret[1].department_id, 11);
            assert_eq!(ret[1].number, 4);
            assert_eq!(ret[1].planned_date, planned_date1);
            assert_eq!(&ret[1].send_users_1.0, send_users[1][0]);
            assert_eq!(&ret[1].send_users_2.0, send_users[1][1]);

            assert_eq!(ret[1].status_appr, ApprovalStatus::New);
            assert!(!ret[1].is_auto);
            assert!(!ret[1].is_preapproved);
            assert!(!ret[1].is_removed);

            assert_ne!(ret[2].uuid, Default::default());

            assert_eq!(ret[2].document_uuid, plan_uuid2);
            assert_eq!(ret[2].plan_id, 1002);
            assert_eq!(ret[2].department_id, 12);
            assert_eq!(ret[2].number, 5);
            assert_eq!(ret[2].planned_date, planned_date2);
            assert_eq!(&ret[2].send_users_1.0, send_users[2][0]);
            assert_eq!(&ret[2].send_users_2.0, send_users[2][1]);

            assert_eq!(ret[2].status_appr, ApprovalStatus::New);
            assert!(!ret[2].is_auto);
            assert!(!ret[2].is_preapproved);
            assert!(!ret[2].is_removed);
        },
    )
    .await
}
