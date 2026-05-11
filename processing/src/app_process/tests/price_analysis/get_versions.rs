use crate::app_process::price_analysis::get_complete::{
    get_contract_amendment_version, get_plan_version,
};
use crate::app_process::tests::run_db_test;

use asez2_shared_db::db_item::AsezTimestamp;
use shared_essential::presentation::dto::processing::{
    PlanVersionRequest, VersionInfo,
};

const EXTRA_MIGS: &[&str] = &["price_analysis/get_complete_versions.sql"];

#[tokio::test]
async fn test_get_plan_version() {
    let req = PlanVersionRequest {
        plan_id: 1,
        user_id: 1,
        version: 1,
    };
    let exp_versions = vec![
        VersionInfo {
            pricing_version: Some(1),
            is_active: false,
            pricing_expert_id: Some(1),
            expert_conclusion_id: None,
            pricing_created_at: Some(
                AsezTimestamp::try_from("1999-09-09 00:00:00").unwrap(),
            ),
            sum_excluded_vat: 4.into(),
            sum_included_vat: 0.into(),
            sum_excluded_vat_rub: 5.into(),
            sum_included_vat_rub: 0.into(),
        },
        VersionInfo {
            pricing_version: None,
            is_active: true,
            pricing_expert_id: Some(1),
            expert_conclusion_id: None,
            pricing_created_at: None,
            sum_excluded_vat: 4.into(),
            sum_included_vat: 0.into(),
            sum_excluded_vat_rub: 5.into(),
            sum_included_vat_rub: 0.into(),
        },
    ];
    run_db_test(EXTRA_MIGS, |pool| async move {
        let mut res = get_plan_version(req, pool).await.unwrap();

        assert_eq!(res.data.total, Some(1));
        assert_eq!(res.data.item_list.len(), 1);

        let data = res.data.item_list.pop().unwrap();
        assert_eq!(data.plan.id, Some(1));
        assert_eq!(data.items.len(), 1);
        assert_eq!(data.versions.len(), 2);
        assert_eq!(data.versions, exp_versions);
        assert!(
            data.items.iter().all(|x| x.description_internal.is_some()),
            "Field 'description_internal' is missing."
        );
        assert!(
            data.items.iter().all(|x| x.number.is_some()),
            "Field 'number' is missing."
        );
    })
    .await
}

#[tokio::test]
async fn test_get_ca_version() {
    let req = PlanVersionRequest {
        plan_id: 2,
        user_id: 1,
        version: 1,
    };
    let exp_versions = vec![
        VersionInfo {
            pricing_version: Some(1),
            is_active: false,
            pricing_expert_id: Some(1),
            expert_conclusion_id: None,
            pricing_created_at: Some(
                AsezTimestamp::try_from("1999-09-09 00:00:00").unwrap(),
            ),
            sum_excluded_vat: 1.into(),
            sum_included_vat: 0.into(),
            sum_excluded_vat_rub: 2.into(),
            sum_included_vat_rub: 0.into(),
        },
        VersionInfo {
            pricing_version: None,
            is_active: true,
            pricing_expert_id: Some(1),
            expert_conclusion_id: None,
            pricing_created_at: None,
            sum_excluded_vat: 1.into(),
            sum_included_vat: 0.into(),
            sum_excluded_vat_rub: 2.into(),
            sum_included_vat_rub: 0.into(),
        },
    ];
    run_db_test(EXTRA_MIGS, |pool| async move {
        let mut res = get_contract_amendment_version(req, pool).await.unwrap();

        assert_eq!(res.data.total, Some(1));
        assert_eq!(res.data.item_list.len(), 1);

        let data = res.data.item_list.pop().unwrap();
        assert_eq!(data.plan.id, Some(2));
        assert_eq!(data.items.len(), 2, "{:#?}", data.items);
        assert_eq!(data.versions.len(), 2);
        assert_eq!(data.versions, exp_versions);
        assert!(
            data.items.iter().all(|x| x.description_internal.is_some()),
            "Field 'description_internal' is missing."
        );
        assert!(
            data.items.iter().all(|x| x.number.is_some()),
            "Field 'number' is missing."
        );
    })
    .await
}
