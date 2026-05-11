//! Тестирование процесса `protocol_agreement`
use super::action_protocol_agreement::fixtures::default_user;
use super::*;
use crate::app_process::action_protocol_agreement;
use shared_essential::presentation::dto::response_request::{MessageKind, Status};
use std::fmt::Debug;

fn check_result<T: Debug, F: FnOnce(T)>(result: Result<T>, f: F) {
    if let Ok(response) = result {
        f(response);
    } else {
        panic!("Failed!: {:?}", result)
    }
}

const PROTOCOL_SQL: &[&str] = &["estimated_commission/protocol.sql"];

/// Не найдено
#[tokio::test]
async fn no_data_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![fixtures::unknown_object_id()],
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            assert_eq!(response.messages.messages.len(), 1);
        })
    })
    .await;
}

/// Найден удаленный документ
#[tokio::test]
async fn removed_data_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![fixtures::removed_object_id()],
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            assert_eq!(response.messages.messages.len(), 1);
        })
    })
    .await;
}

/// Поиск идет только по UUID
#[tokio::test]
async fn find_by_uuid() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![fixtures::valid_uuid_only_object_id()],
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        check_result(result, |r1| {
            assert_eq!(r1.status, Status::Ok);

            match cfg!(with_plan_db) {
                true => assert_eq!(r1.messages.messages.len(), 2),
                false => assert_eq!(r1.messages.messages.len(), 1),
            };
        })
    })
    .await;
}

/// Найдены валидные протоколы типа 1 (InPersonMeeting)
#[tokio::test]
async fn valid_data_type_1_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_1(),
            ],
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        check_result(result, |r1| {
            assert_eq!(r1.status, Status::Ok);

            match cfg!(with_plan_db) {
                true => assert_eq!(r1.messages.messages.len(), 2),
                false => assert_eq!(r1.messages.messages.len(), 1),
            };
        })
    })
    .await;
}

/// Найдены валидные протоколы типа 2 (CorrespondenceMeeting)
#[tokio::test]
async fn valid_data_type_2_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_2(),
            ],
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        check_result(result, |r1| {
            assert_eq!(r1.status, Status::Ok);

            match cfg!(with_plan_db) {
                true => assert_eq!(r1.messages.messages.len(), 2),
                false => assert_eq!(r1.messages.messages.len(), 1),
            };
        })
    })
    .await;
}

/// Найдены протоколы со статусами ошибки
#[tokio::test]
async fn error_status_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_1(),
                fixtures::status_200_object_id(),
                fixtures::status_400_object_id(),
                fixtures::status_500_object_id(),
            ],
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Error);
            assert_eq!(response.messages.messages.len(), 2);
            assert!(response
                .messages
                .messages
                .iter()
                .all(|message| message.kind == MessageKind::Error));
        })
    })
    .await;
}

/// Найдены протоколы со статусами предупреждения
#[tokio::test]
async fn warning_status_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let items = vec![
            fixtures::unknown_object_id(),
            fixtures::valid_object_id_type_1(),
            fixtures::status_300_object_id(),
        ];
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: items.clone(),
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        let s = Select::default();
        let histories = StatusHistory::select(&s, &*pctx.db_pool).await.unwrap();

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            {
                // Unknown status object is dumped.
                assert_eq!(histories.len(), 2);
                // assert_eq!(histories[0].object_uuid, items[0].uuid);
                // assert_eq!(histories[0].comment, items[0].status_note);
                assert_eq!(histories[0].object_uuid, items[1].uuid);
                assert_eq!(histories[0].comment, items[1].status_note);
                assert_eq!(histories[1].object_uuid, items[2].uuid);
                assert_eq!(histories[1].comment, items[2].status_note);

                histories.iter().for_each(|x| {
                    assert_eq!(x.created_by, 666);
                    assert_eq!(
                        x.status_id,
                        EcProtocolStatus::AgreementPending as i16
                    );
                });
            }

            match cfg!(with_plan_db) {
                true => {
                    assert_eq!(response.messages.messages.len(), 2);
                    assert_eq!(
                        response.messages.messages[0].kind,
                        MessageKind::Warning
                    );
                    assert_eq!(
                        response.messages.messages[1].kind,
                        MessageKind::Success
                    );
                }
                false => {
                    assert_eq!(response.messages.messages.len(), 1);
                    assert_eq!(
                        response.messages.messages[0].kind,
                        MessageKind::Success
                    );
                }
            };
        })
    })
    .await;
}

/// Найдены протоколы со статусами ошибки и предупреждения (оставляем только ошибки)
#[tokio::test]
async fn error_and_warning_status_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = ProtocolAgreementReq {
            user_id: default_user(),
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_1(),
                fixtures::status_200_object_id(),
                fixtures::status_300_object_id(),
                fixtures::status_400_object_id(),
                fixtures::status_500_object_id(),
            ],
        };

        let pctx = mock_processing_context(pool).await;
        let result = action_protocol_agreement(dto, pctx.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Error);
            assert_eq!(response.messages.messages.len(), 2);
            assert_eq!(response.messages.messages[0].kind, MessageKind::Error);
            assert_eq!(response.messages.messages[1].kind, MessageKind::Error);
        })
    })
    .await;
}

#[cfg(test)]
mod fixtures {
    use shared_essential::presentation::dto::general::ObjectIdentifierWithStatusNote;
    use uuid::Uuid;

    pub fn default_user() -> i32 {
        666
    }

    pub fn unknown_object_id() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            0,
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            "status_note unknown_object_id".to_string(),
        )
    }

    pub fn valid_object_id_type_1() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            1,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "status_note valid_object_id_type_1".to_string(),
        )
    }

    pub fn valid_object_id_type_2() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            7,
            Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap(),
            "status_note valid_object_id_type_2".to_string(),
        )
    }

    pub fn valid_uuid_only_object_id() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            12345678,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "status_note valid_uuid_only_object_id".to_string(),
        )
    }

    pub fn removed_object_id() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            6,
            Uuid::parse_str("00000000-0000-0000-0000-000000000006").unwrap(),
            "status_note removed_object_id".to_string(),
        )
    }

    pub fn status_200_object_id() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            2,
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            "status_note status_200_object_id".to_string(),
        )
    }

    pub fn status_300_object_id() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            3,
            Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            "status_note status_300_object_id".to_string(),
        )
    }

    pub fn status_400_object_id() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            4,
            Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
            "status_note status_400_object_id".to_string(),
        )
    }

    pub fn status_500_object_id() -> ObjectIdentifierWithStatusNote {
        ObjectIdentifierWithStatusNote::new(
            5,
            Uuid::parse_str("00000000-0000-0000-0000-000000000005").unwrap(),
            "status_note status_500_object_id".to_string(),
        )
    }
}
