use asez2_shared_db::{db_item::Select, uuid, DbItem};
use shared_essential::domain::routes::RouteApprType;
use shared_essential::{
    domain::routes::RouteHeader,
    presentation::dto::response_request::BusinessMessage,
};

use crate::application::action::route_stop::{route_stop, RouteStopMessage};
use crate::application::routes::RoutesNotFound;
use crate::tests::db_test::run_db_test;

const ROUTE_STOP_EXTRA_MIGS: &[&str] = &["route_start.sql"];

#[tokio::test]
async fn test_route_stop() {
    run_db_test(ROUTE_STOP_EXTRA_MIGS, |pool| async move {
        let req = vec![
            uuid!("00000000-0000-0000-0000-000000000001"),
            uuid!("00000000-0000-0000-0000-000000000002"),
            uuid!("00000000-0000-0000-0000-000000000003"),
            uuid!("00000000-0000-0000-0000-000000000013"),
        ];

        let res = route_stop(req.clone(), &pool).await.unwrap();

        let expected_messages = vec![
            RoutesNotFound::from_iter([uuid!(
                "00000000-0000-0000-0000-000000000013"
            )])
            .into_message(),
            RouteStopMessage::AlreadyInactive.plural(&[
                RouteHeader {
                    id: 9100000001,
                    uuid: uuid!("00000000-0000-0000-0000-000000000001"),
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
            RouteStopMessage::Deactivated.singular(&RouteHeader {
                id: 9100000002,
                uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                type_id: RouteApprType::SpecializedDepartments,
                ..Default::default()
            }),
        ];
        assert_eq!(res.messages.messages, expected_messages);

        let routes = RouteHeader::select(
            &Select::full::<RouteHeader>().in_any(RouteHeader::uuid, req),
            &*pool,
        )
        .await
        .unwrap();
        assert!(routes.iter().all(|r| !r.is_active), "{:#?}", routes);
    })
    .await
}
