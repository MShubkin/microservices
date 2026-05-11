use crate::domain::*;
use crate::presentation::dto::general::ObjectIdentifier;
use crate::presentation::dto::processing::GetPlansItem;
use crate::presentation::dto::response_request::*;
use asez2_shared_db::db_item::AsezDate;
use uuid::Uuid;

const BOB_IS_WRONG: &str = "Bob is wrong, his plan is wrong, his agenda is wrong.";

#[test]
fn test_full_message() {
    const FULL_PARAMS: &str = r#"{
  "kind": "Error",
  "text": "Bob is wrong, his plan is wrong, his agenda is wrong.",
  "parameters": {
    "description": "Bob",
    "item_list": [
      {
        "id": "XXX-007",
        "type": "agenda",
        "username": "not_bob",
        "date": "01.01.1901"
      },
      {
        "id": "1001",
        "type": "plan",
        "username": "not_bob",
        "text": "robotics",
        "date": "01.01.1901"
      }
    ]
  }
}"#;
    let message = Message::error(BOB_IS_WRONG.to_string())
        .with_param_item(ParamItem::new(
            "XXX-007".to_string(),
            EntityKind::Agenda,
            "not_bob".to_string(),
            None,
            Some(AsezDate::try_from("1901-01-01").unwrap()),
            None,
        ))
        .with_param_item(ParamItem::new(
            "1001".to_string(),
            EntityKind::Plan,
            "not_bob".to_string(),
            Some("robotics".to_owned()),
            Some(AsezDate::try_from("1901-01-01").unwrap()),
            None,
        ))
        .with_param_description("Bob");

    let message_str_res = serde_json::to_string_pretty(&message).unwrap();
    let message_res: Message = serde_json::from_str(FULL_PARAMS).unwrap();

    assert_eq!(message_str_res, FULL_PARAMS);
    assert_eq!(message_res, message);
}

#[test]
fn test_uuids_message() {
    const UUIDS_LIST_PARAMS: &str = r#"{
  "kind": "Error",
  "text": "Bob is wrong, his plan is wrong, his agenda is wrong.",
  "parameters": {
    "item_list": [
      {
        "id": "1007",
        "type": "plan",
        "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5e"
      },
      {
        "id": "1001",
        "type": "plan",
        "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5d"
      }
    ]
  }
}"#;
    let message = Message::error(BOB_IS_WRONG.to_string())
        .with_param_item(&ObjectIdentifier::new_with_type(
            1007,
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5e").unwrap(),
            EntityKind::Plan,
        ))
        .with_param_item(&ObjectIdentifier::new_with_type(
            1001,
            Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5d").unwrap(),
            EntityKind::Plan,
        ));

    let message_str_res = serde_json::to_string_pretty(&message).unwrap();
    let message_res: Message = serde_json::from_str(UUIDS_LIST_PARAMS).unwrap();

    assert_eq!(message_str_res, UUIDS_LIST_PARAMS);
    assert_eq!(message_res, message);
}

#[test]
fn test_list_message() {
    const ITEM_LIST_PARAMS: &str = r#"{
  "kind": "Error",
  "text": "Bob is wrong, his plan is wrong, his agenda is wrong.",
  "parameters": {
    "item_list": [
      {
        "id": "XXX-007",
        "type": "agenda",
        "username": "not_bob",
        "date": "01.01.1901"
      },
      {
        "id": "1001",
        "type": "plan",
        "username": "not_bob",
        "text": "notes",
        "date": "01.01.1901"
      }
    ]
  }
}"#;
    let message = Message::error(BOB_IS_WRONG.to_string())
        .with_param_item(ParamItem::new(
            "XXX-007".to_string(),
            EntityKind::Agenda,
            "not_bob".to_string(),
            None,
            Some(AsezDate::try_from("1901-01-01").unwrap()),
            None,
        ))
        .with_param_item(ParamItem::new(
            "1001".to_string(),
            EntityKind::Plan,
            "not_bob".to_string(),
            Some("notes".to_owned()),
            Some(AsezDate::try_from("1901-01-01").unwrap()),
            None,
        ));

    let message_str_res = serde_json::to_string_pretty(&message).unwrap();
    let message_res: Message = serde_json::from_str(ITEM_LIST_PARAMS).unwrap();

    assert_eq!(message_str_res, ITEM_LIST_PARAMS);
    assert_eq!(message_res, message);
}

#[test]
fn test_description_message() {
    const DESCRIPTION_PARAMS: &str = r#"{
  "kind": "Error",
  "text": "Bob is wrong, his plan is wrong, his agenda is wrong.",
  "parameters": {
    "description": "Bob"
  }
}"#;
    let message =
        Message::error(BOB_IS_WRONG.to_string()).with_param_description("Bob");

    let message_str_res = serde_json::to_string_pretty(&message).unwrap();
    let message_res: Message = serde_json::from_str(DESCRIPTION_PARAMS).unwrap();

    assert_eq!(message_str_res, DESCRIPTION_PARAMS);
    assert_eq!(message_res, message);
}

#[test]
fn test_no_params_message() {
    const EMPTY_PARAMS: &str = r#"{
  "kind": "Error",
  "text": "Bob is wrong, his plan is wrong, his agenda is wrong."
}"#;
    let message = Message::error(BOB_IS_WRONG.to_string());

    let message_str_res = serde_json::to_string_pretty(&message).unwrap();
    let message_res: Message = serde_json::from_str(EMPTY_PARAMS).unwrap();

    assert_eq!(message_str_res, EMPTY_PARAMS);
    assert_eq!(message_res, message);
}

#[test]
fn test_response_with_s_status() {
    const RES: &str = r#"{
  "status": "s"
}"#;
    let res_real = ApiResponse {
        status: Status::Ok,
        ..Default::default()
    };

    let res_real_str_res = serde_json::to_string_pretty(&res_real).unwrap();
    let exp_res_res: ApiResponse<(), ()> = serde_json::from_str(RES).unwrap();

    assert_eq!(res_real_str_res, RES);
    assert_eq!(res_real, exp_res_res);
}

#[test]
fn test_response_with_e_status() {
    const RES: &str = r#"{
  "status": "e"
}"#;
    let res_real = ApiResponse {
        status: Status::Error,
        ..Default::default()
    };

    let res_real_str_res = serde_json::to_string_pretty(&res_real).unwrap();
    let exp_res_res: ApiResponse<(), ()> = serde_json::from_str(RES).unwrap();

    assert_eq!(res_real_str_res, RES);
    assert_eq!(res_real, exp_res_res);
}

#[test]
fn test_object_identifier() {
    const RES: &str = r#"{
  "id": 1001,
  "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5d",
  "object_type": "plan"
}"#;
    let res_real = ObjectIdentifier::new_with_type(
        1001,
        Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5d").unwrap(),
        EntityKind::Plan,
    );

    let res_real_str_res = serde_json::to_string_pretty(&res_real).unwrap();
    let exp_res_res: ObjectIdentifier = serde_json::from_str(RES).unwrap();

    assert_eq!(res_real_str_res, RES);
    assert_eq!(res_real, exp_res_res);
}

#[test]
fn test_get_plans_items() {
    const RES: &str = r#"{
  "plan": {
    "object_type": "plan",
    "uuid": "2be0b94f-a543-4c37-859c-b3ad1aab8b5d",
    "id": 1001
  },
  "agenda": {
    "meeting_date": "01.01.1901",
    "created_by": 999,
    "agenda_id": 1000004323
  },
  "protocol": {
    "protocol_date": "01.01.1901",
    "created_by": 998,
    "protocol_id": 1000004323
  }
}"#;
    let initial = GetPlansItem {
        plan: PlanRep {
            id: Some(1001),
            uuid: Some(
                Uuid::parse_str("2be0b94f-a543-4c37-859c-b3ad1aab8b5d").unwrap(),
            ),
            ..Default::default()
        }
        .into(),
        agenda: Some(EcAgendaRep {
            agenda_id: Some(1000004323),
            meeting_date: Some(AsezDate::try_from("1901-01-01").unwrap()),
            created_by: Some(999),
            ..Default::default()
        }),
        protocol: Some(EcProtocolRep {
            protocol_id: Some(1000004323),
            protocol_date: Some(AsezDate::try_from("1901-01-01").unwrap()),
            created_by: Some(998),
            ..Default::default()
        }),
        agenda_item: None,
        protocol_item: None,
        _meta: None,
    };
    let res = serde_json::to_string_pretty(&initial).unwrap();
    let re_ser: GetPlansItem = serde_json::from_str(&res).unwrap();
    assert_eq!(res, RES);
    assert_eq!(initial, re_ser);
}
