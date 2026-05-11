//! Тестирование процесса [`get_protocol_list_by_date`]

use super::*;
use crate::app_process::get_protocol_list_by_date;

const GET_PROTOCOL_LIST_BY_DATE_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_protocol_list.sql"];

const DATE_TYPE: &str = "protocol_date";
const INVALID_DATE_TYPE: &str = "meeting_date";

#[tokio::test]
async fn empty_list() {
    run_db_test(GET_PROTOCOL_LIST_BY_DATE_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListByDateReq {
            date: "2002-02-20".try_into().unwrap(),
            date_type: DATE_TYPE.into(),
            protocol_type_id: ProtocolType::InPersonMeeting,
        };

        let result = get_protocol_list_by_date(dto, pool.clone())
            .await
            .expect("should succeed");
        assert!(result.data.item_list.is_empty());
    })
    .await;
}

#[tokio::test]
async fn invalid_date_type() {
    run_db_test(GET_PROTOCOL_LIST_BY_DATE_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListByDateReq {
            date: "2002-02-20".try_into().unwrap(),
            date_type: INVALID_DATE_TYPE.into(),
            protocol_type_id: ProtocolType::InPersonMeeting,
        };

        let result = get_protocol_list_by_date(dto, pool.clone()).await;
        assert!(result.is_err());
    })
    .await;
}

#[tokio::test]
async fn invalid_protocol_type_id() {
    run_db_test(GET_PROTOCOL_LIST_BY_DATE_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListByDateReq {
            date: "2002-02-20".try_into().unwrap(),
            date_type: DATE_TYPE.into(),
            protocol_type_id: ProtocolType::Undefined,
        };

        let result = get_protocol_list_by_date(dto, pool.clone()).await;
        assert!(result.is_err());
    })
    .await;
}

#[tokio::test]
async fn non_empty_list() {
    run_db_test(GET_PROTOCOL_LIST_BY_DATE_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListByDateReq {
            date: "2001-01-01".try_into().unwrap(),
            date_type: DATE_TYPE.into(),
            protocol_type_id: ProtocolType::InPersonMeeting,
        };

        let result = get_protocol_list_by_date(dto, pool.clone())
            .await
            .expect("should succeed");
        assert_eq!(result.data.item_list.len(), 2);
    })
    .await;
}
