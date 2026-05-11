use crate::application::action::get_route_list::get_route_list;
use crate::tests::db_test::run_db_test;
use crate::tests::init_master_data_from_pool;
use asez2_shared_db::db_item::{Select, SelectionKind};
use asez2_shared_db::uuid;
use shared_essential::domain::routes::RouteApprType;
use shared_essential::domain::Section;
use shared_essential::presentation::dto::master_data::request::*;

const ROUTE_LIST_EXTRA_MIGS: &[&str] = &["get_route_list.sql"];
const USER1: i32 = 658;

#[tokio::test]
async fn basic() {
    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let req = RouteListReq {
            section: Section::PriceAnalysisAssignExpert,
            select: Select::with_fields([
                "route_id",
                "uuid",
                "pricing_organization_unit_id",
            ])
            .eq("pricing_organization_unit_id", 1)
            .add_replace_order_asc("route_id"),
            user_id: USER1,
            type_id: RouteApprType::PriceAnalysis,
        };

        let res_ok = get_route_list(req, &pool).await;
        let res = res_ok.unwrap();

        assert!(res.messages.messages.is_empty(), "{:#?}", res.messages);
        assert_eq!(res.data.len(), 3, "{:#?}", res.data);
        let ids =
            res.data.iter().map(|x| x.header.route_id.unwrap()).collect::<Vec<_>>();
        assert_eq!(ids, vec![9100000001, 9100000002, 9100000003], "{ids:?}");

        assert!(res
            .data
            .iter()
            .find(|x| x.header.route_id == Some(9100000001))
            .map_or(false, |x| x.is_plan && !x.is_contract_amendment));
        assert!(res
            .data
            .iter()
            .find(|x| x.header.route_id == Some(9100000002))
            .map_or(false, |x| !x.is_plan && x.is_contract_amendment));
        assert!(res
            .data
            .iter()
            .find(|x| x.header.route_id == Some(9100000003))
            .map_or(false, |x| x.is_plan && x.is_contract_amendment));
    })
    .await
}

#[tokio::test]
async fn criteria_filters() {
    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let req = RouteListReq {
            section: Section::PriceAnalysisAssignExpert,
            select: Select::with_fields([
                "route_id",
                "uuid",
                "pricing_organization_unit_id",
            ])
            .eq("object_type_id", 1)
            .in_any("section_id", [500, 1800])
            .eq("pricing_organization_unit_id", 1)
            .add_replace_order_asc("route_id"),
            user_id: USER1,
            type_id: RouteApprType::PriceAnalysis,
        };

        let res_ok = get_route_list(req, &pool).await;
        let res = res_ok.unwrap();

        assert!(res.messages.messages.is_empty(), "{:#?}", res.messages);
        assert_eq!(res.data.len(), 2, "{:#?}", res.data);
        let ids =
            res.data.iter().map(|x| x.header.route_id.unwrap()).collect::<Vec<_>>();
        assert_eq!(ids, vec![9100000001, 9100000003], "{ids:?}");
    })
    .await
}

#[tokio::test]
async fn filter_json() {
    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let req = RouteListReq {
            section: Section::AssignExpertDepartment,
            select: Select::with_fields(["route_id", "uuid", "data"])
                .add_expand_filter(
                    "data",
                    SelectionKind::Jsonpath,
                    ["$.assign_department[*] ? (@.department_id == 20)"],
                ),
            user_id: USER1,
            type_id: RouteApprType::SpecializedDepartments,
        };

        let res_ok = get_route_list(req, &pool).await;
        let res = res_ok.unwrap();

        assert!(res.messages.messages.is_empty(), "{:#?}", res.messages);
        assert_eq!(res.data.len(), 1, "{:#?}", res.data);
        let found = &res.data[0];
        assert_eq!(found.header.route_id, Some(9100000007));
    })
    .await
}

#[tokio::test]
async fn ordering() {
    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let req = RouteListReq {
            section: Section::AssignExpertDepartment,
            select: Select::with_fields(["route_id", "uuid", "data"])
                .add_replace_order_desc("route_id"),
            user_id: USER1,
            type_id: RouteApprType::SpecializedDepartments,
        };

        let res_ok = get_route_list(req, &pool).await;
        let res = res_ok.unwrap();

        assert!(res.messages.messages.is_empty(), "{:#?}", res.messages);

        let mut prev = i64::MAX;
        for id in res.data.iter().map(|x| x.header.route_id.unwrap()) {
            assert!(id < prev);
            prev = id;
        }
    })
    .await
}

#[tokio::test]
async fn filter_object_type() {
    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let req = RouteListReq {
            section: Section::AssignExpertDepartment,
            select: Select::with_fields([
                "route_id",
                "uuid",
                "data",
                "is_plan",
                "is_contract_amendment",
            ])
            .eq("is_plan", true)
            .eq("is_contract_amendment", false),
            user_id: USER1,
            type_id: RouteApprType::PriceAnalysis,
        };

        let res_ok = get_route_list(req, &pool).await;
        let res = res_ok.unwrap();

        assert!(res.messages.messages.is_empty(), "{:#?}", res.messages);
        assert_eq!(res.data.len(), 1, "{:#?}", res.data);
        let found = &res.data[0];
        assert_eq!(
            found.header.uuid,
            Some(uuid!("00000000-0000-0000-0000-000000000001"))
        );
    })
    .await
}

#[tokio::test]
async fn paging() {
    const LIMIT: usize = 3;
    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let req = RouteListReq {
            section: Section::AssignExpertDepartment,
            select: Select::with_fields(["route_id", "uuid", "data"])
                .add_replace_order_desc("route_id"),
            user_id: USER1,
            type_id: RouteApprType::PriceAnalysis,
        };

        let mut reqp = req.clone();
        reqp.select = reqp.select.take_n(LIMIT).offset(0).count_total(true);

        let res = get_route_list(req, &pool).await.unwrap();
        let all_routes = res.data.item_list;
        assert!(all_routes.len() > LIMIT);
        let mut paged_routes = vec![];
        loop {
            let res = get_route_list(reqp.clone(), &pool).await.unwrap();
            let routes = res.data.item_list;
            let len = routes.len();
            if len == 0 {
                break;
            }

            paged_routes.extend(routes);
            if let Some(true) = reqp.select.count_total {
                assert_eq!(res.data.total, Some(all_routes.len()));
            }
            reqp.select.count_total = None;
            *reqp.select.offset.as_mut().unwrap() += len;

            if paged_routes.len() != all_routes.len() {
                assert_eq!(len, LIMIT);
            } else {
                assert!(len <= LIMIT);
            }
        }
        assert_eq!(all_routes, paged_routes);
    })
    .await
}
