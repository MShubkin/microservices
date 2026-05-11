use super::*;

use crate::processing::plan::PricingUnitId;
use crate::processing::protocol::ProtocolType;

const CREATE_TABLE: &str = "(
    uuid uuid NOT NULL PRIMARY KEY,
    id BIGINT NOT NULL,
    protocol_type_id SMALLINT NOT NULL DEFAULT 0,
    registration_number VARCHAR(64),
    status_id SMALLINT NOT NULL,
    pricing_organization_unit_id SMALLINT NOT NULL DEFAULT 0,
    is_secret BOOLEAN NOT NULL DEFAULT false,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    protocol_date DATE NOT NULL,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
  )";

// FIXME: Кажется, что тест не актуален из-за того, что больше нет авто-активации полей. Удаляем?
//
// #[tokio::test]
// async fn test_conversion() {
// This structure contains the minimum fields that we should encounter.
// This structure contains the minimum fields that we should encounter.
// let new = NewEcProtocolRep {
//     id: 1000004323,
//     protocol_type_id: ProtocolType::InPersonMeeting,
//     is_secret: false,
//     created_by: USER1,
//     registration_number: Some(1000002345.to_string()),
//     protocol_date: AsezDate::try_from("1901-01-01").unwrap(),
// };
// let rep = EcProtocolRep {
//     id: Some(1000004323),
//     protocol_type_id: Some(ProtocolType::InPersonMeeting),
//     is_secret: Some(false),
//     created_by: Some(USER1),
//     registration_number: Some(Some("1000002345".to_string())),
//     pricing_organization_unit_id: None,
//     protocol_date: Some(AsezDate::try_from("1901-01-01").unwrap()),
//     ..Default::default()
// };
// let expected_uninserted = EcProtocol {
//     uuid: Default::default(),
//     id: 1000004323,
//     protocol_type_id: ProtocolType::InPersonMeeting,
//     is_secret: false,
//     registration_number: Some("1000002345".to_string()),
//     pricing_organization_unit_id: Default::default(),
//     protocol_date: AsezDate::default(),
//     created_by: USER1,
//     ..Default::default()
// };
// let uninserted = rep.clone().into_item().unwrap();
// let new_rep = EcProtocolRep::from(new);

// // Test if the experimental NewEcProtocolRep performs as expected.
// assert_eq!(new_rep, rep);
// assert_eq!(uninserted, expected_uninserted);

// let mut activated = uninserted;
// activated.activate_fields();

// // If our uuid generator gives us all 0s we have a problem.
// assert_ne!(
//     activated.uuid,
//     Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
// );

// let exp_activated = EcProtocol {
//     uuid: activated.uuid,
//     id: 1000004323,
//     protocol_type_id: ProtocolType::InPersonMeeting,
//     is_secret: false,
//     registration_number: Some("1000002345".to_string()),
//     pricing_organization_unit_id: Default::default(),
//     is_removed: false,
//     protocol_date: AsezDate::default(),
//     created_by: USER1,
//     status_id: 100.into(),
//     created_at: activated.created_at,
//     changed_at: activated.created_at,
//     changed_by: USER1,
// };
// assert_eq!(activated, exp_activated);
// }

#[tokio::test]
async fn test_insert() {
    run_db_test(EcProtocol::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let new: EcProtocolRep = NewEcProtocolRep {
            id: 1000004323,
            protocol_type_id: ProtocolType::InPersonMeeting,
            is_secret: false,
            created_by: USER1,
            registration_number: Some("1000002345".to_string()),
            protocol_date: AsezDate::try_from("2002-02-21").unwrap(),
        }
        .into();
        let mut rep = new.into_item().unwrap();

        let res = rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        let got = sqlx::query("SELECT * FROM protocol;")
            .map(|x| {
                EcProtocolRep::from_item::<&str>(
                    EcProtocol::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        assert!(got[0].uuid.is_some());
        assert_eq!(
            got[0].pricing_organization_unit_id,
            Some(PricingUnitId::Undefined)
        );
        assert_eq!(got[0].id, Some(1000004323));
        // FIXME: Должны ли мы тут ожидать этот статус?
        // assert_eq!(got[0].status_id, Some(EcProtocolStatus::Formed));
        assert!(got[0].registration_number.is_some());
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(got[0].is_secret, Some(false));
        assert_eq!(
            got[0].protocol_date,
            Some(AsezDate::try_from("2002-02-21").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_by, Some(USER1));
        assert_eq!(got[0].changed_at, got[0].created_at);
        assert!(got[0].created_at.is_some());
    })
    .await
}

#[tokio::test]
async fn test_insert_vec() {
    run_db_test(EcProtocol::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let mut new1: EcProtocolRep = NewEcProtocolRep {
            id: 1000004323,
            protocol_type_id: ProtocolType::InPersonMeeting,
            is_secret: false,
            created_by: USER1,
            registration_number: Some("1000002345".to_string()),
            protocol_date: AsezDate::try_from("2002-02-21").unwrap(),
        }
        .into();
        new1.uuid = Some(Uuid::new_v4());

        let mut new2: EcProtocolRep = NewEcProtocolRep {
            id: 1000004324,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            is_secret: false,
            created_by: USER1,
            registration_number: Some("1000002346".to_string()),
            protocol_date: AsezDate::try_from("2002-02-21").unwrap(),
        }
        .into();
        new2.uuid = Some(Uuid::new_v4());

        let mut reps = [new1.into_item().unwrap(), new2.into_item().unwrap()];

        let res = EcProtocol::insert_vec(&mut reps, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM protocol;")
            .map(|x| {
                EcProtocolRep::from_item::<&str>(
                    EcProtocol::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert!(got[0].uuid.is_some());
        assert_eq!(
            got[0].pricing_organization_unit_id,
            Some(PricingUnitId::Undefined)
        );
        assert!(got[0].registration_number.is_some());
        assert_eq!(got[0].id, Some(1000004323));

        // FIXME: Должны ли мы тут ожидать этот статус?
        // assert_eq!(got[0].status_id, Some(EcProtocolStatus::Formed));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(got[0].is_secret, Some(false));
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_at, got[0].created_at);
        assert!(got[0].created_at.is_some());

        assert!(got[1].uuid.is_some());
        assert!(got[1].registration_number.is_some());
        assert_eq!(
            got[1].pricing_organization_unit_id,
            Some(PricingUnitId::Undefined)
        );
        assert_eq!(got[1].id, Some(1000004324));
        assert_eq!(got[1].changed_at, got[1].created_at);
        assert!(got[1].created_at.is_some());
    })
    .await
}

#[tokio::test]
async fn test_update() {
    run_db_test(EcProtocol::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let new: EcProtocolRep = NewEcProtocolRep {
            id: 1000004323,
            protocol_type_id: ProtocolType::InPersonMeeting,
            is_secret: false,
            created_by: USER1,
            registration_number: Some("1000002345".to_string()),
            protocol_date: AsezDate::try_from("2002-02-21").unwrap(),
        }
        .into();
        let mut rep = new.into_item().unwrap();

        let res = rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        // Check content after insert.
        let got = sqlx::query("SELECT * FROM protocol;")
            .map(|x| {
                EcProtocolRep::from_item::<&str>(
                    EcProtocol::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();

        assert_eq!(got.len(), 1);
        assert_eq!(
            got[0].pricing_organization_unit_id,
            Some(PricingUnitId::Undefined)
        );
        assert_eq!(
            got[0].protocol_date,
            Some(AsezDate::try_from("2002-02-21").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_by, Some(USER1));

        // Make update.
        let upd_agenda = EcProtocolRep {
            // We select what to update by uuid.
            uuid: got[0].uuid,
            pricing_organization_unit_id: Some(PricingUnitId::D646),
            changed_by: Some(USER2),
            protocol_date: Some(AsezDate::try_from("2005-02-21").unwrap()),
            ..Default::default()
        }
        .into_item()
        .unwrap();

        let update_fields = Some(
            &["pricing_organization_unit_id", "changed_by", "protocol_date"][..],
        );

        let res = upd_agenda.update(update_fields, &mut pool).await.unwrap();
        assert_eq!(res, 1);

        // Check content after update.
        let got = sqlx::query("SELECT * FROM protocol;")
            .map(|x| {
                EcProtocolRep::from_item::<&str>(
                    EcProtocol::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].pricing_organization_unit_id, Some(PricingUnitId::D646));
        assert_eq!(
            got[0].protocol_date,
            Some(AsezDate::try_from("2005-02-21").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_by, Some(USER2));
    })
    .await
}

#[tokio::test]
async fn test_update_vec() {
    run_db_test(EcProtocol::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let mut new1: EcProtocolRep = NewEcProtocolRep {
            id: 1000004323,
            protocol_type_id: ProtocolType::InPersonMeeting,
            is_secret: false,
            created_by: USER1,
            registration_number: Some("1000002345".to_string()),
            protocol_date: AsezDate::try_from("2002-02-21").unwrap(),
        }
        .into();
        new1.uuid = Some(Uuid::new_v4());
        let mut new2: EcProtocolRep = NewEcProtocolRep {
            id: 1000004324,
            protocol_type_id: ProtocolType::CorrespondenceMeeting,
            is_secret: false,
            created_by: USER1,
            registration_number: Some("1000002346".to_string()),
            protocol_date: AsezDate::try_from("2002-02-21").unwrap(),
        }
        .into();
        new2.uuid = Some(Uuid::new_v4());
        let mut reps = [new1.into_item().unwrap(), new2.into_item().unwrap()];
        let res = EcProtocol::insert_vec(&mut reps, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        // Check content after insert.

        let got = sqlx::query("SELECT * FROM protocol;")
            .map(|x| {
                EcProtocolRep::from_item::<&str>(
                    EcProtocol::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();

        assert_eq!(got.len(), 2);
        assert_eq!(
            got[0].pricing_organization_unit_id,
            Some(PricingUnitId::Undefined)
        );
        assert_eq!(
            got[1].pricing_organization_unit_id,
            Some(PricingUnitId::Undefined)
        );
        assert_eq!(
            got[1].protocol_type_id,
            Some(ProtocolType::CorrespondenceMeeting)
        );

        let upd_agenda1 = EcProtocolRep {
            // We select what to update by uuid.
            uuid: got[0].uuid,
            pricing_organization_unit_id: None,
            protocol_type_id: got[1].protocol_type_id,
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let upd_agenda2 = EcProtocolRep {
            // We select what to update by uuid.
            uuid: got[1].uuid,
            pricing_organization_unit_id: Some(PricingUnitId::Gpk),
            protocol_type_id: Some(ProtocolType::InPersonMeeting),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let to_update = vec![upd_agenda1, upd_agenda2];

        let update_columns =
            vec!["pricing_organization_unit_id", "protocol_type_id"];

        let res =
            EcProtocol::update_vec(&to_update, Some(&update_columns), &mut pool)
                .await
                .unwrap();
        assert_eq!(res, 2);

        // Check content after update.
        let got = sqlx::query("SELECT * FROM protocol;")
            .map(|x| {
                EcProtocolRep::from_item::<&str>(
                    EcProtocol::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(
            got[0].pricing_organization_unit_id,
            Some(PricingUnitId::Undefined)
        );
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_by, Some(USER1));
        // NB: We use char(4) in the DB, so we will always get 4 chars back.
        assert_eq!(got[1].pricing_organization_unit_id, Some(PricingUnitId::Gpk));
        assert_eq!(got[1].protocol_type_id, Some(ProtocolType::InPersonMeeting));
        assert_eq!(got[1].created_by, Some(USER1));
        assert_eq!(got[1].changed_by, Some(USER1));
    })
    .await
}
