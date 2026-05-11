use anyhow::Result;
use asez2_shared_db::db_item::Select;
use asez2_shared_db::{uuid, DbItem};
use asez2_tables::{
    ContractAmendment, ContractAmendmentRep, Plan, PlanOrAmendmentRep, PlanRep,
    PlanStatus, Section,
};
use shared_essential::presentation::dto::general::ObjectIdentifier;
use shared_essential::presentation::dto::processing::price_analysis::PreRequestDocumentsForExpertReq;
use shared_essential::presentation::dto::processing::{
    AssignExpertMassReq, PlansCountRequest,
};
use testing::TestDbPool;

const FIXTURE: &str =
    include_str!("../extra_migrations/price_analysis/expert_mass_assignment.sql");

use crate::app_process::calls;

use crate::app_process::tests::mock_processing_context;

#[testing::test]
async fn get_plans_count(#[with_arg(FIXTURE)] db_pool: TestDbPool) -> Result<()> {
    let req = PlansCountRequest {
        select: Select::default(),
        pricing_expert_ids: vec![123, 124],
        section: Section::PriceAnalysisDeterminePrice,
        user_id: 666,
    };
    let res = calls::get_plans_count::get_plans_count(req, db_pool.clone()).await?;

    assert_eq!(res.data.get(&123), Some(&3));
    assert_eq!(res.data.get(&124), Some(&0));

    Ok(())
}

#[testing::test]
async fn pre_request_documents_for_expert(
    #[with_arg(FIXTURE)] db_pool: TestDbPool,
) -> Result<()> {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000101");
    let plan_id = 101;
    let plan_oid = ObjectIdentifier::new(plan_id, plan_uuid);
    let ca_uuid = uuid!("00000000-0000-0000-0004-000000000000");
    let ca_id = 14;
    let ca_oid = ObjectIdentifier::new(ca_id, ca_uuid);

    let res = calls::price_analysis::pa_pre_request_documents_for_expert(
        PreRequestDocumentsForExpertReq {
            item_list: vec![plan_oid, ca_oid],
        },
        db_pool.clone(),
    )
    .await?;

    assert_eq!(res.data.len(), 2);
    Ok(())
}

#[testing::test]
async fn mass_assign_experts(
    #[with_arg(FIXTURE)] db_pool: TestDbPool,
) -> Result<()> {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000101");
    let ca_uuid = uuid!("00000000-0000-0000-0004-000000000000");
    let proc_ctx = mock_processing_context(db_pool.clone()).await;
    let plans = vec![
        PlanOrAmendmentRep::Plan(PlanRep {
            uuid: Some(plan_uuid),
            id: Some(101),
            pricing_expert_id: Some(Some(123)),
            ..Default::default()
        }),
        PlanOrAmendmentRep::Amendment(ContractAmendmentRep {
            uuid: Some(ca_uuid),
            id: Some(14),
            pricing_expert_id: Some(Some(123)),
            ..Default::default()
        }),
    ];
    let _res = calls::price_analysis::assign_expert_mass(
        AssignExpertMassReq {
            plans,
            user_id: 666,
        },
        proc_ctx,
    )
    .await?;

    let updated_plan = Plan::select_single(
        &Select::full::<Plan>().eq(Plan::uuid, plan_uuid),
        &**db_pool,
    )
    .await?;
    let updated_ca = ContractAmendment::select_single(
        &Select::full::<ContractAmendment>().eq(ContractAmendment::uuid, ca_uuid),
        &**db_pool,
    )
    .await?;

    assert_eq!(updated_plan.pricing_expert_id, Some(123));
    assert_eq!(updated_plan.status_id, PlanStatus::ExecutorAppointedD646);

    assert_eq!(updated_ca.pricing_expert_id, Some(123));
    assert_eq!(updated_ca.status_id, PlanStatus::ExecutorAppointedD646);

    Ok(())
}
