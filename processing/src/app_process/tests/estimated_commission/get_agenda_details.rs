use super::*;
use crate::app_process;

const GET_AGENDA_DETAILS_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_agenda_details.sql"];

#[tokio::test]
async fn test_get_agenda_details() {
    run_db_test(GET_AGENDA_DETAILS_EXTRA_MIGS, |pool| async move {
        let req_ok = GetAgendaDetailsReq { id: 1 };
        let req_err = GetAgendaDetailsReq { id: -1 };

        let r_ok = app_process::get_agenda_details(req_ok, pool.clone()).await;
        let r_err = app_process::get_agenda_details(req_err, pool.clone()).await;

        {
            let res = r_ok.unwrap();
            assert!(res.messages.is_empty());
            assert_eq!(res.data.agenda.id, Some(1));

            assert_eq!(res.data.agenda_item_list.len(), 1);
            assert_eq!(res.data.agenda_item_d647_list.len(), 1);

            assert_eq!(res.data.attachment_list.len(), 3);
            assert_eq!(res.data.status_histories.len(), 1);
            assert_eq!(
                res.data.partner_list.len(),
                3,
                "{:#?}",
                res.data.partner_list
            );

            assert_eq!(res.data.partner_list[0].commission_role_id, Some(1));
            assert_eq!(res.data.partner_list[1].commission_role_id, Some(1));
            assert_eq!(res.data.partner_list[2].commission_role_id, Some(2));

            assert_eq!(res.data.partner_list[0].user_id, Some(1));
            assert_eq!(res.data.partner_list[1].user_id, Some(3));
            assert_eq!(res.data.partner_list[2].user_id, Some(1));

            assert!(res.data.partner_list[0].commission_role_id.is_some());
        }
        {
            let res = r_err.unwrap();
            assert_eq!(res.messages.messages.len(), 1);
            assert!(res.messages.is_error());
            assert_eq!(&res.messages.messages[0].text, "Повеска № -1 не найдена.");
        }
    })
    .await
}
