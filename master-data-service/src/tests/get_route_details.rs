use shared_essential::presentation::dto::master_data::request::*;

use crate::application::action::get_route_details;
use crate::tests::db_test::run_db_test;

const ROUTE_LIST_EXTRA_MIGS: &[&str] = &["get_route_details.sql"];

#[tokio::test]
async fn basic() {
    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        let req = RouteDetailsReq {
            route_id: 9100000001,
        };

        let res_ok = get_route_details(req, &pool).await;
        let res = res_ok.unwrap();

        assert!(res.messages.messages.is_empty(), "{:#?}", res.messages);
        assert_eq!(res.data.header.route_id, Some(9100000001));
    })
    .await
}
