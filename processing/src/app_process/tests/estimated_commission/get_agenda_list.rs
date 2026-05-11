//! Тестирование процесса `get_agenda_list`
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::db_item::{AsezDate, SelectionKind};
use asez2_shared_db::uuid;

use super::*;
use crate::app_process::get_agenda_list;

const GET_AGENDA_LIST_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_agenda_list.sql"];

/// Тестирование кейса, когда не было найдено повесток
#[tokio::test]
async fn empty_agenda_list() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::full_in::<_, EcAgenda>(
                "uuid",
                vec![Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                    .unwrap()
                    .into()],
            ),
        };

        let result = get_agenda_list(dto, pool.clone()).await.unwrap();

        assert!(result.messages.is_empty());
        assert!(result.data.item_list.is_empty());
    })
    .await;
}

/// Тестирование кейса с примитивным получением повесток СК
#[tokio::test]
async fn get_agenda_list_primitive_success() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["created_by", "changed_by"])
                .add_expand_filter(
                    "is_removed",
                    SelectionKind::Equals,
                    vec![false],
                ),
        };

        let response = get_agenda_list(dto, pool.clone()).await.unwrap();

        let agenda_list = response.data;
        assert_eq!(agenda_list.item_list.len(), 4);
        assert!(agenda_list.item_list.into_iter().enumerate().all(
            |(idx, item)| {
                item.agenda.created_by.unwrap() == 99
                    && item.agenda.changed_by.unwrap() == 99 + idx as i32
            }
        ));
    })
    .await;
}

/// Тестирование кейса с примитивным получением повесток СК
/// c фильтром по году встречи СК
#[tokio::test]
async fn get_agenda_list_with_meeting_date_filter() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto1 = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["created_by", "changed_by", "uuid"])
                .add_expand_filter(
                    "meeting_date_year",
                    SelectionKind::In,
                    vec![2000, 2001],
                )
                .add_replace_order_desc("meeting_date"),
        };
        let dto2 = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["created_by", "changed_by", "uuid"])
                .add_expand_filter(
                    "meeting_date_year",
                    SelectionKind::In,
                    vec![2000],
                ),
        };

        let response1 = get_agenda_list(dto1, pool.clone()).await.unwrap();
        let response2 = get_agenda_list(dto2, pool.clone()).await.unwrap();
        {
            let agenda_list = response1.data;
            assert_eq!(agenda_list.item_list.len(), 4);

            verify_agenda(
                &agenda_list.item_list,
                "00000000-0000-0000-0000-000000000001",
            );
            verify_agenda(
                &agenda_list.item_list,
                "00000000-0000-0000-0000-000000000002",
            );
            verify_agenda(
                &agenda_list.item_list,
                "00000000-0000-0000-0000-000000000003",
            );
            verify_agenda(
                &agenda_list.item_list,
                "00000000-0000-0000-0000-000000000004",
            );
        }
        {
            let agenda_list = response2.data;
            assert_eq!(agenda_list.item_list.len(), 2);

            verify_agenda(
                &agenda_list.item_list,
                "00000000-0000-0000-0000-000000000001",
            );
            verify_agenda(
                &agenda_list.item_list,
                "00000000-0000-0000-0000-000000000002",
            );
        }
    })
    .await;
}

#[tokio::test]
async fn get_agenda_list_with_deleted_status() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto_with_status = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["created_by", "changed_by", "uuid"])
                .add_expand_filter(
                    "agenda_status_id",
                    SelectionKind::Equals,
                    vec![EcAgendaStatus::Deleted],
                ),
        };
        let dto_without_status = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["created_by", "changed_by", "uuid"]),
        };

        let response_with_status =
            get_agenda_list(dto_with_status, pool.clone()).await.unwrap();
        let response_without_status =
            get_agenda_list(dto_without_status, pool.clone()).await.unwrap();

        assert_eq!(response_with_status.data.item_list.len(), 1);

        verify_agenda(
            &response_with_status.data.item_list,
            "00000000-0000-0000-0000-000000000005",
        );

        assert_eq!(response_without_status.data.item_list.len(), 4);
    })
    .await;
}

/// Тестирование кейса с успешным получением повесток СК с доп данными
/// по agenda_item
#[tokio::test]
async fn get_agenda_list_success_with_thresholds() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields([
                "agenda_id",
                "agenda_status_id",
                "pricing_organization_unit_id",
                "agenda_item_quantity_threshold",
                "agenda_item_d647_quantity_threshold",
                "status_id",
            ]),
        };

        let response = get_agenda_list(dto, pool.clone()).await;
        assert!(response.is_ok());

        let agenda_list = response.unwrap().data;
        assert_eq!(agenda_list.item_list.len(), 4);

        let test_vals = vec![
            (
                Some(EcAgendaStatus::Sent),
                Some(ColorFullThreshold {
                    value: [3, 3],
                    color_scheme_id: ColorScheme::Yellow,
                }),
                Some(ColorFullThreshold {
                    value: [2, 2],
                    color_scheme_id: ColorScheme::Green,
                }),
            ),
            (
                Some(EcAgendaStatus::Sent),
                Some(ColorFullThreshold {
                    value: [3, 3],
                    color_scheme_id: ColorScheme::Yellow,
                }),
                Some(ColorFullThreshold {
                    value: [2, 2],
                    color_scheme_id: ColorScheme::Yellow,
                }),
            ),
            (
                Some(EcAgendaStatus::Sent),
                Some(ColorFullThreshold {
                    value: [2, 1],
                    color_scheme_id: ColorScheme::Red,
                }),
                Some(ColorFullThreshold {
                    value: [2, 1],
                    color_scheme_id: ColorScheme::Red,
                }),
            ),
            (
                Some(EcAgendaStatus::ProtocolFormed),
                Some(ColorFullThreshold {
                    value: [1, 1],
                    color_scheme_id: ColorScheme::Undefined,
                }),
                Some(ColorFullThreshold {
                    value: [0, 0],
                    color_scheme_id: ColorScheme::Undefined,
                }),
            ),
        ];
        for (i, (status, threshold, d647_threshold)) in
            test_vals.into_iter().enumerate()
        {
            let ag = &agenda_list.item_list[i];

            assert_eq!(ag.agenda.status_id, status);
            assert_eq!(ag.agenda_item_quantity_threshold, threshold);
            assert_eq!(ag.agenda_item_d647_quantity_threshold, d647_threshold);
        }
    })
    .await;
}

/// Тестирование кейса с успешным получением повесток СК с доп данными
/// по Протоколу СК
#[tokio::test]
async fn get_agenda_list_success_with_protocol_extra() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields([
                "agenda_id",
                "agenda_status_id",
                "protocol_quantity",
            ])
            .in_any(
                EcAgenda::uuid,
                vec![
                    uuid!("00000000-0000-0000-0000-000000000001"),
                    uuid!("00000000-0000-0000-0000-000000000002"),
                    uuid!("00000000-0000-0000-0000-000000000003"),
                ],
            )
            .add_replace_order_asc(EcAgenda::uuid),
        };

        let response = get_agenda_list(dto, pool.clone()).await;
        assert!(response.is_ok());

        let agenda_list = response.unwrap().data;

        assert_eq!(agenda_list.item_list.len(), 3);

        let expected_vals = vec![2, 2, 1];
        agenda_list.item_list.iter().zip(expected_vals).for_each(
            |(item, protocol_quantity)| {
                assert_eq!(item.protocol_quantity.unwrap(), protocol_quantity)
            },
        );
    })
    .await;
}

/// Тестирование кейса с успешным получением повесток СК с доп фильтрами по планам,
/// когда хотя бы один ППЗ/ДС должен подходить под фильтр
#[tokio::test]
async fn get_agenda_list_success_with_plan_filters() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["agenda_id", "agenda_status_id"])
                .add_expand_filter(
                    "plan_id",
                    SelectionKind::In,
                    vec![Value::Int(11)],
                )
                .add_expand_filter(
                    "supplier_id",
                    SelectionKind::In,
                    vec![Value::Int(3)],
                )
                .add_expand_filter(
                    "customer_id",
                    SelectionKind::In,
                    vec![Value::Int(2)],
                ),
        };

        let response = get_agenda_list(dto, pool.clone()).await;
        assert!(response.is_ok());

        let agenda_list = response.unwrap().data;
        assert_eq!(agenda_list.item_list.len(), 1);

        verify_agenda(
            &agenda_list.item_list,
            "00000000-0000-0000-0000-000000000001",
        );
    })
    .await;
}

/// Тестирование сортировок c применением всех фильтров и запросов на доп данные
#[tokio::test]
async fn get_agenda_list_orderings() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields([
                "agenda_id",
                "agenda_status_id",
                "protocol_id_list",
                "agenda_item_quantity_threshold",
                "agenda_item_d647_quantity_threshold",
            ])
            .add_expand_filter("plan_id", SelectionKind::In, vec![Value::Int(1)])
            .add_replace_order_desc(EcAgenda::id),
        };

        let response = get_agenda_list(dto, pool.clone()).await.unwrap();

        let agenda_list = response.data;
        assert_eq!(agenda_list.item_list.len(), 2);

        // Сортировка по айди Повестки по убыванию
        assert_eq!(agenda_list.item_list[0].agenda.agenda_id.unwrap(), 2);
        assert_eq!(agenda_list.item_list[1].agenda.agenda_id.unwrap(), 1);
    })
    .await;
}

/// Тестирование фильтра даты в промежутке [a; a], то есть в любое время в день "а"
#[tokio::test]
async fn get_agenda_list_between_same_date() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["agenda_id"]).add_expand_filter(
                "created_at",
                SelectionKind::Between,
                vec![
                    Value::Date(
                        AsezDate::try_from("27.08.2003")
                            .expect("valid date expected"),
                    ),
                    Value::Date(
                        AsezDate::try_from("27.08.2003")
                            .expect("valid date expected"),
                    ),
                ],
            ),
        };

        let response = get_agenda_list(dto, pool.clone()).await.unwrap();

        let agenda_list = response.data;
        assert_eq!(agenda_list.item_list.len(), 1);
        assert_eq!(agenda_list.item_list[0].agenda.agenda_id.unwrap(), 4);
    })
    .await;
}

/// Тестирование фильтра даты в промежутке [a; b], то есть от "а" до "b", включая весь "b" день
#[tokio::test]
async fn get_agenda_list_with_date_end_included() {
    run_db_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool| async move {
        let dto = GetAgendaListReq {
            section_id: Section::EstimatedCommissionInPerson,
            select: Select::with_fields(["agenda_id"])
                .add_expand_filter(
                    "created_at",
                    SelectionKind::Between,
                    vec![
                        Value::Date(
                            AsezDate::try_from("01.01.1999")
                                .expect("valid date expected"),
                        ),
                        Value::Date(
                            AsezDate::try_from("27.08.2003")
                                .expect("valid date expected"),
                        ),
                    ],
                )
                .add_replace_order_asc(EcAgenda::created_at),
        };

        let response = get_agenda_list(dto, pool.clone()).await.unwrap();

        let agenda_list = response.data;
        assert_eq!(agenda_list.item_list.len(), 2);
        assert_eq!(agenda_list.item_list[0].agenda.agenda_id.unwrap(), 3);
        assert_eq!(agenda_list.item_list[1].agenda.agenda_id.unwrap(), 4);
    })
    .await;
}

fn verify_agenda(agenda_list: &[GetAgendaListItem], uuid: &str) {
    let item = agenda_list
        .iter()
        .find(|agenda| agenda.agenda.uuid.unwrap() == uuid!(uuid));
    assert!(item.is_some(), "Не найдена Повестка {} в {:?}", uuid, agenda_list);
}
