//! Тестирование процесса [`pricing_result`]
use shared_essential::presentation::dto::{
    processing::{price_analysis::PricingResultReq, ColorScheme, ColorThreshold},
    response_request::{EntityKind, Status},
};
use uuid::Uuid;

use crate::app_process::{
    price_analysis::pa_pricing_result,
    tests::{mock_processing_context, run_db_test},
};

const PRICING_RESULT_EXTRA_MIGS: &[&str] = &["price_analysis/pricing_result.sql"];

#[tokio::test]
async fn test_pa_pricing_result() {
    run_db_test(PRICING_RESULT_EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let request_1 = PricingResultReq {
            id: 1,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            object_type: EntityKind::Plan,
        };

        let request_2 = PricingResultReq {
            id: 2,
            uuid: Uuid::parse_str("00000000-0000-0000-0001-000000000000").unwrap(),
            object_type: EntityKind::ContractAmendment,
        };

        let request_3 = PricingResultReq {
            id: 3,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            object_type: EntityKind::Plan,
        };

        let request_4 = PricingResultReq {
            id: 4,
            uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
            object_type: EntityKind::Plan,
        };

        let r1 = pa_pricing_result(request_1, pool.clone()).await.unwrap();
        let r2 = pa_pricing_result(request_2, pool.clone()).await.unwrap();
        let r3 = pa_pricing_result(request_3, pool.clone()).await.unwrap();
        let r4 = pa_pricing_result(request_4, pool.clone()).await.unwrap();

        {
            assert_eq!(r1.status, Status::Ok, "Ожидается статус Ok");
            assert!(
                r1.data.calculated.savings_in_percent.is_some(),
                "savings_in_percent должно содержать значение"
            );
            assert!(
                r1.data.calculated.number_of_days_with_expert_threshold.is_some(),
                "number_of_days_with_expert должно содержать значение"
            );
            assert!(
                r1.data.calculated.number_of_days_with_expert_threshold
                    == Some(ColorThreshold {
                        value: 44,
                        color_scheme_id: ColorScheme::Red
                    }),
            );
        }
        {
            assert_eq!(r2.status, Status::Ok, "Ожидается статус Ok");
            assert!(
                r2.data.calculated.savings_in_percent.is_some(),
                "savings_in_percent должно содержать значение"
            );
            assert!(
                r2.data.calculated.number_of_days_with_expert_threshold.is_some(),
                "number_of_days_with_expert должно содержать значение"
            );
        }
        {
            assert_eq!(r3.status, Status::Ok, "Ожидается статус Ok");
            assert_eq!(
                r3.data.calculated.savings_in_percent,
                Some("50%".to_string())
            );
        }
        {
            assert_eq!(r4.status, Status::Ok, "Ожидается статус Ok");
            assert_eq!(
                r4.data.calculated.savings_in_percent,
                Some("40%".to_string())
            );
        }
    })
    .await;
}
