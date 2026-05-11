//! Тестирование процесса `get_agenda_list`
//!
//! Вводные данные считаются невалидными, если не подходят
//! под процесс

use asez2_shared_db::{db_item::AsezDate, uuid};
use shared_essential::{
    domain::{EcProtocolRep, EcProtocolStatus, PricingUnitId},
    presentation::dto::{
        processing::GetProtocolListByAgendaReq, response_request::EntityKind,
    },
};

use super::run_db_test;
use crate::app_process::get_protocol_list_by_agenda;
use crate::common::ProcessingError;

const GET_PROTOCOL_LIST_BY_AGENDA_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_protocol_list_by_agenda.sql"];

/// Тестирование кейса, когда не было найдено повесток
#[tokio::test]
async fn not_found_agenda() {
    run_db_test(GET_PROTOCOL_LIST_BY_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListByAgendaReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000000"),
            object_type: EntityKind::Agenda,
        };

        let err = get_protocol_list_by_agenda(dto, pool.clone()).await.unwrap_err();

        assert_eq!(
            ProcessingError::GetProtocolListByAgenda(
                "Повестка СК № 0 не найдена".into()
            )
            .to_string(),
            err.to_string()
        );
    })
    .await;
}

/// Тестирование кейса с получением Протоколов СК
#[tokio::test]
async fn get_protocol_list_by_agenda_success() {
    run_db_test(GET_PROTOCOL_LIST_BY_AGENDA_EXTRA_MIGS, |pool| async move {
        let dto = GetProtocolListByAgendaReq {
            id: 1,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            object_type: EntityKind::Agenda,
        };

        let response =
            get_protocol_list_by_agenda(dto, pool.clone()).await.unwrap();

        assert!(response.messages.is_empty());

        let expected_protocols = vec![
            EcProtocolRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                protocol_id: Some(1),
                registration_number: Some(Some(String::from("111"))),
                protocol_date: Some(AsezDate::try_from("1911-11-11").unwrap()),
                protocol_status_id: Some(EcProtocolStatus::Formed),
                pricing_organization_unit_id: Some(PricingUnitId::D646),
                ..Default::default()
            },
            EcProtocolRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000002")),
                protocol_id: Some(2),
                registration_number: Some(Some(String::from("222"))),
                protocol_date: Some(AsezDate::try_from("1911-11-12").unwrap()),
                protocol_status_id: Some(EcProtocolStatus::AgreementPending),
                pricing_organization_unit_id: Some(PricingUnitId::D646),
                ..Default::default()
            },
            EcProtocolRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000003")),
                protocol_id: Some(3),
                registration_number: Some(Some(String::from("333"))),
                protocol_date: Some(AsezDate::try_from("1911-11-13").unwrap()),
                protocol_status_id: Some(EcProtocolStatus::SignaturePending),
                pricing_organization_unit_id: Some(PricingUnitId::D646),
                ..Default::default()
            },
            EcProtocolRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000004")),
                protocol_id: Some(4),
                registration_number: Some(Some(String::from("444"))),
                protocol_date: Some(AsezDate::try_from("1911-11-14").unwrap()),
                protocol_status_id: Some(EcProtocolStatus::Confirmed),
                pricing_organization_unit_id: Some(PricingUnitId::D646),
                ..Default::default()
            },
        ];
        assert_eq!(response.data.item_list, expected_protocols);

        assert_eq!(response.data.id, 1);
        assert_eq!(
            response.data.commission_date,
            AsezDate::try_from("2000-01-01").unwrap()
        );
    })
    .await;
}
