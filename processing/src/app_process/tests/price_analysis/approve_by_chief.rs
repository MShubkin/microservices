//! Тестирование процесса [`documentation_checked`]
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::uuid;

use shared_essential::{
    domain::{Plan, PlanOrAmendment},
    presentation::dto::{
        general::ObjectIdentifierWithStatusNote,
        processing::price_analysis::ApproveByChiefReq,
        response_request::{BusinessMessage, EntityKind, Messages},
    },
};

use crate::app_process::pa_approve_by_chief;
use crate::app_process::tests::{mock_processing_context, run_db_test};
use crate::presentation::business_messages::plan::PlanApproveByChiefMessage;

const EXTRA_MIGS: &[&str] = &["price_analysis/approve_by_chief.sql"];
const USER_ID: i32 = 777;

/// По ППЗ/ДС не заполнены поля
#[tokio::test]
async fn missing_fields() {
    run_db_test(EXTRA_MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;

        let dto = ApproveByChiefReq {
            user_id: USER_ID,
            item_list: vec![ObjectIdentifierWithStatusNote::new_with_type(
                101,
                uuid!("00000000-0000-0000-0000-000000000101"),
                EntityKind::Plan,
                String::from("ground-zero"),
            )],
        };

        let res = pa_approve_by_chief(dto, pctx).await.unwrap();

        let expected_messages = Messages::default().with_messages(vec![
            PlanApproveByChiefMessage::FieldIsMissing("Эксперт АЦ").singular(
                &PlanOrAmendment::Plan(Plan {
                    id: 101,
                    ..Default::default()
                }),
            ),
            PlanApproveByChiefMessage::FieldIsMissing("Дата СК").singular(
                &PlanOrAmendment::Plan(Plan {
                    id: 101,
                    ..Default::default()
                }),
            ),
        ]);
        assert_eq!(expected_messages, res.messages);
    })
    .await;
}
