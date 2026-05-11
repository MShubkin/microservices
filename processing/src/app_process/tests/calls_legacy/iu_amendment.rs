use super::*;

use asez2_shared_db::db_item::int_array::AsezArray;
use asez2_shared_db::db_item::AsezTimestamp;
use shared_essential::domain::{ContractAmendment, ContractAmendmentItem};
use shared_essential::presentation::dto::response_request::MessageKind;

const UPDATE_PLANS_EXTRA_MIGS: &[&str] = &["legacy_call.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0001-000000000000").unwrap();
    let uuid_99 = Uuid::parse_str("99000000-0000-0000-0099-000000000000").unwrap();
    let uuid_20 = Uuid::parse_str("20000000-0000-0000-0000-000000000000").unwrap();
    let uuid_10 = Uuid::parse_str("10000000-0000-0000-0000-000000000000").unwrap();
    let uuid_11 = Uuid::parse_str("90000000-0000-0000-0011-000000000000").unwrap();
    let uuid_188 = Uuid::parse_str("99000000-0000-0000-0188-000000000000").unwrap();
    let request = vec![
        AmendmentFromSrm {
            header: ContractAmendmentLegacyRep {
                uuid: Some(uuid_1),
                id: Some(1.to_string()),
                status_id: Some(PlanStatus::from(222)),
                customer_id: Some(11),
                commission_kind_id: Some(CommissionKind::Correspondence),
                contract_subject: Some("Такса от Москвы до Питера".to_string()),
                sum_excluded_vat_rub: Some(999_9999.99.into()),
                // Since this is different from the recorded value (Default::default)
                // we will make a new version.
                pricing_started_at: Some(AsezTimestamp::now().into()),
                kinds: AsezArray(vec![1, 2, 3]).into(),
                ..Default::default()
            },
            items: vec![ContractAmendmentItemLegacyRep {
                uuid: Some(uuid_20),
                number: Some(1002),
                description_internal: Some(
                    "Поводок от Мурманска до Магадана.".to_string(),
                ),
                currency_id: Some(256),
                currency_rate: Some(10.into()),
                quantity: Some(3.into()),
                previous_quantity: Some(1.into()),
                price: Some(59.into()),
                previous_price: Some(44.into()),
                sum_vat: Some(5.5.into()),
                sum_excluded_vat: Some(177.into()),
                sum_included_vat: Some(182.5.into()),
                previous_sum_excluded_vat: Some(44.into()),
                ..Default::default()
            }],
            retrospective_list: None,
            specialized_departments: None,
        },
        AmendmentFromSrm {
            header: ContractAmendmentLegacyRep {
                uuid: Some(uuid_99),
                id: Some(99.to_string()),
                status_id: Some(PlanStatus::from(222)),
                customer_id: Some(11),
                commission_kind_id: Some(CommissionKind::Correspondence),
                contract_subject: Some("Новый план для новый эпохи.".to_string()),
                sum_excluded_vat_rub: Some(999_999_9999.99.into()),
                kinds: AsezArray(vec![]).into(),
                ..Default::default()
            },
            items: vec![ContractAmendmentItemLegacyRep {
                uuid: Some(uuid_10),
                number: Some(1001),
                description_internal: Some("Швабра для новый эпохи.".to_string()),
                currency_id: Some(256),
                currency_rate: Some(10.into()),
                ..Default::default()
            }],
            retrospective_list: None,
            specialized_departments: None,
        },
        AmendmentFromSrm {
            header: ContractAmendmentLegacyRep {
                uuid: Some(uuid_188),
                id: Some(88.to_string()),
                status_id: Some(PlanStatus::from(222)),
                customer_id: Some(11),
                commission_kind_id: Some(CommissionKind::Correspondence),
                contract_subject: Some("Новый план для новый эпохи.".to_string()),
                sum_excluded_vat_rub: Some(999_999_9999.99.into()),
                kinds: AsezArray(vec![]).into(),
                sum_vat: Some(100.into()),
                ..Default::default()
            },
            items: vec![ContractAmendmentItemLegacyRep {
                uuid: Some(uuid_11),
                number: Some(1),
                description_internal: Some("Швабра для новый эпохи.".to_string()),
                currency_id: Some(256),
                currency_rate: Some(10.into()),
                price: Some(10.into()),
                quantity: Some(20.into()),
                sum_vat: Some(100.into()),
                ..Default::default()
            }],
            retrospective_list: None,
            specialized_departments: None,
        },
    ];

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        // We need order for ease of comparison.
        let plan_select =
            Select::default().add_replace_order_asc(ContractAmendment::uuid);

        let initial_plans =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        let initial_items =
            ContractAmendmentItem::select(&Default::default(), &*pool)
                .await
                .unwrap();
        let initial_histories =
            FieldChange::select(&Default::default(), &*pool).await.unwrap();
        let initial_versions =
            ContractAmendmentVersion::select_all(&*pool).await.unwrap();
        let initial_item_versions =
            ContractAmendmentItemVersion::select_all(&*pool).await.unwrap();

        assert!(initial_histories.is_empty());
        assert!(initial_items.is_empty());
        assert_eq!(initial_versions.len(), 2);
        assert!(initial_item_versions.is_empty());
        assert_eq!(initial_plans.len(), 2);

        let pctx = super::mock_processing_context(pool.clone()).await;

        let r = crate::app_process::upsert_legacy_amendment(request, pctx)
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        let final_items =
            ContractAmendmentItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions =
            ContractAmendmentVersion::select_all(&*pool).await.unwrap();
        let final_item_versions =
            ContractAmendmentItemVersion::select_all(&*pool).await.unwrap();

        assert_eq!(final_items.len(), 3);
        assert_eq!(final_plans.len(), 4);
        assert_eq!(final_plans[0].uuid, uuid_1);
        assert_eq!(final_plans[0].contract_amendment_types.0, vec![1, 2, 3]);
        assert_eq!(final_plans[2].uuid, uuid_99);
        // Correspondence is not the same type as in the DB, so it is cleared.
        assert_eq!(final_plans[0].commission_kind_id as i16, 0);
        assert_eq!(final_plans[1].commission_kind_id as i16, 1); // Not updated.
        assert_eq!(final_plans[2].commission_kind_id as i16, 2); // Should be recorded since it is new.
        assert_eq!(final_items[0].uuid, uuid_10);
        assert_eq!(final_items[1].uuid, uuid_20);

        assert_eq!(final_items[1].currency_id, 256);
        assert_eq!(final_items[1].currency_rate, 10.into());
        assert_eq!(final_items[1].quantity, 3.into());
        assert_eq!(final_items[1].previous_quantity, 1.into());
        assert_eq!(final_items[1].price, 59.into());
        assert_eq!(final_items[1].previous_price, 44.into());
        assert_eq!(final_items[1].sum_vat, 5.5.into());
        assert_eq!(final_items[1].previous_sum_vat, 0.into());
        assert_eq!(final_items[1].sum_excluded_vat, 177.into());
        assert_eq!(final_items[1].sum_included_vat, 182.5.into());
        assert_eq!(final_items[1].previous_sum_excluded_vat, 44.into());
        assert_eq!(final_items[1].delta_quantity, Some(2.into()));
        assert_eq!(final_items[1].delta_price, Some(15.into()));
        assert_eq!(final_items[1].delta_sum_vat, Some(5.5.into()));
        assert_eq!(final_items[1].delta_sum_excluded_vat, Some(133.into()));
        assert_eq!(final_items[1].delta_sum_included_vat, Some(182.5.into()));

        // test that for the new header these fields are taken from monolith
        assert_eq!(final_plans[3].uuid, uuid_188);
        assert_eq!(final_plans[3].sum_vat, 100.into());
        assert_eq!(final_items[2].uuid, uuid_11);
        assert_eq!(final_items[2].price, 10.into());
        assert_eq!(final_items[2].quantity, 20.into());
        assert_eq!(final_items[2].sum_vat, 100.into());

        assert_eq!(final_versions.len(), 3);
        // There are no initial items, so no versions are created.
        assert!(final_item_versions.is_empty());

        // Confirm that at least some history is written on update.
        let inserted_histories =
            FieldChange::select(&Default::default(), &*pool).await.unwrap();
        assert!(!inserted_histories.is_empty());
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith_minimal() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0001-000000000000").unwrap();
    let uuid_99 = Uuid::parse_str("99000000-0000-0000-0099-000000000000").unwrap();
    let request = vec![
        AmendmentFromSrm {
            header: ContractAmendmentLegacyRep {
                uuid: Some(uuid_1),
                id: Some(1.to_string()),
                status_id: Some(PlanStatus::from(222)),
                // Since this is different from the recorded value (Default::default)
                // we will make a new version.
                pricing_started_at: Some(AsezTimestamp::now().into()),
                ..Default::default()
            },
            items: vec![],
            ..Default::default()
        },
        AmendmentFromSrm {
            header: ContractAmendmentLegacyRep {
                uuid: Some(uuid_99),
                id: Some(99.to_string()),
                status_id: Some(PlanStatus::from(222)),
                customer_id: Some(11),
                ..Default::default()
            },
            items: vec![],
            ..Default::default()
        },
    ];

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        // We need order for ease of comparison.
        let plan_select =
            Select::default().add_replace_order_asc(ContractAmendment::uuid);

        let initial_plans =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        let initial_items =
            ContractAmendmentItem::select(&Default::default(), &*pool)
                .await
                .unwrap();
        let initial_histories =
            FieldChange::select(&Default::default(), &*pool).await.unwrap();
        let initial_versions =
            ContractAmendmentVersion::select_all(&*pool).await.unwrap();
        let initial_item_versions =
            ContractAmendmentItemVersion::select_all(&*pool).await.unwrap();

        assert!(initial_histories.is_empty());
        assert!(initial_items.is_empty());
        assert_eq!(initial_versions.len(), 2);
        assert!(initial_item_versions.is_empty());
        assert_eq!(initial_plans.len(), 2);

        let pctx = super::mock_processing_context(pool.clone()).await;

        let r = crate::app_process::upsert_legacy_amendment(request, pctx)
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        let final_items =
            ContractAmendmentItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions =
            ContractAmendmentVersion::select_all(&*pool).await.unwrap();
        let final_item_versions =
            ContractAmendmentItemVersion::select_all(&*pool).await.unwrap();

        assert!(final_items.is_empty());
        assert_eq!(final_plans.len(), 3);
        assert_eq!(final_plans[0].uuid, uuid_1);
        assert!(final_plans[0].contract_amendment_types.0.is_empty());
        assert_eq!(final_plans[2].uuid, uuid_99);
        // commission_kind_id is cleared if it is new.
        assert_eq!(final_plans[0].commission_kind_id as i16, 0);
        assert_eq!(final_plans[1].commission_kind_id as i16, 1); // not updated
        assert_eq!(final_plans[2].commission_kind_id as i16, 0); //undefined since it's new.
        assert_eq!(final_versions.len(), 3);
        assert_eq!(final_item_versions.len(), initial_item_versions.len());

        // Confirm that at least some history is written on update.
        let inserted_histories =
            FieldChange::select(&Default::default(), &*pool).await.unwrap();
        assert!(!inserted_histories.is_empty());
    })
    .await;
}
