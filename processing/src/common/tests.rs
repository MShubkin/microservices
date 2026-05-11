use super::number_range::*;
use super::rules::*;
use asez2_shared_db::test_setup::run_db_test;
use shared_essential::domain::legacy::plans::PlanStatus;

const CREATE_STRING: &str = "(
	object_type SMALLINT NOT NULL,
	start_idx BIGINT NOT NULL,
	end_idx BIGINT NOT NULL,
    next_idx BIGINT NOT NULL
)";

const INSERT: &str =
    "(object_type, start_idx, end_idx, next_idx) values(0, 0, 4999999, 0),
    (1, 5000000, 5999999, 5000000),
    (2, 9000000, 9999999, 9000000)";

#[tokio::test]
async fn test_get_next_numbers() {
    let req_one = vec![
        NumberRequest::new(EcObjectType::Agenda, 1),
        NumberRequest::new(EcObjectType::Protocol, 6),
    ];
    let req_two = vec![NumberRequest::new(EcObjectType::Agenda, 1)];
    let req_three = vec![
        NumberRequest::new(EcObjectType::Agenda, 1),
        NumberRequest::new(EcObjectType::Protocol, 3),
    ];

    run_db_test(
        "number_range",
        CREATE_STRING,
        Some(INSERT),
        |mut pool| async move {
            assert_eq!(
                get_next_numbers(&mut pool, req_one).await.unwrap(),
                vec![
                    (EcObjectType::Agenda, vec![5000000]),
                    (
                        EcObjectType::Protocol,
                        vec![9000000, 9000001, 9000002, 9000003, 9000004, 9000005]
                    ),
                ]
                .into_iter()
                .collect::<ahash::AHashMap<_, _>>()
            );
            assert_eq!(
                get_next_numbers(&mut pool, req_two).await.unwrap(),
                vec![(EcObjectType::Agenda, vec![5000001]),]
                    .into_iter()
                    .collect::<ahash::AHashMap<_, _>>()
            );
            assert_eq!(
                get_next_numbers(&mut pool, req_three).await.unwrap(),
                vec![
                    (EcObjectType::Agenda, vec![5000002]),
                    (EcObjectType::Protocol, vec![9000006, 9000007, 9000008]),
                ]
                .into_iter()
                .collect::<ahash::AHashMap<_, _>>()
            );
        },
    )
    .await
}

#[tokio::test]
async fn test_get_next_numbers_overflow() {
    let req_one = vec![
        // For now, we should overflow at 999999.
        NumberRequest::new(EcObjectType::Agenda, 2000000),
    ];

    run_db_test(
        "number_range",
        CREATE_STRING,
        Some(INSERT),
        |mut pool| async move {
            assert_eq!(
                &get_next_numbers(&mut pool, req_one)
                    .await
                    .unwrap_err()
                    .to_string(),
                "Number range for objects of type \"Agenda\" is full."
            );
        },
    )
    .await
}

#[tokio::test]
async fn test_status_rules_container1() {
    run_db_test("number_range", CREATE_STRING, None, |mut pool| async move {
        // Setup the table
        let migration = include_str!(
            "../../migrations/20240719082834_create_pl_status_transitions.up.sql"
        );
        for split in migration.split(';') {
            sqlx::query(split).execute(&mut pool).await.unwrap();
        }

        let rules = ProcessingRules::new(&mut pool).await.unwrap();
        let status_rules = rules.status_rules();

        assert_eq!(status_rules.len(), 61);

        let all = [
            (221i16, 222i16, " "),
            (222, 120, " "),
            (222, 223, " "),
            (223, 222, " "),
            (223, 225, " "),
            (225, 251, "estimated_commission"),
            (225, 252, "estimated_commission"),
            (225, 253, "estimated_commission"),
            (251, 120, "return_to customer"),
            (251, 131, "return_to customer"),
            (251, 140, "approve"),
            (251, 150, "cancel"),
            (251, 160, "approve"),
            (251, 222, "return_to_expert"),
            (251, 252, "change_form"),
            (251, 253, "change_form"),
            (251, 342, "return_to_expert"),
            (251, 352, "return_to_expert"),
            (252, 120, "return_to customer"),
            (252, 131, "return_to customer"),
            (252, 140, "approve"),
            (252, 150, "cancel"),
            (252, 160, "approve"),
            (252, 222, "return_to_expert"),
            (252, 251, "change_form"),
            (252, 253, "change_form"),
            (252, 342, "return_to_expert"),
            (252, 352, "return_to_expert"),
            (253, 120, "return_to customer"),
            (253, 131, "return_to customer"),
            (253, 140, "approve"),
            (253, 150, "cancel"),
            (253, 160, "approve"),
            (253, 222, "return_to_expert"),
            (253, 251, "change_form"),
            (253, 252, "change_form"),
            (253, 342, "return_to_expert"),
            (253, 352, "return_to_expert"),
            (341, 342, " "),
            (342, 120, " "),
            (342, 343, " "),
            (343, 342, " "),
            (343, 345, " "),
            (345, 251, "estimated_commission"),
            (345, 252, "estimated_commission"),
            (345, 253, "estimated_commission"),
            (351, 352, " "),
            (351, 356, " "),
            (352, 120, " "),
            (352, 353, " "),
            (352, 356, " "),
            (353, 352, " "),
            (353, 355, " "),
            (353, 356, " "),
            (355, 251, "estimated_commission"),
            (355, 252, "estimated_commission"),
            (355, 253, "estimated_commission"),
            (356, 342, " "),
            (356, 343, " "),
            (112, 115, "customer - specialized_departments"),
            (112, 116, "customer - tender_division"),
        ]
        .into_iter();

        for (old, new, rule) in all {
            let old = PlanStatus::try_from(old).unwrap();
            let new = PlanStatus::try_from(new).unwrap();

            assert!(status_rules.transition_is_ok(old, new));

            if let Some(exp_rule) = status_rules.get_rule(old, new) {
                assert_eq!(rule, exp_rule, "{}->{:?}", old, new);
            } else {
                panic!("{}->{:?} should be allowed but is not", old, new);
            }
        }

        let bad = [
            (222, 221),
            (341, 222),
            (352, 351),
            (150, 251),
            (140, 251),
            (222, 251),
            (352, 251),
            (131, 251),
            (120, 251),
            (150, 342),
            (251, 355),
        ];

        for (old, new) in bad {
            let old = PlanStatus::try_from(old).unwrap();
            let new = PlanStatus::try_from(new).unwrap();

            assert!(!status_rules.transition_is_ok(old, new), "{}->{}", old, new);
        }
    })
    .await
}
