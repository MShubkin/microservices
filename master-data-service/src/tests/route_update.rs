use asez2_shared_db::db_item::Select;
use asez2_shared_db::{uuid, DbItem};
use serde_json::json;
use shared_essential::domain::routes::{
    AutoAssignDepartmentDivision, AutoAssignDepartmentItem, RouteApprType,
    RouteCrit, RouteDataContent, RouteHeader, RouteHeaderRep,
};
use shared_essential::domain::DepartmentLevel;
use shared_essential::presentation::dto::master_data::error::MasterDataError;
use shared_essential::presentation::dto::master_data::request::RouteUpdateReq;
use shared_essential::presentation::dto::response_request::{
    BusinessMessage, EntityKind, MessageKind, Messages,
};

use crate::application::action::route_update::{route_update, RouteUpdateMessage};
use crate::application::routes::CoincidentRoute;
use crate::tests::db_test::run_db_test;

const ROUTE_LIST_EXTRA_MIGS: &[&str] = &["route_update.sql"];

#[tokio::test]
async fn basic_update() {
    let data = AutoAssignDepartmentItem {
        department_id: 10,
        division: Some(AutoAssignDepartmentDivision {
            id: 20,
            level: DepartmentLevel::Division,
        }),
    };
    let req = RouteUpdateReq {
        user_id: 666,
        header: RouteHeaderRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
            route_id: Some(9100000001),
            is_active: Some(false),
            ..Default::default()
        },
        criteria: serde_json::from_value(json!({
            "pricing_organization_unit_id": [{"operator": "equal", "filter_values": [3]}],
            "section_id": [{"operator" : "not_equal", "filter_values": [600]}]
        }))
        .expect("ok"),
        data: RouteDataContent::AssignDepartment(vec![data].into()),
    };

    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        let res_ok = route_update(req.clone(), &pool).await;
        let res = res_ok.unwrap();
        assert_eq!(res.messages.kind, MessageKind::Success);
        let param = &res.messages.messages[0].parameters.item_list[0];
        assert_eq!(&param.id, "9100000001");
        assert_eq!(param.kind, EntityKind::SpecializedDepartmentRoute);

        let updated_crits = RouteCrit::select(
            &Select::full::<RouteCrit>()
                .eq(RouteCrit::route_uuid, req.header.uuid.unwrap())
                .in_any(RouteCrit::field_name, req.criteria.keys()),
            &*pool,
        )
        .await
        .unwrap();

        assert!(updated_crits.iter().all(|c| !c.is_removed
            && c.route_uuid == req.header.uuid.unwrap()
            && c.changed_by == req.user_id
            && c.created_by == 0));

        let removed_crits = RouteCrit::select(
            &Select::full::<RouteCrit>()
                .eq(RouteCrit::route_uuid, req.header.uuid.unwrap())
                .not_in_any(RouteCrit::field_name, req.criteria.keys()),
            &*pool,
        )
        .await
        .unwrap();

        // Старые криетрии должны быть удалены
        assert!(removed_crits.iter().all(|c| c.is_removed
            && c.changed_by == req.user_id
            && c.created_by == 0));
    })
    .await
}

#[tokio::test]
async fn with_coincident_route() {
    let data = AutoAssignDepartmentItem {
        department_id: Default::default(),
        division: Default::default(),
    };
    let req = RouteUpdateReq {
        user_id: 666,
        header: RouteHeaderRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000002")),
            route_id: Some(9100000002),
            is_active: Some(true),
            ..Default::default()
        },
        criteria: Default::default(),
        data: RouteDataContent::AssignDepartment(vec![data].into()),
    };

    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        let err = route_update(req.clone(), &pool).await.unwrap_err();
        let MasterDataError::Business(msgs) = err else {
            panic!("Была возвращена не та ошибка: {}", err)
        };

        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![RouteUpdateMessage::SimilarActiveFound(
                CoincidentRoute {
                    own: 9100000002,
                    type_id: RouteApprType::SpecializedDepartments,
                    others: vec![9100000003],
                },
            )
            .singular(&RouteHeader {
                id: 9100000002,
                ..Default::default()
            })],
        };

        assert_eq!(msgs, expected_messages);
    })
    .await
}

/// Случай, когда пользователь обновляет копию маршрута, которая обычно неактивна.
/// В таком случае проверка на дублирующие маршруты не должна быть задействована
#[tokio::test]
async fn update_non_active() {
    let data = AutoAssignDepartmentItem {
        department_id: 10,
        division: Some(AutoAssignDepartmentDivision {
            id: 20,
            level: DepartmentLevel::Division,
        }),
    };
    let req = RouteUpdateReq {
        user_id: 666,
        header: RouteHeaderRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000004")),
            route_id: Some(9100000004),
            is_active: Some(false),
            ..Default::default()
        },
        criteria: serde_json::from_value(json!({
            "pricing_organization_unit_id": [{"operator": "equal", "filter_values": [3]}],
            "section_id": [{"operator" : "not_equal", "filter_values": [600]}]
        }))
        .expect("ok"),
        data: RouteDataContent::AssignDepartment(vec![data].into()),
    };

    run_db_test(ROUTE_LIST_EXTRA_MIGS, |pool| async move {
        let res = route_update(req.clone(), &pool).await.unwrap();

        let expected_messages = Messages {
            kind: MessageKind::Success,
            messages: vec![RouteUpdateMessage::Ok.singular(&RouteHeader {
                id: 9100000004,
                uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                type_id: RouteApprType::SpecializedDepartments,
                ..Default::default()
            })],
        };

        assert_eq!(res.messages, expected_messages);
    })
    .await
}
