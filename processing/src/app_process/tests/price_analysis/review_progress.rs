use asez2_shared_db::uuid;
use shared_essential::presentation::dto::processing::price_analysis::ReviewProgressReq;

use crate::app_process::{calls::price_analysis::pa_review_progress, tests};

const EXTRA_MIG: &[&str] = &["price_analysis/review_progress.sql"];

#[tokio::test]
async fn pa_review_progress_plan_success() {
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let res = pa_review_progress(
            ReviewProgressReq {
                id: 1,
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            },
            pctx.clone(),
        )
        .await
        .unwrap()
        .data;

        assert_eq!(1, res.len());

        assert_eq!("ppz status 223", res[0].comment);
    })
    .await
}

#[tokio::test]
async fn pa_review_progress_contract_amendment_success() {
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let res = pa_review_progress(
            ReviewProgressReq {
                id: 1,
                uuid: uuid!("00000000-0000-0000-0001-000000000001"),
            },
            pctx.clone(),
        )
        .await
        .unwrap()
        .data;

        assert_eq!(2, res.len());

        assert_eq!("dc status 343", res[0].comment);
        assert_eq!("dc status 353", res[1].comment);
    })
    .await
}
