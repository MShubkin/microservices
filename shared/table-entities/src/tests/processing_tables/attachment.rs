use super::*;
use crate::processing::attachment::*;

use asez2_shared_db::db_item::{AsezTimestamp, Select};

const ATTACHMENT: &str = r#"{
    "uuid": "550E8400E29B41D4A716446655440000",   
    "id": 25,
    "parent_id": 0,  
    "kind": 2,
    "text": "Справка-обоснование потребности",
    "category_id": 6,
    "is_classified": false,
    "is_removed": false,
    "pricing_version": 2,
    "mime_id": 0,
    "size": 5,
    "changed_at": 1710401820,
    "changed_by": 25,
    "created_at": 1685259420,
    "created_by": 35
}"#;
const ATTACHMENT_RET: &str = r#"{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "object_uuid": "00000000-0000-0000-0000-000000000777",
  "id": 25,
  "kind": 2,
  "text": "Справка-обоснование потребности",
  "parent_id": 0,
  "category_id": 6,
  "mime_id": 0,
  "size": 5,
  "is_removed": false,
  "is_classified": false,
  "pricing_version": 2,
  "created_at": 1685259420,
  "changed_at": 1710401820,
  "created_by": 35,
  "changed_by": 25
}"#;

const CREATE_TABLE: &str = "(
    uuid uuid NOT NULL PRIMARY KEY,
    object_uuid uuid NOT NULL,
    number SMALLINT NOT NULL,
    kind_id SMALLINT NOT NULL DEFAULT 0,
    name VARCHAR(255) NOT NULL DEFAULT '',
    parent_number SMALLINT,
    category_id SMALLINT NOT NULL DEFAULT 0,
    mime_id SMALLINT NOT NULL DEFAULT 0,
    size BIGINT NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    is_classified BOOLEAN NOT NULL DEFAULT false,
    pricing_version SMALLINT NOT NULL DEFAULT 0,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL
  )";

#[tokio::test]
async fn test_insert_retrieve_attachment() {
    run_db_test(Attachment::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let rep: AttachmentRep = serde_json::from_str(ATTACHMENT).unwrap();

        let mut exp = AttachmentRep {
            uuid: Some(
                Uuid::parse_str("550E8400E29B41D4A716446655440000").unwrap(),
            ),
            id: Some(25),
            parent_id: Some(Some(0)),
            kind: Some(AttachmentKind::Directory),
            text: Some("Справка-обоснование потребности".to_string()),
            category_id: Some(CategoryId::JustificationOfDemands),
            is_classified: Some(false),
            is_removed: Some(false),
            pricing_version: Some(2),
            mime_id: Some(0),
            size: Some(5),
            changed_at: Some(AsezTimestamp::from_unix_timestamp(1710401820)),
            changed_by: Some(25),
            created_at: Some(AsezTimestamp::from_unix_timestamp(1685259420)),
            created_by: Some(35),
            object_uuid: None,
        };
        // Проверка верный десереализации
        assert_eq!(rep, exp);

        let mut item = rep.into_item().unwrap();
        // Add uuid of the associate object.
        item.object_uuid =
            Uuid::parse_str("00000000-0000-0000-0000-000000000777").unwrap();
        exp.object_uuid = Some(item.object_uuid);

        let res = item.insert_returning(&mut pool).await.unwrap();
        // Проверка верный вставки
        assert_eq!(res, item);

        let sel = Select::default().eq(Attachment::uuid, exp.uuid);
        let res = Attachment::select(&sel, &mut pool).await.unwrap().pop().unwrap();
        // Проверка верного возврата
        assert_eq!(res, item);

        let item = AttachmentRep::from_item::<&str>(res, None);
        // Проверка верной сериализации
        assert_eq!(item, exp);

        let item_str = serde_json::to_string_pretty(&item).unwrap();
        assert_eq!(item_str, ATTACHMENT_RET, "{} vs {}", item_str, ATTACHMENT_RET);
    })
    .await
}
