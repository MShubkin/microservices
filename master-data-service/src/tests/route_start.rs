use asez2_shared_db::{db_item::Select, uuid, DbItem};
use shared_essential::{
    domain::routes::{RouteApprType, RouteHeader},
    presentation::dto::{
        master_data::request::RouteStartReq, response_request::BusinessMessage,
    },
};

use crate::application::{
    action::route_start::{route_start, RouteStartMessage},
    routes::{CoincidentRoute, RoutesNotFound},
};
use crate::tests::db_test::run_db_test;

const ROUTE_START_EXTRA_MIGS: &[&str] = &["route_start.sql"];

#[tokio::test]
async fn test_route_start_pa() {
    run_db_test(ROUTE_START_EXTRA_MIGS, |pool| async move {
        sqlx::query("update public.route_list set type_id = 2;")
            .execute(&*pool)
            .await
            .unwrap();

        let req = RouteStartReq {
            items: vec![
                uuid!("00000000-0000-0000-0000-000000000001"),
                uuid!("00000000-0000-0000-0000-000000000002"),
                uuid!("00000000-0000-0000-0000-000000000003"),
            ],
            type_id: RouteApprType::PriceAnalysis,
        };

        let res = route_start(req.clone(), &pool).await.unwrap();

        let expected_messages = vec![
            RouteStartMessage::AlreadyActive.singular(&RouteHeader {
                id: 9100000002,
                uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                type_id: RouteApprType::PriceAnalysis,
                ..Default::default()
            }),
            RouteStartMessage::Activated.plural(&[
                RouteHeader {
                    id: 9100000001,
                    uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                    type_id: RouteApprType::PriceAnalysis,
                    ..Default::default()
                },
                RouteHeader {
                    id: 9100000003,
                    uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                    type_id: RouteApprType::PriceAnalysis,
                    ..Default::default()
                },
            ]),
        ];
        assert_eq!(res.messages.messages, expected_messages);

        let routes = RouteHeader::select(
            &Select::full::<RouteHeader>().in_any(RouteHeader::uuid, req.items),
            &*pool,
        )
        .await
        .unwrap();
        assert!(routes.iter().all(|r| r.is_active), "{:#?}", routes);
    })
    .await
}

#[tokio::test]
async fn test_route_start_sd() {
    run_db_test(ROUTE_START_EXTRA_MIGS, |pool| async move {
        let req = RouteStartReq {
            items: vec![
                uuid!("00000000-0000-0000-0000-000000000001"),
                uuid!("00000000-0000-0000-0000-000000000002"),
                // Схож по данным и критериям с 00000000-0000-0000-0000-000000000006
                uuid!("00000000-0000-0000-0000-000000000003"),
                // Схож по данным с 00000000-0000-0000-0000-000000000006, но не по критериям
                uuid!("00000000-0000-0000-0000-000000000004"),
                // Схож по критериям с 00000000-0000-0000-0000-000000000006, но не по данным
                uuid!("00000000-0000-0000-0000-000000000005"),
                uuid!("00000000-0000-0000-0000-000000000013"),
                uuid!("00000000-0000-0000-0000-000000000023"),
                // Схож по данным и критериям с 00000000-0000-0000-0000-000000000008, но с разным порядком идентификаторов
                uuid!("00000000-0000-0000-0000-000000000007"),
            ],
            type_id: RouteApprType::SpecializedDepartments,
        };

        let res = route_start(req.clone(), &pool).await.unwrap();

        let x = CoincidentRoute::new(
            9100000003,
            vec![9100000006],
            RouteApprType::SpecializedDepartments,
        );
        let x1 = CoincidentRoute::new(
            9100000007,
            vec![9100000008],
            RouteApprType::SpecializedDepartments,
        );

        let exp_messages = vec![
            RoutesNotFound::from_iter([
                uuid!("00000000-0000-0000-0000-000000000013"),
                uuid!("00000000-0000-0000-0000-000000000023"),
            ])
            .into_message(),
            RouteStartMessage::AlreadyActive.singular(&RouteHeader {
                id: 9100000002,
                uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                type_id: RouteApprType::SpecializedDepartments,
                ..Default::default()
            }),
            RouteStartMessage::SimilarActiveFound(vec![x, x1]).plural(&[
                RouteHeader {
                    id: 9100000003,
                    uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                    type_id: RouteApprType::SpecializedDepartments,
                    ..Default::default()
                },
                RouteHeader {
                    id: 9100000007,
                    uuid: uuid!("00000000-0000-0000-0000-000000000007"),
                    type_id: RouteApprType::SpecializedDepartments,
                    ..Default::default()
                },
            ]),
            RouteStartMessage::Activated.plural(&[
                &RouteHeader {
                    id: 9100000001,
                    uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                    type_id: RouteApprType::SpecializedDepartments,
                    ..Default::default()
                },
                &RouteHeader {
                    id: 9100000004,
                    uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                    type_id: RouteApprType::SpecializedDepartments,
                    ..Default::default()
                },
                &RouteHeader {
                    id: 9100000005,
                    uuid: uuid!("00000000-0000-0000-0000-000000000005"),
                    type_id: RouteApprType::SpecializedDepartments,
                    ..Default::default()
                },
            ]),
        ];
        let out_msgs = &res.messages.messages;
        assert_eq!(
            out_msgs, &exp_messages,
            "real:{:#?}\nexp:{:#?}",
            out_msgs, &exp_messages
        );

        let routes = RouteHeader::select(
            &Select::full::<RouteHeader>().in_any(RouteHeader::uuid, req.items),
            &*pool,
        )
        .await
        .unwrap();

        let activated_routes = RouteHeader::select(
            &Select::full::<RouteHeader>().in_any(RouteHeader::id, [1, 2]),
            &*pool,
        )
        .await
        .unwrap();
        assert!(!routes.iter().all(|r| r.is_active), "{:#?}", routes);
        assert!(
            activated_routes.iter().all(|r| r.is_active),
            "{:#?}",
            activated_routes
        );
    })
    .await
}
