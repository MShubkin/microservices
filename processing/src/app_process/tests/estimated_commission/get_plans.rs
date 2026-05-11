use super::*;
use crate::app_process;

use asez2_shared_db::db_item::selection::SelectionKind;
use shared_essential::presentation::dto::response_request::{
    EntityKind, ParamItem,
};

const GET_PLANS_EXTRA_MIGS: &[&str] = &["estimated_commission/get_plans.sql"];

#[tokio::test]
async fn test_get_plans_multicurrency() {
    run_db_test(GET_PLANS_EXTRA_MIGS, |pool| async move {
        let input = CompletePlansRequest {
            section: Section::None,
            user_id: USER1,
            select: Select::with_fields(["plan_id", "uuid", "currency_id"])
                .add_expand_filter(
                    "id",
                    SelectionKind::In,
                    vec![Value::from(1), Value::from(2)]
                ),
            item_fields: PlanItem::FIELDS.iter().map(|x| x.to_string()).collect(),
        };

        let r = app_process::get_complete_plans(input, pool.clone()).await.unwrap();

        // This check checks fields.
        assert!(r.data.item_list[0].plan.id.is_none());
        assert!(r.data.item_list[0].plan.plan_id.is_some());
        let exp_msg = vec![
            Message::warn(
                "ППЗ (2): Валюты (2, 2, 2, 3) в позициях (6, 7, 8, 9) отличаются от валюты заголовка (1)".to_string()
            ).with_parameters(vec![ParamItem::from_id(2).with_uuid(Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()).with_type(EntityKind::Plan)])
        ];
        assert_eq!(exp_msg, r.messages.messages);
    }).await
}
