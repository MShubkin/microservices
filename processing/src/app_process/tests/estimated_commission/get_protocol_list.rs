//! Тестирование процесса [`get_agenda_items_for_protocol_create`]
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::db_item::{AsezDate, SelectionKind};

use super::*;
use crate::app_process::get_protocol_list;

const GET_PROTOCOL_LIST_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_protocol_list.sql"];

/// Тестирование кейса, когда пользователю возвращается
/// пустой массив данных
#[tokio::test]
async fn empty_protocol_list() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            select: Select {
                field_list: vec![String::from("id")],
                filter_list: Filter::in_any("id", [123]).into(),
                ..Default::default()
            },
        };

        let result = get_protocol_list(dto, pool.clone()).await.unwrap();
        assert_eq!(result.data.item_list.len(), 0);
    })
    .await;
}

/// Тестирование кейса, когда были возвращены все протоколы без
/// доп данных и специфичных фильтров
#[tokio::test]
async fn simple_protocol_list_selection() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            select: Select {
                field_list: vec![String::from("id"), String::from("protocol_date")],
                filter_list: Filter::in_any("id", [1, 2, 5, 6]).into(),
                ..Default::default()
            },
        };

        let result = get_protocol_list(dto, pool.clone()).await.unwrap();
        let protocols = result.data.item_list;

        // 5 и 6 Протоколы имеет protocol_type_id = 2
        assert_eq!(protocols.len(), 2);

        verify_item(&protocols, 1, None, None, |p| {
            p.protocol_date.unwrap() == AsezDate::try_from("2000-05-01").unwrap()
        });
        verify_item(&protocols, 2, None, None, |p| {
            p.protocol_date.unwrap() == AsezDate::try_from("2001-05-01").unwrap()
        });
    })
    .await;
}

#[tokio::test]
async fn simple_protocol_list_selection_with_status() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto_with_status = GetProtocolListReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            select: Select::with_fields(["created_by", "changed_by", "uuid"])
                .add_expand_filter(
                    "protocol_status_id",
                    SelectionKind::Equals,
                    vec![EcProtocolStatus::Deleted],
                ),
        };
        let dto_without_status = GetProtocolListReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            select: Select::with_fields(["created_by", "changed_by", "uuid"]),
        };

        let response_with_status =
            get_protocol_list(dto_with_status, pool.clone()).await.unwrap();
        let response_without_status =
            get_protocol_list(dto_without_status, pool.clone()).await.unwrap();

        assert_eq!(response_with_status.data.item_list.len(), 2);
        assert_eq!(response_without_status.data.item_list.len(), 4);
    })
    .await;
}

/// Тестирование кейса с специфичным фильтром `protocol_date_year`
#[tokio::test]
async fn complex_protocol_date_year_selection() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            select: Select {
                field_list: vec![String::from("id"), String::from("protocol_date")],
                filter_list: Filter::eq("protocol_date_year", 2000).into(),
                ..Default::default()
            },
        };

        let result = get_protocol_list(dto, pool.clone()).await.unwrap();
        let protocols = result.data.item_list;

        // Только протокол 1 имеет нужный protocol_date_year
        assert_eq!(protocols.len(), 1);

        verify_item(&protocols, 1, None, None, |p| {
            p.protocol_date.unwrap() == AsezDate::try_from("2000-05-01").unwrap()
        });
    })
    .await;
}

/// Тестирование кейса, когда пользователь запрашивает удаленные Протоколы СК.
/// То есть по умолчанию у нас применятеся is_removed фильтр, но если есть фильтр
/// на status_id=500, то он убирается
#[tokio::test]
async fn complex_status_selection() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            select: Select {
                field_list: vec![String::from("id"), String::from("protocol_date")],
                filter_list: Filter::eq("protocol_status_id", 500).into(),
                ..Default::default()
            },
        };

        let result = get_protocol_list(dto, pool.clone()).await.unwrap();
        let protocols = result.data.item_list;

        // Только протоколы 3 и 4 имеют нужный status_id
        assert_eq!(protocols.len(), 2);

        verify_item(&protocols, 3, None, None, |p| {
            p.protocol_date.unwrap() == AsezDate::try_from("2002-01-01").unwrap()
        });
        verify_item(&protocols, 4, None, None, |p| {
            p.protocol_date.unwrap() == AsezDate::try_from("2002-01-01").unwrap()
        });
    })
    .await;
}

/// Тестирование кейса, когда пользователь запрашивает Протоколы СК, которые содержат
/// определенные ППЗ/ДС
#[tokio::test]
async fn complex_plan_selection() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListReq {
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            select: Select {
                field_list: vec![String::from("id"), String::from("protocol_date")],
                filter_list: FilterTree::And(vec![
                    Filter::in_any("customer_id", [8]).into(),
                    Filter::in_any("plan_id", [2, 12]).into(),
                ]),
                ..Default::default()
            },
        };

        let result = get_protocol_list(dto, pool.clone()).await.unwrap();
        let protocols = result.data.item_list;

        // Только Протокол 8 имеет ППЗ/ДС с нужным id и customer_id
        assert_eq!(protocols.len(), 1);

        verify_item(&protocols, 8, None, None, |p| {
            p.protocol_date.unwrap() == AsezDate::try_from("2003-01-01").unwrap()
        });
    })
    .await;
}

/// Тестирование кейса, когда пользователь запрашивает дополнительные данные
/// по protocol_item, но из за protocol_type_id он получит только protocol_item_quantity_threshold
#[tokio::test]
async fn partial_complex_protocol_item_selection() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListReq {
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            select: Select {
                field_list: vec![
                    String::from("id"),
                    String::from("protocol_date"),
                    String::from("protocol_item_quantity_threshold"),
                    String::from("protocol_item_d647_quantity_threshold"),
                ],
                ..Default::default()
            }
            .add_replace_order_desc("protocol_id"),
        };

        let result = get_protocol_list(dto, pool.clone()).await.unwrap();
        let protocols = result.data.item_list;

        assert_eq!(protocols.len(), 4);

        verify_item(
            &protocols,
            5,
            Some(ColorThreshold {
                value: 1,
                color_scheme_id: ColorScheme::Green,
            }),
            None,
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2000-01-01").unwrap()
            },
        );
        verify_item(
            &protocols,
            6,
            Some(ColorThreshold {
                value: 0,
                color_scheme_id: ColorScheme::Green,
            }),
            None,
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2001-01-01").unwrap()
            },
        );
        verify_item(
            &protocols,
            7,
            Some(ColorThreshold {
                value: 1,
                color_scheme_id: ColorScheme::Red,
            }),
            None,
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2002-01-01").unwrap()
            },
        );
        verify_item(
            &protocols,
            8,
            Some(ColorThreshold {
                value: 1,
                color_scheme_id: ColorScheme::Green,
            }),
            None,
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2003-01-01").unwrap()
            },
        );
    })
    .await;
}

/// Тестирование кейса, когда пользователь запрашивает дополнительные данные
/// по protocol_item и они все будут возвращены
#[tokio::test]
async fn full_complex_protocol_item_selection() {
    run_db_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListReq {
            protocol_type_id: ProtocolType::InPersonMeeting,
            select: Select {
                field_list: vec![
                    String::from("id"),
                    String::from("protocol_date"),
                    String::from("protocol_item_quantity_threshold"),
                    String::from("protocol_item_d647_quantity_threshold"),
                ],
                ..Default::default()
            },
        };

        let result = get_protocol_list(dto, pool.clone()).await.unwrap();
        let protocols = result.data.item_list;

        assert_eq!(protocols.len(), 4);

        verify_item(
            &protocols,
            1,
            Some(ColorThreshold {
                value: 1,
                color_scheme_id: ColorScheme::Green,
            }),
            Some(ColorThreshold {
                value: 0,
                color_scheme_id: ColorScheme::Green,
            }),
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2000-05-01").unwrap()
            },
        );
        verify_item(
            &protocols,
            2,
            Some(ColorThreshold {
                value: 0,
                color_scheme_id: ColorScheme::Green,
            }),
            Some(ColorThreshold {
                value: 1,
                color_scheme_id: ColorScheme::Green,
            }),
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2001-05-01").unwrap()
            },
        );
        verify_item(
            &protocols,
            9,
            Some(ColorThreshold {
                value: 0,
                color_scheme_id: ColorScheme::Green,
            }),
            Some(ColorThreshold {
                value: 0,
                color_scheme_id: ColorScheme::Green,
            }),
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2001-01-01").unwrap()
            },
        );
        verify_item(
            &protocols,
            10,
            Some(ColorThreshold {
                value: 0,
                color_scheme_id: ColorScheme::Green,
            }),
            Some(ColorThreshold {
                value: 0,
                color_scheme_id: ColorScheme::Green,
            }),
            |p| {
                p.protocol_date.unwrap()
                    == AsezDate::try_from("2001-01-01").unwrap()
            },
        );
    })
    .await;
}

fn verify_item<F>(
    data: &[GetProtocolListResponseItem],
    id: i64,
    protocol_item_quantity_threshold: Option<ColorThreshold>,
    protocol_item_d647_quantity_threshold: Option<ColorThreshold>,
    verify_fn: F,
) where
    F: FnOnce(&EcProtocolRep) -> bool,
{
    let item = data.iter().find(|p| p.protocol.protocol_id.unwrap() == id).unwrap();
    assert!(verify_fn(&item.protocol));
    assert_eq!(
        item.protocol_item_quantity_threshold,
        protocol_item_quantity_threshold
    );
    assert_eq!(
        item.protocol_item_d647_quantity_threshold,
        protocol_item_d647_quantity_threshold
    );
}
