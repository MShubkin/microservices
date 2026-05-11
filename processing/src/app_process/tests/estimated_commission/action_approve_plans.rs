//! Тестирование процесса `action/approve` ()
use super::*;
use std::fmt::Debug;

fn check_result<T: Debug>(result: Result<T>, f: fn(T) -> ()) {
    result.map_or_else(
        |error| panic!("Expected Result::Ok, but got: Err({:#?})", error),
        f,
    );
}

#[cfg(test)]
mod action_approve_plans_tests {
    use super::action_approve_plans::check_result;
    use crate::app_process::action_approve;
    use crate::app_process::tests::{mock_processing_context, run_db_test};
    use crate::common::ProcessingError;
    use shared_essential::presentation::dto::processing::ApprovePlansReq;
    use shared_essential::presentation::dto::response_request::Status;

    /// Не найдено по uuid
    #[tokio::test]
    async fn no_data_found() {
        run_db_test(fixtures::APPROVE_PLANS_EXTRA_MIGS, |pool| async move {
            let dto = ApprovePlansReq {
                section_id: fixtures::default_section_id(),
                item_list: vec![fixtures::unknown_object_ids()],
                user_id: fixtures::default_user(),
                is_force: true,
            };

            let pctx = mock_processing_context(pool).await;
            let result = action_approve(dto, pctx).await;

            let error_message = format!(
                "Записи ППЗ/ДС c идентификаторами {} не найдены",
                fixtures::unknown_object_id().id
            );
            match result.unwrap_err() {
                ProcessingError::GetItemList(message) => {
                    assert_eq!(error_message, message)
                }
                _ => panic!(),
            };
        })
        .await;
    }

    /// Действие в случае section_id = no_commission_required (СК не требуется)
    #[tokio::test]
    async fn special_section_id() {
        run_db_test(fixtures::APPROVE_PLANS_EXTRA_MIGS, |pool| async move {
            let dto = ApprovePlansReq {
                section_id: fixtures::special_section_id(),
                item_list: vec![fixtures::valid_object_ids1()],
                user_id: fixtures::default_user(),
                is_force: true,
            };

            let pctx = mock_processing_context(pool).await;
            super::launch_monolith_listener(&pctx, vec![]).await;

            let result = action_approve(dto, pctx).await;

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
        run_db_test(fixtures::APPROVE_PLANS_EXTRA_MIGS, |pool| async move {
            let dto = ApprovePlansReq {
                section_id: fixtures::default_section_id(),
                item_list: vec![fixtures::valid_uuid_only_object_ids()],
                user_id: fixtures::default_user(),
                is_force: true,
            };

            let pctx = mock_processing_context(pool).await;
            super::launch_monolith_listener(&pctx, vec![]).await;

            let result = action_approve(dto, pctx).await;

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

    /// Найдены валидные планы
    #[tokio::test]
    async fn valid_data_found() {
        run_db_test(fixtures::APPROVE_PLANS_EXTRA_MIGS, |pool| async move {
            let dto = ApprovePlansReq {
                section_id: fixtures::default_section_id(),
                item_list: vec![
                    fixtures::valid_object_ids1(),
                    fixtures::valid_object_ids5(),
                ],
                user_id: fixtures::default_user(),
                is_force: true,
            };

            let pctx = mock_processing_context(pool).await;
            super::launch_monolith_listener(&pctx, vec![]).await;

            let result = action_approve(dto, pctx).await;

            check_result(result, |response| {
                assert_eq!(response.status, Status::Ok);
                assert_eq!(response.messages.messages.len(), 1);
                assert_eq!(
                    response.messages.messages[0].parameters.item_list.len(),
                    2
                );
            })
        })
        .await;
    }

    #[cfg(test)]
    mod fixtures {
        use asez2_tables::Section;
        use shared_essential::presentation::dto::general::{
            ObjectIdentifier, ObjectIdentifierWithStatusNote,
        };
        use shared_essential::presentation::dto::response_request::EntityKind;
        use uuid::Uuid;

        pub fn valid_object_ids1() -> ObjectIdentifierWithStatusNote {
            ObjectIdentifierWithStatusNote::new(
                1,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                String::from("note1"),
            )
        }

        pub fn valid_object_ids5() -> ObjectIdentifierWithStatusNote {
            ObjectIdentifierWithStatusNote::new(
                5,
                Uuid::parse_str("00000000-0000-0000-0000-000000000005").unwrap(),
                String::from("note5"),
            )
        }

        pub fn unknown_object_ids() -> ObjectIdentifierWithStatusNote {
            ObjectIdentifierWithStatusNote::new(
                0,
                Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
                String::from("noteu"),
            )
        }

        pub fn valid_uuid_only_object_ids() -> ObjectIdentifierWithStatusNote {
            ObjectIdentifierWithStatusNote::new(
                12345678,
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                String::from("noteuuid"),
            )
        }

        pub fn default_user() -> i32 {
            666
        }

        pub fn default_section_id() -> Section {
            Section::EstimatedCommissionInPerson
        }

        pub fn special_section_id() -> Section {
            Section::EstimatedCommissionNotRequired
        }

        pub fn unknown_object_id() -> ObjectIdentifier {
            ObjectIdentifier::new_with_type(
                0,
                Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
                EntityKind::Plan,
            )
        }

        pub(crate) const APPROVE_PLANS_EXTRA_MIGS: &[&str] =
            &["estimated_commission/approve_plans.sql"];
    }
}
