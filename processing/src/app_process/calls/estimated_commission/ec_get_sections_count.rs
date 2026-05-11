//! This is the module where the business logic lives.
//! Currently there is no real business logic so everything is in the mod.rs file.
use std::sync::Arc;

use crate::app_process::estimated_commission::get_agenda_list::get_agenda_list_inner;
use crate::app_process::estimated_commission::get_protocol_list::get_protocol_list_inner;
use crate::app_process::sections::process_sections;

use ahash::AHashMap;
use asez2_shared_db::db_item::{Filter, FilterTree, Select};
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use shared_essential::presentation::dto::processing::PlansRequest;
use shared_essential::{
    domain::{EcAgendaStatus, EcProtocolStatus, ProtocolType, Section},
    presentation::dto::{
        processing::{
            GetAgendaListReq, GetProtocolListReq, GetSectionsCountRequest,
            GetSectionsCountResponse, UserIdWrapper,
        },
        response_request::*,
    },
};
use sqlx::PgPool;

use crate::common::{ProcessingError, Result};

const SECTIONS_GET_COUNT: &str = "/v1/sections/get/count";
const IN_PERSON: &[&str] = &[
    "plan_id",
    "customer_id",
    "supplier_id",
    "contract_subject_short",
    "currency_id",
    "sum_excluded_vat",
    "pricing_expert_id",
    "pricing_resume_short",
    "status_id",
    "commission_date",
    "section_id",
    "single_supplier_reason_id",
    "number_customer",
    "number_cgg",
];
const CORRESPONDENCE: &[&str] = &[
    "plan_id",
    "customer_id",
    "supplier_id",
    "contract_subject_short",
    "currency_id",
    "sum_excluded_vat",
    "pricing_expert_id",
    "pricing_resume_short",
    "status_id",
    "pricing_organization_unit_id",
    "section_id",
    "single_supplier_reason_id",
    "number_customer",
    "number_cgg",
];
const NOT_REQUIRED: &[&str] = &[
    "plan_id",
    "customer_id",
    "supplier_id",
    "contract_subject_short",
    "currency_id",
    "sum_excluded_vat",
    "pricing_expert_id",
    "pricing_resume_short",
    "status_id",
    "pricing_organization_unit_id",
    "section_id",
    "single_supplier_reason_id",
    "number_customer",
    "number_cgg",
];
const IN_PERSON_PREPARATION: &[&str] = &[
    "agenda_id",
    "agenda_item_quantity_threshold",
    "agenda_item_d647_quantity_threshold",
    "meeting_date",
    "pricing_organization_unit_id",
    "agenda_status_id",
    "protocol_quantity",
    "created_by",
];
const SUMMING_UP_IN_PERSON: &[&str] = &[
    "protocol_id",
    "registration_number",
    "protocol_item_quantity_threshold",
    "protocol_item_d647_quantity_threshold",
    "protocol_date",
    "is_secret",
    "pricing_organization_unit_id",
    "protocol_status_id",
    "created_by",
];
const SUMMING_UP_CORRESPONDENCE: &[&str] = &[
    "protocol_id",
    "registration_number",
    "protocol_item_quantity_threshold",
    "protocol_date",
    "is_secret",
    "pricing_organization_unit_id",
    "protocol_status_id",
    "created_by",
];

#[tracing::instrument(skip_all)]
pub(crate) async fn ec_get_sections_count(
    req: UserIdWrapper<GetSectionsCountRequest>,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetSectionsCountResponse, ()>> {
    tracing::info!(
        kind = "get",
        "Processing: Got request to send to plans on ({get}): {req:?}\n",
        req = req,
        get = SECTIONS_GET_COUNT
    );

    let GetSectionsCountRequest { section_list } = req.dto;
    let mut map: AHashMap<Section, usize> = AHashMap::new();
    let mut futures = FuturesUnordered::new();

    for section in section_list {
        let select = create_select(section);
        let Some(select) = select else { continue };
        let db_pool = std::sync::Arc::clone(&db_pool);
        futures.push(match section {
            Section::EstimatedCommissionInPerson
            | Section::EstimatedCommissionCorrespondence
            | Section::EstimatedCommissionNotRequired => async move {
                let req = PlansRequest {
                    section,
                    select,
                    user_id: req.user_id,
                };
                let res = process_sections(req, &db_pool).await?;
                Ok::<(Section, usize), ProcessingError>((section, res.data.len()))
            }
            .boxed(),
            Section::EstimatedCommissionInPersonPreparation => async move {
                let req = GetAgendaListReq {
                    section_id: section,
                    select,
                };
                let res = get_agenda_list_inner(req, &db_pool).await?;
                Ok::<(Section, usize), ProcessingError>((section, res.len()))
            }
            .boxed(),
            Section::EstimatedCommissionSummingUpInPerson => async move {
                let req = GetProtocolListReq {
                    protocol_type_id: ProtocolType::InPersonMeeting,
                    select,
                };
                let res = get_protocol_list_inner(req, &db_pool).await?;
                Ok::<(Section, usize), ProcessingError>((section, res.len()))
            }
            .boxed(),
            Section::EstimatedCommissionSummingUpCorrespondence => async move {
                let req = GetProtocolListReq {
                    protocol_type_id: ProtocolType::CorrespondenceMeeting,
                    select,
                };
                let res = get_protocol_list_inner(req, &db_pool).await?;
                Ok::<(Section, usize), ProcessingError>((section, res.len()))
            }
            .boxed(),
            _ => continue,
        });
    }

    while let Some(result) = futures.next().await {
        let (section, total) = result?;
        map.insert(section, total);
    }

    let data = GetSectionsCountResponse {
        in_person_commission: map
            .get(&Section::EstimatedCommissionInPerson)
            .copied(),
        correspondence_commission: map
            .get(&Section::EstimatedCommissionCorrespondence)
            .copied(),
        no_commission_required: map
            .get(&Section::EstimatedCommissionNotRequired)
            .copied(),
        preparation_for_in_person_commission: map
            .get(&Section::EstimatedCommissionInPersonPreparation)
            .copied(),
        summing_up_in_person_commission_results: map
            .get(&Section::EstimatedCommissionSummingUpInPerson)
            .copied(),
        summing_up_correspondence_commission_results: map
            .get(&Section::EstimatedCommissionSummingUpCorrespondence)
            .copied(),
    };

    Ok((data, vec![]).into())
}

fn create_select(section: Section) -> Option<Select> {
    let fields = match section {
        Section::EstimatedCommissionInPerson => IN_PERSON,
        Section::EstimatedCommissionCorrespondence => CORRESPONDENCE,
        Section::EstimatedCommissionNotRequired => NOT_REQUIRED,
        Section::EstimatedCommissionInPersonPreparation => IN_PERSON_PREPARATION,
        Section::EstimatedCommissionSummingUpInPerson => SUMMING_UP_IN_PERSON,
        Section::EstimatedCommissionSummingUpCorrespondence => {
            SUMMING_UP_CORRESPONDENCE
        }
        _ => return None,
    };
    let fields = fields.iter().map(ToString::to_string).collect();

    let filter = match section {
        Section::EstimatedCommissionInPerson
        | Section::EstimatedCommissionCorrespondence
        | Section::EstimatedCommissionNotRequired => FilterTree::None,
        Section::EstimatedCommissionInPersonPreparation => {
            FilterTree::Filter(Filter::in_any(
                "status_id",
                [EcAgendaStatus::Formed, EcAgendaStatus::Sent],
            ))
        }
        Section::EstimatedCommissionSummingUpInPerson
        | Section::EstimatedCommissionSummingUpCorrespondence => {
            FilterTree::Filter(Filter::in_any(
                "status_id",
                [
                    EcProtocolStatus::Formed,
                    EcProtocolStatus::AgreementPending,
                    EcProtocolStatus::SignaturePending,
                ],
            ))
        }
        _ => return None,
    };

    Some(Select {
        field_list: fields,
        filter_list: filter,
        ..Default::default()
    })
}
