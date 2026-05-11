use crate::app_process::price_analysis::pa_complete_lotting;
use crate::app_process::tests::{mock_processing_context, run_db_test};

use asez2_shared_db::uuid;
use shared_essential::presentation::dto::general::{
    ObjectIdentifier, ObjectIdentifierList,
};
use shared_essential::presentation::dto::processing::price_analysis::CompleteLottingRequest;
use shared_essential::presentation::dto::response_request::*;

const MIGS: [&str; 1] = ["price_analysis/complete_lotting.sql"];

#[tokio::test]
async fn test_complete_lotting() {
    let item_list = vec![
        ObjectIdentifier::new_with_type(
            1,
            uuid!("00000000-0000-0000-0000-000000000001"),
            EntityKind::Plan,
        ),
        ObjectIdentifier::new_with_type(
            2,
            uuid!("00000000-0000-0000-0000-000000000002"),
            EntityKind::Plan,
        ),
        ObjectIdentifier::new_with_type(
            3,
            uuid!("00000000-0000-0000-0000-000000000003"),
            EntityKind::Plan,
        ),
        ObjectIdentifier::new_with_type(
            5,
            uuid!("00000000-0000-0000-0000-000000000005"),
            EntityKind::Plan,
        ),
        ObjectIdentifier::new_with_type(
            6,
            uuid!("00000000-0000-0000-0000-000000000006"),
            EntityKind::Plan,
        ),
        ObjectIdentifier::new_with_type(
            7,
            uuid!("00000000-0000-0000-0000-000000000007"),
            EntityKind::Plan,
        ),
    ];
    let req = CompleteLottingRequest {
        user_id: 1,
        dto: ObjectIdentifierList { item_list },
    };
    let exp_messages = vec![
        Message {
            kind: MessageKind::Success,
            text: "2 ППЗ переведены на статус \"Анализ цены МТР. Назначение исполнителя\".".to_string(),
            parameters: Params {
                description: "".to_string(),
                item_list: vec![
                    ParamItem::from_id(1).with_type(EntityKind::Plan).with_uuid(uuid!("00000000-0000-0000-0000-000000000001")),
                    ParamItem::from_id(5).with_type(EntityKind::Plan).with_uuid(uuid!("00000000-0000-0000-0000-000000000005"))
                ]
            },
            fields: Default::default()
        },
        Message {
            kind: MessageKind::Success,
            text: "1 ППЗ переведен на статус \"Анализ цены МТР. Исполнитель назначен\".".to_string(),
            parameters: Params {
                description: "".to_string(),
                item_list: vec![
                    ParamItem::from_id(2).with_type(EntityKind::Plan).with_uuid(uuid!("00000000-0000-0000-0000-000000000002"))
                ]
            },
            fields: Default::default()

        },
        Message {
            kind: MessageKind::Success,
            text: "1 ППЗ переведен на статус \"Анализ цены МТР. Анализ проведен\".".to_string(),
            parameters: Params {
                description: "".to_string(),
                item_list: vec![
                    ParamItem::from_id(3).with_type(EntityKind::Plan).with_uuid(uuid!("00000000-0000-0000-0000-000000000003")),
                ]
            },
            fields: Default::default()

        },
    ];
    let exp_messages = Messages {
        messages: exp_messages,
        kind: MessageKind::Success,
    };
    run_db_test(&MIGS, |pool| async move {
        let pctx = mock_processing_context(pool).await;
        let res = pa_complete_lotting(req, pctx).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.messages.kind, MessageKind::Success);
        assert_eq!(res.messages.messages.len(), 3);

        let mut exp_messages = exp_messages.messages;
        let mut actual_messages = res.messages.messages;

        exp_messages.sort_unstable();
        actual_messages.sort_unstable();

        assert_eq!(exp_messages, actual_messages);
    })
    .await
}
