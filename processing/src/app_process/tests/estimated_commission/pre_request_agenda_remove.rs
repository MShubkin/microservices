//! Тестирование процесса `pre_request_agenda_remove`
use super::*;
use std::fmt::Debug;

fn check_result<T: Debug>(result: Result<T>, f: fn(T) -> ()) {
    result.map_or_else(
        |error| panic!("Expected Result::Ok, but got: Err({:#?})", error),
        f,
    );
}

#[cfg(test)]
mod pre_request_agenda_remove_tests {
    use super::pre_request_agenda_remove::check_result;
    use crate::app_process::pre_request_agenda_remove;
    use crate::app_process::tests::run_db_test;
    use shared_essential::presentation::dto::processing::PreRequestAgendaRemoveReq;
    use shared_essential::presentation::dto::response_request::Status;

    /// Не найдено
    #[tokio::test]
    async fn no_data_found() {
        run_db_test(fixtures::PRE_AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = PreRequestAgendaRemoveReq {
                item_list: vec![fixtures::unknown_object_id()],
            };

            let result = pre_request_agenda_remove(dto, pool.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                let agenda_list = response.data;
                assert_eq!(agenda_list.total, Some(0));
                assert_eq!(agenda_list.item_list.len(), 0);
            })
        })
        .await;
    }

    /// Найден удаленный документ
    #[tokio::test]
    async fn removed_data_found() {
        run_db_test(fixtures::PRE_AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = PreRequestAgendaRemoveReq {
                item_list: vec![fixtures::removed_object_id()],
            };

            let result = pre_request_agenda_remove(dto, pool.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                let agenda_list = response.data;
                assert_eq!(agenda_list.total, Some(0));
                assert_eq!(agenda_list.item_list.len(), 0);
            })
        })
        .await;
    }

    /// Поиск идет только по UUID
    #[tokio::test]
    async fn find_by_uuid() {
        run_db_test(fixtures::PRE_AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = PreRequestAgendaRemoveReq {
                item_list: vec![fixtures::valid_uuid_only_object_id()],
            };

            let result = pre_request_agenda_remove(dto, pool.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                let agenda_list = response.data;
                assert_eq!(agenda_list.total, Some(1));
                assert_eq!(agenda_list.item_list.len(), 1);
            })
        })
        .await;
    }

    /// Найдены валидные повестки
    #[tokio::test]
    async fn valid_data_found() {
        run_db_test(fixtures::PRE_AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = PreRequestAgendaRemoveReq {
                item_list: vec![
                    fixtures::unknown_object_id(),
                    fixtures::valid_object_id1(),
                    fixtures::valid_object_id2(),
                ],
            };

            let result = pre_request_agenda_remove(dto, pool.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                let agenda_list = response.data;
                assert_eq!(agenda_list.total, Some(2));
                assert_eq!(agenda_list.item_list.len(), 2);
            })
        })
        .await;
    }

    /// Найдены повестки со связями к протоколу
    #[tokio::test]
    async fn linked_protocols_found() {
        run_db_test(fixtures::PRE_AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = PreRequestAgendaRemoveReq {
                item_list: vec![
                    fixtures::unknown_object_id(),
                    fixtures::valid_object_id1(),
                    fixtures::valid_object_id2(),
                    fixtures::linked_object_id3(),
                ],
            };

            let result = pre_request_agenda_remove(dto, pool.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 1);
                let agenda_list = response.data;
                assert_eq!(agenda_list.total, Some(0));
                assert_eq!(agenda_list.item_list.len(), 0);
            })
        })
        .await;
    }

    /// Найдены повестки со статусами ошибки
    #[tokio::test]
    async fn bad_status_found() {
        run_db_test(fixtures::PRE_AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = PreRequestAgendaRemoveReq {
                item_list: vec![
                    fixtures::unknown_object_id(),
                    fixtures::valid_object_id1(),
                    fixtures::valid_object_id2(),
                    fixtures::status_300_object_id(),
                ],
            };

            let result = pre_request_agenda_remove(dto, pool.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 2);
                let agenda_list = response.data;
                assert_eq!(agenda_list.total, Some(0));
                assert_eq!(agenda_list.item_list.len(), 0);
            })
        })
        .await;
    }

    /// Найдены повестки со статусами ошибки и связями с протоколом
    #[tokio::test]
    async fn linked_and_bad_status_found() {
        run_db_test(fixtures::PRE_AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = PreRequestAgendaRemoveReq {
                item_list: vec![
                    fixtures::unknown_object_id(),
                    fixtures::valid_object_id1(),
                    fixtures::valid_object_id2(),
                    fixtures::linked_object_id3(),
                    fixtures::status_300_object_id(),
                    fixtures::status_400_object_id(),
                ],
            };

            let result = pre_request_agenda_remove(dto, pool.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 4);
                let agenda_list = response.data;
                assert_eq!(agenda_list.total, Some(0));
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

        pub fn valid_object_id1() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                1,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                EntityKind::Plan,
            )
        }

        pub fn valid_object_id2() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                2,
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                EntityKind::Plan,
            )
        }

        pub fn valid_uuid_only_object_id() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                12345678,
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                EntityKind::Plan,
            )
        }

        pub fn linked_object_id3() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                3,
                Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                EntityKind::Plan,
            )
        }

        pub fn removed_object_id() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                4,
                Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                EntityKind::Plan,
            )
        }

        pub fn status_300_object_id() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                5,
                Uuid::parse_str("00000000-0000-0000-0000-000000000005").unwrap(),
                EntityKind::Plan,
            )
        }

        pub fn status_400_object_id() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                6,
                Uuid::parse_str("00000000-0000-0000-0000-000000000006").unwrap(),
                EntityKind::Plan,
            )
        }

        pub(crate) const PRE_AGENDA_REMOVE_EXTRA_MIGS: &[&str] =
            &["estimated_commission/agenda_remove.sql"];
    }
}
