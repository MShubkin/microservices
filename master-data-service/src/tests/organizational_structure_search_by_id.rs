use shared_essential::domain::{DepartmentLevel, DepartmentType};

use crate::application::master_data::organizational_structure;

use super::db_test::run_db_test;

#[tokio::test]
async fn search_by_id() {
    run_db_test(&["organizational_structure.sql"], |pool| async move {
        let ids = &[11, 12, 13];

        let res = organizational_structure::search_by_id(ids, &pool).await;
        assert!(res.is_ok(), "{res:?}");

        let items = res.unwrap();
        assert_eq!(items.len(), 3);

        let assert_item =
            |id, text: &str, text_short: &str, level, parent_id, dep_type| {
                let Some(item) = items.iter().find(|x| x.id == Some(id)) else {
                    panic!("no item with id={id}");
                };
                assert_eq!(item.text, Some(text.to_owned()), "text of {id}");
                assert_eq!(
                    item.text_short,
                    Some(text_short.to_owned()),
                    "text_short of {id}"
                );
                assert_eq!(item.level, Some(level), "level of {id}");
                assert_eq!(item.parent_id, Some(parent_id), "parent_id of {id}");
                assert_eq!(item.dep_type, Some(dep_type), "dep_type of {id}");
                assert!(item.created_at.is_some());
                assert!(item.created_by.is_some());
                assert!(item.changed_by.is_some());
                assert!(item.changed_at.is_some());
            };

        assert_item(
            11,
            "Департамент 645",
            "Д645",
            DepartmentLevel::Department,
            Some(1),
            DepartmentType::Department,
        );
        assert_item(
            12,
            "Управление 645/4",
            "У645/4",
            DepartmentLevel::Division,
            Some(11),
            DepartmentType::Division,
        );
        assert_item(
            13,
            "Отдел 645/4/3",
            "О645/4/3",
            DepartmentLevel::SubDivision,
            Some(12),
            DepartmentType::Section,
        );
    })
    .await
}
