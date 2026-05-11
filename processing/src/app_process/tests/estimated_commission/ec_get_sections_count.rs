use asez2_tables::Section;
use shared_essential::presentation::dto::processing::{
    GetSectionsCountRequest, GetSectionsCountResponse, UserIdWrapper,
};

use crate::app_process::{ec_get_sections_count, tests::run_db_test};

const GET_SECTIONS_COUNT_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_sections_count.sql"];

// TODO test is incomplete
#[tokio::test]
async fn test_get_sections_count() {
    run_db_test(GET_SECTIONS_COUNT_EXTRA_MIGS, |pool| async move {
        // let dto_1 = GetSectionsCountRequest {
        //     section_list: vec![
        //         Section::EstimatedCommissionInPerson,
        //         Section::EstimatedCommissionCorrespondence,
        //         Section::EstimatedCommissionNotRequired,
        //         Section::EstimatedCommissionInPersonPreparation,
        //         Section::EstimatedCommissionSummingUpInPerson,
        //         Section::EstimatedCommissionSummingUpCorrespondence,
        //     ],
        // };

        let dto_2 = UserIdWrapper {
            user_id: 1,
            dto: GetSectionsCountRequest {
                section_list: vec![
                    Section::EstimatedCommissionSummingUpCorrespondence,
                ],
            },
        };

        // let result_1 = ec_get_sections_count(dto_1, pctx.into()).await.unwrap();
        let result_2 = ec_get_sections_count(dto_2, pool.clone()).await.unwrap();

        // let expect_1 = GetSectionsCountResponse {
        //     in_person_commission: Some(1),
        //     correspondence_commission: Some(1),
        //     no_commission_required: Some(1),
        //     preparation_for_in_person_commission: Some(2),
        //     summing_up_in_person_commission_results: Some(3),
        //     summing_up_correspondence_commission_results: Some(3),
        // };

        let expect_2 = GetSectionsCountResponse {
            in_person_commission: None,
            correspondence_commission: None,
            no_commission_required: None,
            preparation_for_in_person_commission: None,
            summing_up_in_person_commission_results: None,
            summing_up_correspondence_commission_results: Some(3),
        };

        // assert_eq!(result_1.data, expect_1, "{:#?}", result_1.data);
        assert_eq!(result_2.data, expect_2, "{:#?}", result_2.data);
    })
    .await;
}
