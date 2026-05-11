use tokio::test;

use crate::PricingUnitId;

use super::*;

const CREATE_TABLE: &str = "(
    uuid uuid NOT NULL PRIMARY KEY,
    id BIGINT NOT NULL,
    meeting_date date NOT NULL,
    status_id SMALLINT NOT NULL DEFAULT 0,
    pricing_organization_unit_id SMALLINT NOT NULL DEFAULT 0,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    created_at timestamp without time zone NOT NULL,
    changed_at timestamp without time zone NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
  )";

#[test]
async fn test_agenda_conversion() {
    // This structure contains the minimum fields that we should encounter.
    let new_agenda = NewEcAgendaRep {
        id: 1000004323,
        meeting_date: AsezDate::try_from("1901-01-01").unwrap(),
        created_by: USER1,
    };
    let agenda_rep = EcAgendaRep {
        uuid: None,
        id: Some(1000004323),
        meeting_date: Some(AsezDate::try_from("1901-01-01").unwrap()),
        created_by: Some(USER1),
        ..Default::default()
    };
    let expected_uninserted_agenda = EcAgenda {
        uuid: Uuid::default(),
        id: 1000004323,
        meeting_date: AsezDate::default(),
        created_by: USER1,
        ..Default::default()
    };
    let uninserted_agenda = agenda_rep.clone().into_item().unwrap();
    let new_agenda_rep = EcAgendaRep::from(new_agenda);

    // Test if the experimental NewEcAgendaRep performs as expected.
    assert_eq!(new_agenda_rep, agenda_rep);
    assert_eq!(uninserted_agenda, expected_uninserted_agenda);

    let mut _activated = uninserted_agenda;

    // FIXME: ❓❓❓❓
    // Я не вижу реализации этой функции, cargo expand тоже ничего не выкидывает, соответственно,
    // UUID не генерируется и поля не активируются и все падает. Возможно, что раньше была активация полей,
    // но может ее удалили.
    // Коммент от Алексея Жлобенко: Newecagendarep, по факту игрушка. Там их нет.
    // TODO: Не решено, что делать с тестом

    // activated.activate_fields();

    // // If our uuid generator gives us all 0s we have a problem.
    // assert_ne!(
    //     activated.uuid,
    //     Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    // );

    // let exp_activated_agenda = EcAgenda {
    //     uuid: activated.uuid,
    //     id: 1000004323,
    //     pricing_organization_unit_id: Default::default(),
    //     is_removed: false,
    //     meeting_date: AsezDate::default(),
    //     created_by: USER1,
    //     status_id: EcAgendaStatus::Formed,
    //     created_at: activated.created_at,
    //     changed_at: activated.created_at,
    //     changed_by: USER1,
    // };
    // assert_eq!(activated, exp_activated_agenda);
}

#[test]
async fn test_insert_agenda() {
    run_db_test(EcAgenda::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let new: EcAgendaRep = NewEcAgendaRep {
            id: 1000004323,
            meeting_date: AsezDate::try_from("2002-02-20").unwrap(),
            created_by: USER1,
        }
        .into();
        let mut agenda_rep = new.into_item().unwrap();

        let res = agenda_rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        let got = sqlx::query("SELECT * FROM agenda;")
            .map(|x| {
                EcAgendaRep::from_item::<&str>(
                    EcAgenda::from_row(&x).unwrap(),
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
            Some(PricingUnitId::default())
        );
        assert_eq!(got[0].id, Some(1000004323));
        assert_eq!(got[0].status_id, Some(EcAgendaStatus::Undefined));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(
            got[0].meeting_date,
            Some(AsezDate::try_from("2002-02-20").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_at, got[0].created_at);
        assert!(got[0].created_at.is_some());
    })
    .await
}

#[test]
async fn test_insert_agenda_returning() {
    run_db_test(EcAgenda::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let new: EcAgendaRep = NewEcAgendaRep {
            id: 1000004323,
            meeting_date: AsezDate::try_from("2002-02-20").unwrap(),
            created_by: USER1,
        }
        .into();
        let mut agenda_rep = new.into_item().unwrap();

        let res = agenda_rep.insert_returning(&mut pool).await.unwrap();

        let mut got = sqlx::query("SELECT * FROM agenda;")
            .try_map(|x| EcAgenda::from_row(&x))
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);

        let got = got.pop().unwrap();
        assert_eq!(got, res);

        let got = EcAgendaRep::from_item::<&str>(got, None);

        assert!(got.uuid.is_some());
        assert_eq!(
            got.pricing_organization_unit_id,
            Some(PricingUnitId::default())
        );
        assert_eq!(got.id, Some(1000004323));
        assert_eq!(got.status_id, Some(EcAgendaStatus::Undefined));
        assert_eq!(got.is_removed, Some(false));
        assert_eq!(
            got.meeting_date,
            Some(AsezDate::try_from("2002-02-20").unwrap())
        );
        assert_eq!(got.created_by, Some(USER1));
        assert_eq!(got.changed_at, got.created_at);
        assert!(got.created_at.is_some());
    })
    .await
}

#[test]
async fn test_insert_agendas() {
    run_db_test(EcAgenda::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let new1: EcAgendaRep = NewEcAgendaRep {
            id: 1000004323,
            meeting_date: AsezDate::try_from("2002-02-20").unwrap(),
            created_by: USER1,
        }
        .into();
        let mut new2: EcAgendaRep = NewEcAgendaRep {
            id: 1000004324,
            meeting_date: AsezDate::try_from("2002-02-21").unwrap(),
            created_by: USER1,
        }
        .into();
        new2.uuid = Some(Uuid::new_v4());
        let mut reps = [new1.into_item().unwrap(), new2.into_item().unwrap()];

        let res = EcAgenda::insert_vec(&mut reps, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        let got = sqlx::query("SELECT * FROM agenda;")
            .map(|x| {
                EcAgendaRep::from_item::<&str>(
                    EcAgenda::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);

        assert!(got[0].uuid.is_some());
        assert_eq!(got[0].pricing_organization_unit_id, Some(Default::default()));
        assert_eq!(got[0].id, Some(1000004323));
        // FIXME: Должны ли мы тут ожидать этот статус?
        // assert_eq!(got[0].status_id, Some(EcAgendaStatus::Formed));
        assert_eq!(got[0].is_removed, Some(false));
        assert_eq!(
            got[0].meeting_date,
            Some(AsezDate::try_from("2002-02-20").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));
        assert_eq!(got[0].changed_at, got[0].created_at);
        assert!(got[0].created_at.is_some());

        assert!(got[1].uuid.is_some());
        assert_eq!(got[1].pricing_organization_unit_id, Some(Default::default()));
        assert_eq!(got[1].id, Some(1000004324));
        assert_eq!(
            got[1].meeting_date,
            Some(AsezDate::try_from("2002-02-21").unwrap())
        );
        assert_eq!(got[1].changed_at, got[1].created_at);
        assert!(got[1].created_at.is_some());
    })
    .await
}

#[test]
async fn test_update_agenda() {
    run_db_test(EcAgenda::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let new: EcAgendaRep = NewEcAgendaRep {
            id: 1000004323,
            meeting_date: AsezDate::try_from("2002-02-20").unwrap(),
            created_by: USER1,
        }
        .into();
        let mut agenda_rep = new.into_item().unwrap();

        let res = agenda_rep.insert(&mut pool).await.unwrap();
        assert_eq!(res, 1);

        // Check content after insert.

        let got = sqlx::query("SELECT * FROM agenda;")
            .map(|x| {
                EcAgendaRep::from_item::<&str>(
                    EcAgenda::from_row(&x).unwrap(),
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
            got[0].meeting_date,
            Some(AsezDate::try_from("2002-02-20").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));

        // Make update.
        let upd_agenda = EcAgendaRep {
            // We select what to update by uuid.
            uuid: got[0].uuid,
            pricing_organization_unit_id: Some(PricingUnitId::D646),
            meeting_date: Some(AsezDate::try_from("1066-10-14").unwrap()),
            changed_by: Some(USER2),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let update_fields =
            ["pricing_organization_unit_id", "meeting_date", "changed_by"];
        let res = upd_agenda.update(Some(&update_fields), &mut pool).await.unwrap();
        assert_eq!(res, 1);

        // Check content after update.
        let got = sqlx::query("SELECT * FROM agenda;")
            .map(|x| {
                EcAgendaRep::from_item::<&str>(
                    EcAgenda::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].pricing_organization_unit_id, Some(PricingUnitId::D646));
        assert_eq!(
            got[0].meeting_date,
            Some(AsezDate::try_from("1066-10-14").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));
    })
    .await
}

#[test]
// TODO: Надо узнать почему UUID автоматически не генерируются
async fn test_update_agendas() {
    run_db_test(EcAgenda::TABLE, CREATE_TABLE, None, |mut pool| async move {
        let mut new1: EcAgendaRep = NewEcAgendaRep {
            id: 1000004323,
            meeting_date: AsezDate::try_from("2002-02-20").unwrap(),
            created_by: USER1,
        }
        .into();
        new1.uuid = Some(Uuid::new_v4());
        let mut new2: EcAgendaRep = NewEcAgendaRep {
            id: 1000004324,
            meeting_date: AsezDate::try_from("2002-02-21").unwrap(),
            created_by: USER1,
        }
        .into();
        new2.uuid = Some(Uuid::new_v4());
        let mut reps = [new1.into_item().unwrap(), new2.into_item().unwrap()];

        let res = EcAgenda::insert_vec(&mut reps, &mut pool).await.unwrap();
        assert_eq!(res, 2);

        // Check content after insert.

        let got = sqlx::query("SELECT * FROM agenda;")
            .map(|x| {
                EcAgendaRep::from_item::<&str>(
                    EcAgenda::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();

        assert_eq!(got.len(), 2);
        assert_eq!(got[0].pricing_organization_unit_id, Some(Default::default()));
        assert_eq!(got[1].pricing_organization_unit_id, Some(Default::default()));
        assert_eq!(
            got[0].meeting_date,
            Some(AsezDate::try_from("2002-02-20").unwrap())
        );
        assert_eq!(
            got[1].meeting_date,
            Some(AsezDate::try_from("2002-02-21").unwrap())
        );

        // Make update.
        let upd_agenda1 = EcAgendaRep {
            // We select what to update by uuid.
            uuid: got[0].uuid,
            pricing_organization_unit_id: Some(PricingUnitId::Undefined),
            meeting_date: Some(AsezDate::try_from("1066-10-14").unwrap()),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let upd_agenda2 = EcAgendaRep {
            // We select what to update by uuid.
            uuid: got[1].uuid,
            pricing_organization_unit_id: Some(PricingUnitId::Gpk),
            meeting_date: Some(AsezDate::try_from("1066-10-13").unwrap()),
            ..Default::default()
        }
        .into_item()
        .unwrap();
        let to_update = vec![upd_agenda1, upd_agenda2];

        let update_columns = vec!["pricing_organization_unit_id", "meeting_date"];

        let res =
            EcAgenda::update_vec(&to_update, Some(&update_columns), &mut pool)
                .await
                .unwrap();
        assert_eq!(res, 2);

        // Check content after update.
        let got = sqlx::query("SELECT * FROM agenda;")
            .map(|x| {
                EcAgendaRep::from_item::<&str>(
                    EcAgenda::from_row(&x).unwrap(),
                    None,
                )
            })
            .fetch_all(&mut pool)
            .await
            .unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(
            got[0].meeting_date,
            Some(AsezDate::try_from("1066-10-14").unwrap())
        );
        assert_eq!(got[0].created_by, Some(USER1));
        // NB: We use char(4) in the DB, so we will always get 4 chars back.
        assert_eq!(got[1].pricing_organization_unit_id, Some(PricingUnitId::Gpk));
        assert_eq!(
            got[1].meeting_date,
            Some(AsezDate::try_from("1066-10-13").unwrap())
        );
        assert_eq!(got[1].created_by, Some(USER1));
    })
    .await
}
