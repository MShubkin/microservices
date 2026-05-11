use shared_essential::domain::legacy::plans::PlanStatus;

use super::*;
use crate::app_process;

const UPDATE_PLANS_EXTRA_MIGS: &[&str] = &["estimated_commission/update_plans.sql"];

#[tokio::test]
#[ignore = "Тест переодически падает, пока отключен"]
async fn test_update_plans_a() {
    let plan_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let mut the_plan = PlanRep {
        uuid: Some(plan_uuid),
        id: Some(1),
        status_id: Some(PlanStatus::from(222)),
        customer_id: Some(11),
        contract_subject: Some("Такса от Москвы до Питера".to_string()),
        contract_subject_short: Some("Такса от Москвы до Питера".to_string()),
        sum_excluded_vat_rub: Some(999_9999.99.into()),
        ..Default::default()
    };

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        let request = PrUpdatePlansReq {
            user_id: USER1,
            fields: vec![
                "status_id".to_string(),
                "customer_id".to_string(),
                "contract_subject".to_string(),
                "contract_subject_short".to_string(),
                "sum_excluded_vat_rub".to_string(),
            ],
            plans: vec![PlanOrAmendmentRep::Plan(the_plan.clone())],
        };
        let fields = [
            "uuid",
            "status_id",
            "customer_id",
            "contract_subject",
            "contract_subject_short",
            "sum_excluded_vat_rub",
        ];
        let initial_plans = sqlx::query("select * from plan where uuid = $1;")
            .bind(request.plans[0].uuid().as_ref().unwrap())
            .map(|x| {
                PlanOrAmendmentRep::Plan(PlanRep::from_item(
                    Plan::from_row(&x).unwrap(),
                    Some(&fields),
                ))
            })
            .fetch_all(&*pool)
            .await
            .unwrap();
        // Confirm that there is something to update and we are not doing a null update.
        assert_eq!(initial_plans.len(), 1);
        assert_ne!(initial_plans, request.plans);

        let pctx = super::mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let r = app_process::update_plans(request, pctx.clone()).await.unwrap();

        let messages = r.messages.messages;
        let plans = r.data;

        assert!(messages.is_empty());

        let final_plans_for_check: Vec<Plan> =
            sqlx::query_as("select * from plan where uuid = $1;")
                .bind(the_plan.uuid.as_ref().unwrap())
                .fetch_all(&*pctx.db_pool)
                .await
                .unwrap();
        // Пользователь не запросил changed_at, но нам он требуется для проверки
        let update_moment = final_plans_for_check[0].changed_at;

        let final_plans_for_check = final_plans_for_check
            .into_iter()
            .map(|x| PlanOrAmendmentRep::Plan(PlanRep::from_item(x, Some(&fields))))
            .collect::<Vec<_>>();

        // ID is not one of the retrieved fields, so we expect it to be None.
        the_plan.id = None;

        assert_eq!(plans.item_list.len(), 1);
        // Confirm that we return the updated item.
        assert_eq!(plans.item_list[0], PlanOrAmendmentRep::Plan(the_plan.clone()));

        assert_eq!(final_plans_for_check.len(), 1);
        // Confirm that the final value in the table is what we updated with.
        assert_eq!(final_plans_for_check[0], PlanOrAmendmentRep::Plan(the_plan));

        let s = Select::default();
        let status_histories =
            StatusHistory::select(&s, &*pctx.db_pool).await.unwrap();
        {
            assert_eq!(status_histories.len(), 1);
            assert_eq!(status_histories[0].object_uuid, plan_uuid);
            assert_eq!(status_histories[0].comment, "");
            assert_eq!(status_histories[0].created_by, USER1);
            assert_eq!(status_histories[0].status_id, 222);
        }

        // Confirm that correct history is written on update.
        let inserted_histories =
            sqlx::query("select * from field_history ORDER BY field_name ASC;")
                .map(|x| FieldChange::from_row(&x).unwrap())
                .fetch_all(&*pctx.db_pool)
                .await
                .unwrap();

        let expected_histories = vec![
            FieldChange {
                id: inserted_histories[0].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0000-000000000001",
                )
                .unwrap(),
                table_name: "plan".to_string(),
                field_name: "changed_at".to_string(),
                field_value: Some(SqlxJ(Value::from(
                    update_moment.unix_timestamp(),
                ))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[0].created_at,
            },
            FieldChange {
                id: inserted_histories[1].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0000-000000000001",
                )
                .unwrap(),
                table_name: "plan".to_string(),
                field_name: "contract_subject".to_string(),
                field_value: Some(SqlxJ(Value::from("Такса от Москвы до Питера"))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[1].created_at,
            },
            FieldChange {
                id: inserted_histories[2].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0000-000000000001",
                )
                .unwrap(),
                table_name: "plan".to_string(),
                field_name: "customer_id".to_string(),
                field_value: Some(SqlxJ(Value::from(11))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[2].created_at,
            },
            FieldChange {
                id: inserted_histories[3].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0000-000000000001",
                )
                .unwrap(),
                table_name: "plan".to_string(),
                field_name: "status_id".to_string(),
                field_value: Some(SqlxJ(Value::from(222))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[3].created_at,
            },
            FieldChange {
                id: inserted_histories[4].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0000-000000000001",
                )
                .unwrap(),
                table_name: "plan".to_string(),
                field_name: "sum_excluded_vat_rub".to_string(),
                field_value: Some(SqlxJ(Value::from(999_999_999))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[4].created_at,
            },
        ];
        assert_eq!(inserted_histories, expected_histories);
    })
    .await;
}

#[tokio::test]
#[ignore = "Тест переодически падает, пока отключен"]
async fn test_update_plans_a1() {
    let mut the_plan = ContractAmendmentRep {
        uuid: Some(
            Uuid::parse_str("00000000-0000-0000-0001-000000000000").unwrap(),
        ),
        id: Some(1),
        status_id: Some(PlanStatus::from(222)),
        customer_id: Some(11),
        contract_subject: Some("Такса от Москвы до Питера".to_string()),
        contract_subject_short: Some("Такса от Москвы до Питера".to_string()),
        ..Default::default()
    };

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, |pool| async move {
        let request = PrUpdatePlansReq {
            user_id: USER1,
            fields: vec![
                "status_id".to_string(),
                "customer_id".to_string(),
                "contract_subject".to_string(),
                "contract_subject_short".to_string(),
            ],
            plans: vec![PlanOrAmendmentRep::Amendment(the_plan.clone())],
        };

        let fields = [
            "uuid",
            "status_id",
            "customer_id",
            "contract_subject",
            "contract_subject_short",
        ];
        let initial_plans =
            sqlx::query("select * from contract_amendment where uuid = $1;")
                .bind(request.plans[0].uuid().as_ref().unwrap())
                .map(|x| {
                    PlanOrAmendmentRep::Amendment(ContractAmendmentRep::from_item(
                        ContractAmendment::from_row(&x).unwrap(),
                        Some(&fields),
                    ))
                })
                .fetch_all(&*pool)
                .await
                .unwrap();
        // Confirm that there is something to update and we are not doing a null update.
        assert_eq!(initial_plans.len(), 1);
        assert_ne!(initial_plans, request.plans);

        let pctx = super::mock_processing_context(pool).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let r = app_process::update_plans(request, pctx.clone()).await.unwrap();

        let messages = r.messages.messages;
        let plans = r.data;

        let exp_msgs = match cfg!(with_plan_db) {
            true => {
                vec![Message::warn("Push to SAP is not necessary.".to_string())]
            }
            false => vec![],
        };
        assert_eq!(messages, exp_msgs);

        let mut check_fields = fields.to_vec();
        check_fields.push(ContractAmendment::changed_at);

        let final_plans_for_check: Vec<ContractAmendment> =
            sqlx::query_as("select * from contract_amendment where uuid = $1;")
                .bind(the_plan.uuid.as_ref().unwrap())
                .fetch_all(&*pctx.db_pool)
                .await
                .unwrap();

        let update_moment = final_plans_for_check[0].changed_at;

        let final_plans_for_check = final_plans_for_check
            .into_iter()
            .map(|x| {
                PlanOrAmendmentRep::Amendment(ContractAmendmentRep::from_item(
                    x,
                    Some(&fields),
                ))
            })
            .collect::<Vec<_>>();

        // ID is not one of the retrieved fields, so we expect it to be None.
        the_plan.id = None;

        assert_eq!(plans.item_list.len(), 1);
        // Confirm that we return the updated item.
        assert_eq!(
            plans.item_list[0],
            PlanOrAmendmentRep::Amendment(the_plan.clone())
        );

        assert_eq!(final_plans_for_check.len(), 1);
        // Confirm that the final value in the table is what we updated with.
        assert_eq!(
            final_plans_for_check[0],
            PlanOrAmendmentRep::Amendment(the_plan)
        );

        // Confirm that correct history is written on update.
        let inserted_histories =
            sqlx::query("select * from field_history ORDER BY field_name ASC;")
                .map(|x| FieldChange::from_row(&x).unwrap())
                .fetch_all(&*pctx.db_pool)
                .await
                .unwrap();

        let expected_histories = vec![
            FieldChange {
                id: inserted_histories[0].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0001-000000000000",
                )
                .unwrap(),
                table_name: "contract_amendment".to_string(),
                field_name: "changed_at".to_string(),
                field_value: Some(SqlxJ(Value::from(
                    update_moment.unix_timestamp(),
                ))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[0].created_at,
            },
            FieldChange {
                id: inserted_histories[1].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0001-000000000000",
                )
                .unwrap(),
                table_name: "contract_amendment".to_string(),
                field_name: "contract_subject".to_string(),
                field_value: Some(SqlxJ(Value::from("Такса от Москвы до Питера"))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[1].created_at,
            },
            FieldChange {
                id: inserted_histories[2].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0001-000000000000",
                )
                .unwrap(),
                table_name: "contract_amendment".to_string(),
                field_name: "customer_id".to_string(),
                field_value: Some(SqlxJ(Value::from(11))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[2].created_at,
            },
            FieldChange {
                id: inserted_histories[3].id,
                record_uuid: Uuid::parse_str(
                    "00000000-0000-0000-0001-000000000000",
                )
                .unwrap(),
                table_name: "contract_amendment".to_string(),
                field_name: "status_id".to_string(),
                field_value: Some(SqlxJ(Value::from(222))),
                record_status: HistoryStatus::Finished,
                created_by: USER1,
                created_at: inserted_histories[3].created_at,
            },
        ];
        assert_eq!(
            inserted_histories, expected_histories,
            "{:#?}\n{:#?}",
            inserted_histories, expected_histories
        );
    })
    .await;
}

#[tokio::test]
/// Failure mode: Updating non-existent item.
async fn test_update_plans_b() {
    let the_plan = PlanRep {
        uuid: Some(
            Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
        ),
        id: Some(3),
        customer_id: Some(11),
        contract_subject: Some("Такса от Москвы до Питера".to_string()),
        sum_excluded_vat_rub: Some(999_9999.99.into()),
        ..Default::default()
    };

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, |pool| async move {
        let request = PrUpdatePlansReq {
            user_id: USER1,
            fields: vec![
                "customer_id".to_string(),
                "contract_subject".to_string(),
                "sum_excluded_vat_rub".to_string(),
            ],
            plans: vec![PlanOrAmendmentRep::Plan(the_plan.clone())],
        };

        let starting = sqlx::query("select * from plan where uuid = $1;")
            .bind(the_plan.uuid.as_ref().unwrap())
            .map(|x| Plan::from_row(&x).unwrap())
            .fetch_all(&*pool)
            .await
            .unwrap();
        assert!(starting.is_empty());

        let pctx = super::mock_processing_context(pool).await;

        let r = app_process::update_plans(request, pctx.clone()).await.unwrap_err();

        let exp_error =
            "Строки с UUID 00000000-0000-0000-0000-000000000003 не существует.";
        match r {
            ProcessingError::UpdateFail(ref x, m) => {
                assert_eq!(x, "plan");
                assert_eq!(&m.messages[0].text, exp_error);
            }
            e => panic!("expected update fail fail error: {}", e),
        }

        let fin = sqlx::query("select * from plan where uuid = $1;")
            .bind(the_plan.uuid.as_ref().unwrap())
            .map(|x| Plan::from_row(&x).unwrap())
            .fetch_all(&*pctx.db_pool)
            .await
            .unwrap();
        assert!(fin.is_empty());

        let inserted_histories = sqlx::query("select * from field_history;")
            .map(|x| FieldChange::from_row(&x).unwrap())
            .fetch_all(&*pctx.db_pool)
            .await
            .unwrap();

        let s = Select::default();
        let status_histories =
            StatusHistory::select(&s, &*pctx.db_pool).await.unwrap();
        assert!(status_histories.is_empty());

        // 7 in the record, NB: Calculated fields no longer calculated in this function.
        assert_eq!(inserted_histories.len(), 5, "{:#?}", inserted_histories);
        assert!(
            inserted_histories
                .iter()
                .all(|x| matches!(x.record_status, HistoryStatus::Proposed)),
            "{:#?}",
            inserted_histories
        );
    })
    .await;
}

#[cfg(with_plan_db)]
#[tokio::test(flavor = "multi_thread")]
/// Failure mode: No connection to secondary DB.
async fn test_update_plans_c() {
    let the_plan = PlanRep {
        uuid: Some(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        ),
        id: Some(1),
        customer_id: Some(11),
        contract_subject: Some("Такса от Москвы до Питера".to_string()),
        sum_excluded_vat_rub: Some(999_9999.99.into()),
        ..Default::default()
    };

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, |pool| async move {
        let request = PrUpdatePlansReq {
            user: USER1.to_string(),
            fields: vec![
                "customer_id".to_string(),
                "contract_subject_short".to_string(),
                "sum_excluded_vat_rub".to_string(),
            ],
            plans: vec![PlanOrAmendmentRep::Plan(the_plan.clone())],
        };
        let pctx = super::mock_processing_context(pool).await;

        let r = app_process::update_plans(request, pctx.clone()).await.unwrap_err();

        match r {
            ProcessingError::SapPushFail(ref x, _) => assert_eq!(x, "plan"),
            e => panic!("expected sap push fail error: {:#?}", e),
        }
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_update_plans_d() {
    let the_plan = PlanRep {
        uuid: Some(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        ),
        id: Some(1),
        status_id: Some(PlanStatus::from(342)),
        customer_id: Some(11),
        contract_subject: Some("Такса от Москвы до Питера".to_string()),
        sum_excluded_vat_rub: Some(999_9999.99.into()),
        ..Default::default()
    };

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, |pool| async move {
        let request = PrUpdatePlansReq {
            user_id: USER1,
            fields: vec![
                "status_id".to_string(),
                "customer_id".to_string(),
                "contract_subject".to_string(),
                "contract_subject_short".to_string(),
                "sum_excluded_vat_rub".to_string(),
            ],
            plans: vec![PlanOrAmendmentRep::Plan(the_plan.clone())],
        };
        let pctx = super::mock_processing_context(pool).await;

        let r = app_process::update_plans(request, pctx.clone()).await.unwrap_err();

        let exp_messages: Vec<Message> = vec![
            Message::error(
                "Переход статуса с \"Анализ цены Д646. Назначение исполнителя\" на \"Анализ цены Д647. Исполнитель назначен\" не разрешен (ППЗ/ДС номер 1)".to_string()
            ),
            Message::stop("Rules check for plan failed.".to_string()),
        ];

        match r {
            ProcessingError::RulesLawyer(ref x, messages) => {
                assert_eq!(x, "plan");
                assert_eq!(exp_messages, messages.messages);
            },
            e => panic!("expected rules lawyer fail error: {:#?}", e),
        }
    })
    .await;
}
