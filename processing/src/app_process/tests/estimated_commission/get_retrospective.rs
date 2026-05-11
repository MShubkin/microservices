use super::*;
use crate::app_process::get_retrospective;
use shared_essential::presentation::dto::response_request::{EntityKind, Status};
use std::str::FromStr;
use uuid::Uuid;

const GET_PLAN_RETROSPECTIVE: &[&str] =
    &["estimated_commission/get_plan_retrospective.sql"];

#[tokio::test]
async fn get_plan_retrosective_test() {
    run_db_test(GET_PLAN_RETROSPECTIVE, |pool| async move {
        let request = GetRetrospectiveReq {
            item_list: vec![ObjectIdentifier::new_with_type(
                123,
                Uuid::from_str("12300000-0000-0000-0000-000000000001").unwrap(),
                EntityKind::Plan,
            )],
        };
        let response = get_retrospective(request, pool.clone()).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.messages.is_empty());
        assert_eq!(response.data.item_list.len(), 2);
        let data = response.data.item_list;
        for merged_plan_retrospective in data {
            match merged_plan_retrospective.plan {
                PlanOrAmendmentRep::Plan(plan) => match plan.plan_id.unwrap() {
                    124 => {
                        assert_eq!(
                            merged_plan_retrospective
                                .retrospective
                                .plan_retrospective_id
                                .unwrap(),
                            1
                        );
                        assert_eq!(
                            merged_plan_retrospective.status_history.uuid.unwrap(),
                            Uuid::from_str("220E8400E29B41D4A716446655440000")
                                .unwrap()
                        );
                    }
                    125 => {
                        assert_eq!(
                            merged_plan_retrospective
                                .retrospective
                                .plan_retrospective_id
                                .unwrap(),
                            2
                        );
                        assert_eq!(
                            merged_plan_retrospective.status_history.uuid,
                            None
                        );
                    }
                    _ => {
                        panic!("should return a plan_id 124 or 125");
                    }
                },
                PlanOrAmendmentRep::Amendment(_) => {
                    panic!("should return a plan");
                }
            }
        }
    })
    .await
}
