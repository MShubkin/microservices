//! Тестирование процесса [`get_price_analysis_user`]
use shared_essential::domain::{PricingUnitId, UserType};
use shared_essential::presentation::dto::processing::price_analysis::GetPriceAnalysisUsersReq;

use crate::app_process::{
    price_analysis::get_price_analysis_user, tests::run_db_test,
};

const GET_PRICE_ANALYSIS_USER_EXTRA_MIGS: &[&str] =
    &["price_analysis/get_price_analysis_user.sql"];

#[tokio::test]
async fn get_price_analysis_user_by_id() {
    run_db_test(GET_PRICE_ANALYSIS_USER_EXTRA_MIGS, |pool| async move {
        let req = GetPriceAnalysisUsersReq {
            user_ids: Some(vec![1, 2]),
            unit_ids: None,
            user_types: None,
        };

        let res = get_price_analysis_user(req, pool).await.unwrap();
        let users = res.data;

        assert_eq!(users.len(), 2);
        assert!([1, 2]
            .into_iter()
            .all(|id| users.iter().any(|user| user.id == id)));
    })
    .await;
}

#[tokio::test]
async fn get_price_analysis_user_by_type() {
    run_db_test(GET_PRICE_ANALYSIS_USER_EXTRA_MIGS, |pool| async move {
        let req = GetPriceAnalysisUsersReq {
            user_ids: None,
            unit_ids: None,
            user_types: Some(vec![UserType::Expert]),
        };

        let res = get_price_analysis_user(req, pool).await.unwrap();
        let users = res.data;

        assert_eq!(users.len(), 2);
        assert!([2, 3]
            .into_iter()
            .all(|id| users.iter().any(|user| user.id == id)));
    })
    .await;
}

#[tokio::test]
async fn get_price_analysis_user_by_unit() {
    run_db_test(GET_PRICE_ANALYSIS_USER_EXTRA_MIGS, |pool| async move {
        let req = GetPriceAnalysisUsersReq {
            user_ids: None,
            unit_ids: Some(vec![PricingUnitId::D647]),
            user_types: None,
        };

        let res = get_price_analysis_user(req, pool).await.unwrap();
        let users = res.data;

        assert_eq!(users.len(), 2);
        assert!([1, 2]
            .into_iter()
            .all(|id| users.iter().any(|user| user.id == id)));
    })
    .await;
}

#[tokio::test]
async fn get_price_analysis_user_by_all_filters() {
    run_db_test(GET_PRICE_ANALYSIS_USER_EXTRA_MIGS, |pool| async move {
        let req = GetPriceAnalysisUsersReq {
            user_ids: Some(vec![3]),
            unit_ids: Some(vec![PricingUnitId::Gpk]),
            user_types: Some(vec![UserType::Maintenance]),
        };

        let res = get_price_analysis_user(req, pool).await.unwrap();
        let mut users = res.data;

        assert_eq!(users.len(), 1);
        let user = users.pop().unwrap();
        assert_eq!(user.id, 4);
    })
    .await;
}
