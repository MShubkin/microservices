//! Тестирование процесса `pre_request_protocol_agreement`
use super::*;
use crate::app_process::pre_request_protocol_agreement;
use shared_essential::presentation::dto::response_request::{MessageKind, Status};
use std::fmt::Debug;

fn check_result<T: Debug>(result: Result<T>, f: fn(T) -> ()) {
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
        let dto = PreProtocolAgreementReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![fixtures::unknown_object_id()],
        };

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 0);
            assert_eq!(agenda_list.item_list.len(), 0);
        })
    })
    .await;
}

/// Найден удаленный документ
#[tokio::test]
async fn removed_data_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = PreProtocolAgreementReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![fixtures::removed_object_id()],
        };

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 1);
            assert_eq!(agenda_list.item_list.len(), 1);
        })
    })
    .await;
}

/// Поиск идет только по UUID
#[tokio::test]
async fn find_by_uuid() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = PreProtocolAgreementReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![fixtures::valid_uuid_only_object_id()],
        };

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 1);
            assert_eq!(agenda_list.item_list.len(), 1);
        })
    })
    .await;
}

/// Найдены валидные протоколы типа 1 (InPersonMeeting)
#[tokio::test]
async fn valid_data_type_1_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = PreProtocolAgreementReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_1(),
            ],
        };

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 1);
            assert_eq!(agenda_list.item_list.len(), 1);
        })
    })
    .await;
}

/// Найдены валидные протоколы типа 2 (CorrespondenceMeeting)
#[tokio::test]
async fn valid_data_type_2_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = PreProtocolAgreementReq {
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_2(),
            ],
        };

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 1);
            assert_eq!(agenda_list.item_list.len(), 1);
        })
    })
    .await;
}

/// Найдены протоколы со статусами ошибки
#[tokio::test]
async fn error_status_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = PreProtocolAgreementReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_1(),
                fixtures::status_200_object_id(),
                fixtures::status_400_object_id(),
                fixtures::status_500_object_id(),
            ],
        };

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            assert_eq!(response.messages.messages.len(), 2);
            assert!(response
                .messages
                .messages
                .iter()
                .all(|message| message.kind == MessageKind::Error));
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 0);
            assert_eq!(agenda_list.item_list.len(), 0);
        })
    })
    .await;
}

/// Найдены протоколы со статусами предупреждения
#[tokio::test]
async fn warning_status_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = PreProtocolAgreementReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            item_list: vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id_type_1(),
                fixtures::status_300_object_id(),
            ],
        };

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            assert_eq!(response.messages.messages.len(), 1);
            assert!(response
                .messages
                .messages
                .iter()
                .all(|message| message.kind == MessageKind::Warning));
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 2);
            assert_eq!(agenda_list.item_list.len(), 2);
        })
    })
    .await;
}

/// Найдены протоколы со статусами ошибки и предупреждения (оставляем только ошибки)
#[tokio::test]
async fn error_and_warning_status_found() {
    run_db_test(PROTOCOL_SQL, |pool| async move {
        let dto = PreProtocolAgreementReq {
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

        let result = pre_request_protocol_agreement(dto, pool.clone()).await;

        check_result(result, |response| {
            assert_eq!(response.status, Status::Ok);
            assert_eq!(response.messages.messages.len(), 2);
            assert!(response
                .messages
                .messages
                .iter()
                .all(|message| message.kind == MessageKind::Error));
            let agenda_list = response.data;
            assert_eq!(agenda_list.total, 0);
            assert_eq!(agenda_list.item_list.len(), 0);
        })
    })
    .await;
}

#[cfg(test)]
mod fixtures {
    use shared_essential::presentation::dto::general::ObjectIdentifier;
    use shared_essential::presentation::dto::response_request::EntityKind;
    use uuid::Uuid;

    pub fn unknown_object_id() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            0,
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn valid_object_id_type_1() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            1,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn valid_object_id_type_2() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            7,
            Uuid::parse_str("00000000-0000-0000-0000-000000000007").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn valid_uuid_only_object_id() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            12345678,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn removed_object_id() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            6,
            Uuid::parse_str("00000000-0000-0000-0000-000000000006").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn status_200_object_id() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            2,
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn status_300_object_id() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            3,
            Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn status_400_object_id() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            4,
            Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
            EntityKind::Plan,
        )
    }

    pub fn status_500_object_id() -> ObjectIdentifier {
        ObjectIdentifier::new_with_type(
            5,
            Uuid::parse_str("00000000-0000-0000-0000-000000000005").unwrap(),
            EntityKind::Plan,
        )
    }
}
