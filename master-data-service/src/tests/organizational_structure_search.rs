use crate::{
    application::master_data::{
        org_user_assignment_search::{
            OrgHierarchy, OrgUserAssignment, SearchData,
            ORGANIZATIONAL_USER_ASSIGNMENT,
        },
        organizational_structure,
    },
    presentation::dto::OrganizationalStructureSearchReqBody,
};
use ahash::AHashSet;
use asez2_shared_db::{db_item::Select, DbItem};
use itertools::Itertools;
use shared_essential::domain::{DepartmentLevel, OrganizationalStructure};
use tokio::test;

use super::db_test::run_db_test;

const USER_ID: i32 = 699;
const ORG_STRUCT_EXTRA_MIGS: &[&str] = &["organizational_structure.sql"];
const ORG_STRUCT_BIG_EXTRA_MIGS: &[&str] = &["organizational_structure_big.sql"];

const EMPTY_REQ: OrganizationalStructureSearchReqBody =
    OrganizationalStructureSearchReqBody {
        from: 0,
        quantity: 9999,
        search: String::new(),
        organization_structure_id: None,
        level: None,
        is_specialized_department: None,
    };

#[test]
async fn empty_request() {
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        let result = organizational_structure::organizational_structure_search(
            USER_ID,
            OrganizationalStructureSearchReqBody { ..EMPTY_REQ },
            &db_pool,
        )
        .await;

        assert!(result.is_ok());
    })
    .await;
}

/// Тест на фильтрацию только по уровню (`level`)
#[test]
async fn filter_by_level() {
    const LEVEL: DepartmentLevel = DepartmentLevel::Department;
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        let request = OrganizationalStructureSearchReqBody {
            level: Some(LEVEL),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        let expected_ids: Vec<_> = OrganizationalStructure::select(
            &Select::default().eq(OrganizationalStructure::level, LEVEL),
            &*db_pool,
        )
        .await
        .unwrap()
        .into_iter()
        .map(|x| x.id)
        .sorted()
        .collect();

        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}

/// Тест на фильтрацию только по `id` (итеративное построение иерархии)
#[test]
async fn filter_by_id() {
    run_db_test(ORG_STRUCT_BIG_EXTRA_MIGS, |db_pool| async move {
        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(11),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения потомки
        let expected_ids: AHashSet<_> = [
            111, 112, 113, // уровень 3 от 11
            1111, 1112, 1113, // Уровень 4 от 111
        ]
        .into_iter()
        .collect();
        let returned_ids: AHashSet<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();
        assert_eq!(returned_ids, expected_ids);

        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(1),
            level: Some(DepartmentLevel::Department),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения потомки
        let expected_ids: AHashSet<_> = [
            11, 12, 13, // Уровень 2 от 1
        ]
        .into_iter()
        .collect();
        let returned_ids: AHashSet<_> =
            result.value.iter().map(|u| u.id.unwrap()).collect();
        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}

/// Тест на фильтрацию по `id` и `level`
#[test]
async fn filter_by_id_and_level() {
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        // Предположим, выбираем подразделение с ID=2 и уровень=3
        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(21),
            level: Some(DepartmentLevel::Division),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения:
        let expected_ids = vec![22, 23];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}

/// Тест на фильтрацию с поисковым запросом (`search`)
#[test]
async fn filter_by_search() {
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        // Ищем подразделения с текстом, содержащим "отдел"
        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: None,
            from: 0,
            quantity: 100,
            search: "отдел".to_string(),
            level: None,
            is_specialized_department: None,
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения
        let expected_ids = vec![13, 102, 103];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).collect();

        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}

/// Тест на фильтрацию по `is_specialized_department`
#[test]
async fn filter_by_is_specialized_department_true() {
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        // Фильтруем только специализированные департаменты
        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(1),
            is_specialized_department: Some(true),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения
        let expected_ids = vec![12, 13, 22, 23];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        assert_eq!(returned_ids, expected_ids);

        // Фильтруем только специализированные департаменты
        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(1),
            is_specialized_department: Some(true),
            level: Some(DepartmentLevel::Department),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения
        let expected_ids = vec![11, 21];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}

/// Тест на фильтрацию по `is_specialized_department` = false
#[test]
async fn filter_by_is_specialized_department_false() {
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        // Фильтруем только не специализированные департаменты
        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(1),
            is_specialized_department: Some(false),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения
        let expected_ids = vec![102, 103];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        assert_eq!(returned_ids, expected_ids);

        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(1),
            is_specialized_department: Some(false),
            level: Some(DepartmentLevel::Department),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения
        let expected_ids = vec![101];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}

/// Тест с комбинированными фильтрами: `id`, `level` и `is_specialized_department`
#[test]
async fn combined_filters() {
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        // Фильтруем подразделение ID=3, уровень=3, не специализированные департаменты
        let request = OrganizationalStructureSearchReqBody {
            organization_structure_id: Some(101),
            level: Some(DepartmentLevel::Division),
            is_specialized_department: Some(false),
            ..EMPTY_REQ
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения
        let expected_ids = vec![102];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}

/// Тест на фильтрацию по департаменту пользователя
///
/// NB. работа с общей переменной может привести к неожиданным результатам как
/// данного, так и других тестов.
#[test]
#[ignore = "вынести в интеграционный тест"]
async fn filter_by_user_department() {
    run_db_test(ORG_STRUCT_EXTRA_MIGS, |db_pool| async move {
        let org_hierarchy = OrgHierarchy {
            org_ids: vec![
                Some((1, DepartmentLevel::GP)),
                Some((11, DepartmentLevel::Department)),
                Some((12, DepartmentLevel::Division)),
            ],
        };
        let org_user_assignment = OrgUserAssignment {
            dep_ids: vec![org_hierarchy],
            ..Default::default()
        };
        *ORGANIZATIONAL_USER_ASSIGNMENT.write().await = Some(SearchData {
            users: std::iter::once((USER_ID, org_user_assignment)).collect(),
            deps: Default::default(),
            timestamp: Default::default(),
        });

        // Фильтруем только не специализированные департаменты
        let request = OrganizationalStructureSearchReqBody {
            from: 0,
            quantity: 100,
            search: String::new(),
            level: None,
            organization_structure_id: None,
            is_specialized_department: None,
        };

        let result = organizational_structure::organizational_structure_search(
            USER_ID, request, &db_pool,
        )
        .await
        .unwrap();

        // Ожидаемые подразделения
        let expected_ids = vec![12, 13];
        let returned_ids: Vec<_> =
            result.value.iter().map(|u| u.id.unwrap()).sorted().collect();

        assert_eq!(returned_ids, expected_ids);
    })
    .await;
}
