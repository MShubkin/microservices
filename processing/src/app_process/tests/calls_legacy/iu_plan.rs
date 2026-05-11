use super::*;

use asez2_shared_db::asez_timestamp;
use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};

use shared_essential::domain::legacy::plans::PlanStatus;
use shared_essential::domain::{Plan, PlanItem, PlanRetrospective};
use shared_essential::presentation::dto::response_request::MessageKind;

const UPDATE_PLANS_EXTRA_MIGS: &[&str] = &["legacy_call.sql"];

#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let uuid_99 = Uuid::parse_str("99000000-0000-0000-0000-000000000099").unwrap();
    let uuid_20 = Uuid::parse_str("20000000-0000-0000-0000-000000000000").unwrap();
    let uuid_10 = Uuid::parse_str("10000000-0000-0000-0000-000000000000").unwrap();
    let request1 = vec![
        PlanFromSrm {
            header: PlanLegacyRep {
                uuid: Some(uuid_1),
                id: Some(1.to_string()),
                status_id: Some(PlanStatus::from(355)),
                customer_id: Some(11),
                contract_subject: Some("Такса от Москвы до Питера".to_string()),
                sum_excluded_vat_rub: Some(9_999_999.99.into()),
                is_actual: Some(true),
                ..Default::default()
            },
            items: vec![PlanItemLegacyRep {
                uuid: Some(uuid_20),
                number: Some(1002),
                description_internal: Some(
                    "Поводок от Мурманска до Магадана.".to_string(),
                ),
                currency_id: Some(256),
                currency_rate: Some(10.into()),
                pricing_price: Some(Some(1000.into())),
                pricing_quantity: Some(Some(2.into())),
                pricing_vat_id: Some(VatId::R10),
                ..Default::default()
            }],
            retrospective_list: Some(vec![PlanRetrospectiveLegacy {
                active_uuid: uuid_1,
                plan_id: "1".to_string(),
                year: 2024,
                status_id: 355.into(),
                is_removed: false,
            }]),
            ..Default::default()
        },
        PlanFromSrm {
            header: PlanLegacyRep {
                uuid: Some(uuid_99),
                id: Some(99.to_string()),
                status_id: Some(PlanStatus::from(355)),
                customer_id: Some(11),
                contract_subject: Some("Новый план для новый эпохи.".to_string()),
                sum_excluded_vat_rub: Some(9_999_999_999.99.into()),
                is_actual: Some(true),
                ..Default::default()
            },
            items: vec![PlanItemLegacyRep {
                uuid: Some(uuid_10),
                number: Some(1001),
                description_internal: Some("Швабра для новый эпохи.".to_string()),
                currency_id: Some(256),
                currency_rate: Some(10.into()),
                pricing_price: Some(Some(1000.into())),
                pricing_quantity: Some(Some(2.into())),
                pricing_vat_id: Some(VatId::R10),
                ..Default::default()
            }],
            ..Default::default()
        },
    ];
    let mut request2 = request1.clone();
    request2[0].header.status_id = Some(PlanStatus::from(251));
    request2[1].header.status_id = Some(PlanStatus::from(251));
    request2[1].header.commission_kind_id = Some(CommissionKind::InPerson);

    let mut request_conflict = request1.clone();
    let some_time = AsezTimestamp::from_unix_timestamp(12341212423);
    request_conflict[0].header.status_id = Some(PlanStatus::from(252));
    request_conflict[1].header.status_id = Some(PlanStatus::from(252));
    request_conflict[0].header.pricing_started_at = Some(some_time.into());
    request_conflict[1].header.pricing_started_at = Some(some_time.into());

    assert_ne!(request_conflict[1].header.id, request_conflict[0].header.id);
    request_conflict[1].header.id = request_conflict[0].header.id.clone();

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        // We need order for ease of comparison.
        let plan_select = Select::default().add_replace_order_asc(Plan::uuid);
        let version_select = Select::default()
            .add_replace_order_asc(PlanVersion::pricing_version)
            .add_replace_order_asc(PlanVersion::id)
            .add_replace_order_asc(PlanVersion::uuid);
        let history =
            Select::default().add_replace_order_asc(FieldChange::created_at);
        let retro_select =
            Select::default().add_replace_order_asc(PlanRetrospective::id);

        let initial_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let initial_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();
        let initial_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let initial_items =
            PlanItem::select(&Default::default(), &*pool).await.unwrap();
        let initial_histories =
            FieldChange::select(&history, &*pool).await.unwrap();
        let plan_retros =
            PlanRetrospective::select(&retro_select, &*pool).await.unwrap();

        assert!(initial_histories.is_empty());
        assert_eq!(initial_items.len(), 4);
        assert_eq!(initial_plans.len(), 2);
        assert_eq!(initial_versions.len(), 2);
        assert_eq!(initial_item_versions.len(), 4);
        assert_eq!(plan_retros.len(), 0);

        let pctx = super::mock_processing_context(pool.clone()).await;

        ////////////////////////////////////////////////////////////////////
        // We perform a first round here, to test basic function
        ///////////////////////////////////////////////////////////////////

        let r = crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let final_items = PlanItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let final_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();
        let plan_retros =
            PlanRetrospective::select(&retro_select, &*pool).await.unwrap();

        assert_eq!(final_items.len(), 6);
        assert_eq!(final_plans.len(), 3);
        assert_eq!(final_plans[0].uuid, uuid_1);
        assert_eq!(final_plans[2].uuid, uuid_99);
        assert_eq!(final_plans[0].pricing_organization_unit_id, PricingUnitId::Gpk);
        assert_eq!(final_plans[2].pricing_organization_unit_id, PricingUnitId::Gpk);
        assert_eq!(final_plans[0].status_id as i16, 355);
        assert_eq!(final_plans[2].status_id as i16, 355);
        assert!(final_plans[0].is_actual);
        assert!(final_plans[1].is_actual);
        assert!(final_plans[2].is_actual);
        assert_eq!(final_items[2].uuid, uuid_10);
        assert_eq!(final_items[3].uuid, uuid_20);
        assert_eq!(final_versions.len(), 2);
        // Both items are new so they get new versions.
        assert_eq!(final_item_versions.len(), 4);
        assert_eq!(plan_retros.len(), 1);

        // Confirm that correct history is written on update.
        let inserted_histories =
            FieldChange::select(&history, &*pool).await.unwrap();
        // For now we update a lot of fields. This will need to be optimised.
        assert_eq!(inserted_histories.len(), 426);

        ////////////////////////////////////////////////////////////////////
        // We perform a second round here, to guarantee that we do not have
        // problems in the version table
        ///////////////////////////////////////////////////////////////////

        let r = crate::app_process::upsert_legacy_plan(request2, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans2 = Plan::select(&plan_select, &*pool).await.unwrap();
        let final_items2 = PlanItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions2 =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let final_item_versions2 =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();

        assert_eq!(final_items2.len(), 6);
        assert_eq!(final_plans2.len(), 3);
        assert_eq!(final_plans2[0].uuid, uuid_1);
        assert_eq!(final_plans2[2].uuid, uuid_99);
        assert_eq!(final_plans2[0].status_id as i16, 251);
        assert_eq!(final_plans2[2].status_id as i16, 251);
        assert_eq!(final_plans2[2].commission_kind_id, CommissionKind::InPerson);
        assert_eq!(final_items2[2].uuid, uuid_10);
        assert_eq!(final_items2[3].uuid, uuid_20);
        assert_eq!(final_versions2.len(), 2);
        assert_eq!(final_item_versions2.len(), 4);

        for f in final_versions2 {
            println!("{:?}", (f.id, f.pricing_version, f.status_id, f.uuid));
            assert_eq!(f.pricing_version, 1);
        }

        for f in final_item_versions2 {
            assert_eq!(f.pricing_version, 1);
        }

        // Confirm that correct history is written on update.
        let inserted_histories2 =
            FieldChange::select(&history, &*pool).await.unwrap();
        // extra changes a second time are:
        // 2 x plan.status_id,
        // TODO: make this test more thorough and systematic.
        assert!(
            inserted_histories2.len() > inserted_histories.len(),
            "{:#?}\n{:#?}",
            inserted_histories2,
            inserted_histories
        );

        ////////////////////////////////////////////////////////////////////
        // We perform a third round here, to guarantee that we disallow imports
        // of records with the same id (which can happen in theory, but in
        // practice should not.
        // We also start a new round of price analysis (new pricing_started_at,
        // so versions should be created)
        ///////////////////////////////////////////////////////////////////

        let r =
            crate::app_process::upsert_legacy_plan(request_conflict, pctx.clone())
                .await
                .unwrap_err();

        match r {
            ProcessingError::SrmHeaderImport(1) => {}
            x => panic!("Wrong error: {:?}", x),
        }
    })
    .await;
}

/// This test tests whether we receive/save certain "important" fields correctly.
/// In this test we are inserting a new plan.
#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith_plan_fields() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();

    let header = PlanLegacyRep {
        uuid: Some(uuid_1),
        id: Some(4.to_string()),
        status_id: Some(PlanStatus::from(225)),
        customer_id: Some(11),
        commission_kind_id: Some(CommissionKind::Correspondence),
        contract_subject: Some("Такса от Москвы до Питера".to_string()),
        sum_excluded_vat_rub: Some(999_9999.99.into()),
        sum_vat_rub: Some(10.into()),
        budget_item_id: Some(200),
        payment_balance_item_id: Some(210),
        limit_on_construction: Some(1_000_000),
        limit_on_works: Some(1_000_010),
        priority: Some(-1),
        priority_income_contract_partner_text: Some(String::from("нужно вчера")),
        extract_number_d646: Some(String::from("XXX1234567")),
        extract_date_d646: Some(AsezDate::try_from("2011-01-01").unwrap()),
        extract_sum_included_vat_rub_d646: Some(20.into()),
        extract_number_d647: Some(String::from("YYY1234567")),
        extract_date_d647: Some(AsezDate::try_from("2011-01-02").unwrap()),
        extract_sum_included_vat_rub_d647: Some(30.into()),
        product_type_id: Some(3),
        organizer_note: Some(String::from("not organised")),
        description: Some(String::from("very good")),
        status_scheme_id: Some(4),
        bid_opening_date: Some(AsezDate::try_from("2011-01-03").unwrap()),
        single_supplier_note_for_expert: Some(String::from(
            "no one makes them like they used to",
        )),
        control_pp_2013: Some(10),
        is_no_qualification: Some(true),
        is_commission: Some(true),
        is_priority_far_eastern: Some(true),
        is_nko: Some(true),
        is_priority_nonprofit: Some(true),
        //----
        contract_sign_date: Some(AsezDate::try_from("2011-02-01").unwrap()),
        active_uuid: Some(Some(uuid_1)),
        //  is actual
        is_actual: Some(false),
        // created & changed
        created_at: Some(asez_timestamp!("2012-01-03 03:04:05").into()),
        changed_at: Some(asez_timestamp!("2012-01-04 03:04:05").into()),
        created_by: Some(999),
        changed_by: Some(666),
        ..Default::default()
    };

    let plan_rep: PlanRep = header.clone().try_into().unwrap();

    let mut plan = plan_rep.into_item().unwrap();
    // Changed by is set as us.

    let request1 = vec![PlanFromSrm {
        header,
        items: vec![],
        ..Default::default()
    }];
    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        let plan_select = Select::default().eq(Plan::uuid, uuid_1).take_first();

        let plans = Plan::select(&plan_select, &*pool).await.unwrap();
        assert!(plans.is_empty());

        let pctx = super::mock_processing_context(pool.clone()).await;

        let r = crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let mut plans = Plan::select(&plan_select, &*pool).await.unwrap();

        assert_eq!(plans.len(), 1);
        let final_plan = plans.pop().unwrap();
        // This is added automatically inside the function.
        plan.pricing_organization_unit_id = PricingUnitId::D646;

        // Final changed_at is updated to us.
        plan.pricing_changed_at = final_plan.pricing_changed_at;
        plan.pricing_created_at = final_plan.pricing_created_at;

        assert_eq!(final_plan.commission_kind_id as i16, 2);
        assert_eq!(final_plan, plan);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith_version_check() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let request_wo_version = vec![PlanFromSrm {
        header: PlanLegacyRep {
            uuid: Some(uuid_1),
            id: Some(1.to_string()),
            status_id: Some(PlanStatus::from(225)),
            customer_id: Some(11),
            contract_subject: Some("Такса от Москвы до Питера".to_string()),
            sum_excluded_vat_rub: Some(999_9999.99.into()),
            is_actual: Some(true),
            ..Default::default()
        },
        items: vec![],
        ..Default::default()
    }];
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let request_with_version = vec![PlanFromSrm {
        header: PlanLegacyRep {
            uuid: Some(uuid_1),
            id: Some(1.to_string()),
            status_id: Some(PlanStatus::from(225)),
            customer_id: Some(11),
            contract_subject: Some("Такса от Москвы до Питера".to_string()),
            sum_excluded_vat_rub: Some(999_9999.99.into()),
            is_actual: Some(true),
            // Since this is different from the recorded value (Default::default)
            // we will make a new version.
            pricing_started_at: Some(AsezTimestamp::now().into()),
            ..Default::default()
        },
        items: vec![],
        ..Default::default()
    }];

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        // We need order for ease of comparison.
        let plan_select = Select::default().add_replace_order_asc(Plan::uuid);
        let version_select = Select::default()
            .add_replace_order_asc(PlanVersion::id)
            .add_replace_order_asc(PlanVersion::pricing_version)
            .add_replace_order_asc(PlanVersion::uuid);
        let history =
            Select::default().add_replace_order_asc(FieldChange::created_at);

        let initial_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let initial_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let initial_histories =
            FieldChange::select(&history, &*pool).await.unwrap();

        assert!(initial_histories.is_empty());
        assert_eq!(initial_versions.len(), 2);
        assert_eq!(initial_plans.len(), 2);

        let pctx = super::mock_processing_context(pool.clone()).await;

        ////////////////////////////////////////////////////////////////////
        // Test update without creating a version (same pricing_started_at)
        ///////////////////////////////////////////////////////////////////
        let r = crate::app_process::upsert_legacy_plan(
            request_wo_version,
            pctx.clone(),
        )
        .await
        .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let plans2 = Plan::select(&plan_select, &*pool).await.unwrap();
        let versions2 = PlanVersion::select(&version_select, &*pool).await.unwrap();

        assert_eq!(plans2.len(), initial_plans.len());
        assert_eq!(versions2.len(), 2);

        ////////////////////////////////////////////////////////////////////
        // Test update with new version (different pricing_started_at)
        ///////////////////////////////////////////////////////////////////
        let r = crate::app_process::upsert_legacy_plan(
            request_with_version,
            pctx.clone(),
        )
        .await
        .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let plans3 = Plan::select(&plan_select, &*pool).await.unwrap();
        let versions3 = PlanVersion::select(&version_select, &*pool).await.unwrap();

        assert_eq!(plans3.len(), initial_plans.len());
        assert_eq!(versions3.len(), 3);
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith_version_check_with_items() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let uuid_40 = Uuid::parse_str("40000000-0000-0000-0000-000000000000").unwrap();
    let request_wo_version = vec![PlanFromSrm {
        header: PlanLegacyRep {
            uuid: Some(uuid_1),
            id: Some(1.to_string()),
            status_id: Some(PlanStatus::from(225)),
            customer_id: Some(11),
            contract_subject: Some("Такса от Москвы до Питера".to_string()),
            is_actual: Some(true),
            // Fields which are ignored if new version is not created.
            sum_vat: Some(333_9999.99.into()),
            sum_vat_rub: Some(444_9999.99.into()),
            sum_excluded_vat: Some(555_9999.99.into()),
            sum_excluded_vat_rub: Some(666_9999.99.into()),
            sum_included_vat: Some(777_9999.99.into()),
            sum_included_vat_rub: Some(999_9999.99.into()),
            // These fields are cleared if version and pricing unit is changed
            pricing_expert_id: Some(234),
            is_check_documentation: Some(false),
            check_documentation_date: Some(Some(
                AsezTimestamp::from_unix_timestamp(1000).into(),
            )),
            ..Default::default()
        },
        items: vec![PlanItemLegacyRep {
            uuid: Some(uuid_40),
            number: Some(1002),
            description_internal: Some(
                "Поводок от Мурманска до Магадана.".to_string(),
            ),
            currency_id: Some(256),
            currency_rate: Some(10.into()),
            pricing_price: Some(Some(1000.into())),
            pricing_quantity: Some(Some(2.into())),
            pricing_vat_id: Some(VatId::R10),
            ..Default::default()
        }],
        ..Default::default()
    }];
    let uuid_99 = Uuid::parse_str("99000000-0000-0000-0000-000000000099").unwrap();
    let uuid_30 = Uuid::parse_str("30000000-0000-0000-0000-000000000000").unwrap();
    let request_with_version = vec![PlanFromSrm {
        header: PlanLegacyRep {
            uuid: Some(uuid_99),
            id: Some(1.to_string()),
            status_id: Some(PlanStatus::from(225)),
            customer_id: Some(11),
            contract_subject: Some("Такса от Москвы до Питера".to_string()),
            is_actual: Some(true),
            // Fields which are ignored if new version is not created.
            sum_vat: Some(333_9999.99.into()),
            sum_vat_rub: Some(444_9999.99.into()),
            sum_excluded_vat: Some(555_9999.99.into()),
            sum_excluded_vat_rub: Some(666_9999.99.into()),
            sum_included_vat: Some(777_9999.99.into()),
            sum_included_vat_rub: Some(999_9999.99.into()),
            vat_id: Some(4.into()),
            // These fields are cleared if version and pricing unit is changed
            pricing_expert_id: Some(235),
            is_check_documentation: Some(true),
            check_documentation_date: Some(Some(
                AsezTimestamp::from_unix_timestamp(1010).into(),
            )),
            // Since this is different from the recorded value (Default::default)
            // we will make a new version.
            pricing_started_at: Some(
                AsezTimestamp::from_unix_timestamp(1000).into(),
            ),
            ..Default::default()
        },
        items: vec![PlanItemLegacyRep {
            uuid: Some(uuid_30),
            number: Some(1001),
            description_internal: Some("Швабра для новый эпохи.".to_string()),
            currency_id: Some(256),
            currency_rate: Some(10.into()),
            pricing_price: Some(Some(1000.into())),
            pricing_quantity: Some(Some(2.into())),
            pricing_vat_id: Some(VatId::R10),
            ..Default::default()
        }],
        ..Default::default()
    }];
    let request_with_version_and_unit = vec![PlanFromSrm {
        header: PlanLegacyRep {
            uuid: Some(uuid_99),
            id: Some(1.to_string()),
            // Changed here.
            status_id: Some(PlanStatus::from(345)),
            customer_id: Some(11),
            contract_subject: Some("Такса от Москвы до Питера".to_string()),
            is_actual: Some(true),
            // Fields which are ignored if new version is not created.
            sum_vat: Some(222_9999.99.into()),
            sum_vat_rub: Some(333_9999.99.into()),
            sum_excluded_vat: Some(444_9999.99.into()),
            sum_excluded_vat_rub: Some(555_9999.99.into()),
            sum_included_vat: Some(666_9999.99.into()),
            sum_included_vat_rub: Some(777_9999.99.into()),
            vat_id: Some(5.into()),
            // These fields are cleared if version and pricing unit is changed
            pricing_expert_id: Some(234),
            is_check_documentation: Some(true),
            check_documentation_date: Some(Some(
                AsezTimestamp::from_unix_timestamp(1000).into(),
            )),
            // Since this is different from the recorded value (Default::default)
            // we will make a new version.
            pricing_started_at: Some(
                AsezTimestamp::from_unix_timestamp(2000).into(),
            ),
            ..Default::default()
        },
        items: vec![PlanItemLegacyRep {
            uuid: Some(uuid_30),
            number: Some(1001),
            description_internal: Some("Швабра для новый эпохи.".to_string()),
            currency_id: Some(256),
            currency_rate: Some(10.into()),
            pricing_price: Some(Some(1000.into())),
            pricing_quantity: Some(Some(2.into())),
            pricing_vat_id: Some(VatId::R10),
            ..Default::default()
        }],
        ..Default::default()
    }];

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        // We need order for ease of comparison.
        let plan_select = Select::default().add_replace_order_asc(Plan::uuid);
        let version_select = Select::default()
            .add_replace_order_asc(PlanVersion::id)
            .add_replace_order_asc(PlanVersion::pricing_version)
            .add_replace_order_asc(PlanVersion::uuid);
        let history =
            Select::default().add_replace_order_asc(FieldChange::created_at);

        let initial_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let initial_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();
        let initial_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let initial_items =
            PlanItem::select(&Default::default(), &*pool).await.unwrap();
        let initial_histories =
            FieldChange::select(&history, &*pool).await.unwrap();

        assert!(initial_histories.is_empty());
        assert_eq!(initial_items.len(), 4);
        assert_eq!(initial_plans.len(), 2);
        assert_eq!(initial_versions.len(), 2);
        assert_eq!(initial_item_versions.len(), 4);

        let pctx = super::mock_processing_context(pool.clone()).await;

        ////////////////////////////////////////////////////////////////////
        // Test update without creating a version (same pricing_started_at)
        ///////////////////////////////////////////////////////////////////
        let r = crate::app_process::upsert_legacy_plan(
            request_wo_version,
            pctx.clone(),
        )
        .await
        .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let final_items = PlanItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let final_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();

        assert_eq!(final_items.len(), 4);
        assert_eq!(final_plans.len(), 2);
        assert_eq!(final_plans[0].uuid, uuid_1);
        assert_eq!(final_items[2].uuid, uuid_30);
        assert_eq!(final_items[3].uuid, uuid_40);
        assert_eq!(final_versions.len(), 2);
        assert_eq!(final_item_versions.len(), 4);
        // Check the fields that should not be set from incoming plans when old version.
        assert_ne!(final_plans[0].sum_vat, 333_9999.99.into());
        assert_ne!(final_plans[0].sum_vat_rub, 444_9999.99.into());
        assert_ne!(final_plans[0].sum_excluded_vat, 555_9999.99.into());
        assert_ne!(final_plans[0].sum_excluded_vat_rub, 666_9999.99.into());
        assert_ne!(final_plans[0].sum_included_vat, 777_9999.99.into());
        assert_ne!(final_plans[0].sum_excluded_vat_rub, 999_9999.99.into());
        assert_ne!(final_plans[0].vat_id, 4.into());
        // These fields are cleared if version and pricing unit is changed
        // In this case they are not updated from the planning module..
        assert_eq!(final_plans[0].pricing_expert_id, Some(1));
        assert!(!final_plans[0].is_check_documentation);
        assert!(final_plans[0].check_documentation_date.is_some());

        ////////////////////////////////////////////////////////////////////
        // Test update creating a version (different pricing_started_at)
        ///////////////////////////////////////////////////////////////////
        let r = crate::app_process::upsert_legacy_plan(
            request_with_version,
            pctx.clone(),
        )
        .await
        .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let final_items = PlanItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let final_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();

        assert_eq!(final_items.len(), 4);
        assert_eq!(final_plans.len(), 3);
        assert_eq!(final_plans[0].uuid, uuid_1);
        assert_eq!(final_plans[2].uuid, uuid_99);
        assert_eq!(final_items[2].uuid, uuid_30);
        assert_eq!(final_items[3].uuid, uuid_40);
        assert_eq!(final_versions.len(), 3);
        assert_eq!(final_item_versions.len(), 5);
        // Check the fields that should not be set from incoming plans when old version.
        assert_eq!(final_plans[2].sum_vat, 333_9999.99.into());
        assert_eq!(final_plans[2].sum_vat_rub, 444_9999.99.into());
        assert_eq!(final_plans[2].sum_excluded_vat, 555_9999.99.into());
        assert_eq!(final_plans[2].sum_excluded_vat_rub, 666_9999.99.into());
        assert_eq!(final_plans[2].sum_included_vat, 777_9999.99.into());
        assert_eq!(final_plans[2].sum_included_vat_rub, 999_9999.99.into());
        assert_eq!(final_plans[2].vat_id, 4.into());
        // These fields are cleared if version and pricing unit is changed
        // In this case they are updated despite the new version because we are on
        // an EC status.
        assert_eq!(final_plans[2].pricing_expert_id, Some(235));
        assert!(final_plans[2].is_check_documentation);
        assert!(final_plans[2].check_documentation_date.is_some());

        ////////////////////////////////////////////////////////////////////
        // Test update creating a version and organisation id (different pricing_started_at)
        ///////////////////////////////////////////////////////////////////

        let mut update = final_plans[0].clone();
        update.is_check_documentation = true;
        // Don't do this at home!
        update = update
            .update_returning::<_, &str>(
                Some(&[Plan::is_check_documentation]),
                None,
                &*pool,
            )
            .await
            .unwrap();
        assert!(update.is_check_documentation);

        let r = crate::app_process::upsert_legacy_plan(
            request_with_version_and_unit,
            pctx.clone(),
        )
        .await
        .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let final_items = PlanItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let final_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();

        assert_eq!(final_items.len(), 4);
        assert_eq!(final_plans.len(), 3);
        assert_eq!(final_plans[0].uuid, uuid_1);
        assert_eq!(final_plans[2].uuid, uuid_99);
        assert_eq!(final_items[2].uuid, uuid_30);
        assert_eq!(final_items[3].uuid, uuid_40);
        assert_eq!(final_versions.len(), 4);
        assert_eq!(final_item_versions.len(), 6);
        // Check the fields that should not be set from incoming plans when old version.
        assert_eq!(final_plans[2].sum_vat, 222_9999.99.into());
        assert_eq!(final_plans[2].sum_vat_rub, 333_9999.99.into());
        assert_eq!(final_plans[2].sum_excluded_vat, 444_9999.99.into());
        assert_eq!(final_plans[2].sum_excluded_vat_rub, 555_9999.99.into());
        assert_eq!(final_plans[2].sum_included_vat, 666_9999.99.into());
        assert_eq!(final_plans[2].sum_included_vat_rub, 777_9999.99.into());
        assert_eq!(final_plans[2].vat_id, 5.into());
        // These fields are cleared if version and pricing unit is changed
        assert!(final_plans[2].pricing_expert_id.is_none());
        assert!(!final_plans[2].is_check_documentation);
        assert!(final_plans[2].check_documentation_date.is_none());
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith_minimum_test() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let uuid_99 = Uuid::parse_str("99000000-0000-0000-0000-000000000099").unwrap();
    let request1 = vec![
        PlanFromSrm {
            header: PlanLegacyRep {
                uuid: Some(uuid_1),
                id: Some(1.to_string()),
                status_id: Some(PlanStatus::from(355)),
                is_actual: Some(true),
                ..Default::default()
            },
            items: vec![],
            ..Default::default()
        },
        PlanFromSrm {
            header: PlanLegacyRep {
                uuid: Some(uuid_99),
                id: Some(99.to_string()),
                status_id: Some(PlanStatus::from(355)),
                is_actual: Some(true),
                ..Default::default()
            },
            items: vec![],
            ..Default::default()
        },
    ];

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        // We need order for ease of comparison.
        let plan_select = Select::default().add_replace_order_asc(Plan::uuid);
        let version_select = Select::default()
            .add_replace_order_asc(PlanVersion::id)
            .add_replace_order_asc(PlanVersion::pricing_version)
            .add_replace_order_asc(PlanVersion::uuid);
        let history =
            Select::default().add_replace_order_asc(FieldChange::created_at);

        let initial_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let initial_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();
        let initial_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let initial_items =
            PlanItem::select(&Default::default(), &*pool).await.unwrap();
        let initial_histories =
            FieldChange::select(&history, &*pool).await.unwrap();

        assert!(initial_histories.is_empty());
        assert_eq!(initial_items.len(), 4);
        assert_eq!(initial_plans.len(), 2);
        assert_eq!(initial_versions.len(), 2);
        assert_eq!(initial_item_versions.len(), 4);

        let pctx = super::mock_processing_context(pool.clone()).await;

        ////////////////////////////////////////////////////////////////////
        // We perform a first round here, to test basic function
        ///////////////////////////////////////////////////////////////////

        let r = crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let final_items = PlanItem::select(&plan_select, &*pool).await.unwrap();
        let final_versions =
            PlanVersion::select(&version_select, &*pool).await.unwrap();
        let final_item_versions =
            PlanItemVersion::select(&version_select, &*pool).await.unwrap();

        assert_eq!(final_items.len(), 4);
        assert_eq!(final_plans.len(), 3);
        assert_eq!(final_plans[0].uuid, uuid_1);
        assert_eq!(final_plans[2].uuid, uuid_99);
        // Undefined since it's changed between updates.
        assert_eq!(final_plans[0].commission_kind_id as i16, 0);
        assert_eq!(final_plans[1].commission_kind_id as i16, 1); // Not updated
        assert_eq!(final_plans[2].commission_kind_id as i16, 0); //undefined since it's new.

        // Version not created: pricing_started_at is default in both existing record and new.
        assert_eq!(final_versions.len(), 2);
        assert_eq!(final_item_versions.len(), initial_item_versions.len());

        // Confirm that correct history is written on update.
        let inserted_histories =
            FieldChange::select(&history, &*pool).await.unwrap();
        // This will need to be reoptimised.
        assert_eq!(inserted_histories.len(), 184);
    })
    .await;
}

/// This test tests that we handle different field sets on existing items adequately.
#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith_varying_fields() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let uuid_2 = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    let header = PlanLegacyRep {
        uuid: Some(uuid_1),
        id: Some(4.to_string()),
        status_id: Some(PlanStatus::from(225)),
        customer_id: Some(11),
        commission_kind_id: Some(CommissionKind::Correspondence),
        contract_subject: Some("Такса от Москвы до Питера".to_string()),
        sum_excluded_vat_rub: Some(999_9999.99.into()),
        sum_vat_rub: Some(10.into()),
        // created & changed
        created_at: Some(asez_timestamp!("2012-01-03 03:04:05").into()),
        changed_at: Some(asez_timestamp!("2012-01-04 03:04:05").into()),
        created_by: Some(999),
        changed_by: Some(666),
        ..Default::default()
    };

    let item1 = PlanItemLegacyRep {
        uuid: Some(uuid_1),
        number: Some(1001),
        description_internal: Some("Швабра для новый эпохи.".to_string()),
        currency_id: Some(256),
        currency_rate: Some(10.into()),
        pricing_price: Some(Some(1000.into())),
        pricing_quantity: Some(Some(2.into())),
        pricing_vat_id: Some(asez2_tables::maths::VatId::R10),
        ..Default::default()
    };
    let item2 = PlanItemLegacyRep {
        uuid: Some(uuid_2),
        number: Some(1001),
        description_internal: Some("Швабра для новый эпохи.".to_string()),
        currency_id: Some(256),
        currency_rate: Some(10.into()),
        pricing_price: Some(Some(1000.into())),
        pricing_quantity: Some(Some(2.into())),
        pricing_vat_id: Some(asez2_tables::maths::VatId::R10),
        is_not_russian_delivery: Some(true),
        is_centralized_delivery: Some(true),
        repair_text: Some("fix me please".to_string()),
        unit_id: Some(647),
        created_at: Some(asez_timestamp!("2012-01-03 03:04:05").into()),
        changed_at: Some(asez_timestamp!("2012-01-04 03:04:05").into()),
        created_by: Some(999),
        changed_by: Some(666),
        ..Default::default()
    };

    let plan_rep: PlanRep = header.clone().try_into().unwrap();

    let mut plan = plan_rep.into_item().unwrap();
    // Changed by is set as us.
    plan.changed_by = 0;

    let request1 = vec![PlanFromSrm {
        header,
        items: vec![item1, item2],
        ..Default::default()
    }];
    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        let plan_select = Select::default().eq(Plan::uuid, uuid_1).take_first();

        let plans = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(plans.len(), 1);

        let pctx = super::mock_processing_context(pool.clone()).await;

        let r = crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let plans = Plan::select(&plan_select, &*pool).await.unwrap();
        let item_count = PlanItem::select_all(&*pool).await.unwrap().len();

        // We are purely updating items.
        assert_eq!(item_count, 4);
        assert_eq!(plans.len(), 1);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_update_from_monolith_minimum_commission_kind() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let request1 = vec![PlanFromSrm {
        header: PlanLegacyRep {
            uuid: Some(uuid_1),
            id: Some(1.to_string()),
            status_id: Some(PlanStatus::from(355)),
            is_actual: Some(true),
            ..Default::default()
        },
        items: vec![],
        ..Default::default()
    }];
    let mut req2 = request1.clone();
    let mut req3 = request1.clone();

    req2[0].header.commission_kind_id = Some(CommissionKind::InPerson);
    req3[0].header.commission_kind_id = Some(CommissionKind::InPerson);
    req3[0].header.status_id = Some(PlanStatus::EstimatedCommissionNo);
    let mut req4 = req3.clone();
    req4[0].header.commission_kind_id = Some(CommissionKind::Correspondence);

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        // We need order for ease of comparison.
        let plan_select = Select::default().add_replace_order_asc(Plan::uuid);
        let pctx = super::mock_processing_context(pool.clone()).await;

        ////////////////////////////////////////////////////////////////////
        // We perform a first round here, to test basic function
        ///////////////////////////////////////////////////////////////////

        let r = crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(final_plans[0].uuid, uuid_1);
        // Undefined since it's changed between updates.
        assert_eq!(final_plans[0].commission_kind_id as i16, 0);
        assert_eq!(final_plans[0].pricing_organization_unit_id, PricingUnitId::Gpk);

        /////////////////////////////////////////////////////////////////////////
        // Test that commission Kind does not change if the status is not EC
        let r = crate::app_process::upsert_legacy_plan(req2, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(final_plans[0].uuid, uuid_1);
        // Undefined since it's changed between updates.
        assert_eq!(final_plans[0].commission_kind_id as i16, 0);
        assert_eq!(final_plans[0].pricing_organization_unit_id, PricingUnitId::Gpk);

        /////////////////////////////////////////////////////////////////////////
        // Test that commission Kind does change if the status is EC, and blank in DB
        let r = crate::app_process::upsert_legacy_plan(req3, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(final_plans[0].uuid, uuid_1);
        // Undefined since it's changed between updates.
        assert_eq!(final_plans[0].commission_kind_id as i16, 1);
        assert_eq!(final_plans[0].pricing_organization_unit_id, PricingUnitId::Gpk);

        /////////////////////////////////////////////////////////////////////////
        // Test that commission Kind does not change if the status is EC, and NOT blank in DB
        let r = crate::app_process::upsert_legacy_plan(req4, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r.kind, MessageKind::Success);
        assert!(r.messages.is_empty());

        let final_plans = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(final_plans[0].uuid, uuid_1);
        // Undefined since it's changed between updates.
        assert_eq!(final_plans[0].commission_kind_id as i16, 1);
        assert_eq!(final_plans[0].pricing_organization_unit_id, PricingUnitId::Gpk);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_delete_retrospectives_from_monolith() {
    let uuid_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let uuid_20 = Uuid::parse_str("20000000-0000-0000-0000-000000000000").unwrap();
    let uuid_99 = Uuid::parse_str("99000000-0000-0000-0000-000000000099").unwrap();
    let uuid_10 = Uuid::parse_str("10000000-0000-0000-0000-000000000000").unwrap();

    let request1 = vec![
        PlanFromSrm {
            header: PlanLegacyRep {
                uuid: Some(uuid_1),
                id: Some(1.to_string()),
                status_id: Some(PlanStatus::from(355)),
                customer_id: Some(11),
                contract_subject: Some("Такса от Москвы до Питера".to_string()),
                sum_excluded_vat_rub: Some(9_999_999.99.into()),
                is_actual: Some(true),
                ..Default::default()
            },
            items: vec![PlanItemLegacyRep {
                uuid: Some(uuid_20),
                number: Some(1002),
                description_internal: Some(
                    "Поводок от Мурманска до Магадана.".to_string(),
                ),
                currency_id: Some(256),
                currency_rate: Some(10.into()),
                pricing_price: Some(Some(1000.into())),
                pricing_quantity: Some(Some(2.into())),
                pricing_vat_id: Some(VatId::R10),
                ..Default::default()
            }],
            retrospective_list: Some(vec![
                PlanRetrospectiveLegacy {
                    active_uuid: uuid_1,
                    plan_id: "1".to_string(),
                    year: 2024,
                    status_id: 355.into(),
                    is_removed: false,
                },
                PlanRetrospectiveLegacy {
                    active_uuid: uuid_1,
                    plan_id: "2".to_string(),
                    year: 2024,
                    status_id: 355.into(),
                    is_removed: false,
                },
                PlanRetrospectiveLegacy {
                    active_uuid: uuid_1,
                    plan_id: "3".to_string(),
                    year: 2024,
                    status_id: 355.into(),
                    is_removed: false,
                },
            ]),
            ..Default::default()
        },
        PlanFromSrm {
            header: PlanLegacyRep {
                uuid: Some(uuid_99),
                id: Some(99.to_string()),
                status_id: Some(PlanStatus::from(355)),
                customer_id: Some(11),
                contract_subject: Some("Новый план для новый эпохи.".to_string()),
                sum_excluded_vat_rub: Some(9_999_999_999.99.into()),
                is_actual: Some(true),
                ..Default::default()
            },
            items: vec![PlanItemLegacyRep {
                uuid: Some(uuid_10),
                number: Some(1001),
                description_internal: Some("Швабра для новый эпохи.".to_string()),
                currency_id: Some(256),
                currency_rate: Some(10.into()),
                pricing_price: Some(Some(1000.into())),
                pricing_quantity: Some(Some(2.into())),
                pricing_vat_id: Some(VatId::R10),
                ..Default::default()
            }],
            retrospective_list: Some(vec![
                PlanRetrospectiveLegacy {
                    active_uuid: uuid_99,
                    plan_id: "1".to_string(),
                    year: 2024,
                    status_id: 355.into(),
                    is_removed: false,
                },
                PlanRetrospectiveLegacy {
                    active_uuid: uuid_99,
                    plan_id: "2".to_string(),
                    year: 2024,
                    status_id: 355.into(),
                    is_removed: false,
                },
                PlanRetrospectiveLegacy {
                    active_uuid: uuid_99,
                    plan_id: "3".to_string(),
                    year: 2024,
                    status_id: 355.into(),
                    is_removed: false,
                },
            ]),
            ..Default::default()
        },
    ];

    let mut request2 = request1.clone();
    request2[0].header.status_id = Some(PlanStatus::from(251));
    request2[0].retrospective_list = Some(vec![PlanRetrospectiveLegacy {
        active_uuid: uuid_1,
        plan_id: "2".to_string(),
        year: 2024,
        status_id: 355.into(),
        is_removed: false,
    }]);

    request2[1].header.status_id = Some(PlanStatus::from(251));
    request2[1].header.commission_kind_id = Some(CommissionKind::InPerson);
    request2[1].retrospective_list = Some(vec![PlanRetrospectiveLegacy {
        active_uuid: uuid_99,
        plan_id: "3".to_string(),
        year: 2024,
        status_id: 355.into(),
        is_removed: false,
    }]);

    let mut request3 = request1.clone();
    request3[0].retrospective_list = None;
    request3[1].retrospective_list = None;

    run_db_test(UPDATE_PLANS_EXTRA_MIGS, move |pool| async move {
        let pctx = super::mock_processing_context(pool.clone()).await;

        let r1 = crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r1.kind, MessageKind::Success);
        assert!(r1.messages.is_empty());
        let retro_select =
            Select::default().add_replace_order_asc(PlanRetrospective::id);
        let first_request_retros =
            PlanRetrospective::select(&retro_select, &*pool).await.unwrap();
        assert_eq!(first_request_retros.len(), 6);

        let r2 = crate::app_process::upsert_legacy_plan(request2, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r2.kind, MessageKind::Success);
        assert!(r2.messages.is_empty());
        let retro_select =
            Select::default().add_replace_order_asc(PlanRetrospective::id);
        let second_request_retros =
            PlanRetrospective::select(&retro_select, &*pool).await.unwrap();
        assert_eq!(second_request_retros.len(), 2);
        assert_eq!(second_request_retros[0].id, 2);
        assert_eq!(second_request_retros[0].plan_uuid, uuid_1);
        assert_eq!(second_request_retros[1].id, 6);
        assert_eq!(second_request_retros[1].plan_uuid, uuid_99);

        let r3 = crate::app_process::upsert_legacy_plan(request3, pctx.clone())
            .await
            .unwrap();

        assert_eq!(r3.kind, MessageKind::Success);
        assert!(r3.messages.is_empty());
        let retro_select =
            Select::default().add_replace_order_asc(PlanRetrospective::id);
        let third_request_retros =
            PlanRetrospective::select(&retro_select, &*pool).await.unwrap();
        assert_eq!(third_request_retros.len(), 0);
    })
    .await;
}
