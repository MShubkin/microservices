use asez2_shared_db::DbItem;
use serde_json::json;
use shared_essential::domain::routes::{
    AutoAssignDepartmentDivision, AutoAssignDepartmentItem, RouteApprType,
    RouteDataContent, RouteHeader, RouteHeaderRep,
};
use shared_essential::domain::DepartmentLevel;
use shared_essential::presentation::dto::master_data::request::RouteUpdateReq;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, MessageKind, Messages,
};

use crate::application::action::route_create::{route_create, RouteCreateMessage};
use crate::tests::db_test::run_db_test;

const ROUTE_LIST_EXTRA_MIGS: &[&str] = &["route_create.sql"];

#[tokio::test]
async fn basic_create() {
    let user_id = 666;

    let data = AutoAssignDepartmentItem {
        department_id: 10,
        division: Some(AutoAssignDepartmentDivision {
            id: 20,
            level: DepartmentLevel::Division,
        }),
    };
    let req = RouteUpdateReq {
        user_id,
        header: RouteHeaderRep {
            name_short: Some(Some("имя маршрута".to_string())),
            is_active: Some(false),
            type_id: Some(RouteApprType::SpecializedDepartments),
            ..Default::default()
        },
        criteria: serde_json::from_value(json!({
            "pricing_organization_unit_id": [{"operator": "equal", "filter_values": [3]}],
            "section_id": [{"operator" : "not_equal", "filter_values": [600]}]
        }))
        .expect("ok"),
        data: RouteDataContent::AssignDepartment(vec![data].into())
    };

    run_db_test(ROUTE_LIST_EXTRA_MIGS, move |pool| async move {
        let old_routes =
            RouteHeader::select_all(&*pool).await.expect("select before");

        let res_ok = route_create(req, &pool).await;
        let res = res_ok.unwrap();

        let routes = RouteHeader::select_all(&*pool).await.expect("select after");
        assert_eq!(routes.len(), old_routes.len() + 1);

        let new_route = routes.iter().find(|r| r.created_by == user_id).unwrap();

        let expected_messages = Messages {
            kind: MessageKind::Success,
            messages: vec![RouteCreateMessage::Ok.singular(new_route)],
        };
        assert_eq!(res.messages, expected_messages);

        assert_eq!(res.data.route_id, Some(9100000000 + routes.len() as i64));
    })
    .await
}
