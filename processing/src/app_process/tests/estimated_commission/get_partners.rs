use asez2_shared_db::uuid;

use super::*;
use crate::app_process::get_partners;

const GET_PARTNERS_EXTR_MIGS: &[&str] = &["estimated_commission/get_partners.sql"];

#[tokio::test]
async fn get_partners_success() {
    run_db_test(GET_PARTNERS_EXTR_MIGS, |pool| async move {
        let req = GetPartnersReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
        };

        let result = get_partners(req, pool.clone()).await.unwrap();

        let expected_res = vec![
            GetPartnersResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                commission_role_id: 111,
                user_id: 1,
            },
            GetPartnersResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                commission_role_id: 333,
                user_id: 3,
            },
        ];
        assert_eq!(result.data.item_list, expected_res);
    })
    .await;
}
