use asez2_shared_db::db_item::DbVersioned;
use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
use asez2_shared_db::DbItem;

use uuid::Uuid;

use crate::legacy::plans::PlanStatus;
use crate::maths::*;
use crate::processing::agenda::CommissionKind;
use crate::processing::plan::*;
use crate::test_setup::run_db_test;
use crate::{ContractAmendment, ContractAmendmentVersion};

#[tokio::test]
async fn test_plan_version_fields() {
    // let uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();
    let template_time = AsezTimestamp::from_unix_timestamp(123123);
    let template_day = AsezDate::today();

    assert_ne!(template_time, AsezTimestamp::default());
    assert_ne!(template_day, AsezDate::default());

    // NB: No field is automatically filled in this table, even
    // created_at and changed_at, as well as calculated fields are filled out
    // by logic in the "processing" service.
    //
    // This test simply tests that the versioning code works correctly, saving
    // all fields from the original "plan" table in every version.
    let mut plan = ContractAmendment {
        uuid: Uuid::default(),
        id: 4,
        status_id: PlanStatus::from(225),
        savings_accounting_id: SavingsAccountingId::No,
        savings_sum_excluded_vat: Some(CurrencyValue(45_000)),
        savings_sum_excluded_vat_rub: Some(CurrencyValue(45_000_000)),
        savings_sum_included_vat: Some(CurrencyValue(55_000)),
        savings_sum_included_vat_rub: Some(CurrencyValue(55_000_000)),
        pricing_organization_unit_id: PricingUnitId::Gpk,
        expert_conclusion_id: Some(ExpertConclusionId::RefundToCustomer),
        pricing_expert_id: Some(3),
        is_check_documentation: true,
        pricing_resume: Some("priceless".to_owned()),
        commission_kind_id: CommissionKind::NotRequired,
        customer_id: 5,
        check_documentation_date: Some(template_time),
        priority_introductory_date: template_day,
        chairman_order_date: template_day,
        termination_date: template_day,
        commission_date: Some(template_day),
        posting_date: template_day,
        created_at: AsezTimestamp::from_unix_timestamp(999_999),
        changed_at: AsezTimestamp::from_unix_timestamp(999_999_999),
        ..ContractAmendment::default()
    };
    run_db_test(move |pool| async move {
        let mut tx = pool.begin().await.unwrap();
        let inserted = plan.insert_returning(&mut tx).await.unwrap();
        ContractAmendment::insert_version_vec_returning(
            &[inserted.clone()],
            &mut tx,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(inserted, plan, "{:#?}\n{:#?}", inserted, plan);

        let version = ContractAmendmentVersion::select_all(&*pool).await.unwrap();
        assert_eq!(version.len(), 1);
        // When we convert a version back to its active form, we use
        let version = ContractAmendment::to_active(&version[0]);

        assert_eq!(version, plan, "{:#?}\n{:#?}", version, plan);

        let updatable = ContractAmendment {
            uuid: plan.uuid,
            id: plan.id,
            commission_date: Some(template_day),
            ..ContractAmendment::default()
        };

        let fields = &[ContractAmendment::commission_date];

        // Essentially we perform a "non-update", where everything stays the same, but only citing a
        // limited number of fields. Nevertheless all fields shoudl be updated an returned.
        let mut tx = pool.begin().await.unwrap();
        let updated = updatable
            .update_returning::<_, &str>(Some(fields), None, &mut tx)
            .await
            .unwrap();
        ContractAmendment::insert_version_vec_returning(
            &[updated.clone()],
            &mut tx,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(updated, plan);

        let versions = ContractAmendmentVersion::select_all(&*pool).await.unwrap();
        assert_eq!(versions.len(), 2);

        for v in versions {
            let v = ContractAmendment::to_active(&v);
            assert_eq!(v, plan);
        }
    })
    .await
}
