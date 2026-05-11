//! Tests here aim to make sure that fields are reset or saved properly. We do not
//! However test all possible branch points of the mechanism.
use super::*;

use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};

use shared_essential::domain::Plan;

fn make_items(header_uuid: Uuid, item_uuid: Uuid) -> (Plan, PlanItemFull) {
    let header = Plan {
        uuid: header_uuid,
        id: 4,
        version_type: 199,
        version_number: 299,
        posting_date: AsezDate::try_from_yo(399, 100).unwrap(),
        year: 299,
        commission_kind_id: 299.into(),
        commission_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        customer_id: 499,
        supplier_id: 599,
        executor_method_id: 699.into(),
        supplier_text: Some(799.to_string()),
        contract_subject: 899.to_string(),
        sum_excluded_vat: 999.into(),
        sum_excluded_vat_rub: 199.into(),
        sum_vat: 299.into(),
        currency_id: 299,
        currency_rate: 399.into(),
        for_price_analysis: false, // TODO: ignoring field for now.
        purchasing_type_id: 599,
        status_id: 223.into(),
        delivery_start_date: AsezDate::try_from_yo(2299, 100).unwrap(),
        delivery_end_date: AsezDate::try_from_yo(2399, 100).unwrap(),
        section_id: 939,
        kod_st_buda: Some(959.to_string()),
        okdp2: Some(993.to_string()),
        category_id: Some(959.to_string()),
        code_type: Some(992.into()),
        // Fields below here are generally new.
        declarant_id: 199,
        agent_id: 995,
        initiator_user_id: 99,
        tender_user_id: 99,
        organizer_id: Some(99),
        minimal_requirements: 99.to_string(),
        customer_note: Some(99.to_string()),
        number_customer: 99.to_string(),
        number_cgg: Some(99.to_string()),
        vat_id: VatId::R11,
        sum_included_vat: 99.into(),
        sum_included_vat_rub: 99.into(),
        purchasing_method_id: 99,
        purchasing_kind_id: 99,
        regulation_document_id: 99,
        publication_type_id: 99,
        master_system_id: Some(99),
        funding_source_id: 99,
        purchasing_trend_id: 99,
        is_smb: true,
        is_smb_sub: true,
        smb_exception_id: Some(99),
        smb_sub_percent: Some(99.into()),
        smb_sub_sum: Some(99.into()),
        documentation_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        publication_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        summing_up_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        contract_sing_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        single_supplier_reason_id: 99,
        single_supplier_expert_id: Some(99),
        single_supplier_decision_id: Some(99),
        single_supplier_decision_resume: Some(99.to_string()),
        is_affiliated: true,
        is_supplier_smb: true,
        is_competitive_now: true,
        management_order_number: None, // Currently incompatible.
        management_order_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        reason_document: Some(99.to_string()),
        is_approved_by_d646: true,
        is_price_analysis_by_d646: true,
        is_approver_by_d647: false,
        is_onm: true,
        is_agent_fee: true,
        agent_contract_number: Some(99.to_string()),
        is_design_stage: true,
        repair_stage_id: Some(99),
        is_gas_supply: false,
        is_little_cost: true,
        is_banking_support: false,
        is_innovative: false,
        is_to_publish: true,
        rationale_for_not_publication: Some(99.to_string()),
        rationale_for_publication: Some(99.to_string()),
        is_cooperative: true,
        is_not_purchase: false,
        is_under_control: false,
        rationale_is_under_control: Some(false.to_string()),
        technical_developer: Some(false.to_string()),
        is_priority_project: true,
        priority_project_document: Some(99.to_string()),
        is_priority_introductory: true,
        priority_introductory_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        priority_introductory_document: Some(99.to_string()),
        is_priority_repair: false,
        priority_repair_document: Some(99.to_string()),
        is_priority_ozp: false,
        priority_ozp_document: Some(99.to_string()),
        is_priority_income_contract: false,
        priority_income_contract_document: Some(99.to_string()),
        priority_income_contract_partner_id: Some(99),
        is_priority_other: false,
        is_headquarters: true,
        is_first_time: true,
        is_list_price: true,
        is_removed: false,
        general_contract_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        general_contract_number: Some(99.to_string()),
        general_contract_stages: Some(99.to_string()),
        items_number: 99,
        publication_start_date: AsezDate::try_from_yo(299, 100).unwrap(),
        is_lease_supplier_selection: true,
        is_performance_indicator: true,
        competitive_note_for_expert: Some(99.to_string()),
        expert_conclusion_id: Some(99.into()),
        is_check_documentation: true,
        check_documentation_date: Some(AsezTimestamp::from_unix_timestamp(299)),

        savings_accounting_id: 99.into(),
        savings_sum_excluded_vat: Some(99.into()),
        savings_sum_excluded_vat_rub: Some(99.into()),
        savings_sum_included_vat: Some(99.into()),
        savings_sum_included_vat_rub: Some(99.into()),

        // pricing
        pricing_method_id: 99,
        pricing_expert_id: Some(99),
        pricing_organization_unit_id: PricingUnitId::D646,
        pricing_resume: Some(99.to_string()),
        pricing_competitive_note_for_expert: Some(99.to_string()),

        // TODO не знаю нужны ли тут, но в ДС есть еще поля:
        // is_pricing_by_d646: 99.into(),
        // is_pricing_by_d647: 99.into(),
        is_pricing_by_complectation: true,

        pricing_vat_id: VatId::R11,
        pricing_currency_id: Some(99),
        pricing_currency_rate: Some(99.into()),
        pricing_sum_excluded_vat: 99.into(),
        pricing_sum_excluded_vat_rub: Some(99.into()),
        pricing_sum_included_vat: Some(99.into()),
        pricing_sum_included_vat_rub: Some(99.into()),
        pricing_sum_vat: Some(99.into()),
        pricing_sum_vat_rub: Some(99.into()),
        pricing_transportation_vat_id: VatId::R11,
        pricing_transportation_price: Some(99.into()),
        pricing_transportation_price_rub: Some(99.into()),
        pricing_transportation_sum_vat: Some(99.into()),
        pricing_transportation_sum_vat_rub: Some(99.into()),
        pricing_transportation_sum_included_vat: Some(99.into()),
        pricing_transportation_sum_included_vat_rub: Some(99.into()),
        pricing_total_sum: Some(99.into()),
        pricing_total_sum_rub: Some(99.into()),
        // Доп. Поля с 2024-11-10
        sum_vat_rub: 99.into(),
        budget_item_id: 99,
        payment_balance_item_id: 99,
        limit_on_construction: 99.into(),
        limit_on_works: 99.into(),
        priority: 99,
        priority_income_contract_partner_text: 99.to_string(),
        extract_number_d646: 99.to_string(),
        extract_date_d646: AsezDate::try_from_yo(299, 100).unwrap(),
        extract_sum_included_vat_rub_d646: 99.into(),
        extract_number_d647: 99.to_string(),
        extract_date_d647: AsezDate::try_from_yo(299, 100).unwrap(),
        extract_sum_included_vat_rub_d647: 99.into(),
        product_type_id: 99,
        organizer_note: 99.to_string(),
        description: 99.to_string(),
        status_scheme_id: 99,
        bid_opening_date: AsezDate::try_from_yo(299, 100).unwrap(),
        single_supplier_note_for_expert: 99.to_string(),
        control_pp_2013: 99,
        is_no_qualification: true,
        is_commission: true,
        is_priority_far_eastern: true,
        is_nko: true,
        is_priority_nonprofit: true,
        //----
        contract_sign_date: AsezDate::try_from_yo(299, 100).unwrap(),
        active_uuid: Some(header_uuid),
        //  is actual
        is_actual: true,
        // created & changed
        created_at: AsezTimestamp::from_unix_timestamp(959),
        changed_at: AsezTimestamp::from_unix_timestamp(95959),
        // created & changed
        pricing_created_at: AsezTimestamp::from_unix_timestamp(959),
        pricing_changed_at: AsezTimestamp::from_unix_timestamp(95959),
        created_by: 666,
        changed_by: 582,
        // pricing_started_at new on 2024.11.25
        pricing_started_at: AsezTimestamp::from_unix_timestamp(959),
        reason_cancel_id: None,
        replaced_id: None,
    };

    let item = PlanItemFull {
        uuid: item_uuid,
        plan_uuid: header_uuid,
        description_internal: Some("desc".to_owned()),
        okato_id: Some(5),
        // pricing
        pricing_quantity: Some(100.into()),
        pricing_unit_id: Some(1),
        pricing_price: Some(121.12.into()),
        pricing_price_rub: Some(0.23.into()),
        pricing_vat_id: VatId::R11,
        pricing_currency_id: Some(99),
        pricing_currency_rate: Some(99.into()),
        pricing_sum_excluded_vat: Some(99.67.into()),
        pricing_sum_excluded_vat_rub: Some(9986.78.into()),
        pricing_sum_included_vat: Some(998.66.into()),
        pricing_sum_included_vat_rub: Some(992.34.into()),
        pricing_sum_vat: Some(9923.42.into()),
        pricing_sum_vat_rub: Some(0.99.into()),
        pricing_transportation_vat_id: VatId::R11,
        pricing_transportation_price: Some(99423.42.into()),
        pricing_transportation_price_rub: Some(9923.42.into()),
        pricing_transportation_sum_vat: Some(99.23.into()),
        pricing_transportation_sum_vat_rub: Some(992.34.into()),
        pricing_transportation_sum_included_vat: Some(99.23.into()),
        pricing_transportation_sum_included_vat_rub: Some(9941521.34.into()),
        pricing_total_sum: Some(992342.25.into()),
        pricing_total_sum_rub: Some(9923.42.into()),
        // created & changed
        created_at: AsezTimestamp::from_unix_timestamp(959),
        changed_at: AsezTimestamp::from_unix_timestamp(95959),
        created_by: 666,
        changed_by: 582,
        ..Default::default()
    };
    (header, item)
}

#[tokio::test(flavor = "multi_thread")]
async fn full_insert_update_test_reset_by_unit_id() {
    let uuid_4 = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();
    let uuid_10 = Uuid::parse_str("00000000-0000-0000-0000-000000000010").unwrap();

    let (header_original, item_original) = make_items(uuid_4, uuid_10);

    let header = PlanRep::from_item::<&str>(header_original.clone(), None);
    let header = header.into();
    let item = PlanItemFullRep::from_item::<&str>(item_original.clone(), None);
    let item: PlanItemLegacyRep = item.into();

    let request1 = vec![PlanFromSrm {
        header,
        items: vec![item],
        ..Default::default()
    }];
    let mut request2 = request1.clone();
    request2[0].items[0].pricing_transportation_price = Some(Some(1.into()));
    request2[0].items[0].pricing_transportation_price_rub = Some(Some(1000.into()));
    request2[0].items[0].description_internal = Some("table".to_owned());

    let mut request3 = request1.clone();
    request3[0].header.status_id = Some(343.into());
    request3[0].items[0].description_internal = Some("chair".to_owned());

    run_db_test(&[], move |pool| async move {
        let pctx = super::mock_processing_context(pool.clone()).await;

        //////////////////////////////////////////////////////////
        // The first test tests whether all fields are inserted
        //////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        let plan_select = Select::full::<Plan>().eq(Plan::uuid, uuid_4);
        let item_select =
            Select::full::<PlanItemFull>().eq(PlanItemFull::plan_uuid, uuid_4);

        let mut found_plan =
            Plan::select(&plan_select, &*pool).await.unwrap().pop().unwrap();
        let mut found_item = PlanItemFull::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();

        // assert_ne!();
        assert_ne!(
            found_plan.pricing_changed_at,
            header_original.pricing_changed_at
        );
        assert_ne!(
            found_plan.pricing_created_at,
            header_original.pricing_created_at
        );
        assert_eq!(found_plan.pricing_organization_unit_id, PricingUnitId::D646);
        assert_eq!(found_plan.changed_by, header_original.changed_by);
        assert_eq!(found_plan.created_by, header_original.created_by);
        assert_eq!(found_plan.changed_at, header_original.changed_at);
        assert_eq!(found_plan.created_at, header_original.created_at);
        assert_eq!(found_plan.description, header_original.description);
        assert_eq!(found_plan.items_number, header_original.items_number);
        assert_eq!(found_plan.product_type_id, header_original.product_type_id);
        assert_eq!(found_plan.posting_date, header_original.posting_date);
        assert_eq!(
            found_plan.is_approved_by_d646,
            header_original.is_approved_by_d646
        );
        // assert_ne!();
        assert_ne!(found_item.pricing_changed_at, item_original.pricing_changed_at);
        assert_ne!(found_item.pricing_created_at, item_original.pricing_created_at);
        assert_eq!(found_item.changed_at, item_original.changed_at);
        assert_eq!(found_item.changed_by, item_original.changed_by);

        // Field not in transfer structure.
        found_plan.pricing_competitive_note_for_expert =
            header_original.pricing_competitive_note_for_expert.clone();

        found_plan.changed_at = header_original.changed_at;
        found_plan.pricing_changed_at = header_original.pricing_changed_at;
        found_plan.pricing_created_at = header_original.pricing_created_at;
        found_plan.changed_by = header_original.changed_by;

        found_item.changed_at = item_original.changed_at;
        found_item.pricing_changed_at = item_original.pricing_changed_at;
        found_item.pricing_created_at = item_original.pricing_created_at;
        found_item.changed_by = item_original.changed_by;

        assert_eq!(
            found_plan, header_original,
            "{:#?} vs {:#?}",
            found_plan, header_original
        );
        assert_eq!(
            found_item, item_original,
            "{:#?} vs {:#?}",
            found_item, item_original
        );

        //////////////////////////////////////////////////////////////////
        // The second test tests whether pricing fields are kept the same
        // while normal fields are updated.
        /////////////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_plan(request2, pctx.clone())
            .await
            .unwrap();

        let plan_select = Select::full::<Plan>().eq(Plan::uuid, uuid_4);
        let item_select =
            Select::full::<PlanItemFull>().eq(PlanItemFull::plan_uuid, uuid_4);

        let found_plan = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D646);

        let found_item = PlanItemFull::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, Some("table".to_owned()));
        assert_eq!(found_item.pricing_transportation_price, Some(99423.42.into()));
        assert_eq!(
            found_item.pricing_transportation_price_rub,
            Some(9923.42.into())
        );
        assert_eq!(found_item.pricing_transportation_sum_vat, Some(99.23.into()));
        assert_eq!(
            found_item.pricing_transportation_sum_vat_rub,
            Some(992.34.into())
        );
        assert_eq!(
            found_item.pricing_transportation_sum_included_vat,
            Some(99.23.into())
        );
        assert_eq!(
            found_item.pricing_transportation_sum_included_vat_rub,
            Some(9941521.34.into())
        );
        assert_eq!(found_item.pricing_sum_excluded_vat, Some(99.67.into()));
        assert_eq!(found_item.pricing_sum_excluded_vat_rub, Some(9986.78.into()));
        assert_eq!(found_item.pricing_sum_included_vat, Some(998.66.into()));
        assert_eq!(found_item.pricing_sum_included_vat_rub, Some(992.34.into()));
        assert_eq!(found_item.pricing_sum_vat, Some(9923.42.into()));
        assert_eq!(found_item.pricing_sum_vat_rub, Some(0.99.into()));
        assert_eq!(found_item.pricing_total_sum, Some(992342.25.into()));
        assert_eq!(found_item.pricing_total_sum_rub, Some(9923.42.into()));

        //////////////////////////////////////////////////////////////////
        // The third test tests whether pricing fields are zeroed if unit changes
        // while normal fields are updated.
        /////////////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_plan(request3, pctx.clone())
            .await
            .unwrap();

        let plan_select = Select::full::<Plan>().eq(Plan::uuid, uuid_4);
        let item_select =
            Select::full::<PlanItemFull>().eq(PlanItemFull::plan_uuid, uuid_4);

        let found_plan = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D647);

        let found_item = PlanItemFull::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, Some("chair".to_owned()));

        assert_eq!(found_item.pricing_currency_id, None);
        assert_eq!(found_item.pricing_currency_rate, None);
        assert_eq!(found_item.pricing_currency_rate_date, None);
        assert_eq!(found_item.pricing_quantity, None);
        assert_eq!(found_item.pricing_sum_excluded_vat, None);
        assert_eq!(found_item.pricing_sum_excluded_vat_rub, None);
        assert_eq!(found_item.pricing_sum_included_vat, None);
        assert_eq!(found_item.pricing_sum_included_vat_rub, None);
        assert_eq!(found_item.pricing_sum_vat, None);
        assert_eq!(found_item.pricing_sum_vat_rub, None);
        assert_eq!(found_item.pricing_transportation_price, None);
        assert_eq!(found_item.pricing_transportation_price_rub, None);
        assert_eq!(found_item.pricing_transportation_sum_vat, None);
        assert_eq!(found_item.pricing_transportation_sum_vat_rub, None);
        assert_eq!(found_item.pricing_transportation_sum_included_vat, None);
        assert_eq!(found_item.pricing_transportation_sum_included_vat_rub, None);
        assert_eq!(found_item.pricing_total_sum, None);
        assert_eq!(found_item.pricing_total_sum_rub, None);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn full_insert_update_test_reset_by_pricing_started_at() {
    let uuid_4 = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();
    let uuid_10 = Uuid::parse_str("00000000-0000-0000-0000-000000000010").unwrap();

    let (header_original, item_original) = make_items(uuid_4, uuid_10);

    let header = PlanRep::from_item::<&str>(header_original.clone(), None);
    let header = header.into();
    let item = PlanItemFullRep::from_item::<&str>(item_original.clone(), None);
    let item: PlanItemLegacyRep = item.into();

    let request1 = vec![PlanFromSrm {
        header,
        items: vec![item],
        ..Default::default()
    }];
    let mut request2 = request1.clone();
    request2[0].items[0].pricing_transportation_price = Some(Some(1.into()));
    request2[0].items[0].pricing_transportation_price_rub = Some(Some(1000.into()));
    request2[0].items[0].description_internal = Some("table".to_owned());

    let mut request3 = request1.clone();
    request3[0].header.pricing_started_at =
        Some(AsezTimestamp::from_unix_timestamp(1_000_000).into());
    request3[0].items[0].description_internal = Some("chair".to_owned());

    run_db_test(&[], move |pool| async move {
        let pctx = super::mock_processing_context(pool.clone()).await;

        //////////////////////////////////////////////////////////
        // The first test tests whether all fields are inserted //
        // We do not make any checks here since it is done in   //
        // `full_insert_update_test_reset_by_unit_id`           //
        //////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_plan(request1, pctx.clone())
            .await
            .unwrap();

        //////////////////////////////////////////////////////////////////
        // The second test tests whether pricing fields are kept the same
        // while normal fields are updated.
        /////////////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_plan(request2, pctx.clone())
            .await
            .unwrap();

        let plan_select = Select::full::<Plan>().eq(Plan::uuid, uuid_4);
        let item_select =
            Select::full::<PlanItemFull>().eq(PlanItemFull::plan_uuid, uuid_4);

        let found_plan = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D646);

        let found_item = PlanItemFull::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, Some("table".to_owned()));
        assert_eq!(found_item.pricing_transportation_price, Some(99423.42.into()));
        assert_eq!(
            found_item.pricing_transportation_price_rub,
            Some(9923.42.into())
        );
        assert_eq!(found_item.pricing_transportation_sum_vat, Some(99.23.into()));
        assert_eq!(
            found_item.pricing_transportation_sum_vat_rub,
            Some(992.34.into())
        );
        assert_eq!(
            found_item.pricing_transportation_sum_included_vat,
            Some(99.23.into())
        );
        assert_eq!(
            found_item.pricing_transportation_sum_included_vat_rub,
            Some(9941521.34.into())
        );
        assert_eq!(found_item.pricing_sum_excluded_vat, Some(99.67.into()));
        assert_eq!(found_item.pricing_sum_excluded_vat_rub, Some(9986.78.into()));
        assert_eq!(found_item.pricing_sum_included_vat, Some(998.66.into()));
        assert_eq!(found_item.pricing_sum_included_vat_rub, Some(992.34.into()));
        assert_eq!(found_item.pricing_sum_vat, Some(9923.42.into()));
        assert_eq!(found_item.pricing_sum_vat_rub, Some(0.99.into()));
        assert_eq!(found_item.pricing_total_sum, Some(992342.25.into()));
        assert_eq!(found_item.pricing_total_sum_rub, Some(9923.42.into()));

        //////////////////////////////////////////////////////////////////
        // The third test tests whether pricing fields are zeroed if unit changes
        // while normal fields are updated.
        /////////////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_plan(request3, pctx.clone())
            .await
            .unwrap();

        let plan_select = Select::full::<Plan>().eq(Plan::uuid, uuid_4);
        let item_select =
            Select::full::<PlanItemFull>().eq(PlanItemFull::plan_uuid, uuid_4);

        let found_plan = Plan::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D646);
        assert_eq!(
            found_plan[0].pricing_started_at,
            AsezTimestamp::from_unix_timestamp(1_000_000)
        );

        let found_item = PlanItemFull::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, Some("chair".to_owned()));

        assert_eq!(found_item.pricing_currency_id, None);
        assert_eq!(found_item.pricing_currency_rate, None);
        assert_eq!(found_item.pricing_currency_rate_date, None);
        assert_eq!(found_item.pricing_quantity, None);
        assert_eq!(found_item.pricing_sum_excluded_vat, None);
        assert_eq!(found_item.pricing_sum_excluded_vat_rub, None);
        assert_eq!(found_item.pricing_sum_included_vat, None);
        assert_eq!(found_item.pricing_sum_included_vat_rub, None);
        assert_eq!(found_item.pricing_sum_vat, None);
        assert_eq!(found_item.pricing_sum_vat_rub, None);
        assert_eq!(found_item.pricing_transportation_price, None);
        assert_eq!(found_item.pricing_transportation_price_rub, None);
        assert_eq!(found_item.pricing_transportation_sum_vat, None);
        assert_eq!(found_item.pricing_transportation_sum_vat_rub, None);
        assert_eq!(found_item.pricing_transportation_sum_included_vat, None);
        assert_eq!(found_item.pricing_transportation_sum_included_vat_rub, None);
        assert_eq!(found_item.pricing_total_sum, None);
        assert_eq!(found_item.pricing_total_sum_rub, None);
    })
    .await;
}
