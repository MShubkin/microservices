//! Does some basic checks as to whether serialization is working correctly.
use crate::maths::*;
use crate::processing::plan::PricingUnitId;
use crate::*;

use crate::legacy::plans::PlanStatus;
use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};
use uuid::Uuid;

#[test]
fn test_plan_adaptor_full() {
    const EXPECTED_FULL: &str = r#"
{
  "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e",
  "year": 2020,
  "commission_kind_id": 2,
  "commission_date": "03.03.1989",
  "customer_id": 777,
  "supplier_id": 888,
  "executor_method_id": 1,
  "contract_subject": "Very interesting purchase",
  "sum_excluded_vat": 900,
  "sum_excluded_vat_rub": 901,
  "currency_id": 44,
  "currency_rate": 1,
  "for_price_analysis": false,
  "purchasing_type_id": 4,
  "status_id": 251,
  "delivery_start_date": "02.02.1999",
  "delivery_end_date": "02.02.1999",
  "section_id": 0,
  "kod_st_buda": "123",
  "okdp2": "123",
  "category_id": "123",
  "code_type": 1,
  "is_check_documentation": true,
  "check_documentation_date": 1722594099,
  "pricing_expert_id": 9999,
  "pricing_organization_unit_id": 2,
  "pricing_resume": "text",
  "pricing_competitive_note_for_expert": "I win.",
  "created_at": 1722594099,
  "changed_at": 1722594099,
  "created_by": 5,
  "changed_by": 5,
  "plan_id": 1000038765,
  "contract_subject_short": "Very interesting purchase",
  "pricing_vat_id": 1
}"#;

    let plan = PlanRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        plan_id: Some(1000038765),
        commission_kind_id: Some(CommissionKind::Correspondence),
        commission_date: Some(Some(AsezDate::try_from("1989-03-03").unwrap())),
        customer_id: Some(777),
        supplier_id: Some(888),
        executor_method_id: Some(ExecutorMethodId::Automatic),
        contract_subject: Some("Very interesting purchase".to_string()),
        contract_subject_short: Some("Very interesting purchase".to_string()),
        year: Some(2020),
        sum_excluded_vat: Some(9.into()),
        sum_excluded_vat_rub: Some(9.01.into()),
        currency_id: Some(44),
        currency_rate: Some(CurrencyRate(1)),
        for_price_analysis: Some(false),
        pricing_organization_unit_id: Some(PricingUnitId::D647),
        pricing_expert_id: Some(Some(9999)),
        purchasing_type_id: Some(4),
        status_id: Some(PlanStatus::from(251)),
        delivery_start_date: Some(AsezDate::try_from("1999-02-02").unwrap()),
        delivery_end_date: Some(AsezDate::try_from("1999-02-02").unwrap()),
        section_id: Some(0),
        is_check_documentation: Some(true),
        check_documentation_date: Some(Some(AsezTimestamp::from_unix_timestamp(
            1722594099,
        ))),
        kod_st_buda: Some(Some(String::from("123"))),
        okdp2: Some(Some(String::from("123"))),
        category_id: Some(Some(String::from("123"))),
        code_type: Some(Some(TypeOfPurchaseId::Competitive)),
        pricing_resume: Some(Some(String::from("text"))),
        pricing_competitive_note_for_expert: Some(Some(String::from("I win."))),
        created_at: Some(AsezTimestamp::from_unix_timestamp(1722594099)),
        changed_at: Some(AsezTimestamp::from_unix_timestamp(1722594099)),
        created_by: Some(5),
        changed_by: Some(5),
        pricing_vat_id: Some(VatId::NoVat),
        ..Default::default()
    };
    let full_string = serde_json::to_string_pretty(&plan).unwrap();
    let expected_json: serde_json::Value =
        serde_json::from_str(EXPECTED_FULL).unwrap();
    let actual_json: serde_json::Value =
        serde_json::from_str(&full_string).unwrap();
    assert_eq!(expected_json, actual_json, "{}\n{}", EXPECTED_FULL, full_string);
}

#[test]
fn test_plan_adaptor_2() {
    const EXPECTED_FULL: &str = r#"{
  "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e",
  "year": 2020,
  "commission_kind_id": 2,
  "commission_date": null,
  "customer_id": 777,
  "supplier_id": 888,
  "executor_method_id": 1,
  "contract_subject": "Very interesting purchase",
  "sum_excluded_vat": 900,
  "sum_excluded_vat_rub": 901,
  "currency_id": 44,
  "purchasing_type_id": 4,
  "status_id": 251,
  "delivery_start_date": "02.02.1999",
  "delivery_end_date": "02.02.1999",
  "section_id": 0,
  "kod_st_buda": "123",
  "okdp2": "123",
  "category_id": "123",
  "code_type": 1,
  "is_check_documentation": true,
  "check_documentation_date": null,
  "pricing_expert_id": 9999,
  "pricing_organization_unit_id": 2,
  "pricing_resume": "text",
  "created_at": 1722594099,
  "changed_at": 1722594099,
  "created_by": 5,
  "changed_by": 5,
  "plan_id": 1000038765
}"#;

    let plan = PlanRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        plan_id: Some(1000038765),
        commission_kind_id: Some(CommissionKind::Correspondence),
        commission_date: Some(None),
        customer_id: Some(777),
        supplier_id: Some(888),
        executor_method_id: Some(ExecutorMethodId::Automatic),
        contract_subject: Some("Very interesting purchase".to_string()),
        year: Some(2020),
        sum_excluded_vat: Some(9.into()),
        sum_excluded_vat_rub: Some(9.01.into()),
        currency_id: Some(44),
        currency_rate: None,
        for_price_analysis: None,
        pricing_organization_unit_id: Some(PricingUnitId::D647),
        pricing_expert_id: Some(Some(9999)),
        purchasing_type_id: Some(4),
        status_id: Some(PlanStatus::from(251)),
        delivery_start_date: Some(AsezDate::try_from("1999-02-02").unwrap()),
        delivery_end_date: Some(AsezDate::try_from("1999-02-02").unwrap()),
        section_id: Some(0),
        is_check_documentation: Some(true),
        check_documentation_date: Some(None),
        kod_st_buda: Some(Some(String::from("123"))),
        okdp2: Some(Some(String::from("123"))),
        category_id: Some(Some(String::from("123"))),
        code_type: Some(Some(TypeOfPurchaseId::Competitive)),
        pricing_resume: Some(Some(String::from("text"))),
        created_at: Some(AsezTimestamp::from_unix_timestamp(1722594099)),
        changed_at: Some(AsezTimestamp::from_unix_timestamp(1722594099)),
        created_by: Some(5),
        changed_by: Some(5),
        ..Default::default()
    };
    let full_string = serde_json::to_string_pretty(&plan).unwrap();
    let expected_json: serde_json::Value =
        serde_json::from_str(EXPECTED_FULL).unwrap();
    let actual_json: serde_json::Value =
        serde_json::from_str(&full_string).unwrap();
    assert_eq!(expected_json, actual_json, "{}\n{}", EXPECTED_FULL, full_string);
}

#[test]
fn test_plan_adaptor_3() {
    const EXPECTED_FULL: &str = r#"{
  "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e",
  "commission_date": null
}"#;

    let plan = PlanRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        commission_date: Some(None),
        ..Default::default()
    };
    let full_string = serde_json::to_string_pretty(&plan).unwrap();
    assert_eq!(EXPECTED_FULL, &full_string);
    assert_eq!(plan, serde_json::from_str(&full_string).unwrap());
}

#[test]
fn test_plan_adaptor_4() {
    const EXPECTED_FULL: &str = "{}";

    let plan = PlanRep::default();
    let full_string = serde_json::to_string_pretty(&plan).unwrap();
    assert_eq!(EXPECTED_FULL, &full_string);
    assert_eq!(plan, serde_json::from_str(&full_string).unwrap());
}

#[test]
fn test_agenda_1() {
    const EXPECT_FULL: &str = r#"{
  "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e",
  "pricing_organization_unit_id": 1,
  "is_removed": false,
  "meeting_date": "01.01.1901",
  "created_at": 123456789,
  "changed_at": 123456789,
  "created_by": 1,
  "changed_by": 1,
  "agenda_id": 54,
  "agenda_status_id": 100
}"#;

    // "agenda_status_id": 200,
    let expected_rep = EcAgendaRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        agenda_id: Some(54),
        agenda_status_id: Some(EcAgendaStatus::Formed),
        // status_id: None,
        pricing_organization_unit_id: Some(PricingUnitId::D646),
        is_removed: Some(false),
        meeting_date: Some(AsezDate::try_from("1901-01-01").unwrap()),
        created_at: Some(AsezTimestamp::from_unix_timestamp(123456789)),
        changed_at: Some(AsezTimestamp::from_unix_timestamp(123456789)),
        created_by: Some(1),
        changed_by: Some(1),
        ..Default::default()
    };
    let expected_string = serde_json::to_string_pretty(&expected_rep).unwrap();

    let res_rep: EcAgendaRep = serde_json::from_str(EXPECT_FULL).unwrap();

    assert_eq!(expected_string, EXPECT_FULL);
    assert_eq!(expected_rep, res_rep);
}

#[test]
fn test_protocol_1() {
    const EXPECT_FULL: &str = r#"{
  "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e",
  "protocol_type_id": 1,
  "registration_number": "0123456789",
  "pricing_organization_unit_id": 1,
  "is_secret": true,
  "is_removed": false,
  "protocol_date": "01.01.1901",
  "created_at": 123456789,
  "changed_at": 123456789,
  "created_by": 1,
  "changed_by": 1,
  "protocol_id": 54,
  "protocol_status_id": 100
}"#;

    // "agenda_status_id": 200,
    let expected_rep = EcProtocolRep {
        uuid: Some(
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
        ),
        protocol_id: Some(54),
        protocol_type_id: Some(ProtocolType::InPersonMeeting),
        registration_number: Some(Some("0123456789".to_string())),
        protocol_status_id: Some(EcProtocolStatus::Formed),
        pricing_organization_unit_id: Some(PricingUnitId::D646),
        is_removed: Some(false),
        is_secret: Some(true),
        protocol_date: Some(AsezDate::try_from("1901-01-01").unwrap()),
        created_at: Some(AsezTimestamp::from_unix_timestamp(123456789)),
        changed_at: Some(AsezTimestamp::from_unix_timestamp(123456789)),
        created_by: Some(1),
        changed_by: Some(1),
        ..Default::default()
    };
    let expected_string = serde_json::to_string_pretty(&expected_rep).unwrap();

    let res_rep: EcProtocolRep = serde_json::from_str(EXPECT_FULL).unwrap();

    assert_eq!(
        expected_string, EXPECT_FULL,
        "{}\n{}",
        expected_string, EXPECT_FULL
    );
    assert_eq!(expected_rep, res_rep);
}

#[test]
fn test_plan_amendment_rep1() {
    // N.B.: Simplifications because not all fields match yet.
    const EXPECTED_AM: &str = r#"{
      "object_type": "contract_amendment",
      "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e",
      "customer_id": 777,
      "year": 2020,
      "purchasing_type_id": 4,
      "section_id": 0,
      "supplier_id": 888,
      "contract_subject": "Very interesting purchase",
      "sum_excluded_vat": 900,
      "currency_id": 44,
      "status_id": 251,
      "commission_kind_id": 2,
      "executor_method_id": 1,
      "commission_date": "03.03.1989",
      "is_check_documentation": true,
      "check_documentation_date": 1722594099,
      "kod_st_buda": "123",
      "okdp2": "123",
      "category_id": "123",
      "code_type": 1,
      "pricing_expert_id": 9999,
      "pricing_organization_unit_id": 2,
      "pricing_resume": "text",
      "created_at": 1722594099,
      "changed_at": 1722594099,
      "created_by": 5,
      "changed_by": 5,
      "plan_id": 1000038765,
      "contract_subject_short": "Very interesting purchase"
    }"#;

    const EXPECTED_PLAN: &str = r#"{
      "object_type": "plan",
      "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e",
      "year": 2020,
      "commission_kind_id": 2,
      "commission_date": "03.03.1989",
      "customer_id": 777,
      "supplier_id": 888,
      "executor_method_id": 1,
      "contract_subject": "Very interesting purchase",
      "sum_excluded_vat": 900,
      "currency_id": 44,
      "purchasing_type_id": 4,
      "status_id": 251,
      "section_id": 0,
      "kod_st_buda": "123",
      "okdp2": "123",
      "category_id": "123",
      "code_type": 1,
      "is_check_documentation": true,
      "check_documentation_date": 1722594099,
      "pricing_expert_id": 9999,
      "pricing_organization_unit_id": 2,
      "pricing_resume": "text",
      "created_at": 1722594099,
      "changed_at": 1722594099,
      "created_by": 5,
      "changed_by": 5,
      "plan_id": 1000038765,
      "contract_subject_short": "Very interesting purchase"
    }"#;
    // ❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗
    // ❗ Никогда не используйте в ассертах сравнение строк из `to_string_pretty` так как поля могут
    // ❗ выдаваться произвольным образом и если когда-то работало сравнение по строкам, то нет гарантии,
    // ❗ что оно будет работать потом!!
    // ❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗❗

    let p: PlanOrAmendmentRep = serde_json::from_str(EXPECTED_PLAN).unwrap();
    let a: PlanOrAmendmentRep = serde_json::from_str(EXPECTED_AM).unwrap();

    // TODO: Коммент от Александра Коптелова:
    // `Мне кажется, более продуктивно было бы изначально EXPECTED_... делать не строками, а Value с использованием json!{...}.`

    assert!(matches!(p, PlanOrAmendmentRep::Plan(_)));
    assert!(matches!(a, PlanOrAmendmentRep::Amendment(_)));

    let p_value = serde_json::to_value(&p).unwrap();
    let expected_p_value: serde_json::Value =
        serde_json::from_str(EXPECTED_PLAN).unwrap();
    assert_eq!(p_value, expected_p_value, "Plan values do not match");

    let a_value = serde_json::to_value(&a).unwrap();
    let expected_a_value: serde_json::Value =
        serde_json::from_str(EXPECTED_AM).unwrap();
    assert_eq!(a_value, expected_a_value, "Amendment values do not match");
}

#[test]
fn plan_legacy_to_dust() {
    let s = r#"{
    "some_unknown_field": {
        "some_inner_field": "some data",
        "some_other_inner_field": 11.11
    }
}"#;
    let exp: PlanLegacyRep = serde_json::from_str(s).unwrap();
    assert_eq!(exp, PlanLegacyRep::default());
}

#[test]
fn plan_item_legacy_to_dust() {
    let s = r#"{
    "rpp_rf_pp_719_value": {
        "value": "some data",
        "some_other_value": 11.11
    },
    "rpp_eaes_pp_616_value": {
        "value": "some data",
        "some_other_value": 11.11
    },
    "errrp_pp_878_value": {
        "value": "some data",
        "some_other_value": 11.11
    }
}"#;
    let exp: PlanItemLegacyRep = serde_json::from_str(s).unwrap();
    assert_eq!(exp, PlanItemLegacyRep::default());
}

#[test]
fn amendment_legacy_to_dust() {
    let s = r#"{
    "some_unknown_field": {
        "some_inner_field": "some data",
        "some_other_inner_field": 11.11
    }
}"#;
    let exp: ContractAmendmentLegacyRep = serde_json::from_str(s).unwrap();
    assert_eq!(exp, ContractAmendmentLegacyRep::default());
}

#[test]
fn amendment_item_legacy_to_dust() {
    let s = r#"{
    "some_unknown_field": {
        "some_inner_field": "some data",
        "some_other_inner_field": 11.11
    }
}"#;
    let exp: ContractAmendmentItemLegacyRep = serde_json::from_str(s).unwrap();
    assert_eq!(exp, ContractAmendmentItemLegacyRep::default());
}
