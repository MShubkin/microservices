use asez2_shared_db::{db_item::Select, uuid, DbItem};
use shared_essential::domain::routes::RouteApprType;
use shared_essential::{
    domain::routes::RouteHeader,
    presentation::dto::response_request::BusinessMessage,
};

use crate::application::action::route_remove::{route_remove, RouteRemoveMessage};
use crate::application::routes::RoutesNotFound;
use crate::tests::db_test::run_db_test;

const ROUTE_REMOVE_EXTRA_MIGS: &[&str] = &["route_remove.sql"];

#[tokio::test]
async fn test_route_remove() {
    run_db_test(ROUTE_REMOVE_EXTRA_MIGS, |pool| async move {
        let req = vec![
            uuid!("00000000-0000-0000-0000-000000000001"),
            uuid!("00000000-0000-0000-0000-000000000002"),
            uuid!("00000000-0000-0000-0000-000000000003"),
            uuid!("00000000-0000-0000-0000-000000000004"),
            uuid!("00000000-0000-0000-0000-000000000014"),
        ];

        let res = route_remove(req.clone(), &pool).await.unwrap();

        let expected_messages = vec![
            RoutesNotFound::from_iter([uuid!(
                "00000000-0000-0000-0000-000000000014"
            )])
            .into_message(),
            RouteRemoveMessage::IsActive.singular(&RouteHeader {
                id: 9100000001,
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                type_id: RouteApprType::SpecializedDepartments,
                ..Default::default()
            }),
            RouteRemoveMessage::AlreadyRemoved.singular(&RouteHeader {
                id: 9100000004,
                uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                type_id: RouteApprType::SpecializedDepartments,
                ..Default::default()
            }),
            RouteRemoveMessage::Success.plural(&[
                RouteHeader {
                    id: 9100000002,
                    uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                    type_id: RouteApprType::SpecializedDepartments,
                    ..Default::default()
                },
                RouteHeader {
                    id: 9100000003,
                    uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                    type_id: RouteApprType::SpecializedDepartments,
                    ..Default::default()
                },
            ]),
        ];
        assert_eq!(res.messages.messages, expected_messages);

        let routes = RouteHeader::select(
            &Select::full::<RouteHeader>().in_any(RouteHeader::uuid, req),
            &*pool,
        )
        .await
        .unwrap();

        let removed_routes: Vec<&RouteHeader> =
            routes.iter().filter(|r| r.is_removed && !r.is_active).collect();
        let not_removed_routes: Vec<&RouteHeader> =
            routes.iter().filter(|r| !r.is_removed).collect();

        assert_eq!(
            removed_routes.len(),
            3,
            "Ожидалось, что будет удалено 2 маршрута, но оказалось в общем {:#?}",
            removed_routes
        );
        assert_eq!(
            not_removed_routes.len(),
            1,
            "Ожидался 1 не удаленный маршрут, но оказалось {:#?}",
            not_removed_routes
        );
    })
    .await
}
