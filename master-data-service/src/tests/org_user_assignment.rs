use monolith_service::dto::user::MonolithUser;
use shared_essential::domain::DepartmentLevel;
use sqlx::PgPool;

use crate::application::master_data::org_user_assignment_search::{
    self, fetch_user_search_data,
};

use super::db_test::run_db_test;
use testing::monolith::MockMonolithService;

const ORG_USER_ASSIGNMENT_EXTRA_MIGS: &[&str] = &["org_user_assignments.sql"];

pub(super) async fn refresh_maybe_organizational_user_assignment(
    users: Vec<MonolithUser>,
    pool: &PgPool,
) {
    let (_handle, monolith) =
        MockMonolithService::new().search_users_by_id(users).run().unwrap();
    let token = String::new();

    let res =
        org_user_assignment_search::refresh_maybe_organizational_user_assignment(
            token, pool, monolith,
        )
        .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn fetch_search_cache() {
    run_db_test(ORG_USER_ASSIGNMENT_EXTRA_MIGS, |pool| async move {
        let (_handle, monolith) = MockMonolithService::new()
            .search_users_by_id(monolith_users())
            .run()
            .unwrap();
        let token = String::new();

        let res = fetch_user_search_data(&pool, monolith, token).await;
        assert!(res.is_ok(), "{res:?}");

        let users = res.unwrap().users;
        assert_eq!(users.len(), 11);

        let user1 = users.get(&609).expect("user 609");
        assert_eq!(
            &user1.dep_ids[0].org_ids,
            &[
                None,
                Some((21, DepartmentLevel::Department)),
                Some((23, DepartmentLevel::Division)),
            ]
        );
    })
    .await;
}

#[tokio::test]
async fn search_ui_text() {
    run_db_test(ORG_USER_ASSIGNMENT_EXTRA_MIGS, |pool| async move {
        refresh_maybe_organizational_user_assignment(monolith_users(), &pool).await;

        let user_id = 666;
        let search = "ова";
        let organization_structure_id = None;

        let res = org_user_assignment_search::organizational_user_assignment(
            user_id,
            search,
            organization_structure_id,
            0,
            usize::MAX,
        )
        .await;
        assert!(res.is_ok());

        let items = res.unwrap().value;

        assert_eq!(items.len(), 4);
        assert!(items.iter().all(|x| [606, 608, 609, 666].contains(&x.id)));
    })
    .await;
}

#[tokio::test]
async fn search_org_id() {
    run_db_test(ORG_USER_ASSIGNMENT_EXTRA_MIGS, |pool| async move {
        refresh_maybe_organizational_user_assignment(monolith_users(), &pool).await;

        let user_id = 666;
        let search = "";
        let organization_structure_id = Some(22);

        let res = org_user_assignment_search::organizational_user_assignment(
            user_id,
            search,
            organization_structure_id,
            0,
            usize::MAX,
        )
        .await;
        assert!(res.is_ok());

        let items = res.unwrap().value;

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|x| [607, 608].contains(&x.id)));
    })
    .await;
}

#[tokio::test]
async fn search_by_id() {
    run_db_test(ORG_USER_ASSIGNMENT_EXTRA_MIGS, |pool| async move {
        refresh_maybe_organizational_user_assignment(monolith_users(), &pool).await;

        let ids = &[607, 608];

        let res =
            org_user_assignment_search::organizational_user_assignment_by_id(ids)
                .await;
        assert!(res.is_ok());

        let items = res.unwrap().value;

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|x| ids.contains(&x.id)));
    })
    .await;
}

#[tokio::test]
async fn search_same_org_id() {
    run_db_test(ORG_USER_ASSIGNMENT_EXTRA_MIGS, |pool| async move {
        refresh_maybe_organizational_user_assignment(monolith_users(), &pool).await;

        let user_id = 666;
        let search = "";
        let organization_structure_id = None;

        let res = org_user_assignment_search::organizational_user_assignment(
            user_id,
            search,
            organization_structure_id,
            0,
            usize::MAX,
        )
        .await;
        assert!(res.is_ok());

        let items = res.unwrap().value;

        assert_eq!(items.len(), 5);
        assert!(items.iter().all(|x| [606, 607, 608, 609, 666].contains(&x.id)));
    })
    .await;
}

#[tokio::test]
async fn search_same_org_id_from_quant() {
    run_db_test(ORG_USER_ASSIGNMENT_EXTRA_MIGS, |pool| async move {
        refresh_maybe_organizational_user_assignment(monolith_users(), &pool).await;

        let user_id = 666;
        let search = "";
        let organization_structure_id = None;
        let from = 0;
        let quantity = 2;

        let res = org_user_assignment_search::organizational_user_assignment(
            user_id,
            search,
            organization_structure_id,
            from,
            quantity,
        )
        .await;
        assert!(res.is_ok());

        let items = res.unwrap().value;
        assert_eq!(items.len(), quantity);

        let from = quantity;
        let quantity = usize::MAX;

        let res = org_user_assignment_search::organizational_user_assignment(
            user_id,
            search,
            organization_structure_id,
            from,
            quantity,
        )
        .await;
        assert!(res.is_ok());

        let mut items = items;
        items.extend(res.unwrap().value);

        assert_eq!(items.len(), 5);
        assert!(items.iter().all(|x| [606, 607, 608, 609, 666].contains(&x.id)));
    })
    .await;
}

#[tokio::test]
async fn search_sorted_by_last_name() {
    run_db_test(ORG_USER_ASSIGNMENT_EXTRA_MIGS, |pool| async move {
        refresh_maybe_organizational_user_assignment(monolith_users(), &pool).await;

        let user_id = 666;
        let search = "";
        let organization_structure_id = None;

        let res = org_user_assignment_search::organizational_user_assignment(
            user_id,
            search,
            organization_structure_id,
            0,
            usize::MAX,
        )
        .await;
        assert!(res.is_ok());

        let items = res.unwrap().value;

        // Проверяем, что список отсортирован по фамилии
        let mut last_name = String::new();
        for item in &items {
            assert!(
                item.last_name >= last_name,
                "Список не отсортирован по фамилии"
            );
            last_name = item.last_name.clone();
        }
    })
    .await;
}

fn monolith_users() -> Vec<MonolithUser> {
    vec![
        MonolithUser {
            id: 666,
            last_name: "Богомолова".into(),
            first_name: "Мария".into(),
            patronymic_name: "Артёмовна".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 601,
            last_name: "Блинов".into(),
            first_name: "Тимофей".into(),
            patronymic_name: "Денисович".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 602,
            last_name: "Завьялов".into(),
            first_name: "Михаил".into(),
            patronymic_name: "Андреевич".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 603,
            last_name: "Князева".into(),
            first_name: "Ника".into(),
            patronymic_name: "Сергеевна".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 604,
            last_name: "Назаров".into(),
            first_name: "Лука".into(),
            patronymic_name: "Николаевич".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 605,
            last_name: "Верещагина".into(),
            first_name: "Аделина".into(),
            patronymic_name: "Артёмовна".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 606,
            last_name: "Карпова".into(),
            first_name: "Полина".into(),
            patronymic_name: "Артёмовна".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 607,
            last_name: "Яковлева".into(),
            first_name: "Элина".into(),
            patronymic_name: "Павловна".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 608,
            last_name: "Кузнецова".into(),
            first_name: "Виолетта".into(),
            patronymic_name: "Леоновна".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 609,
            last_name: "Попова".into(),
            first_name: "Лилия".into(),
            patronymic_name: "Данииловна".into(),
            ..Default::default()
        },
        MonolithUser {
            id: 658,
            last_name: "Зубов".into(),
            first_name: "Матвей".into(),
            patronymic_name: "Егорович".into(),
            ..Default::default()
        },
    ]
}
