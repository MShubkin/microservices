use std::collections::HashMap;

use shared_essential::presentation::dto::{
    processing::price_analysis::GetPlansWithLastAgendaItemsReq,
    response_request::Status,
};
use tokio::test;

use super::*;
use crate::app_process;

const GET_AGENDA_BY_PLAN_MIGS: &[&str] =
    &["estimated_commission/get_plans_with_last_agenda_item.sql"];
#[test]
async fn test_single_plan_with_valid_agenda() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // ППЗ № 1000012345 включена в Повестку № 1, 2, и 3.
        // - в позиции Повестки №1 по данной ППЗ/ДС установлен признак is_removed = true
        // - в Повестке №3 установлен признак is_removed = true.
        // Поэтому выбирается Повестка №2 и анализируются данные по ППЗ/ДС в ней.
        // Но если в Повестке №2 будет указан признак is_excluded = true, то не выберется ни одной Повестки
        let plan_with_last_agenda_item =
            Uuid::parse_str("00000000-0000-0000-0000-100001234500")
                .expect("Не смог распарсить UUID ППЗ/ДС");

        let req: GetPlansWithLastAgendaItemsReq = GetPlansWithLastAgendaItemsReq {
            plans_uuid: vec![plan_with_last_agenda_item],
        };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::from_iter(vec![(
            Uuid::parse_str("00000000-0000-0000-0000-100001234500")
                .expect("Не смог распарсить UUID ППЗ/ДС"),
            Uuid::parse_str("90101112-0000-0000-0000-000000000011")
                .expect("Не смог распарсить UUID повестки"),
        )]);

        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}

#[test]
async fn test_no_plans_in_request() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // В запросе вообще не указаны ППЗ/ДС
        let req: GetPlansWithLastAgendaItemsReq =
            GetPlansWithLastAgendaItemsReq { plans_uuid: vec![] };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::new();
        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}

#[test]
async fn test_nonexistent_plans() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // В запросе указаны ППЗ/ДС, но они не существуют
        let req: GetPlansWithLastAgendaItemsReq = GetPlansWithLastAgendaItemsReq {
            plans_uuid: vec![Uuid::parse_str(
                "99999999-8888-7777-6666-098765432123",
            )
            .expect("Не смог распарсить UUID ППЗ/ДС")],
        };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::new();
        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}

#[test]
async fn test_plans_without_agendas() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // В запросе указаны ППЗ/ДС, но по ним нет повесток
        let req: GetPlansWithLastAgendaItemsReq = GetPlansWithLastAgendaItemsReq {
            plans_uuid: vec![Uuid::parse_str(
                "00000000-0000-0000-0000-000000000007",
            )
            .expect("Не смог распарсить UUID ППЗ/ДС")],
        };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::new();
        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}

#[test]
async fn test_plans_with_removed_agendas() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // В запросе указаны ППЗ/ДС, но по ним только удаленные повестки (agenda)
        let req: GetPlansWithLastAgendaItemsReq = GetPlansWithLastAgendaItemsReq {
            plans_uuid: vec![Uuid::parse_str(
                "00000000-0000-0000-0000-000000000009",
            )
            .expect("Не смог распарсить UUID ППЗ/ДС")],
        };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::new();
        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}

#[test]
async fn test_plans_with_excluded_and_removed_agenda_items() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // В запросе указаны ППЗ/ДС, но по ним только исключенные и удаленные элементы повесток (agenda_items)
        let req: GetPlansWithLastAgendaItemsReq = GetPlansWithLastAgendaItemsReq {
            plans_uuid: vec![Uuid::parse_str(
                "00000000-0000-0000-0000-000000000003",
            )
            .expect("Не смог распарсить UUID ППЗ/ДС")],
        };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::new();
        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}

#[test]
async fn test_multiple_agenda_items_return_latest() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // Несколько элементов повесток (agenda_items) подходят по запросу, проверяем, что возвращается та у которой наивысшая дата создания
        let plan_with_few_last_agenda_items =
            Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                .expect("Не смог распарсить UUID ППЗ/ДС");

        let req: GetPlansWithLastAgendaItemsReq = GetPlansWithLastAgendaItemsReq {
            plans_uuid: vec![plan_with_few_last_agenda_items],
        };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::from_iter(vec![(
            Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                .expect("Не смог распарсить UUID ППЗ/ДС"),
            Uuid::parse_str("90101112-3333-0000-0000-000000000011")
                .expect("Не смог распарсить UUID повестки"),
        )]);

        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}

#[test]

async fn test_multiple_agendas_return_latest() {
    const REQUEST_FAILED: &str = "Ошибка выполнения запроса к процессингу";
    run_db_test(GET_AGENDA_BY_PLAN_MIGS, |pool| async move {
        // Несколько повесток (agenda) подходят по запросу, проверяем, что возвращается та у которой наивысшая дата создания
        let plan_with_few_last_agenda_items =
            Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                .expect("Не смог распарсить UUID ППЗ/ДС");

        let req: GetPlansWithLastAgendaItemsReq = GetPlansWithLastAgendaItemsReq {
            plans_uuid: vec![plan_with_few_last_agenda_items],
        };
        let response =
            app_process::get_plans_with_last_agenda_items(req, pool.clone())
                .await
                .expect(REQUEST_FAILED);

        let expected_result = HashMap::from_iter(vec![(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                .expect("Не смог распарсить UUID ППЗ/ДС"),
            Uuid::parse_str("00000000-0000-2000-0000-000000000001")
                .expect("Не смог распарсить UUID повестки"),
        )]);

        assert_eq!(response.data.last_agenda_item_hashmap, expected_result);
        assert!(response.status == Status::Ok);
    })
    .await;
}
