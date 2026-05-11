use asez2_tables::{
    processing::price_analysis_user::UserType, PricingUnitId, Section,
};

use shared_essential::presentation::dto::processing::price_analysis::{
    GetSectionsCountRequest, GetSectionsCountResponse,
};

use crate::app_process::{
    price_analysis::pa_get_sections_count, tests::run_db_test,
};

const GET_SECTIONS_COUNT_EXTRA_MIGS: &[&str] =
    &["price_analysis/get_sections_count.sql"];
const USER_ID_1: i32 = 1;
const USER_ID_2: i32 = 2;
const USER_ID_3: i32 = 3;

#[tokio::test]
async fn test_get_sections_count() {
    run_db_test(GET_SECTIONS_COUNT_EXTRA_MIGS, |pool| async move {
        // Начальник АЦ
        let dto_1 = GetSectionsCountRequest {
            section_list: vec![
                Section::PriceAnalysisAssignExpert,
                Section::PriceAnalysisDeterminePrice,
                Section::PriceAnalysisPrimaryExpertControl,
                Section::PriceAnalysisApprovePrice,
                Section::PriceAnalysisLottingMTP,
            ],
            departments: vec![PricingUnitId::D646, PricingUnitId::D647],
            user_type: UserType::Director,
            user_id: USER_ID_1,
        };

        // Эксперт АЦ с ППЗ/ДС
        let dto_2 = GetSectionsCountRequest {
            section_list: vec![
                Section::PriceAnalysisAssignExpert,
                Section::PriceAnalysisDeterminePrice,
                Section::PriceAnalysisPrimaryExpertControl,
                Section::PriceAnalysisApprovePrice,
                Section::PriceAnalysisLottingMTP,
            ],
            departments: vec![PricingUnitId::D646, PricingUnitId::D647],
            user_type: UserType::Expert,
            user_id: USER_ID_2,
        };

        // Эксперт АЦ без ППЗ/ДС
        let dto_3 = GetSectionsCountRequest {
            section_list: vec![
                Section::PriceAnalysisAssignExpert,
                Section::PriceAnalysisDeterminePrice,
                Section::PriceAnalysisPrimaryExpertControl,
                Section::PriceAnalysisApprovePrice,
                Section::PriceAnalysisLottingMTP,
            ],
            departments: vec![PricingUnitId::D646, PricingUnitId::D647],
            user_type: UserType::Expert,
            user_id: USER_ID_3,
        };

        // Такое же как и 1, но только с 1 секцией в запросе
        let dto_4 = GetSectionsCountRequest {
            section_list: vec![Section::PriceAnalysisAssignExpert],
            departments: vec![PricingUnitId::D646, PricingUnitId::D647],
            user_type: UserType::Director,
            user_id: USER_ID_1,
        };

        let result_1 = pa_get_sections_count(dto_1, pool.clone()).await.unwrap();
        let result_2 = pa_get_sections_count(dto_2, pool.clone()).await.unwrap();
        let result_3 = pa_get_sections_count(dto_3, pool.clone()).await.unwrap();
        let result_4 = pa_get_sections_count(dto_4, pool.clone()).await.unwrap();

        let expect_1 = GetSectionsCountResponse {
            assign_expert: Some(2),
            determine_price: Some(2),
            primary_expert_control: Some(2),
            approve_price: Some(2),
            lotting_mtr: Some(2),
        };
        let expect_2 = GetSectionsCountResponse {
            assign_expert: Some(2),
            determine_price: Some(0),
            primary_expert_control: Some(2),
            approve_price: Some(0),
            lotting_mtr: Some(1),
        };
        let expect_3 = GetSectionsCountResponse {
            assign_expert: Some(0),
            determine_price: Some(2),
            primary_expert_control: Some(0),
            approve_price: Some(2),
            lotting_mtr: Some(1),
        };
        let expect_4 = GetSectionsCountResponse {
            assign_expert: Some(2),
            determine_price: None,
            primary_expert_control: None,
            approve_price: None,
            lotting_mtr: None,
        };

        assert_eq!(result_1.data, expect_1, "{:#?}", result_1.data);
        assert_eq!(result_2.data, expect_2, "{:#?}", result_2.data);
        assert_eq!(result_3.data, expect_3, "{:#?}", result_3.data);
        assert_eq!(result_4.data, expect_4, "{:#?}", result_4.data);
    })
    .await;
}
