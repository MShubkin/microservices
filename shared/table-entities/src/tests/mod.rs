//! Тесты для таблиц. В основном те для которых БД не требуется.
mod master_data_tables;
mod processing_tables;
mod serde;

use super::{Plan, PlanRep};
use asez2_shared_db::DbAdaptor;
use uuid::Uuid;

#[test]
fn plan_to_plan_rep() {
    let plan = Plan {
        uuid: Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        id: 1000038765,
        contract_subject: "Very interesting purchase".to_string(),
        pricing_resume: Some(String::from("text")),
        ..Default::default()
    };
    let exp_rep_old = PlanRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        id: Some(1000038765),
        contract_subject: Some("Very interesting purchase".to_string()),
        pricing_resume: Some(Some(String::from("text"))),
        ..Default::default()
    };
    let exp_rep_new = PlanRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        id: Some(1000038765),
        contract_subject_short: Some("Very interesting purchase".to_string()),
        pricing_resume_short: Some(Some(String::from("text"))),
        ..Default::default()
    };
    let exp_rep_all = PlanRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        id: Some(1000038765),
        contract_subject_short: Some("Very interesting purchase".to_string()),
        pricing_resume_short: Some(Some(String::from("text"))),
        contract_subject: Some("Very interesting purchase".to_string()),
        pricing_resume: Some(Some(String::from("text"))),
        ..Default::default()
    };

    let full_rep = PlanRep::from_item(
        plan.clone(),
        Some(&[
            "uuid",
            "id",
            "contract_subject_short",
            "contract_subject",
            "pricing_resume",
            "pricing_resume_short",
        ]),
    );

    let old_rep = PlanRep::from_item(
        plan.clone(),
        Some(&["uuid", "id", "contract_subject", "pricing_resume"]),
    );

    let new_rep = PlanRep::from_item(
        plan,
        Some(&["uuid", "id", "contract_subject_short", "pricing_resume_short"]),
    );

    assert_eq!(full_rep, exp_rep_all);
    assert_eq!(old_rep, exp_rep_old);
    assert_eq!(new_rep, exp_rep_new);
}
