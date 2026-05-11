use crate::{
    application::master_data::{base::plan_reasons_cancel::repository, MasterData},
    domain::MasterDataDirectoryInterface,
    tests::db_test::run_db_test,
};
use ahash::AHashSet;
use asez2_shared_db::db_item::int_array::AsezArray;

use shared_essential::{
    domain::plan_reasons_cancel::PlanReasonCancelHeaderRep,
    presentation::dto::master_data::{
        plan_reasons_cancel::PlanReasonCancelCustomer as PlanReasonCancelCustomerDto,
        request::{
            CreatePlanReasonsCancelReq, SearchByUserInput,
            SearchPlanReasonCancelReq, UpdatePlanReasonsCancelReq,
        },
    },
};

const FIXTURE_FILE: &str = "plan_reasons_cancel.sql";

#[tokio::test]
async fn create_reason() {
    run_db_test(&[FIXTURE_FILE], |pool| async move {
        let user_id = 1;

        let create_req = CreatePlanReasonsCancelReq {
            header: PlanReasonCancelHeaderRep {
                text: Some("New Test Reason".to_string()),
                impact_area_id: Some(2),
                is_objective_reason: Some(true),
                is_new_plan: Some(false),
                is_reason_fill_type: Some(false),
                functionality_id_list: Some(AsezArray(vec![1, 2])),
                check_reason_id: Some(1),
                ..Default::default()
            },
            customers: vec![PlanReasonCancelCustomerDto::In {
                filter_values: vec![999, 888],
            }],
        };
        let customer_ids: AHashSet<i32> =
            match create_req.customers.first().unwrap() {
                PlanReasonCancelCustomerDto::In { filter_values } => {
                    filter_values.iter().cloned().collect()
                }
                _ => panic!("Тест должен использовать оператор 'in'"),
            };

        let (created_item, messages) =
            repository::create(create_req.header, customer_ids, user_id, &pool)
                .await
                .expect("Create failed");

        assert!(!messages.is_error(), "Expected no errors, but got: {messages:?}");
        assert_eq!(created_item.header.text, "New Test Reason");
        assert_eq!(created_item.header.impact_area_id, 2);
        assert_eq!(created_item.header.functionality_id_list.0, vec![1, 2]);
        assert_eq!(created_item.header.check_reason_id, 1);
        assert!(created_item.header.is_objective_reason);
        assert_eq!(created_item.customers.len(), 2);
        assert!(created_item.customers.iter().any(|c| c.customer_id == 999));
        assert!(created_item.customers.iter().any(|c| c.customer_id == 888));
    })
    .await
}

#[tokio::test]
async fn get_by_id_reason() {
    run_db_test(&[FIXTURE_FILE], |pool| async move {
        let fixture_id = 1;

        let result = repository::get_by_id(fixture_id, &pool).await;
        assert!(result.is_ok(), "Expected success for valid ID");
        let aggregate = result.unwrap();
        assert_eq!(aggregate.header.id, fixture_id);
        assert_eq!(aggregate.header.text, "Initial Test Reason");

        let non_existent_id = 999_999;
        let result = repository::get_by_id(non_existent_id, &pool).await;
        assert!(result.is_err(), "Expected error for non-existent ID");
    })
    .await
}

#[tokio::test]
async fn search_reason() {
    run_db_test(&[FIXTURE_FILE], |pool| async move {
        let specific_reason_text = "Another Specific Reason Word";

        // Search for "Specific"
        let search_req = SearchPlanReasonCancelReq {
            search: SearchByUserInput {
                from: 0,
                search: "Specific".to_string(),
                quantity: 10,
            },
            header: Default::default(),
        };

        let results = repository::search(&search_req, &pool).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].header.text, specific_reason_text);

        // Search for "Reason"
        let search_req_all = SearchPlanReasonCancelReq {
            search: SearchByUserInput {
                from: 0,
                search: "Reason".to_string(),
                quantity: 10,
            },
            header: Default::default(),
        };

        let results_all = repository::search(&search_req_all, &pool).await.unwrap();
        assert_eq!(results_all.len(), 2);
        assert!(results_all.iter().any(|r| r.header.text == "Initial Test Reason"));
        assert!(results_all.iter().any(|r| r.header.text == specific_reason_text));
    })
    .await
}

#[tokio::test]
async fn functionality_directory_tests() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.plan_reasons_cancel_functionality.load(&pool).await.unwrap();
        let directory = &master_data.plan_reasons_cancel_functionality;
        let item_id = 1;
        let result = directory.get_by_id(&item_id).await.unwrap();
        assert_eq!(result.id, item_id as i16);
        assert_eq!(result.text, "Планирование/ППЗ");
    })
    .await
}

#[tokio::test]
async fn impact_area_directory_tests() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.plan_reasons_cancel_impact_area.load(&pool).await.unwrap();
        let directory = &master_data.plan_reasons_cancel_impact_area;
        let item_id = 1;
        let result = directory.get_by_id(&item_id).await.unwrap();
        assert_eq!(result.id, item_id as i16);
        assert_eq!(result.text, "Сфера влияния КГГ");
    })
    .await
}

#[tokio::test]
async fn check_reason_directory_tests() {
    run_db_test(&[], |pool| async move {
        let mut master_data = MasterData::default();
        master_data.plan_reasons_cancel_check_reason.load(&pool).await.unwrap();
        let directory = &master_data.plan_reasons_cancel_check_reason;
        let item_id = 1;
        let result = directory.get_by_id(&item_id).await.unwrap();
        assert_eq!(result.id, item_id as i16);
        assert_eq!(result.text, "Публикация ППЗ в ЕИС");
    })
    .await
}

#[tokio::test]
async fn update_reason() {
    run_db_test(&[FIXTURE_FILE], |pool| async move {
        let fixture_id = 1;
        let user_id = 2;

        let update_req = UpdatePlanReasonsCancelReq {
            header: PlanReasonCancelHeaderRep {
                id: Some(fixture_id),
                text: Some("Updated Reason Text".to_string()),
                impact_area_id: Some(3),
                is_objective_reason: Some(false),
                is_new_plan: Some(true),
                is_reason_fill_type: Some(true),
                functionality_id_list: Some(AsezArray(vec![2])),
                check_reason_id: Some(1),
                ..Default::default()
            },
            customers: vec![PlanReasonCancelCustomerDto::In {
                filter_values: vec![202],
            }],
        };
        let new_customer_ids: AHashSet<i32> = match update_req.customers.first().unwrap() {
            PlanReasonCancelCustomerDto::In { filter_values } => {
                filter_values.iter().cloned().collect()
            }
            _ => panic!("Тест должен использовать оператор 'in'"),
        };
        let (updated_item, messages) = repository::update(
            update_req.header,
            new_customer_ids,
            user_id,
            &pool,
        )
        .await
        .expect("Update failed");

        assert!(!messages.is_error(), "Expected no errors, but got: {messages:?}");
        assert_eq!(updated_item.header.text, "Updated Reason Text");
        assert_eq!(updated_item.header.impact_area_id, 3);
        assert_eq!(updated_item.header.check_reason_id, 1);
        assert_eq!(updated_item.header.changed_by, user_id);

        // Проверяем, что старый заказчик (101) был удален и больше не возвращается.
        let old_customer_present = updated_item
            .customers
            .iter()
            .any(|c| c.customer_id == 101);
        assert!(!old_customer_present, "Старый заказчик (101) не должен был вернуться в ответе, так как он удален");

        // Проверяем, что новый заказчик добавлен и активен
        let new_customer = updated_item
            .customers
            .iter()
            .find(|c| c.customer_id == 202)
            .expect("Новый заказчик (202) не найден в ответе");
        assert!(!new_customer.is_removed);
    })
    .await
}

#[tokio::test]
async fn delete_and_restore_reason() {
    run_db_test(&[FIXTURE_FILE], |pool| async move {
        let fixture_id = 1;

        // Удаляем
        let (aggregates, _messages) =
            repository::delete(&[fixture_id], 10, &pool).await.unwrap();
        assert!(aggregates[0].header.is_removed);

        // Восстанавливаем
        let (restored_vec, _messages) =
            repository::restore(&[aggregates[0].header.id], 11, &pool)
                .await
                .unwrap();
        assert!(!restored_vec[0].header.is_removed);
    })
    .await
}
