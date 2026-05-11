use crate::app_process::calls::price_analysis::{
    pa_update_contract_amendment, pa_update_plan,
};
use crate::app_process::price_analysis::update_plan::UpdatePlanCAError;
use asez2_shared_db::db_item::{AsezDate, Select};
use asez2_shared_db::{asez_date, uuid, DbItem};
use shared_essential::domain::maths::*;
use shared_essential::domain::*;
use shared_essential::presentation::dto;

use dto::processing::{price_analysis::UpdatePlanReq, UserIdWrapper};
use shared_essential::presentation::dto::processing::price_analysis::UpdateContractAmendmentReq;
use time::Weekday;

use crate::app_process::tests;
use crate::common::ProcessingError;

use uuid::Uuid;

const EXTRA_MIG: &[&str] = &["price_analysis/update_plan.sql"];

const REAL_SAMPLE_ATTACHMENT: &str = r#"{
  "uuid": "00000000-0000-0000-0000-000000000099",
  "id": 25,
  "category_id": 8,
  "kind": 2,
  "text": "Справка-обоснование потребности.docx",
  "is_classified": false,
  "is_removed": false,
  "parent_id": 0,
  "changed_at": 1710363600,
  "changed_by": 25,
  "created_at": 1685221200,
  "created_by": 35,
  "mime_id": 0,
  "size": 0,
  "is_archived": true
}"#;

#[tokio::test]
async fn basic_update_with_existing_attachment() {
    let plan_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let bad_plan_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000004321").unwrap();

    let attachment_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let attachment_new_uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();

    let item_uuid1 =
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let item_uuid2 =
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    let bad_req = UserIdWrapper::<UpdatePlanReq> {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(bad_plan_uuid),
                id: Some(1),
                customer_id: Some(11),
                sum_excluded_vat_rub: Some(999_9999.99.into()),
                ..Default::default()
            },
            item_list: vec![],
            pricing_attachment_list: vec![],
        },
    };

    let req = UserIdWrapper::<UpdatePlanReq> {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(plan_uuid),
                id: Some(1),
                customer_id: Some(11),
                contract_subject: Some("Такса от Москвы до Питера".to_string()),
                sum_excluded_vat_rub: Some(999_9999.99.into()),
                ..Default::default()
            },
            item_list: vec![
                PlanItemFullRep {
                    uuid: Some(item_uuid1),
                    plan_uuid: Some(plan_uuid),
                    ..Default::default()
                },
                PlanItemFullRep {
                    uuid: Some(item_uuid2),
                    plan_uuid: Some(plan_uuid),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![
                AttachmentRep {
                    uuid: Some(attachment_uuid),
                    size: Some(123_312),
                    mime_id: Some(34),
                    ..Default::default()
                },
                serde_json::from_str(REAL_SAMPLE_ATTACHMENT).unwrap(),
            ],
        },
    };

    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;
        // Initial state.
        {
            let plan_select = Select::default().eq(Plan::uuid, plan_uuid);
            let item_select =
                Select::default().in_any(PlanItem::uuid, [item_uuid1, item_uuid2]);
            let attachment1_select =
                Select::default().eq(Attachment::uuid, attachment_uuid);
            let attachment2_select =
                Select::default().eq(Attachment::uuid, attachment_new_uuid);

            let plan = Plan::select(&plan_select, &*pool).await.unwrap();
            let items = PlanItem::select(&item_select, &*pool).await.unwrap();
            let ok_attachment =
                Attachment::select(&attachment1_select, &*pool).await.unwrap();
            let empty =
                Attachment::select(&attachment2_select, &*pool).await.unwrap();

            assert_eq!(plan.len(), 1);
            assert_eq!(items.len(), 2);
            assert_eq!(ok_attachment.len(), 1);
            assert!(empty.is_empty());

            assert_eq!(plan[0].contract_subject, "Слишком много комаров");
            assert_eq!(
                items[0].description_internal,
                Some("труба, железная,".to_string())
            );
            assert_eq!(
                items[1].description_internal,
                Some("труба, железная,".to_string())
            );
            assert_eq!(ok_attachment[0].size, 54);
        }

        let bad_rep = pa_update_plan(bad_req, pctx.clone()).await;
        let r = pa_update_plan(req, pctx).await;
        // Fail case.
        {
            let error = bad_rep.unwrap_err();
            match error {
                ProcessingError::UpdatePlanCA(UpdatePlanCAError::NotFound(1)) => {}
                x => panic!("Wrong error type found: {x:#?}"),
            }
        }
        {
            assert!(r.is_ok());

            let plan_select = Select::default().eq(Plan::uuid, plan_uuid);
            let item_select =
                Select::default().in_any(PlanItem::uuid, [item_uuid1, item_uuid2]);
            let attachment1_select =
                Select::default().eq(Attachment::uuid, attachment_uuid);
            let attachment2_select =
                Select::default().eq(Attachment::uuid, attachment_new_uuid);

            let plan = Plan::select(&plan_select, &*pool).await.unwrap();
            let items = PlanItem::select(&item_select, &*pool).await.unwrap();
            let ok_attachment =
                Attachment::select(&attachment1_select, &*pool).await.unwrap();
            let not_empty =
                Attachment::select(&attachment2_select, &*pool).await.unwrap();

            assert_eq!(plan.len(), 1);
            assert_eq!(items.len(), 2);
            assert_eq!(ok_attachment.len(), 1);
            assert_eq!(not_empty.len(), 1);

            assert_eq!(plan[0].contract_subject, "Слишком много комаров");
            assert_eq!(
                items[0].description_internal,
                Some("труба, железная,".to_string())
            );
            assert_eq!(
                items[1].description_internal,
                Some("труба, железная,".to_string())
            );
            assert_eq!(ok_attachment[0].size, 123_312);
            assert_eq!(ok_attachment[0].object_uuid, plan[0].uuid);

            assert_eq!(not_empty[0].object_uuid, plan[0].uuid);
            assert_eq!(not_empty[0].number, 25);
            assert_eq!(not_empty[0].category_id as i16, 8);
            assert_eq!(not_empty[0].kind_id as i16, 2);
            assert_eq!(&not_empty[0].name, "Справка-обоснование потребности.docx");
            assert!(!not_empty[0].is_classified);
            assert!(!not_empty[0].is_removed);
            assert_eq!(not_empty[0].mime_id, 0);
            assert_eq!(not_empty[0].size, 0);
            // у нового attachment устанавливаются эти поля
            assert_eq!(not_empty[0].created_by, 666);
            assert_ne!(not_empty[0].created_at.unix_timestamp(), 1685221200);
        }
    })
    .await
}

macro_rules! assert_zero {
    ($item:expr, $($field:ident),* $(,)?) => {
        $(
            assert_eq!($item.$field, Some(Default::default()));
        )*
    };
}

// plans

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_plan() {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(plan_uuid),
                id: Some(1),
                ..Default::default()
            },
            item_list: vec![
                PlanItemFullRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(Some(1500.into())),
                    pricing_quantity: Some(Some(1.into())),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
                PlanItemFullRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(Some(4500.into())),
                    pricing_quantity: Some(Some(2.into())),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_plan(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let plan = Plan::select(&Select::default().eq(Plan::uuid, plan_uuid), pool)
            .await
            .unwrap()
            .pop()
            .expect("plan");
        let mut items = PlanItemFull::select(
            &Select::default()
                .eq(PlanItemFull::plan_uuid, plan_uuid)
                .add_replace_order_asc(PlanItemFull::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.pricing_unit_id, Some(item0.unit_id));
        assert_eq!(item0.pricing_price_rub, Some(1500.into()));

        assert_eq!(item0.pricing_currency_id, Some(item0.currency_id));
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(item0.pricing_currency_rate_date, item0.currency_rate_date);

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_vat_rub, Some(0.into()));
        assert_eq!(item0.pricing_sum_vat_rub, Some(0.into()));

        assert_eq!(item0.pricing_total_sum, Some(1500.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(1500.into()));

        assert_zero!(
            item0,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item0.pricing_transportation_vat_id, VatId::Unspecified);

        // item1
        assert_eq!(item1.pricing_unit_id, Some(item1.unit_id));
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, Some(item1.currency_id));
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(item1.pricing_currency_rate_date, item1.currency_rate_date);

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_vat_rub, Some(0.into()));
        assert_eq!(item1.pricing_sum_vat_rub, Some(0.into()));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some((4500 * 2).into()));

        assert_zero!(
            item1,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item1.pricing_transportation_vat_id, VatId::Unspecified);

        // plan
        assert_eq!(plan.pricing_currency_id, Some(plan.currency_id));
        assert_eq!(plan.pricing_currency_rate, Some(plan.currency_rate));

        // sum fields
        assert_eq!(plan.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(
            plan.pricing_sum_excluded_vat_rub,
            Some((1500 + 4500 * 2).into())
        );
        assert_eq!(plan.pricing_sum_included_vat, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            plan.pricing_sum_included_vat_rub,
            Some((1500 + 4500 * 2).into())
        );

        assert_eq!(plan.pricing_total_sum, Some((1500 + 4500 * 2).into()));
        assert_eq!(plan.pricing_total_sum_rub, Some((1500 + 4500 * 2).into()));

        // common vat_id
        assert_eq!(plan.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(plan.pricing_vat_id, item1.pricing_vat_id);

        assert_zero!(
            plan,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(plan.pricing_transportation_vat_id, VatId::Unspecified);
    })
    .await
}

/// Test that only needed fields are taken from DTO.
#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn plan_ignore_extra_items() {
    // Values to use from DTO
    let uuid = uuid!("00000000-0000-0000-0000-000000000001");
    let id = 1;
    let pricing_expert_id = Some(42);
    let pricing_method_id = 56;
    let expert_conclusion_id = Some(ExpertConclusionId::DocumentationRequest);
    let pricing_resume = Some("my pricing resume".to_string());
    let commission_kind_id = CommissionKind::InPerson;
    let commission_date =
        Some(AsezDate::today().with_next_weekday(Weekday::Monday));
    let savings_accounting_id = SavingsAccountingId::Partial;
    let savings_sum_excluded_vat = Some(12345.into());
    let savings_sum_included_vat = Some((12345 + 345).into());
    let organizer_id = Some(453);
    let is_cooperative = true;
    let is_list_price = true;

    let item0_uuid = uuid!("00000000-0000-0000-0000-000000000001");
    let item0_pricing_quantity = Some(1.into());
    let item0_pricing_price = Some(1500.into());
    let item0_pricing_vat_id = VatId::NoVat;

    let item1_uuid = uuid!("00000000-0000-0000-0000-000000000002");
    let item1_pricing_quantity = Some(2.into());
    let item1_pricing_price = Some(4500.into());
    let item1_pricing_vat_id = VatId::NoVat;

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                // DTO fields
                uuid: Some(uuid),
                id: Some(id),
                pricing_expert_id: Some(pricing_expert_id),
                pricing_method_id: Some(pricing_method_id),
                expert_conclusion_id: Some(expert_conclusion_id),
                pricing_resume: Some(pricing_resume.clone()),
                commission_kind_id: Some(commission_kind_id),
                commission_date: Some(commission_date),
                savings_accounting_id: Some(savings_accounting_id),
                savings_sum_excluded_vat: Some(savings_sum_excluded_vat),
                savings_sum_included_vat: Some(savings_sum_included_vat),
                organizer_id: Some(organizer_id),
                is_cooperative: Some(is_cooperative),
                is_list_price: Some(is_list_price),
                // fields to ignore
                year: Some(2020),
                currency_id: Some(123),
                currency_rate: Some(123.87634.into()),
                status_id: Some(PlanStatus::EstimatedCommissionCorrespondence),
                sum_excluded_vat: Some(123456789.into()),
                sum_included_vat: Some((123456789 + 56789).into()),

                ..Default::default()
            },
            item_list: vec![
                PlanItemFullRep {
                    // DTO fields
                    uuid: Some(item0_uuid),
                    pricing_quantity: Some(item0_pricing_quantity),
                    pricing_price: Some(item0_pricing_price),
                    pricing_vat_id: Some(item0_pricing_vat_id),
                    // fields to ignore
                    currency_id: Some(646),
                    currency_rate_date: Some(Some(asez_date!("2024-12-12"))),
                    unit_id: Some(30),
                    price: Some(1900.into()),
                    quantity: Some(9.into()),

                    ..Default::default()
                },
                PlanItemFullRep {
                    uuid: Some(item1_uuid),
                    pricing_price: Some(item1_pricing_price),
                    pricing_quantity: Some(item1_pricing_quantity),
                    pricing_vat_id: Some(item1_pricing_vat_id),
                    // fields to ignore
                    currency_id: Some(696),
                    currency_rate_date: Some(Some(asez_date!("2024-12-12"))),
                    unit_id: Some(20),
                    price: Some(8500.into()),
                    quantity: Some(6.into()),

                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        let pool = &*pctx.db_pool;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let old_plan = Plan::select(&Select::default().eq(Plan::uuid, uuid), pool)
            .await
            .unwrap()
            .pop()
            .expect("contract amendment");

        let mut items = PlanItemFull::select(
            &Select::default()
                .eq(PlanItemFull::plan_uuid, uuid)
                .add_replace_order_asc(PlanItemFull::uuid),
            pool,
        )
        .await
        .unwrap();

        assert_eq!(items.len(), 2);

        let old_item1 = items.pop().unwrap();
        let old_item0 = items.pop().unwrap();

        let res = pa_update_plan(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let plan = Plan::select(&Select::default().eq(Plan::uuid, uuid), pool)
            .await
            .unwrap()
            .pop()
            .expect("contract amendment");

        let mut items = PlanItemFull::select(
            &Select::default()
                .eq(PlanItemFull::plan_uuid, uuid)
                .add_replace_order_asc(PlanItemFull::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // values from DTO
        assert_eq!(plan.uuid, uuid);
        assert_eq!(plan.id, id);
        assert_eq!(plan.pricing_expert_id, pricing_expert_id);
        assert_eq!(plan.pricing_method_id, pricing_method_id);
        assert_eq!(plan.expert_conclusion_id, expert_conclusion_id);
        assert_eq!(plan.pricing_resume, pricing_resume);
        assert_eq!(plan.commission_kind_id, commission_kind_id);
        assert_eq!(plan.commission_date, commission_date);
        assert_eq!(plan.savings_accounting_id, savings_accounting_id);
        assert_eq!(plan.savings_sum_excluded_vat, savings_sum_excluded_vat);
        assert_eq!(plan.savings_sum_included_vat, savings_sum_included_vat);
        assert_eq!(plan.organizer_id, organizer_id);
        assert_eq!(plan.is_cooperative, is_cooperative);
        assert_eq!(plan.is_list_price, is_list_price);

        assert_eq!(item0.uuid, item0_uuid);
        assert_eq!(item0.pricing_quantity, item0_pricing_quantity);
        assert_eq!(item0.pricing_price, item0_pricing_price);
        assert_eq!(item0.pricing_vat_id, item0_pricing_vat_id);

        assert_eq!(item1.uuid, item1_uuid);
        assert_eq!(item1.pricing_quantity, item1_pricing_quantity);
        assert_eq!(item1.pricing_price, item1_pricing_price);
        assert_eq!(item1.pricing_vat_id, item1_pricing_vat_id);

        // values from DB
        assert_eq!(plan.year, old_plan.year);
        assert_eq!(plan.currency_id, old_plan.currency_id);
        assert_eq!(plan.currency_rate, old_plan.currency_rate);
        assert_eq!(plan.status_id, old_plan.status_id);
        assert_eq!(plan.sum_excluded_vat, old_plan.sum_excluded_vat);
        assert_eq!(plan.sum_included_vat, old_plan.sum_included_vat);

        assert_eq!(item0.currency_id, old_item0.currency_id);
        assert_eq!(item0.currency_rate_date, old_item0.currency_rate_date);
        assert_eq!(item0.unit_id, old_item0.unit_id);
        assert_eq!(item0.price, old_item0.price);
        assert_eq!(item0.quantity, old_item0.quantity);

        assert_eq!(item1.currency_id, old_item1.currency_id);
        assert_eq!(item1.currency_rate_date, old_item1.currency_rate_date);
        assert_eq!(item1.unit_id, old_item1.unit_id);
        assert_eq!(item1.price, old_item1.price);
        assert_eq!(item1.quantity, old_item1.quantity);

        // calculated fields
        // item0
        assert_eq!(item0.pricing_unit_id, Some(item0.unit_id));
        assert_eq!(item0.pricing_price_rub, Some(1500.into()));

        assert_eq!(item0.pricing_currency_id, Some(item0.currency_id));
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(item0.pricing_currency_rate_date, item0.currency_rate_date);

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_vat_rub, Some(0.into()));
        assert_eq!(item0.pricing_sum_vat_rub, Some(0.into()));

        assert_eq!(item0.pricing_total_sum, Some(1500.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(1500.into()));

        assert_zero!(
            item0,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item0.pricing_transportation_vat_id, VatId::Unspecified);

        // item1
        assert_eq!(item1.pricing_unit_id, Some(item1.unit_id));
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, Some(item1.currency_id));
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(item1.pricing_currency_rate_date, item1.currency_rate_date);

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_vat_rub, Some(0.into()));
        assert_eq!(item1.pricing_sum_vat_rub, Some(0.into()));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some((4500 * 2).into()));

        assert_zero!(
            item1,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item1.pricing_transportation_vat_id, VatId::Unspecified);

        // plan
        assert_eq!(plan.pricing_currency_id, Some(plan.currency_id));
        assert_eq!(plan.pricing_currency_rate, Some(plan.currency_rate));

        // sum fields
        assert_eq!(plan.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(
            plan.pricing_sum_excluded_vat_rub,
            Some((1500 + 4500 * 2).into())
        );
        assert_eq!(plan.pricing_sum_included_vat, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            plan.pricing_sum_included_vat_rub,
            Some((1500 + 4500 * 2).into())
        );

        assert_eq!(plan.pricing_total_sum, Some((1500 + 4500 * 2).into()));
        assert_eq!(plan.pricing_total_sum_rub, Some((1500 + 4500 * 2).into()));

        // common vat_id
        assert_eq!(plan.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(plan.pricing_vat_id, item1.pricing_vat_id);

        assert_zero!(
            plan,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(plan.pricing_transportation_vat_id, VatId::Unspecified);
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_plan_partial() {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(plan_uuid),
                id: Some(1),
                ..Default::default()
            },
            item_list: vec![
                PlanItemFullRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(Some(1500.into())),
                    pricing_quantity: Some(Some(1.into())),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
                PlanItemFullRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(Some(4500.into())),
                    pricing_quantity: Some(Some(2.into())),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    let req1 = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(plan_uuid),
                id: Some(1),
                ..Default::default()
            },
            item_list: vec![PlanItemFullRep {
                uuid: Some(item_uuid1),
                quantity: Some(1.into()),
                unit_id: Some(10),
                price: Some(1700.into()),
                sum_excluded_vat: Some(1700.into()),
                vat_id: Some(VatId::NoVat),
                sum_included_vat: Some(1700.into()),

                pricing_price: Some(Some(1400.into())),
                pricing_quantity: Some(Some(1.into())),
                pricing_sum_excluded_vat: Some(Some(1400.into())),
                pricing_vat_id: Some(VatId::NoVat),
                pricing_sum_included_vat: Some(Some(1400.into())),

                currency_id: Some(643),
                ..Default::default()
            }],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_plan(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let res = pa_update_plan(req1, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let plan = Plan::select(&Select::default().eq(Plan::uuid, plan_uuid), pool)
            .await
            .unwrap()
            .pop()
            .expect("plan");
        let mut items = PlanItemFull::select(
            &Select::default()
                .eq(PlanItemFull::plan_uuid, plan_uuid)
                .add_replace_order_asc(PlanItemFull::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.pricing_unit_id, Some(item0.unit_id));
        assert_eq!(item0.pricing_price_rub, Some(1400.into()));

        assert_eq!(item0.pricing_currency_id, Some(item0.currency_id));
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(item0.pricing_currency_rate_date, item0.currency_rate_date);

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1400.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1400.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1400.into()));
        assert_eq!(item0.pricing_sum_included_vat_rub, Some(1400.into()));

        assert_eq!(item0.pricing_total_sum, Some(1400.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(1400.into()));

        // item1
        assert_eq!(item1.pricing_unit_id, Some(item1.unit_id));
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, Some(item1.currency_id));
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(item1.pricing_currency_rate_date, item1.currency_rate_date);

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some((4500 * 2).into()));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some((4500 * 2).into()));

        // plan
        assert_eq!(plan.pricing_currency_id, Some(plan.currency_id));
        assert_eq!(plan.pricing_currency_rate, Some(plan.currency_rate));

        // sum fields
        assert_eq!(plan.pricing_sum_excluded_vat, (1400 + 4500 * 2).into());
        assert_eq!(
            plan.pricing_sum_excluded_vat_rub,
            Some((1400 + 4500 * 2).into())
        );
        assert_eq!(plan.pricing_sum_included_vat, Some((1400 + 4500 * 2).into()));
        assert_eq!(
            plan.pricing_sum_included_vat_rub,
            Some((1400 + 4500 * 2).into())
        );

        assert_eq!(plan.pricing_total_sum, Some((1400 + 4500 * 2).into()));
        assert_eq!(plan.pricing_total_sum_rub, Some((1400 + 4500 * 2).into()));

        // common vat_id
        assert_eq!(plan.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(plan.pricing_vat_id, item1.pricing_vat_id);
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_plan_foreign_currency() {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000003");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000003");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000004");

    let rwf: CurrencyRate = 0.037.into(); // 0.037 rub
    let kwd: CurrencyRate = 329.44.into(); // 329.44 rub

    let from_rwf = move |v| rwf.convert_value(CurrencyValue::from(v));
    let from_kwd = move |v| kwd.convert_value(CurrencyValue::from(v));

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(plan_uuid),
                id: Some(3),
                ..Default::default()
            },
            item_list: vec![
                PlanItemFullRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(Some(1500.into())),
                    pricing_quantity: Some(Some(1.into())),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
                PlanItemFullRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(Some(4500.into())),
                    pricing_quantity: Some(Some(2.into())),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_plan(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let plan = Plan::select(&Select::default().eq(Plan::uuid, plan_uuid), pool)
            .await
            .unwrap()
            .pop()
            .expect("plan");
        let mut items = PlanItemFull::select(
            &Select::default()
                .eq(PlanItemFull::plan_uuid, plan_uuid)
                .add_replace_order_asc(PlanItemFull::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.currency_rate, rwf);

        assert_eq!(item0.pricing_unit_id, Some(item0.unit_id));
        assert_eq!(item0.pricing_price_rub, Some(from_rwf(1500)));

        assert_eq!(item0.pricing_currency_id, Some(item0.currency_id));
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(item0.pricing_currency_rate_date, item0.currency_rate_date);

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(from_rwf(1500)));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat_rub, Some(from_rwf(1500)));

        assert_eq!(item0.pricing_total_sum, Some(1500.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(from_rwf(1500)));

        // item1
        assert_eq!(item1.currency_rate, kwd);

        assert_eq!(item1.pricing_unit_id, Some(item1.unit_id));
        assert_eq!(item1.pricing_price_rub, Some(from_kwd(4500)));

        assert_eq!(item1.pricing_currency_id, Some(item1.currency_id));
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(item1.pricing_currency_rate_date, item1.currency_rate_date);

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some(from_kwd(4500 * 2)));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some(from_kwd(4500 * 2)));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some(from_kwd(4500 * 2)));

        // plan

        // sum fields
        assert_eq!(plan.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(
            plan.pricing_sum_excluded_vat_rub,
            Some(from_rwf(1500) + from_kwd(4500 * 2))
        );
        assert_eq!(plan.pricing_sum_included_vat, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            plan.pricing_sum_included_vat_rub,
            Some(from_rwf(1500) + from_kwd(4500 * 2))
        );

        assert_eq!(plan.pricing_total_sum, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            plan.pricing_total_sum_rub,
            Some(from_rwf(1500) + from_kwd(4500 * 2))
        );

        // common vat_id
        assert_eq!(plan.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(plan.pricing_vat_id, item1.pricing_vat_id);
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_plan_vat() {
    let plan_uuid = uuid!("00000000-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000000-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000000-0000-0000-0000-000000000002");

    let vat_10 = |v: i64| v / 10;
    let vat_20 = |v: i64| v / 5;

    let with_vat_10 = |v: i64| v + v / 10;
    let with_vat_20 = |v: i64| v + v / 5;

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(plan_uuid),
                id: Some(1),
                ..Default::default()
            },
            item_list: vec![
                PlanItemFullRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(Some(1500.into())),
                    pricing_quantity: Some(Some(1.into())),
                    pricing_vat_id: Some(VatId::R10),
                    ..Default::default()
                },
                PlanItemFullRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(Some(4500.into())),
                    pricing_quantity: Some(Some(2.into())),
                    pricing_vat_id: Some(VatId::R20),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_plan(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let plan = Plan::select(&Select::default().eq(Plan::uuid, plan_uuid), pool)
            .await
            .unwrap()
            .pop()
            .expect("plan");
        let mut items = PlanItemFull::select(
            &Select::default()
                .eq(PlanItemFull::plan_uuid, plan_uuid)
                .add_replace_order_asc(PlanItemFull::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.pricing_unit_id, Some(item0.unit_id));
        assert_eq!(item0.pricing_price_rub, Some(1500.into()));

        assert_eq!(item0.pricing_currency_id, Some(item0.currency_id));
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(item0.pricing_currency_rate_date, item0.currency_rate_date);

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(with_vat_10(1500).into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(with_vat_10(1500).into()));
        assert_eq!(item0.pricing_sum_vat_rub, Some(vat_10(1500).into()));
        assert_eq!(item0.pricing_sum_vat_rub, Some(vat_10(1500).into()));

        assert_eq!(item0.pricing_total_sum, Some(with_vat_10(1500).into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(with_vat_10(1500).into()));

        // item1
        assert_eq!(item1.pricing_unit_id, Some(item1.unit_id));
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, Some(item1.currency_id));
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(item1.pricing_currency_rate_date, item1.currency_rate_date);

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(
            item1.pricing_sum_included_vat,
            Some(with_vat_20(4500 * 2).into())
        );
        assert_eq!(
            item1.pricing_sum_included_vat_rub,
            Some(with_vat_20(4500 * 2).into())
        );
        assert_eq!(item1.pricing_sum_vat_rub, Some(vat_20(4500 * 2).into()));
        assert_eq!(item1.pricing_sum_vat_rub, Some(vat_20(4500 * 2).into()));

        assert_eq!(item1.pricing_total_sum, Some(with_vat_20(4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some(with_vat_20(4500 * 2).into()));

        // plan

        // sum fields
        assert_eq!(plan.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(
            plan.pricing_sum_excluded_vat_rub,
            Some((1500 + 4500 * 2).into())
        );
        assert_eq!(
            plan.pricing_sum_included_vat,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            plan.pricing_sum_included_vat_rub,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            plan.pricing_sum_vat,
            Some((vat_10(1500) + vat_20(4500 * 2)).into())
        );
        assert_eq!(
            plan.pricing_sum_vat_rub,
            Some((vat_10(1500) + vat_20(4500 * 2)).into())
        );

        assert_eq!(
            plan.pricing_total_sum,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            plan.pricing_total_sum_rub,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );

        // common vat_id
        assert_eq!(plan.pricing_vat_id, VatId::Compound);
    })
    .await
}

// contract amendments

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_ca() {
    let ca_uuid = uuid!("00000001-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000001-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000001-0000-0000-0000-000000000002");

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(ca_uuid),
                id: Some(101),
                ..Default::default()
            },
            item_list: vec![
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(1500.into()),
                    pricing_quantity: Some(1.into()),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(4500.into()),
                    pricing_quantity: Some(2.into()),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_contract_amendment(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let ca = ContractAmendment::select(
            &Select::default().eq(ContractAmendment::uuid, ca_uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .expect("contract amendment");
        let mut items = ContractAmendmentItem::select(
            &Select::default()
                .eq(ContractAmendmentItem::header_uuid, ca_uuid)
                .add_replace_order_asc(ContractAmendmentItem::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.pricing_unit_id, item0.unit_id);
        assert_eq!(item0.pricing_price_rub, Some(1500.into()));

        assert_eq!(item0.pricing_currency_id, item0.currency_id);
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(
            item0.pricing_currency_rate_date,
            Some(item0.currency_rate_date)
        );

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat_rub, Some(1500.into()));

        assert_eq!(item0.pricing_total_sum, Some(1500.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(1500.into()));

        assert_eq!(item0.pricing_transportation_price, 0.into());
        assert_zero!(
            item0,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item0.pricing_transportation_vat_id, VatId::Unspecified);

        assert_zero!(
            item0,
            pricing_delta_transportation_price,
            pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub,
        );

        // item1
        assert_eq!(item1.pricing_unit_id, item1.unit_id);
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, item1.currency_id);
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(
            item1.pricing_currency_rate_date,
            Some(item1.currency_rate_date)
        );

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some((4500 * 2).into()));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some((4500 * 2).into()));

        assert_eq!(item1.pricing_transportation_price, 0.into());
        assert_zero!(
            item1,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item1.pricing_transportation_vat_id, VatId::Unspecified);

        assert_zero!(
            item1,
            pricing_delta_transportation_price,
            pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub,
        );

        // header

        assert_eq!(ca.pricing_currency_id, Some(ca.currency_id));
        assert_eq!(ca.pricing_currency_rate, Some(ca.currency_rate));

        // sum fields
        assert_eq!(ca.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(ca.pricing_sum_excluded_vat_rub, Some((1500 + 4500 * 2).into()));
        assert_eq!(ca.pricing_sum_included_vat, Some((1500 + 4500 * 2).into()));
        assert_eq!(ca.pricing_sum_included_vat_rub, Some((1500 + 4500 * 2).into()));

        assert_eq!(ca.pricing_total_sum, Some((1500 + 4500 * 2).into()));
        assert_eq!(ca.pricing_total_sum_rub, Some((1500 + 4500 * 2).into()));

        // common vat_id
        assert_eq!(ca.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(ca.pricing_vat_id, item1.pricing_vat_id);

        assert_zero!(
            ca,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(ca.pricing_transportation_vat_id, VatId::Unspecified);

        assert_zero!(
            ca,
            pricing_delta_transportation_price,
            //pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub,
        );
    })
    .await
}

/// Test that only needed fields are taken from DTO.
#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn ca_ignore_extra_items() {
    // Values to use from DTO
    let uuid = uuid!("00000001-0000-0000-0000-000000000001");
    let id = 1;
    let pricing_expert_id = Some(42);
    let pricing_method_id = 56;
    let expert_conclusion_id = Some(ExpertConclusionId::DocumentationRequest);
    let pricing_resume = Some("my pricing resume".to_string());
    let commission_kind_id = CommissionKind::InPerson;
    let commission_date =
        Some(AsezDate::today().with_next_weekday(Weekday::Monday));
    let savings_accounting_id = SavingsAccountingId::Partial;
    let savings_sum_excluded_vat = Some(CurrencyValue::from(12345));
    let savings_sum_included_vat = Some(CurrencyValue::from(12345 + 345));

    let item0_uuid = uuid!("00000001-0000-0000-0000-000000000001");
    let item0_pricing_quantity = Quantity::from(1);
    let item0_pricing_price = CurrencyValue::from(1500);
    let item0_pricing_vat_id = VatId::NoVat;

    let item1_uuid = uuid!("00000001-0000-0000-0000-000000000002");
    let item1_pricing_quantity = Quantity::from(2);
    let item1_pricing_price = CurrencyValue::from(4500);
    let item1_pricing_vat_id = VatId::NoVat;

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                // DTO fields
                uuid: Some(uuid),
                id: Some(id),
                pricing_expert_id: Some(pricing_expert_id),
                pricing_method_id: Some(pricing_method_id),
                expert_conclusion_id: Some(expert_conclusion_id),
                pricing_resume: Some(pricing_resume.clone()),
                commission_kind_id: Some(commission_kind_id),
                commission_date: Some(commission_date),
                savings_accounting_id: Some(savings_accounting_id),
                savings_sum_excluded_vat: Some(savings_sum_excluded_vat),
                savings_sum_included_vat: Some(savings_sum_included_vat),
                // fields to ignore
                year: Some(2020),
                currency_id: Some(123),
                currency_rate: Some(123.87634.into()),
                status_id: Some(PlanStatus::EstimatedCommissionCorrespondence),
                sum_excluded_vat: Some(123456789.into()),
                sum_included_vat: Some((123456789 + 56789).into()),

                ..Default::default()
            },
            item_list: vec![
                ContractAmendmentItemRep {
                    // DTO fields
                    uuid: Some(item0_uuid),
                    pricing_quantity: Some(item0_pricing_quantity),
                    pricing_price: Some(item0_pricing_price),
                    pricing_vat_id: Some(item0_pricing_vat_id),
                    // fields to ignore
                    currency_id: Some(646),
                    currency_rate_date: Some(asez_date!("2024-12-12")),
                    unit_id: Some(10),
                    price: Some(5600.into()),
                    quantity: Some(19.into()),

                    ..Default::default()
                },
                ContractAmendmentItemRep {
                    // DTO fields
                    uuid: Some(item1_uuid),
                    pricing_quantity: Some(item1_pricing_quantity),
                    pricing_price: Some(item1_pricing_price),
                    pricing_vat_id: Some(item1_pricing_vat_id),
                    // fields to ignore
                    currency_id: Some(646),
                    currency_rate_date: Some(asez_date!("2024-12-12")),
                    unit_id: Some(20),
                    price: Some(9900.into()),
                    quantity: Some(100.into()),

                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        let pool = &*pctx.db_pool;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let old_ca = ContractAmendment::select(
            &Select::default().eq(ContractAmendment::uuid, uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .expect("contract amendment");

        let mut items = ContractAmendmentItem::select(
            &Select::default()
                .eq(ContractAmendmentItem::header_uuid, uuid)
                .add_replace_order_asc(ContractAmendmentItem::uuid),
            pool,
        )
        .await
        .unwrap();

        assert_eq!(items.len(), 2);

        let old_item1 = items.pop().unwrap();
        let old_item0 = items.pop().unwrap();

        let res = pa_update_contract_amendment(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let ca = ContractAmendment::select(
            &Select::default().eq(ContractAmendment::uuid, uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .expect("contract amendment");

        let mut items = ContractAmendmentItem::select(
            &Select::default()
                .eq(ContractAmendmentItem::header_uuid, uuid)
                .add_replace_order_asc(ContractAmendmentItem::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // values from DTO
        assert_eq!(ca.uuid, uuid);
        assert_eq!(ca.id, id);
        assert_eq!(ca.pricing_expert_id, pricing_expert_id);
        assert_eq!(ca.pricing_method_id, pricing_method_id);
        assert_eq!(ca.expert_conclusion_id, expert_conclusion_id);
        assert_eq!(ca.pricing_resume, pricing_resume);
        assert_eq!(ca.commission_kind_id, commission_kind_id);
        assert_eq!(ca.commission_date, commission_date);
        assert_eq!(ca.savings_accounting_id, savings_accounting_id);
        assert_eq!(ca.savings_sum_excluded_vat, savings_sum_excluded_vat);
        assert_eq!(ca.savings_sum_included_vat, savings_sum_included_vat);

        assert_eq!(item0.uuid, item0_uuid);
        assert_eq!(item0.pricing_quantity, item0_pricing_quantity);
        assert_eq!(item0.pricing_price, item0_pricing_price);
        assert_eq!(item0.pricing_vat_id, item0_pricing_vat_id);

        assert_eq!(item1.uuid, item1_uuid);
        assert_eq!(item1.pricing_quantity, item1_pricing_quantity);
        assert_eq!(item1.pricing_price, item1_pricing_price);
        assert_eq!(item1.pricing_vat_id, item1_pricing_vat_id);

        // values from DB
        assert_eq!(ca.year, old_ca.year);
        assert_eq!(ca.currency_id, old_ca.currency_id);
        assert_eq!(ca.currency_rate, old_ca.currency_rate);
        assert_eq!(ca.status_id, old_ca.status_id);
        assert_eq!(ca.sum_excluded_vat, old_ca.sum_excluded_vat);
        assert_eq!(ca.sum_included_vat, old_ca.sum_included_vat);

        assert_eq!(item0.currency_id, old_item0.currency_id);
        assert_eq!(item0.currency_rate_date, old_item0.currency_rate_date);
        assert_eq!(item0.unit_id, old_item0.unit_id);
        assert_eq!(item0.price, old_item0.price);
        assert_eq!(item0.quantity, old_item0.quantity);

        assert_eq!(item1.currency_id, old_item1.currency_id);
        assert_eq!(item1.currency_rate_date, old_item1.currency_rate_date);
        assert_eq!(item1.unit_id, old_item1.unit_id);
        assert_eq!(item1.price, old_item1.price);
        assert_eq!(item1.quantity, old_item1.quantity);

        // calculated values
        // item0
        assert_eq!(item0.pricing_unit_id, item0.unit_id);
        assert_eq!(item0.pricing_price_rub, Some(1500.into()));

        assert_eq!(item0.pricing_currency_id, item0.currency_id);
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(
            item0.pricing_currency_rate_date,
            Some(item0.currency_rate_date)
        );

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat_rub, Some(1500.into()));

        assert_eq!(item0.pricing_total_sum, Some(1500.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(1500.into()));

        assert_eq!(item0.pricing_transportation_price, Default::default());
        assert_zero!(
            item0,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item0.pricing_transportation_vat_id, VatId::Unspecified);

        assert_zero!(
            item0,
            pricing_delta_transportation_price,
            pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub,
        );

        // item1
        assert_eq!(item1.pricing_unit_id, item1.unit_id);
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, item1.currency_id);
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(
            item1.pricing_currency_rate_date,
            Some(item1.currency_rate_date)
        );

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some((4500 * 2).into()));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some((4500 * 2).into()));

        assert_eq!(item1.pricing_transportation_price, Default::default());
        assert_zero!(
            item1,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(item1.pricing_transportation_vat_id, VatId::Unspecified);

        assert_zero!(
            item1,
            pricing_delta_transportation_price,
            pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub,
        );

        // header

        assert_eq!(ca.pricing_currency_id, Some(ca.currency_id));
        assert_eq!(ca.pricing_currency_rate, Some(ca.currency_rate));

        // sum fields
        assert_eq!(ca.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(ca.pricing_sum_excluded_vat_rub, Some((1500 + 4500 * 2).into()));
        assert_eq!(ca.pricing_sum_included_vat, Some((1500 + 4500 * 2).into()));
        assert_eq!(ca.pricing_sum_included_vat_rub, Some((1500 + 4500 * 2).into()));

        assert_eq!(ca.pricing_total_sum, Some((1500 + 4500 * 2).into()));
        assert_eq!(ca.pricing_total_sum_rub, Some((1500 + 4500 * 2).into()));

        // common vat_id
        assert_eq!(ca.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(ca.pricing_vat_id, item1.pricing_vat_id);

        assert_zero!(
            ca,
            pricing_transportation_price,
            pricing_transportation_price_rub,
            pricing_transportation_sum_vat,
            pricing_transportation_sum_vat_rub,
            pricing_transportation_sum_included_vat,
            pricing_transportation_sum_included_vat_rub
        );
        assert_eq!(ca.pricing_transportation_vat_id, VatId::Unspecified);

        assert_zero!(
            ca,
            pricing_delta_transportation_price,
            //pricing_delta_transportation_price_rub,
            pricing_delta_transportation_sum_vat,
            pricing_delta_transportation_sum_vat_rub,
            pricing_delta_transportation_sum_included_vat,
            pricing_delta_transportation_sum_included_vat_rub,
        );
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_ca_partial() {
    let ca_uuid = uuid!("00000001-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000001-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000001-0000-0000-0000-000000000002");

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(ca_uuid),
                id: Some(101),
                ..Default::default()
            },
            item_list: vec![
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(1500.into()),
                    pricing_quantity: Some(1.into()),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(4500.into()),
                    pricing_quantity: Some(2.into()),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    let req1 = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(ca_uuid),
                id: Some(101),
                ..Default::default()
            },
            item_list: vec![ContractAmendmentItemRep {
                uuid: Some(item_uuid1),
                pricing_price: Some(1400.into()),
                pricing_quantity: Some(1.into()),
                pricing_vat_id: Some(VatId::NoVat),
                ..Default::default()
            }],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_contract_amendment(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let res = pa_update_contract_amendment(req1, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let ca = ContractAmendment::select(
            &Select::default().eq(ContractAmendment::uuid, ca_uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .expect("contract amendment");
        let mut items = ContractAmendmentItem::select(
            &Select::default()
                .eq(ContractAmendmentItem::header_uuid, ca_uuid)
                .add_replace_order_asc(ContractAmendmentItem::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.pricing_unit_id, item0.unit_id);
        assert_eq!(item0.pricing_price_rub, Some(1400.into()));

        assert_eq!(item0.pricing_currency_id, item0.currency_id);
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(
            item0.pricing_currency_rate_date,
            Some(item0.currency_rate_date)
        );

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1400.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1400.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(1400.into()));
        assert_eq!(item0.pricing_sum_included_vat_rub, Some(1400.into()));

        assert_eq!(item0.pricing_total_sum, Some(1400.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(1400.into()));

        // item1
        assert_eq!(item1.pricing_unit_id, item1.unit_id);
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, item1.currency_id);
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(
            item1.pricing_currency_rate_date,
            Some(item1.currency_rate_date)
        );

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some((4500 * 2).into()));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some((4500 * 2).into()));

        // header
        assert_eq!(ca.pricing_currency_id, Some(ca.currency_id));
        assert_eq!(ca.pricing_currency_rate, Some(ca.currency_rate));

        // sum fields
        assert_eq!(ca.pricing_sum_excluded_vat, (1400 + 4500 * 2).into());
        assert_eq!(ca.pricing_sum_excluded_vat_rub, Some((1400 + 4500 * 2).into()));
        assert_eq!(ca.pricing_sum_included_vat, Some((1400 + 4500 * 2).into()));
        assert_eq!(ca.pricing_sum_included_vat_rub, Some((1400 + 4500 * 2).into()));

        assert_eq!(ca.pricing_total_sum, Some((1400 + 4500 * 2).into()));
        assert_eq!(ca.pricing_total_sum_rub, Some((1400 + 4500 * 2).into()));

        // common vat_id
        assert_eq!(ca.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(ca.pricing_vat_id, item1.pricing_vat_id);
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_ca_foreign_currency() {
    let ca_uuid = uuid!("00000001-0000-0000-0000-000000000002");

    let item_uuid1 = uuid!("00000001-0000-0000-0000-000000000003");
    let item_uuid2 = uuid!("00000001-0000-0000-0000-000000000004");

    let rwf: CurrencyRate = 0.037.into(); // 0.037 rub
    let kwd: CurrencyRate = 329.44.into(); // 329.44 rub

    let from_rwf = move |v| rwf.convert_value(CurrencyValue::from(v));
    let from_kwd = move |v| kwd.convert_value(CurrencyValue::from(v));

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(ca_uuid),
                id: Some(102),
                ..Default::default()
            },
            item_list: vec![
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(1500.into()),
                    pricing_quantity: Some(1.into()),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(4500.into()),
                    pricing_quantity: Some(2.into()),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_contract_amendment(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let ca = ContractAmendment::select(
            &Select::default().eq(ContractAmendment::uuid, ca_uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .expect("contract amendment");
        let mut items = ContractAmendmentItem::select(
            &Select::default()
                .eq(ContractAmendmentItem::header_uuid, ca_uuid)
                .add_replace_order_asc(ContractAmendmentItem::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.currency_rate, rwf);

        assert_eq!(item0.pricing_unit_id, item0.unit_id);
        assert_eq!(item0.pricing_price_rub, Some(from_rwf(1500)));

        assert_eq!(item0.pricing_currency_id, item0.currency_id);
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(
            item0.pricing_currency_rate_date,
            Some(item0.currency_rate_date)
        );

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(from_rwf(1500)));
        assert_eq!(item0.pricing_sum_included_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat_rub, Some(from_rwf(1500)));

        assert_eq!(item0.pricing_total_sum, Some(1500.into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(from_rwf(1500)));

        // item1
        assert_eq!(item1.currency_rate, kwd);

        assert_eq!(item1.pricing_unit_id, item1.unit_id);
        assert_eq!(item1.pricing_price_rub, Some(from_kwd(4500)));

        assert_eq!(item1.pricing_currency_id, item1.currency_id);
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(
            item1.pricing_currency_rate_date,
            Some(item1.currency_rate_date)
        );

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some(from_kwd(4500 * 2)));
        assert_eq!(item1.pricing_sum_included_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_included_vat_rub, Some(from_kwd(4500 * 2)));

        assert_eq!(item1.pricing_total_sum, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some(from_kwd(4500 * 2)));

        // header

        // sum fields
        assert_eq!(ca.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(
            ca.pricing_sum_excluded_vat_rub,
            Some(from_rwf(1500) + from_kwd(4500 * 2))
        );
        assert_eq!(ca.pricing_sum_included_vat, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            ca.pricing_sum_included_vat_rub,
            Some(from_rwf(1500) + from_kwd(4500 * 2))
        );

        assert_eq!(ca.pricing_total_sum, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            ca.pricing_total_sum_rub,
            Some(from_rwf(1500) + from_kwd(4500 * 2))
        );

        // common vat_id
        assert_eq!(ca.pricing_vat_id, item0.pricing_vat_id);
        assert_eq!(ca.pricing_vat_id, item1.pricing_vat_id);
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_ca_vat() {
    let ca_uuid = uuid!("00000001-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000001-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000001-0000-0000-0000-000000000002");

    let vat_10 = |v: i64| v / 10;
    let vat_20 = |v: i64| v / 5;

    let with_vat_10 = |v: i64| v + v / 10;
    let with_vat_20 = |v: i64| v + v / 5;

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(ca_uuid),
                id: Some(101),
                ..Default::default()
            },
            item_list: vec![
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(1500.into()),
                    pricing_quantity: Some(1.into()),
                    pricing_vat_id: Some(VatId::R10),
                    ..Default::default()
                },
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(4500.into()),
                    pricing_quantity: Some(2.into()),
                    pricing_vat_id: Some(VatId::R20),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_contract_amendment(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let ca = ContractAmendment::select(
            &Select::default().eq(ContractAmendment::uuid, ca_uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .expect("contract amendment");
        let mut items = ContractAmendmentItem::select(
            &Select::default()
                .eq(ContractAmendmentItem::header_uuid, ca_uuid)
                .add_replace_order_asc(ContractAmendmentItem::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.pricing_unit_id, item0.unit_id);
        assert_eq!(item0.pricing_price_rub, Some(1500.into()));

        assert_eq!(item0.pricing_currency_id, item0.currency_id);
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(
            item0.pricing_currency_rate_date,
            Some(item0.currency_rate_date)
        );

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(with_vat_10(1500).into()));
        assert_eq!(
            item0.pricing_sum_included_vat_rub,
            Some(with_vat_10(1500).into())
        );
        assert_eq!(item0.pricing_sum_vat, Some(vat_10(1500).into()));
        assert_eq!(item0.pricing_sum_vat_rub, Some(vat_10(1500).into()));

        assert_eq!(item0.pricing_total_sum, Some(with_vat_10(1500).into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(with_vat_10(1500).into()));

        // item1
        assert_eq!(item1.pricing_unit_id, item1.unit_id);
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, item1.currency_id);
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(
            item1.pricing_currency_rate_date,
            Some(item1.currency_rate_date)
        );

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(
            item1.pricing_sum_included_vat,
            Some(with_vat_20(4500 * 2).into())
        );
        assert_eq!(
            item1.pricing_sum_included_vat_rub,
            Some(with_vat_20(4500 * 2).into())
        );
        assert_eq!(item1.pricing_sum_vat, Some(vat_20(4500 * 2).into()));
        assert_eq!(item1.pricing_sum_vat_rub, Some(vat_20(4500 * 2).into()));

        assert_eq!(item1.pricing_total_sum, Some(with_vat_20(4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some(with_vat_20(4500 * 2).into()));

        // header

        // sum fields
        assert_eq!(ca.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(ca.pricing_sum_excluded_vat_rub, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            ca.pricing_sum_included_vat,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            ca.pricing_sum_included_vat_rub,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            ca.pricing_sum_vat,
            Some((vat_10(1500) + vat_20(4500 * 2)).into())
        );
        assert_eq!(
            ca.pricing_sum_vat_rub,
            Some((vat_10(1500) + vat_20(4500 * 2)).into())
        );

        assert_eq!(
            ca.pricing_total_sum,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            ca.pricing_total_sum_rub,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );

        // common vat_id
        assert_eq!(ca.pricing_vat_id, VatId::Compound);
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn calculated_fields_ca_deltas() {
    let ca_uuid = uuid!("00000001-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000001-0000-0000-0000-000000000001");
    let item_uuid2 = uuid!("00000001-0000-0000-0000-000000000002");

    let vat_10 = |v: i64| v / 10;
    let vat_20 = |v: i64| v / 5;
    let with_vat_10 = |v: i64| v + v / 10;
    let with_vat_20 = |v: i64| v + v / 5;

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(ca_uuid),
                id: Some(101),
                ..Default::default()
            },
            item_list: vec![
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(1500.into()),
                    pricing_quantity: Some(1.into()),
                    pricing_vat_id: Some(VatId::R10),
                    ..Default::default()
                },
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid2),
                    pricing_price: Some(4500.into()),
                    pricing_quantity: Some(2.into()),
                    pricing_vat_id: Some(VatId::R20),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let pool = &*pctx.db_pool;

        let res = pa_update_contract_amendment(req, pctx.clone()).await;
        assert!(res.is_ok(), "should be ok: {res:?}");

        let ca = ContractAmendment::select(
            &Select::default().eq(ContractAmendment::uuid, ca_uuid),
            pool,
        )
        .await
        .unwrap()
        .pop()
        .expect("contract amendment");
        let mut items = ContractAmendmentItem::select(
            &Select::default()
                .eq(ContractAmendmentItem::header_uuid, ca_uuid)
                .add_replace_order_asc(ContractAmendmentItem::uuid),
            pool,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 2);

        let item1 = items.pop().unwrap();
        let item0 = items.pop().unwrap();

        // item0
        assert_eq!(item0.pricing_unit_id, item0.unit_id);
        assert_eq!(item0.pricing_price_rub, Some(1500.into()));

        assert_eq!(item0.pricing_currency_id, item0.currency_id);
        assert_eq!(item0.pricing_currency_rate, Some(item0.currency_rate));
        assert_eq!(
            item0.pricing_currency_rate_date,
            Some(item0.currency_rate_date)
        );

        assert_eq!(item0.pricing_sum_excluded_vat, Some(1500.into()));
        assert_eq!(item0.pricing_sum_excluded_vat_rub, Some(1500.into()));
        assert_eq!(item0.pricing_sum_included_vat, Some(with_vat_10(1500).into()));
        assert_eq!(
            item0.pricing_sum_included_vat_rub,
            Some(with_vat_10(1500).into())
        );

        // check input values
        assert_eq!(item0.previous_price, 1000.into());
        assert_eq!(item0.previous_quantity, 2.into());
        assert_eq!(item0.previous_sum_excluded_vat, 1000.into());
        assert_eq!(item0.previous_sum_included_vat, 1200.into());
        assert_eq!(item0.previous_sum_vat, 200.into());

        assert_eq!(item0.pricing_delta_price, Some((1500 - 1000).into()));
        assert_eq!(item0.pricing_delta_quantity, Some((1 - 2).into()));
        assert_eq!(
            item0.pricing_delta_sum_excluded_vat,
            Some((1500 - 1000).into())
        );
        assert_eq!(
            item0.pricing_delta_sum_excluded_vat_rub,
            Some((1500 - 1000).into())
        );
        assert_eq!(
            item0.pricing_delta_sum_included_vat,
            Some((with_vat_10(1500) - 1200).into())
        );
        assert_eq!(
            item0.pricing_delta_sum_included_vat_rub,
            Some((with_vat_10(1500) - 1200).into())
        );
        assert_eq!(item0.pricing_delta_sum_vat, Some((vat_10(1500) - 200).into()));
        assert_eq!(
            item0.pricing_delta_sum_vat_rub,
            Some((vat_10(1500) - 200).into())
        );

        assert_eq!(item0.pricing_total_sum, Some(with_vat_10(1500).into()));
        assert_eq!(item0.pricing_total_sum_rub, Some(with_vat_10(1500).into()));

        // item1
        assert_eq!(item1.pricing_unit_id, item1.unit_id);
        assert_eq!(item1.pricing_price_rub, Some(4500.into()));

        assert_eq!(item1.pricing_currency_id, item1.currency_id);
        assert_eq!(item1.pricing_currency_rate, Some(item1.currency_rate));
        assert_eq!(
            item1.pricing_currency_rate_date,
            Some(item1.currency_rate_date)
        );

        assert_eq!(item1.pricing_sum_excluded_vat, Some((4500 * 2).into()));
        assert_eq!(item1.pricing_sum_excluded_vat_rub, Some((4500 * 2).into()));
        assert_eq!(
            item1.pricing_sum_included_vat,
            Some(with_vat_20(4500 * 2).into())
        );
        assert_eq!(
            item1.pricing_sum_included_vat_rub,
            Some(with_vat_20(4500 * 2).into())
        );

        assert_eq!(item1.previous_price, 3000.into());
        assert_eq!(item1.previous_quantity, 1.into());
        assert_eq!(item1.previous_sum_excluded_vat, 6000.into());
        assert_eq!(item1.previous_sum_included_vat, 6600.into());
        assert_eq!(item1.previous_sum_vat, 600.into());

        assert_eq!(item1.pricing_delta_price, Some((4500 - 3000).into()));
        assert_eq!(item1.pricing_delta_quantity, Some((2 - 1).into()));
        assert_eq!(
            item1.pricing_delta_sum_excluded_vat,
            Some((4500 * 2 - 6000).into())
        );
        assert_eq!(
            item1.pricing_delta_sum_excluded_vat_rub,
            Some((4500 * 2 - 6000).into())
        );
        assert_eq!(
            item1.pricing_delta_sum_included_vat,
            Some((with_vat_20(4500 * 2) - 6600).into())
        );
        assert_eq!(
            item1.pricing_delta_sum_included_vat_rub,
            Some((with_vat_20(4500 * 2) - 6600).into())
        );
        assert_eq!(
            item1.pricing_delta_sum_vat,
            Some((vat_20(4500 * 2) - 600).into())
        );
        assert_eq!(
            item1.pricing_delta_sum_vat_rub,
            Some((vat_20(4500 * 2) - 600).into())
        );

        assert_eq!(item1.pricing_total_sum, Some(with_vat_20(4500 * 2).into()));
        assert_eq!(item1.pricing_total_sum_rub, Some(with_vat_20(4500 * 2).into()));

        // header

        // sum fields
        assert_eq!(ca.pricing_sum_excluded_vat, (1500 + 4500 * 2).into());
        assert_eq!(ca.pricing_sum_excluded_vat_rub, Some((1500 + 4500 * 2).into()));
        assert_eq!(
            ca.pricing_sum_included_vat,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            ca.pricing_sum_included_vat_rub,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );

        assert_eq!(
            ca.pricing_total_sum,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );
        assert_eq!(
            ca.pricing_total_sum_rub,
            Some((with_vat_10(1500) + with_vat_20(4500 * 2)).into())
        );

        // common vat_id
        assert_eq!(ca.pricing_vat_id, VatId::Compound);
    })
    .await
}

/// Тест на наличие ошибок при проверке commission_date от пользователя
///
/// NB: Здесь тестируется только наличие самой проверки, а не правильность ее логики. Для этого
/// есть отдельные тесты
#[tokio::test]
async fn commission_date_check_failure() {
    // Невалидная, так как слишком слишком рано установлена
    let invalid_commission_date = Some(Some(asez_date!("2012-12-12")));

    // По ППЗ/ДС есть позиции Протокола, которые имеют result_id = 2, НО они is_removed=true
    let req1 = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                id: Some(1),
                commission_date: invalid_commission_date,
                ..Default::default()
            },
            ..Default::default()
        },
    };
    // По ППЗ/ДС есть позиции Протокола, которые имеют result_id = 1 и is_removed = false
    let req2 = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(uuid!("00000001-0000-0000-0000-000000000001")),
                id: Some(101),
                commission_date: invalid_commission_date,
                ..Default::default()
            },
            ..Default::default()
        },
    };
    // По ППЗ/ДС есть позиции Протокола, которые имеют result_id = 2 и is_removed = false, НО сам Прокотокол имеет
    // is_removed = true
    let req3 = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(uuid!("00000001-0000-0000-0000-000000000002")),
                id: Some(102),
                commission_date: invalid_commission_date,
                ..Default::default()
            },
            ..Default::default()
        },
    };
    // По ППЗ/ДС есть позиции Протокола, которые имеют result_id = 2 и is_removed = false, то есть
    // она пропускает проверку на commission_date
    let req4 = UserIdWrapper {
        user_id: 666,
        dto: UpdatePlanReq {
            plan: PlanRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000002")),
                id: Some(2),
                commission_date: invalid_commission_date,
                ..Default::default()
            },
            ..Default::default()
        },
    };

    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;

        let res = pa_update_plan(req1, pctx.clone()).await.unwrap_err();
        assert!(matches!(
            res,
            ProcessingError::UpdatePlanCA(
                UpdatePlanCAError::OldCommissionDate { .. }
            )
        ));

        let res =
            pa_update_contract_amendment(req2, pctx.clone()).await.unwrap_err();
        assert!(matches!(
            res,
            ProcessingError::UpdatePlanCA(
                UpdatePlanCAError::OldCommissionDate { .. }
            )
        ));

        let res =
            pa_update_contract_amendment(req3, pctx.clone()).await.unwrap_err();
        assert!(matches!(
            res,
            ProcessingError::UpdatePlanCA(
                UpdatePlanCAError::OldCommissionDate { .. }
            )
        ));

        let _res = pa_update_plan(req4, pctx.clone())
            .await
            .expect("Должно быть без ошибки");
    })
    .await
}

#[tokio::test]
#[allow(clippy::inconsistent_digit_grouping)]
async fn plan_items_failure_not_found() {
    let ca_uuid = uuid!("00000001-0000-0000-0000-000000000001");

    let item_uuid1 = uuid!("00000001-0000-0000-0000-000000000001");
    let missing_item_uuid2 = uuid!("00000001-0000-0000-0000-000000000003");

    let req = UserIdWrapper {
        user_id: 666,
        dto: UpdateContractAmendmentReq {
            contract_amendment: ContractAmendmentRep {
                uuid: Some(ca_uuid),
                id: Some(101),
                ..Default::default()
            },
            item_list: vec![
                ContractAmendmentItemRep {
                    uuid: Some(item_uuid1),
                    pricing_price: Some(1500.into()),
                    pricing_quantity: Some(1.into()),
                    pricing_vat_id: Some(VatId::NoVat),
                    ..Default::default()
                },
                ContractAmendmentItemRep {
                    uuid: Some(missing_item_uuid2),
                    pricing_price: Some(4500.into()),
                    pricing_quantity: Some(2.into()),
                    pricing_vat_id: Some(VatId::R20),
                    ..Default::default()
                },
            ],
            pricing_attachment_list: vec![],
        },
    };
    tests::run_db_test(EXTRA_MIG, move |pool| async move {
        let pctx = tests::mock_processing_context(pool.clone()).await;
        super::launch_monolith_listener(&pctx, vec![]).await;

        let res = pa_update_contract_amendment(req, pctx.clone()).await;
        assert!(res.is_err(), "should be err: {res:?}");
        let err = res.unwrap_err();
        assert!(
            matches!(
                err,
                ProcessingError::UpdatePlanCA(UpdatePlanCAError::ItemNotFound(_))
            ),
            "{err}"
        );
    })
    .await
}
