use crate::app_process::export_data;
use crate::app_process::tests::{run_db_rabbit_test, USER1};
use asez2_shared_db::db_item::{Filter, Select};
use rabbit_services::specialized_departments::SpecializedDepartmentsService;
use shared_essential::domain::Section;
use shared_essential::presentation::dto::print_docs::common::TemplateFormat;
use shared_essential::presentation::dto::processing::ExportReq;
use shared_essential::presentation::dto::Source;

const GET_PLANS_EXTRA_MIGS: &[&str] = &["estimated_commission/get_plans.sql"];

const GET_AGENDA_LIST_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_agenda_list.sql"];

const GET_PROTOCOL_LIST_EXTRA_MIGS: &[&str] =
    &["estimated_commission/get_protocol_list.sql"];

#[tokio::test]
async fn test_export_data_plans() {
    run_db_rabbit_test(GET_PLANS_EXTRA_MIGS, |pool, rabbit| async move {
        let request = ExportReq {
            section_id: Section::EstimatedCommissionInPerson,
            format: Some(TemplateFormat::Xlsx),
            template: None, // TODO: Specify template name
            user_id: -1,
            select: Select::with_fields(["id", "uuid", "currency_id"]),
            captions: None,
            token: String::new(),
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );
        let _response =
            export_data(request, pool.clone(), sd_service).await.unwrap();
    })
    .await
}

#[tokio::test]
async fn export_data_agendas() {
    run_db_rabbit_test(GET_AGENDA_LIST_EXTRA_MIGS, |pool, rabbit| async move {
        let request = ExportReq {
            section_id: Section::EstimatedCommissionInPersonPreparation,
            format: Some(TemplateFormat::Xlsx),
            template: None, // TODO: Specify template name
            user_id: USER1,
            select: Select::with_fields(["uuid", "created_by", "changed_by"]),
            captions: None,
            token: String::new(),
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );
        let _response =
            export_data(request, pool.clone(), sd_service).await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn export_data_protocol_list_correspondence() {
    run_db_rabbit_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool, rabbit| async move {
        let request = ExportReq {
            section_id: Section::EstimatedCommissionSummingUpCorrespondence,
            format: Some(TemplateFormat::Xlsx),
            template: None, // TODO: Specify template name
            user_id: USER1,
            select: Select {
                field_list: vec![String::from("id"), String::from("protocol_date")],
                filter_list: Filter::in_any("id", [1, 2, 5, 6]).into(),
                ..Default::default()
            },
            captions: None,
            token: String::new(),
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );
        let _response =
            export_data(request, pool.clone(), sd_service).await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn export_data_protocol_list_in_person() {
    run_db_rabbit_test(GET_PROTOCOL_LIST_EXTRA_MIGS, |pool, rabbit| async move {
        let request = ExportReq {
            section_id: Section::EstimatedCommissionSummingUpInPerson,
            format: Some(TemplateFormat::Xlsx),
            template: None, // TODO: Specify template name
            user_id: USER1,
            select: Select {
                field_list: vec![String::from("id"), String::from("protocol_date")],
                filter_list: Filter::in_any("id", [1, 2, 5, 6]).into(),
                ..Default::default()
            },
            captions: None,
            token: String::new(),
        };

        let sd_service = SpecializedDepartmentsService::new(
            rabbit,
            Default::default(),
            Source::Processing,
        );
        let _response =
            export_data(request, pool.clone(), sd_service).await.unwrap();
    })
    .await;
}
