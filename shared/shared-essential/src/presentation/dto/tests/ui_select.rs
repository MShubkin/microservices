use crate::presentation::dto::{
    general::{ColumnFilter, Filters, UiSelect},
    value::UiValue,
};

use asez2_shared_db::db_item::selection::{
    FieldSortKind, FieldSortOrder, Filter, FilterTree, Select, SelectionKind,
};

#[test]
fn test_convert() {
    let input = UiSelect {
        chunk: None,
        field_list: vec!["id".to_string(), "name".to_string(), "age".to_string()],
        order_list: vec![
            FieldSortOrder {
                field: "age".to_string(),
                order: FieldSortKind::Desc,
                null_position: None,
            },
            FieldSortOrder {
                field: "name".to_string(),
                order: FieldSortKind::Asc,
                null_position: None,
            },
        ],
        filter_list: vec![
            Filters {
                field: "name".to_string(),
                is_key: false,
                values: vec![ColumnFilter {
                    selection_kind: SelectionKind::In,
                    values: vec![UiValue::from("Bob"), UiValue::from("Aerith")],
                }],
            },
            Filters {
                field: "age".to_string(),
                is_key: false,
                values: vec![
                    ColumnFilter {
                        selection_kind: SelectionKind::Equals,
                        values: vec![UiValue::from(34)],
                    },
                    ColumnFilter {
                        selection_kind: SelectionKind::Between,
                        values: vec![UiValue::from(23), UiValue::from(34)],
                    },
                ],
            },
        ],
    };
    // Конечно это не самое логичное, но фронт иначе не умеет.
    let expected = Select {
        field_list: vec!["id".to_string(), "name".to_string(), "age".to_string()],
        order_list: vec![
            FieldSortOrder {
                field: "age".to_string(),
                order: FieldSortKind::Desc,
                null_position: None,
            },
            FieldSortOrder {
                field: "name".to_string(),
                order: FieldSortKind::Asc,
                null_position: None,
            },
        ],
        filter_list: FilterTree::And(vec![
            FilterTree::Filter(Filter::in_any("name", vec!["Bob", "Aerith"])),
            FilterTree::Or(vec![
                FilterTree::Filter(Filter::eq("age", 34)),
                FilterTree::Filter(Filter::between("age", 23, 34)),
            ]),
        ]),
        ..Default::default()
    };

    let actual: Select = input.try_into().unwrap();

    assert_eq!(actual, expected, "{:#?} vs {:#?}", actual, expected);
}
