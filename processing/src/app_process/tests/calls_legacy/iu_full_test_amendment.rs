//! Tests here aim to make sure that fields are reset or saved properly. We do not
//! However test all possible branch points of the mechanism.
use super::*;

use asez2_shared_db::db_item::int_array::AsezArray;
use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};

use shared_essential::domain::ContractAmendment;

fn make_items(
    header_uuid: Uuid,
    item_uuid: Uuid,
) -> (ContractAmendment, ContractAmendmentItem) {
    let header = ContractAmendment {
        uuid: header_uuid,
        id: 4,
        version_type: 199,
        version_number: 299,
        active_uuid: header_uuid,
        source_uuid: header_uuid,
        system_number: 12345.to_string(),
        external_number: 23456.to_string(),
        branch: 34567.to_string(),
        is_actual: true,
        is_pur_asbu: true,
        customer_id: 99,
        declarant_id: 999,
        agent_id: 9999,
        assignee_id: 9,
        project_institute_id: 99,
        organizer_id: 999,
        initiator_user_id: 9999,
        tender_user_id: 9999,
        year: 2006,
        purchasing_type_id: 599,
        purchasing_method_id: 99,
        section_id: 939,
        funding_source_id: 99,
        single_supplier_reason_id: 99,
        number_cgg: 99.to_string(),
        contract_system_number: 1234.to_string(),
        contract_external_number: 1234.to_string(),
        number_eis: 1234.to_string(),
        supplier_id: 599,
        contract_subject: 899.to_string(),
        contract_type_id: 5,
        accepted_volume_included_vat_rub: 999_999,
        is_banking_support: false,
        is_with_amendments: true,
        is_secret_state: false,
        is_secret_commercial: false,
        rationale: 99.to_string(),
        funding_availability: 99.to_string(),
        is_chairman_order: false,
        is_chairman_order_secret: true,
        chairman_order_number: 99.to_string(),
        chairman_order_date: AsezDate::try_from_yo(299, 100).unwrap(),
        is_vice_chairman_order: false,
        is_with_approval: true,
        is_need_for_departments: false,
        is_sum_increase_was_specified: true,
        is_sum_changed_via_key_rate: false,
        is_material_registry: true,
        is_to_publish: false,
        repair_stage_id: 99,
        vat_id: VatId::R11,
        sum_excluded_vat: 999.into(),
        sum_vat: 299.into(),
        sum_included_vat: 99.into(),
        currency_id: 299,
        currency_rate: 399.into(),
        sum_excluded_vat_rub: 199.into(),
        sum_vat_rub: 99.into(),
        sum_included_vat_rub: 99.into(),
        initial_vat_id: VatId::R11,
        initial_sum_excluded_vat: 999.into(),
        initial_sum_excluded_vat_rub: 99.into(),
        initial_sum_vat: 299.into(),
        initial_sum_vat_rub: 99.into(),
        initial_sum_included_vat: 99.into(),
        initial_sum_included_vat_rub: 99.into(),
        initial_currency_id: 299,
        initial_currency_rate: 399.into(),
        previous_vat_id: VatId::R11,
        previous_sum_excluded_vat: 999.into(),
        previous_sum_excluded_vat_rub: 99.into(),
        previous_sum_vat: 299.into(),
        previous_sum_vat_rub: 99.into(),
        previous_sum_included_vat: 99.into(),
        previous_sum_included_vat_rub: 99.into(),
        previous_currency_id: 299,
        previous_currency_rate: 399.into(),
        delta_sum_excluded_vat: 1.into(),
        delta_sum_vat: 1.into(),
        delta_sum_included_vat: 1.into(),
        delta_sum_excluded_vat_rub: 1.into(),
        delta_sum_vat_rub: 1.into(),
        delta_sum_included_vat_rub: 1.into(),
        sign_date: AsezDate::try_from_yo(299, 100).unwrap(),
        close_date: AsezDate::try_from_yo(299, 100).unwrap(),
        termination_date: AsezDate::try_from_yo(2999, 100).unwrap(),
        start_date: AsezDate::try_from_yo(1999, 100).unwrap(),
        end_date: AsezDate::try_from_yo(2009, 100).unwrap(),
        whole_start_date: AsezDate::try_from_yo(1999, 100).unwrap(),
        whole_end_date: AsezDate::try_from_yo(2009, 100).unwrap(),
        initial_start_date: AsezDate::try_from_yo(1999, 100).unwrap(),
        initial_end_date: AsezDate::try_from_yo(2009, 100).unwrap(),
        initial_whole_start_date: AsezDate::try_from_yo(1999, 100).unwrap(),
        initial_whole_end_date: AsezDate::try_from_yo(2009, 100).unwrap(),
        previous_start_date: AsezDate::try_from_yo(1999, 100).unwrap(),
        previous_end_date: AsezDate::try_from_yo(2009, 100).unwrap(),
        previous_whole_start_date: AsezDate::try_from_yo(1999, 100).unwrap(),
        previous_whole_end_date: AsezDate::try_from_yo(2009, 100).unwrap(),
        is_priority_project: true,
        priority_project_document: 99.to_string(),
        is_priority_introductory: true,
        priority_introductory_date: AsezDate::try_from_yo(299, 100).unwrap(),
        priority_introductory_document: 99.to_string(),
        is_priority_repair: false,
        priority_repair_document: 99.to_string(),
        is_priority_ozp: false,
        priority_ozp_document: 99.to_string(),
        is_priority_income_contract: false,
        priority_income_contract_document: 99.to_string(),
        priority_income_contract_partner_id: 99,
        priority_income_contract_partner_text: 99.to_string(),
        is_priority_other: false,
        is_headquarters: true,
        status_scheme_id: 99,
        status_id: 223.into(),
        is_approved_by_d646: true,
        commission_kind_id: 299.into(),
        budget_item_id: 99,
        payment_balance_item_id: 99,
        product_type_id: 99,
        items_number: 99,
        /// plan_id в оригинале у ГПИ
        associated_plan_id: 1,
        purchase_id: 1.to_string(),
        purchase_number_eis: 2.to_string(),
        quotation_id: 3.to_string(),
        contract_id: 4.to_string(),
        claim_id: 5,
        is_removed: false,
        posting_date: AsezDate::try_from_yo(399, 100).unwrap(),
        is_priority_far_eastern: true,

        executor_method_id: 699.into(),
        number_customer: 99.to_string(),
        commission_date: Some(AsezDate::try_from_yo(299, 100).unwrap()),
        expert_conclusion_id: Some(99.into()),
        is_check_documentation: true,
        check_documentation_date: Some(AsezTimestamp::from_unix_timestamp(299)),
        kod_st_buda: Some(959.to_string()),
        okdp2: Some(993.to_string()),
        category_id: Some(959.to_string()),
        code_type: Some(992.into()),
        contract_amendment_types: AsezArray(vec![1, 2, 3]),

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

        is_pricing_by_d646: true,
        is_pricing_by_d647: false,
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

        pricing_delta_currency_id: Some(99),
        pricing_delta_currency_rate: Some(99.into()),
        pricing_delta_sum_excluded_vat: Some(99.into()),
        pricing_delta_sum_excluded_vat_rub: Some(99.into()),
        pricing_delta_sum_included_vat: Some(99.into()),
        pricing_delta_sum_included_vat_rub: Some(99.into()),
        pricing_delta_sum_vat: Some(99.into()),
        pricing_delta_sum_vat_rub: Some(99.into()),
        pricing_delta_total_sum: Some(99.into()),
        pricing_delta_total_sum_rub: Some(99.into()),
        pricing_delta_transportation_price: Some(99.into()),
        pricing_delta_transportation_sum_vat: Some(99.into()),
        pricing_delta_transportation_sum_vat_rub: Some(99.into()),
        pricing_delta_transportation_sum_included_vat: Some(99.into()),
        pricing_delta_transportation_sum_included_vat_rub: Some(99.into()),
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
    };

    let item = ContractAmendmentItem {
        uuid: item_uuid,
        header_uuid,
        description_internal: "desc".to_owned(),
        okato_id: 5,
        // pricing
        pricing_quantity: 100.into(),
        pricing_unit_id: 1,
        pricing_price: 121.12.into(),
        pricing_price_rub: Some(0.23.into()),
        pricing_vat_id: VatId::R11,
        pricing_currency_id: 99,
        pricing_currency_rate: Some(99.into()),
        pricing_sum_excluded_vat: Some(99.67.into()),
        pricing_sum_excluded_vat_rub: Some(9986.78.into()),
        pricing_sum_included_vat: Some(998.66.into()),
        pricing_sum_included_vat_rub: Some(992.34.into()),
        pricing_sum_vat: Some(9923.42.into()),
        pricing_sum_vat_rub: Some(0.99.into()),
        pricing_transportation_vat_id: VatId::R11,
        pricing_transportation_price: 99423.42.into(),
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

    let header =
        ContractAmendmentRep::from_item::<&str>(header_original.clone(), None);
    let header = header.into();
    let item =
        ContractAmendmentItemRep::from_item::<&str>(item_original.clone(), None);
    let item: ContractAmendmentItemLegacyRep = item.into();

    let request1 = vec![AmendmentFromSrm {
        header,
        items: vec![item],
        ..Default::default()
    }];
    let mut request2 = request1.clone();
    request2[0].items[0].pricing_transportation_price = Some(1.into());
    request2[0].items[0].pricing_transportation_price_rub = Some(Some(1000.into()));
    request2[0].items[0].description_internal = Some("table".to_owned());

    let mut request3 = request1.clone();
    request3[0].header.status_id = Some(PlanStatus::from(343));
    request3[0].items[0].description_internal = Some("chair".to_owned());

    run_db_test(&[], move |pool| async move {
        let pctx = super::mock_processing_context(pool.clone()).await;

        //////////////////////////////////////////////////////////
        // The first test tests whether all fields are inserted
        //////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_amendment(request1, pctx.clone())
            .await
            .unwrap();

        let plan_select =
            Select::full::<ContractAmendment>().eq(ContractAmendment::uuid, uuid_4);
        let item_select = Select::full::<ContractAmendmentItem>()
            .eq(ContractAmendmentItem::header_uuid, uuid_4);

        let mut found_plan = ContractAmendment::select(&plan_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        let mut found_item = ContractAmendmentItem::select(&item_select, &*pool)
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
        assert_eq!(found_plan.items_number, header_original.items_number);
        assert_eq!(found_plan.product_type_id, header_original.product_type_id);
        assert_eq!(found_plan.posting_date, header_original.posting_date);
        assert_eq!(
            found_plan.is_approved_by_d646,
            header_original.is_approved_by_d646
        );
        // context fields are kept
        assert_eq!(found_item.changed_at, item_original.changed_at);
        assert_eq!(found_item.changed_by, item_original.changed_by);
        // assert_ne!();
        assert_ne!(found_item.pricing_changed_at, item_original.pricing_changed_at);
        assert_ne!(found_item.pricing_created_at, item_original.pricing_created_at);

        // Field not in transfer structure.
        found_plan.pricing_competitive_note_for_expert =
            header_original.pricing_competitive_note_for_expert.clone();

        found_plan.changed_at = header_original.changed_at;
        found_plan.pricing_changed_at = header_original.pricing_changed_at;
        found_plan.pricing_created_at = header_original.pricing_created_at;
        found_plan.changed_by = header_original.changed_by;
        found_plan.source_uuid = header_original.source_uuid;

        found_item.changed_at = item_original.changed_at;
        found_item.pricing_changed_at = item_original.pricing_changed_at;
        found_item.pricing_created_at = item_original.pricing_created_at;
        found_item.changed_by = item_original.changed_by;

        // These are calculated:
        found_item.delta_quantity = item_original.delta_quantity;
        found_item.delta_price = item_original.delta_price;
        found_item.delta_price_rub = item_original.delta_price_rub;
        found_item.delta_sum_vat = item_original.delta_sum_vat;
        found_item.delta_sum_vat_rub = item_original.delta_sum_vat_rub;
        found_item.delta_sum_excluded_vat = item_original.delta_sum_excluded_vat;
        found_item.delta_sum_excluded_vat_rub =
            item_original.delta_sum_excluded_vat_rub;
        found_item.delta_sum_included_vat = item_original.delta_sum_included_vat;
        found_item.delta_sum_included_vat_rub =
            item_original.delta_sum_included_vat_rub;
        found_item.uuid_item_proposal = item_original.uuid_item_proposal;

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
        crate::app_process::upsert_legacy_amendment(request2, pctx.clone())
            .await
            .unwrap();

        let plan_select =
            Select::full::<ContractAmendment>().eq(ContractAmendment::uuid, uuid_4);
        let item_select = Select::full::<ContractAmendmentItem>()
            .eq(ContractAmendmentItem::header_uuid, uuid_4);

        let found_plan =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D646);

        let found_item = ContractAmendmentItem::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, "table".to_owned());
        assert_eq!(found_item.pricing_transportation_price, 99423.42.into());
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
        crate::app_process::upsert_legacy_amendment(request3, pctx.clone())
            .await
            .unwrap();

        let plan_select =
            Select::full::<ContractAmendment>().eq(ContractAmendment::uuid, uuid_4);
        let item_select = Select::full::<ContractAmendmentItem>()
            .eq(ContractAmendmentItem::header_uuid, uuid_4);

        let found_plan =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D647);

        let found_item = ContractAmendmentItem::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, "chair".to_owned());

        assert_eq!(found_item.pricing_currency_id, 0);
        assert_eq!(found_item.pricing_currency_rate, None);
        assert_eq!(found_item.pricing_currency_rate_date, None);
        assert_eq!(found_item.pricing_quantity, Default::default());
        assert_eq!(found_item.pricing_sum_excluded_vat, None);
        assert_eq!(found_item.pricing_sum_excluded_vat_rub, None);
        assert_eq!(found_item.pricing_sum_included_vat, None);
        assert_eq!(found_item.pricing_sum_included_vat_rub, None);
        assert_eq!(found_item.pricing_sum_vat, None);
        assert_eq!(found_item.pricing_sum_vat_rub, None);
        assert_eq!(found_item.pricing_transportation_price, Default::default());
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

    let header =
        ContractAmendmentRep::from_item::<&str>(header_original.clone(), None);
    let header = header.into();
    let item =
        ContractAmendmentItemRep::from_item::<&str>(item_original.clone(), None);
    let item: ContractAmendmentItemLegacyRep = item.into();

    let request1 = vec![AmendmentFromSrm {
        header,
        items: vec![item],
        ..Default::default()
    }];
    let mut request2 = request1.clone();
    request2[0].items[0].pricing_transportation_price = Some(1.into());
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
        crate::app_process::upsert_legacy_amendment(request1, pctx.clone())
            .await
            .unwrap();

        //////////////////////////////////////////////////////////////////
        // The second test tests whether pricing fields are kept the same
        // while normal fields are updated.
        /////////////////////////////////////////////////////////////////
        crate::app_process::upsert_legacy_amendment(request2, pctx.clone())
            .await
            .unwrap();

        let plan_select =
            Select::full::<ContractAmendment>().eq(ContractAmendment::uuid, uuid_4);
        let item_select = Select::full::<ContractAmendmentItem>()
            .eq(ContractAmendmentItem::header_uuid, uuid_4);

        let found_plan =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D646);

        let found_item = ContractAmendmentItem::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, "table".to_owned());
        assert_eq!(found_item.pricing_transportation_price, 99423.42.into());
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
        crate::app_process::upsert_legacy_amendment(request3, pctx.clone())
            .await
            .unwrap();

        let plan_select =
            Select::full::<ContractAmendment>().eq(ContractAmendment::uuid, uuid_4);
        let item_select = Select::full::<ContractAmendmentItem>()
            .eq(ContractAmendmentItem::header_uuid, uuid_4);

        let found_plan =
            ContractAmendment::select(&plan_select, &*pool).await.unwrap();
        assert_eq!(found_plan.len(), 1);
        assert_eq!(found_plan[0].pricing_organization_unit_id, PricingUnitId::D646);
        assert_eq!(
            found_plan[0].pricing_started_at,
            AsezTimestamp::from_unix_timestamp(1_000_000)
        );

        let found_item = ContractAmendmentItem::select(&item_select, &*pool)
            .await
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(found_item.description_internal, "chair".to_owned());

        assert_eq!(found_item.pricing_currency_id, 0);
        assert_eq!(found_item.pricing_currency_rate, None);
        assert_eq!(found_item.pricing_currency_rate_date, None);
        assert_eq!(found_item.pricing_quantity, Default::default());
        assert_eq!(found_item.pricing_sum_excluded_vat, None);
        assert_eq!(found_item.pricing_sum_excluded_vat_rub, None);
        assert_eq!(found_item.pricing_sum_included_vat, None);
        assert_eq!(found_item.pricing_sum_included_vat_rub, None);
        assert_eq!(found_item.pricing_sum_vat, None);
        assert_eq!(found_item.pricing_sum_vat_rub, None);
        assert_eq!(found_item.pricing_transportation_price, Default::default());
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

#[tokio::test]
#[ignore = "Perf. test. Run manually when needed."]
async fn legacy_amendment_test_perf() {
    use rand::distributions::{Alphanumeric, DistString};
    use rand::prelude::*;

    let uuid_4 = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();
    let uuid_10 = Uuid::parse_str("00000000-0000-0000-0000-000000000010").unwrap();
    let (header, item) = make_items(uuid_4, uuid_10);

    let header = ContractAmendmentRep::from_item::<&str>(header, None);
    let header = header.into();

    let mut items = Vec::with_capacity(70_000);
    let mut rng = thread_rng();
    for i in 0..70_000 {
        let mut item = item.clone();

        item.uuid = Uuid::from_u128(i as u128);
        item.description_internal = Alphanumeric.sample_string(&mut rng, 10);
        // pricing
        item.pricing_quantity = i.into();
        item.pricing_price = (i as f64 / 100.).into();
        item.pricing_price_rub = Some((i as f64 / 25.3).into());

        let item = ContractAmendmentItemRep::from_item::<&str>(item, None);
        let item: ContractAmendmentItemLegacyRep = item.into();
        items.push(item);
    }
    let request1 = vec![AmendmentFromSrm {
        header,
        items,
        ..Default::default()
    }];

    run_db_test(&[], move |pool| async move {
        let pctx = super::mock_processing_context(pool.clone()).await;

        let now = std::time::Instant::now();
        crate::app_process::upsert_legacy_amendment(request1, pctx.clone())
            .await
            .unwrap();
        let elapsed = now.elapsed();
        println!("Amendment inserts: {elapsed:?}");

        let header_count =
            sqlx::query("SELECT count(*) FROM contract_amendment WHERE uuid=$1")
                .bind(uuid_4)
                .try_map(|x| <(i64,)>::from_row(&x))
                .fetch_one(&*pool)
                .await
                .unwrap()
                .0;
        let item_count = sqlx::query(
            "SELECT count(*) FROM contract_amendment_item WHERE header_uuid=$1",
        )
        .bind(uuid_4)
        .try_map(|x| <(i64,)>::from_row(&x))
        .fetch_one(&*pool)
        .await
        .unwrap()
        .0;
        let field_count = sqlx::query("SELECT count(*) FROM field_history")
            .try_map(|x| <(i64,)>::from_row(&x))
            .fetch_one(&*pool)
            .await
            .unwrap()
            .0;

        assert_eq!(header_count, 1);
        assert_eq!(item_count, 70_000);
        assert_eq!(field_count, 10_570_195);
        assert!(elapsed.as_secs() < 260);
    })
    .await;
}
