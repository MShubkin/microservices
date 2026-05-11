use ahash::AHashMap;
use serde::Serialize;
use shared_essential::{
    domain::{
        maths::CurrencyValue,
        routes::{CritValue, RouteApprType},
        PlanRep, PricingUnitId,
    },
    presentation::dto::master_data::request::{
        CritArg, ItemWithExtraFields, RouteFindReqItem,
    },
};

use crate::{
    application::action::route_find,
    presentation::dto::{RouteFindItem, RouteFindReqBody},
    tests::init_master_data_from_pool,
};

use super::db_test::run_db_test;

const ROUTE_FIND_EXTRA_MIGS: &[&str] = &["route_find.sql"];

fn item_to_fields<T: Serialize>(item: &T) -> RouteFindItem {
    use serde_json::*;
    from_slice(&to_vec(item).expect("serialized")).expect("deserialized")
}

#[tokio::test]
async fn test_route_find() {
    run_db_test(ROUTE_FIND_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let matched_plan = PlanRep {
            pricing_organization_unit_id: Some(PricingUnitId::D646),
            customer_id: Some(1500),
            section_id: Some(100),
            sum_excluded_vat: Some(CurrencyValue::from(10000.00)),
            purchasing_type_id: Some(200),
            budget_item_id: Some(800),
            ..Default::default()
        };
        let unmatched_plan = PlanRep {
            pricing_organization_unit_id: Some(PricingUnitId::D646),
            customer_id: Some(1500),
            section_id: Some(600),
            sum_excluded_vat: Some(CurrencyValue::from(10000.00)),
            purchasing_type_id: Some(200),
            budget_item_id: Some(800),
            ..Default::default()
        };

        let req = RouteFindReqBody {
            type_id: RouteApprType::PriceAnalysis,
            item_list: vec![
                RouteFindReqItem {
                    id: 10,
                    item: item_to_fields(&matched_plan),
                },
                RouteFindReqItem {
                    id: 20,
                    item: item_to_fields(&unmatched_plan),
                },
            ],
        };

        let res = route_find(req, &pool).await;
        assert!(res.is_ok(), "{res:?}");

        let res = res.unwrap();
        assert!(res.messages.is_empty(), "{messages:?}", messages = res.messages);
        assert_eq!(res.data[0].item_list.len(), 1, "{:?}", res.data[0]);
        assert!(res.data[1].item_list.is_empty(), "{:?}", res.data[0]);
    })
    .await
}

#[tokio::test]
async fn test_route_find_agg() {
    run_db_test(ROUTE_FIND_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let matched_plan = ItemWithExtraFields {
            item: PlanRep {
                pricing_organization_unit_id: Some(PricingUnitId::D646),
                customer_id: Some(1500),
                section_id: Some(100),
                sum_excluded_vat: Some(CurrencyValue::from(10000.00)),
                purchasing_type_id: Some(200),
                budget_item_id: Some(800),
                ..Default::default()
            },
            extra: AHashMap::from_iter([(
                "okdp2".to_string(),
                CritArg::AnyAgg(vec![
                    CritValue::String("130".to_string()),
                    CritValue::String("140".to_string()),
                ]),
            )]),
        };
        let unmatched_plan = ItemWithExtraFields {
            item: PlanRep {
                pricing_organization_unit_id: Some(PricingUnitId::D646),
                customer_id: Some(1500),
                section_id: Some(100),
                sum_excluded_vat: Some(CurrencyValue::from(10000.00)),
                purchasing_type_id: Some(200),
                budget_item_id: Some(800),
                ..Default::default()
            },
            extra: AHashMap::from_iter([(
                "okdp2".to_string(),
                CritArg::AllAgg(vec![
                    CritValue::String("950".to_string()),
                    CritValue::String("960".to_string()),
                ]),
            )]),
        };

        let req = RouteFindReqBody {
            type_id: RouteApprType::PriceAnalysis,
            item_list: vec![
                RouteFindReqItem {
                    id: 10,
                    item: item_to_fields(&matched_plan),
                },
                RouteFindReqItem {
                    id: 20,
                    item: item_to_fields(&unmatched_plan),
                },
            ],
        };

        let res = route_find(req, &pool).await;
        assert!(res.is_ok(), "{res:?}");

        let res = res.unwrap();
        assert!(res.messages.is_empty(), "{messages:?}", messages = res.messages);
        assert_eq!(res.data[0].item_list.len(), 1, "{:?}", res.data[0]);
        assert!(res.data[1].item_list.is_empty(), "{:?}", res.data[0]);
    })
    .await
}

#[tokio::test]
async fn test_route_find_hierarchy() {
    run_db_test(ROUTE_FIND_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let matched_plan = PlanRep {
            pricing_organization_unit_id: Some(PricingUnitId::D646),
            customer_id: Some(1500),
            section_id: Some(100),
            sum_excluded_vat: Some(CurrencyValue::from(10000.00)),
            purchasing_type_id: Some(200),
            okdp2: Some(Some("130".to_string())),
            budget_item_id: Some(800),
            ..Default::default()
        };
        let unmatched_plan = PlanRep {
            pricing_organization_unit_id: Some(PricingUnitId::D646),
            customer_id: Some(1500),
            section_id: Some(100),
            sum_excluded_vat: Some(CurrencyValue::from(10000.00)),
            purchasing_type_id: Some(200),
            okdp2: Some(Some("135".to_string())),
            budget_item_id: Some(800),
            ..Default::default()
        };

        let req = RouteFindReqBody {
            type_id: RouteApprType::PriceAnalysis,
            item_list: vec![
                RouteFindReqItem {
                    id: 10,
                    item: item_to_fields(&matched_plan),
                },
                RouteFindReqItem {
                    id: 20,
                    item: item_to_fields(&unmatched_plan),
                },
            ],
        };

        let res = route_find(req, &pool).await;
        assert!(res.is_ok(), "{res:?}");

        let res = res.unwrap();
        assert!(res.messages.is_empty(), "{messages:?}", messages = res.messages);
        assert_eq!(res.data[0].item_list.len(), 1, "{:?}", res.data[0]);
        assert!(res.data[1].item_list.is_empty(), "{:?}", res.data[0]);
    })
    .await
}

#[tokio::test]
async fn none_criterion() {
    run_db_test(ROUTE_FIND_EXTRA_MIGS, |pool| async move {
        init_master_data_from_pool(&pool).await;
        let matched_plan = PlanRep {
            section_id: Some(100),
            ..Default::default()
        };

        let req = RouteFindReqBody {
            type_id: RouteApprType::SpecializedDepartments,
            item_list: vec![RouteFindReqItem {
                id: 10,
                item: item_to_fields(&matched_plan),
            }],
        };

        let res = route_find(req, &pool).await;
        assert!(res.is_ok(), "{res:?}");

        let res = res.unwrap();
        assert!(res.messages.is_empty(), "{messages:?}", messages = res.messages);
        assert_eq!(res.data.len(), 1);
        // маршруты с none и отсутствующим критерием section_id
        assert_eq!(res.data[0].item_list.len(), 2, "{:?}", res.data[0]);
    })
    .await
}
