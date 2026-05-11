//! Тестирование процесса `agenda_remove`
use super::*;
use std::fmt::Debug;

fn check_result<T: Debug, F: FnOnce(T)>(result: Result<T>, f: F) {
    result.map_or_else(
        |error| panic!("Expected Result::Ok, but got: Err({:#?})", error),
        f,
    );
}

#[cfg(test)]
mod action_agenda_remove_tests {
    use super::action_agenda_remove::check_result;
    use super::*;
    use super::{mock_processing_context, run_db_test};
    use crate::app_process::action_agenda_remove;
    use shared_essential::presentation::dto::processing::AgendaRemoveReq;
    use shared_essential::presentation::dto::response_request::Status;

    /// Не найдено
    #[tokio::test]
    async fn no_data_found() {
        run_db_test(fixtures::AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = AgendaRemoveReq {
                user_id: fixtures::default_user(),
                item_list: vec![fixtures::unknown_object_id()],
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_agenda_remove(dto, pctx.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 1);
                assert_eq!(
                    response.messages.messages[0].parameters.item_list.len(),
                    0
                );
            })
        })
        .await;
    }

    /// Повторное удаление
    #[tokio::test]
    async fn removed_data_found() {
        run_db_test(fixtures::AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = AgendaRemoveReq {
                user_id: fixtures::default_user(),
                item_list: vec![fixtures::removed_object_id()],
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_agenda_remove(dto, pctx.clone()).await;

            let s = Select::default();
            let histories =
                StatusHistory::select(&s, &*pctx.db_pool).await.unwrap();

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);

                assert_eq!(response.messages.messages.len(), 1);

                // Повторное удаление статус не меняет!
                assert!(histories.is_empty(), "{:?}", histories);
            })
        })
        .await;
    }

    /// Поиск идет только по UUID
    #[tokio::test]
    async fn find_by_uuid() {
        run_db_test(fixtures::AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = AgendaRemoveReq {
                user_id: fixtures::default_user(),
                item_list: vec![fixtures::valid_uuid_only_object_id()],
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_agenda_remove(dto, pctx.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 1);
                assert_eq!(
                    response.messages.messages[0].parameters.item_list.len(),
                    1
                );
            })
        })
        .await;
    }

    /// Найдены валидные повестки
    #[tokio::test]
    async fn valid_data_found() {
        run_db_test(fixtures::AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let items = vec![
                fixtures::unknown_object_id(),
                fixtures::valid_object_id1(),
                fixtures::valid_object_id2(),
            ];
            let dto = AgendaRemoveReq {
                user_id: fixtures::default_user(),
                item_list: items.clone(),
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_agenda_remove(dto, pctx.clone()).await;

            let s = Select::default();
            let histories =
                StatusHistory::select(&s, &*pctx.db_pool).await.unwrap();

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                {
                    assert_eq!(histories.len(), 2);
                    assert_eq!(histories[0].object_uuid, items[1].uuid);
                    assert_eq!(histories[1].object_uuid, items[2].uuid);

                    histories.iter().for_each(|x| {
                        assert_eq!(x.comment, "");
                        assert_eq!(x.created_by, 666);
                        assert_eq!(x.status_id, EcAgendaStatus::Deleted as i16);
                    });
                }
                assert_eq!(response.messages.messages.len(), 1);
                assert_eq!(
                    response.messages.messages[0].parameters.item_list.len(),
                    2
                );
            })
        })
        .await;
    }

    /// Найдены повестки со связями к протоколу
    #[tokio::test]
    async fn linked_protocols_found() {
        run_db_test(fixtures::AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = AgendaRemoveReq {
                user_id: fixtures::default_user(),
                item_list: vec![
                    fixtures::unknown_object_id(),
                    fixtures::valid_object_id1(),
                    fixtures::valid_object_id2(),
                    fixtures::linked_object_id3(),
                ],
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_agenda_remove(dto, pctx.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 1);
                assert_eq!(
                    response.messages.messages[0].parameters.item_list.len(),
                    0
                );
            })
        })
        .await;
    }

    /// Найдены повестки со статусами ошибки
    #[tokio::test]
    async fn bad_status_found() {
        run_db_test(fixtures::AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = AgendaRemoveReq {
                user_id: fixtures::default_user(),
                item_list: vec![
                    fixtures::unknown_object_id(),
                    fixtures::valid_object_id1(),
                    fixtures::valid_object_id2(),
                    fixtures::status_300_object_id(),
                ],
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_agenda_remove(dto, pctx.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 2);
                assert_eq!(
                    response.messages.messages[0].parameters.item_list.len(),
                    0
                );
                assert_eq!(
                    response.messages.messages[1].parameters.item_list.len(),
                    0
                );
            })
        })
        .await;
    }

    /// Найдены повестки со статусами ошибки и связями с протоколом
    #[tokio::test]
    async fn linked_and_bad_status_found() {
        run_db_test(fixtures::AGENDA_REMOVE_EXTRA_MIGS, |pool| async move {
            let dto = AgendaRemoveReq {
                user_id: fixtures::default_user(),
                item_list: vec![
                    fixtures::unknown_object_id(),
                    fixtures::valid_object_id1(),
                    fixtures::valid_object_id2(),
                    fixtures::linked_object_id3(),
                    fixtures::status_300_object_id(),
                    fixtures::status_400_object_id(),
                ],
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_agenda_remove(dto, pctx.clone()).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 4);
            })
        })
        .await;
    }

    #[cfg(test)]
    mod fixtures {
        use shared_essential::presentation::dto::general::ObjectIdentifier;
        use shared_essential::presentation::dto::response_request::EntityKind;
        use uuid::Uuid;

        pub fn default_user() -> i32 {
            666
        }

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

        pub(crate) const AGENDA_REMOVE_EXTRA_MIGS: &[&str] =
            &["estimated_commission/agenda_remove.sql"];
    }
}
