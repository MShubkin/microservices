use super::*;
use crate::{app_process, common::ProcessingError};

use asez2_shared_db::uuid;
use shared_essential::presentation::dto::response_request::EntityKind;

const GET_PLANS_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_attachments_meta.sql"];

#[tokio::test]
async fn test_get_attachments_meta_success() {
    run_db_test(GET_PLANS_EXTRA_MIGS, |pool| async move {
        let plan1_id = ObjectIdentifier::new_with_type(
            1,
            uuid!("00000000-0000-0000-0000-000000000001"),
            EntityKind::Plan,
        );
        let plan2_id = ObjectIdentifier::new_with_type(
            101,
            uuid!("00000000-0000-0000-0001-000000000000"),
            EntityKind::ContractAmendment,
        );

        let input = GetAttachmentsMetaRequest {
            item_list: vec![plan1_id.clone(), plan2_id.clone()],
        };

        let r =
            app_process::get_attachments_meta(input, pool.clone()).await.unwrap();

        assert!(r.messages.is_empty());
        assert_eq!(r.data.len(), 2);

        let expected_result = vec![
            GetAttachmentsMetaResponseItem {
                id: plan1_id,
                attachment_list: vec![
                    AttachmentMeta {
                        uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                        category_id: CategoryId::Agenda,
                        parent_number: 1,
                    },
                    AttachmentMeta {
                        uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                        category_id: CategoryId::ProtocolInPersonEc,
                        parent_number: 1,
                    },
                ],
            },
            GetAttachmentsMetaResponseItem {
                id: plan2_id,
                attachment_list: vec![
                    AttachmentMeta {
                        uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                        category_id: CategoryId::Bulletin,
                        parent_number: 1,
                    },
                    AttachmentMeta {
                        uuid: uuid!("00000000-0000-0000-0000-000000000005"),
                        category_id: CategoryId::Estimates,
                        parent_number: 1,
                    },
                ],
            },
        ];

        assert_eq!(r.data, expected_result);
    })
    .await
}

#[tokio::test]
async fn test_get_attachments_meta_not_found_failure() {
    run_db_test(GET_PLANS_EXTRA_MIGS, |pool| async move {
        let plan1_id = ObjectIdentifier::new_with_type(
            777,
            uuid!("10000000-0000-0000-0000-000000000000"),
            EntityKind::Plan,
        );
        let plan2_id = ObjectIdentifier::new_with_type(
            888,
            uuid!("20000000-0000-0000-0000-000000000000"),
            EntityKind::ContractAmendment,
        );
        let plan3_id = ObjectIdentifier::new_with_type(
            1,
            uuid!("00000000-0000-0000-0000-000000000001"),
            EntityKind::Plan,
        );

        let input = GetAttachmentsMetaRequest {
            item_list: vec![plan1_id, plan2_id, plan3_id],
        };

        let r = app_process::get_attachments_meta(input, pool.clone())
            .await
            .unwrap_err();

        assert_eq!(
            r.to_string(),
            ProcessingError::GetItemList(String::from(
                "Записи ППЗ/ДС c идентификаторами 777, 888 не найдены"
            ))
            .to_string(),
            "{:?}",
            r
        );
    })
    .await
}
