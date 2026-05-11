use super::*;
use crate::app_process;

const GET_PROTOCOL_DETAILS_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_protocol_details.sql"];

#[tokio::test]
async fn test_get_protocol_details() {
    run_db_test(GET_PROTOCOL_DETAILS_EXTRA_MIGS, |pool| async move {
        let req_ok = GetProtocolDetailsReq { id: 1 };
        let req_ok2 = GetProtocolDetailsReq { id: 2 };
        let req_err = GetProtocolDetailsReq { id: -1 };

        let r_ok = app_process::get_protocol_details(req_ok, pool.clone()).await;
        let r_ok2 = app_process::get_protocol_details(req_ok2, pool.clone()).await;
        let r_err = app_process::get_protocol_details(req_err, pool.clone()).await;

        {
            let res = r_ok.unwrap();
            assert!(res.messages.is_empty());
            assert_eq!(res.data.protocol.id, Some(1));
            assert_eq!(res.data.protocol_item_list.len(), 2);
            assert_eq!(res.data.protocol_item_d647_list.len(), 1);
            assert_eq!(res.data.partner_list.len(), 2);
            assert_eq!(res.data.attachment_list.len(), 3);

            // Additional checks for correct fields.
            assert_eq!(
                res.data.protocol.protocol_type_id,
                Some(ProtocolType::InPersonMeeting)
            );

            let item = &res.data.protocol_item_list[0].protocol_item.item;

            assert!(item.result_id.is_some());
            assert!(item.sum_excluded_vat.is_some());
            assert!(item.source_uuid.is_some());
            assert!(item.sum_excluded_vat.is_some());
            assert!(item.result_id.is_some());
            assert!(item.pricing_sum_excluded_vat.is_some());
            assert!(item.is_registered_by_d647.is_some());
            assert!(item.is_excluded.is_some());
            assert!(item.commission_sum_excluded_vat.is_some());

            assert_eq!(
                res.data.protocol_item_list[0]
                    .protocol_item
                    .calculated
                    .commission_economy_sum_excluded_vat
                    .unwrap(),
                2.15.into()
            );
            assert_eq!(
                res.data.protocol_item_list[0]
                    .protocol_item
                    .calculated
                    .commission_percent_economy
                    .as_ref()
                    .unwrap(),
                "21,50"
            );

            assert_eq!(
                res.data.protocol_item_list[1]
                    .protocol_item
                    .calculated
                    .commission_economy_sum_excluded_vat
                    .unwrap(),
                (-5.15).into()
            );
            assert_eq!(
                res.data.protocol_item_list[1]
                    .protocol_item
                    .calculated
                    .commission_percent_economy
                    .as_ref()
                    .unwrap(),
                "-51,50"
            );
        }
        {
            let res = r_ok2.unwrap();
            assert!(res.messages.is_empty());
            assert_eq!(res.data.protocol.id, Some(2));
            assert_eq!(res.data.protocol_item_list.len(), 1);
            assert_eq!(res.data.protocol_item_d647_list.len(), 0);
            assert_eq!(res.data.partner_list.len(), 3);
            assert_eq!(res.data.attachment_list.len(), 0);

            assert_eq!(res.data.partner_list[0].commission_role_id, Some(2));
            assert_eq!(res.data.partner_list[1].commission_role_id, Some(2));
            assert_eq!(res.data.partner_list[2].commission_role_id, Some(3));

            assert_eq!(res.data.partner_list[0].user_id, Some(1));
            assert_eq!(res.data.partner_list[1].user_id, Some(3));
            assert_eq!(res.data.partner_list[2].user_id, Some(2));

            // Additional checks for correct fields.
            assert_eq!(
                res.data.protocol.protocol_type_id,
                Some(ProtocolType::CorrespondenceMeeting)
            );

            let item = &res.data.protocol_item_list[0].protocol_item.item;

            assert!(item.result_id.is_some());
            assert!(item.sum_excluded_vat.is_some());
            assert!(item.source_uuid.is_some());
            assert!(item.sum_excluded_vat.is_some());
            assert!(item.result_id.is_some());
            assert!(item.pricing_sum_excluded_vat.is_some());
            assert!(item.is_registered_by_d647.is_some());
            assert!(item.is_excluded.is_some());
            assert!(item.commission_sum_excluded_vat.is_some());

            assert!(
                res.data.protocol_item_list[0]
                    .protocol_item
                    .calculated
                    .commission_economy_sum_excluded_vat
                    .unwrap()
                    == (-0.5).into()
                    && res.data.protocol_item_list[0]
                        .protocol_item
                        .calculated
                        .commission_percent_economy
                        .as_ref()
                        .unwrap()
                        == "-50,00"
            );
        }
        {
            let res = r_err.unwrap_err();
            assert!(matches!(
                res,
                crate::common::ProcessingError::GetProtocolDetails(_)
            ));
        }
    })
    .await
}
