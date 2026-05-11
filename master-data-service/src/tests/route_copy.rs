use crate::application::action::route_copy;
use crate::application::routes::fetch_routes;
use crate::tests::db_test::run_db_test;
use asez2_shared_db::db_item::Select;
use shared_essential::domain::routes::RouteHeader;
use shared_essential::presentation::dto::master_data::request::*;
use shared_essential::presentation::dto::response_request::Status;

const ROUTE_COPY_EXTRA_MIGS: &[&str] = &["route_copy.sql"];

#[tokio::test]
async fn basic_route_copy() {
    run_db_test(ROUTE_COPY_EXTRA_MIGS, |pool| async move {
        let select =
            Select::full::<RouteHeader>().eq(RouteHeader::id, 9100000001_i64);
        let source_routes = fetch_routes(select, &pool).await.unwrap();

        assert_eq!(source_routes.len(), 1);

        let source_route_full = source_routes.get(0).unwrap();

        let copy_result = route_copy(
            RouteCopyReq {
                uuid: source_route_full.route.uuid,
                name_short: "new short name".to_string(),
                user_id: 777,
            },
            &pool,
        )
        .await
        .unwrap();
        assert_eq!(copy_result.status, Status::Ok);
        let msg = copy_result.messages.messages.get(0).unwrap();
        assert!(msg.text.contains("Маршрут скопирован"));

        let new_route_id = copy_result.data;

        let select =
            Select::full::<RouteHeader>().eq(RouteHeader::id, new_route_id);
        let copy_routes = fetch_routes(select, &pool).await.unwrap();
        assert_eq!(copy_routes.len(), 1);
        let copy_route_full = copy_routes.get(0).unwrap();

        assert_eq!(copy_route_full.route.id, new_route_id.unwrap());
        assert_ne!(copy_route_full.route.id, source_route_full.route.id);
        assert!(!copy_route_full.route.is_active);
        assert_ne!(
            copy_route_full.route.created_at,
            source_route_full.route.created_at
        );
        assert_ne!(
            copy_route_full.route.changed_at,
            source_route_full.route.changed_at
        );

        assert_eq!(
            copy_route_full.route.name_short.as_ref().unwrap().clone(),
            "new short name".to_string()
        );
        assert_eq!(copy_route_full.route.created_by, 777);
        assert_eq!(copy_route_full.route.changed_by, 777);

        assert_eq!(source_route_full.crits.len(), copy_route_full.crits.len());

        copy_route_full.crits.iter().for_each(|item| {
            assert_eq!(item.route_uuid, copy_route_full.route.uuid);
            assert_eq!(item.created_by, 777);
            assert_eq!(item.changed_by, 777);
        });
        assert_eq!(copy_route_full.data.route_uuid, copy_route_full.route.uuid);
        assert_eq!(copy_route_full.data.created_by, 777);
        assert_eq!(copy_route_full.data.changed_by, 777);
    })
    .await
}
