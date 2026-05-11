use crate::app_process::sections::table::EntityType;
use crate::app_process::{get_agenda_list, get_plans, get_protocol_list};
use crate::common::{ProcessingError, Result};
use ahash::AHashMap;
use asez2_shared_db::db_item::{DbAdaptorFieldsWithValues, Field, Select};
use asez2_shared_db::Value;
use itertools::Itertools;
use rabbit_services::specialized_departments::SpecializedDepartmentsService;
use shared_essential::domain::{
    PlanOrAmendmentRep, PricingUnitId, ProtocolType, Section,
};
use shared_essential::presentation::dto::general::{
    DataRecord, DataRecords, TaggedValue,
};
use shared_essential::presentation::dto::processing::{
    ExportReq, GetAgendaListReq, GetPlansCalculatedItem, GetProtocolListReq,
    PlansRequest,
};
use shared_essential::presentation::dto::response_request::{EntityKind, Messages};
use sqlx::PgPool;
use std::sync::Arc;

pub(crate) async fn process_export_table_request(
    request: &ExportReq,
    db_pool: Arc<PgPool>,
    sd_service: SpecializedDepartmentsService,
) -> Result<(DataRecords, Messages)> {
    match request.section_id {
        // обработка запросов из модуля estimated-commission
        Section::EstimatedCommissionProcurements
        | Section::EstimatedCommissionInPerson
        | Section::EstimatedCommissionCorrespondence
        | Section::EstimatedCommissionNotRequired => {
            process_get_plan_list(request, db_pool, sd_service).await
        }
        Section::EstimatedCommissionInPersonPreparation => {
            process_get_agenda_list(request, db_pool).await
        }
        Section::EstimatedCommissionSummingUpCorrespondence => {
            process_get_protocol_list(
                request,
                ProtocolType::CorrespondenceMeeting,
                db_pool,
            )
            .await
        }
        Section::EstimatedCommissionSummingUpInPerson => {
            process_get_protocol_list(
                request,
                ProtocolType::InPersonMeeting,
                db_pool,
            )
            .await
        }
        // обработка запросов из модуля pricing-analysis-service
        Section::PriceAnalysisAssignExpert
        | Section::PriceAnalysisDeterminePrice
        | Section::PriceAnalysisApprovePrice
        | Section::PriceAnalysisGgp
        | Section::PriceAnalysisPrimaryExpertControl
        | Section::PriceAnalysisLottingMTP
        | Section::PriceAnalysisReportingPrice
        | Section::PriceAnalysisConclusionTemplates
        | Section::PriceAnalysisOffersByRequest
        | Section::PriceAnalysisAutomaticAssignExpert => {
            process_get_plan_list(request, db_pool, sd_service).await
        }
        // обработка запросов из модуля specialized-departments
        Section::AssignExpertDepartment
        | Section::InWorkByExpertDepartment
        | Section::ProcurementsReviewedByDepartment
        | Section::AutoAssignDepartment => {
            process_get_plan_list(request, db_pool, sd_service).await
        }
        _ => Err(ProcessingError::Section(format!(
            "Секция {} не имеет возможности экспорта таблицы",
            request.section_id
        ))),
    }
}

/// 1. Экспорт списка в разделах
/// * Закупки ЕИ - procurements_for_commission,
/// * Очная СК - in_person_commission,
/// * Заочная СК - correspondence_commission,
/// * СК не требуется - no_commission_required
/// `/rest/estimated_commission/v1/get/plans`
async fn process_get_plan_list(
    request: &ExportReq,
    db_pool: Arc<PgPool>,
    sd_service: SpecializedDepartmentsService,
) -> Result<(DataRecords, Messages)> {
    let origin_fields = request.select.field_list.clone();
    let req = PlansRequest {
        section: request.section_id,
        select: Select {
            field_list: origin_fields.clone(),
            ..request.select.clone()
        },
        user_id: request.user_id,
    };

    let plans = get_plans(req, db_pool, sd_service).await?;

    let data = convert_plan_calculated_items_to_data_records(
        &plans.data.item_list,
        &origin_fields,
    );

    let entity_kind = plans
        .data
        .item_list
        .iter()
        .map(|item| match item.plan.item {
            PlanOrAmendmentRep::Plan(_) => EntityKind::Plan,
            PlanOrAmendmentRep::Amendment(_) => EntityKind::ContractAmendment,
        })
        .collect();

    Ok((
        DataRecords {
            captions: request.captions.clone().unwrap_or_default(),
            field_list: origin_fields,
            data,
            entity_kind,
        },
        plans.messages,
    ))
}

/// 2. Экспорт списка в разделе
/// * Проведение Очной СК - preparation_for_in_person_commission
/// `/rest/estimated_commission/v1/get/agenda_list/`
async fn process_get_agenda_list(
    request: &ExportReq,
    db_pool: Arc<PgPool>,
) -> Result<(DataRecords, Messages)> {
    let origin_fields = request.select.field_list.clone();

    let req = GetAgendaListReq {
        section_id: request.section_id,
        select: Select {
            field_list: origin_fields.clone(),
            ..request.select.clone()
        },
    };
    let result = get_agenda_list(req, db_pool).await?;

    let map = result
        .data
        .item_list
        .into_iter()
        .map(|item| {
            let mut calculated_data: AHashMap<String, Value> = AHashMap::new();
            calculated_data.insert(
                "agenda_item_quantity_threshold".to_string(),
                item.agenda_item_quantity_threshold.into(),
            );
            if item.agenda.pricing_organization_unit_id == Some(PricingUnitId::D647)
            {
                calculated_data.insert(
                    "agenda_item_d647_quantity_threshold".to_string(),
                    item.agenda_item_d647_quantity_threshold.into(),
                );
            }
            calculated_data.insert(
                "protocol_quantity".to_string(),
                item.protocol_quantity.into(),
            );
            (item.agenda, Some(calculated_data))
        })
        .collect_vec();

    let data = extract_data_records(map, &origin_fields.clone());
    Ok((
        DataRecords {
            captions: request.captions.clone().unwrap_or_default(),
            field_list: origin_fields,
            entity_kind: vec![EntityKind::Agenda; data.len()],
            data,
        },
        result.messages,
    ))
}

/// 3. Экспорт списка в разделах
/// * Подведение итогов очной СК - summing_up_in_person_commission_results,
/// * Подведение итогов заочной СК - summing_up_correspondence_commission_results
/// `/rest/estimated_commission/v1/get/protocol_list/`
async fn process_get_protocol_list(
    request: &ExportReq,
    protocol_type_id: ProtocolType,
    db_pool: Arc<PgPool>,
) -> Result<(DataRecords, Messages)> {
    let origin_fields = request.select.field_list.clone();

    let req = GetProtocolListReq {
        protocol_type_id,
        select: Select {
            field_list: origin_fields.clone(),
            ..request.select.clone()
        },
    };
    let result = get_protocol_list(req, db_pool).await?;

    let map = result
        .data
        .item_list
        .into_iter()
        .map(|item| {
            let mut calculated_data: AHashMap<String, Value> = AHashMap::new();
            calculated_data.insert(
                "protocol_item_quantity_threshold".to_string(),
                item.protocol_item_quantity_threshold.into(),
            );
            if item.protocol.pricing_organization_unit_id
                == Some(PricingUnitId::D647)
            {
                calculated_data.insert(
                    "protocol_item_d647_quantity_threshold".to_string(),
                    item.protocol_item_d647_quantity_threshold.into(),
                );
            }
            (item.protocol, Some(calculated_data))
        })
        .collect_vec();

    let data = extract_data_records(map, &origin_fields.clone());

    Ok((
        DataRecords {
            captions: request.captions.clone().unwrap_or_default(),
            field_list: origin_fields,
            entity_kind: vec![EntityKind::Protocol; data.len()],
            data,
        },
        result.messages,
    ))
}

type CalculatedData = Option<AHashMap<String, Value>>;

macro_rules! fill_field_indexes {
    ($fields_map:ident,  $item:ident, $entity_type:expr, $original_field_names:ident) => {
        $fields_map.entry($entity_type).or_insert_with(|| {
            build_field_index(&$item.fields_with_values(), $original_field_names)
        });
    };
}

/// Получение индексов полей для сущностей PlanRep, ContractAmendmentRep, EcAgendaRep, EcAgendaItemRep, EcProtocolRep, EcProtocolItemRep
fn get_field_indexes(
    items: &[GetPlansCalculatedItem],
    original_field_names: &[String],
) -> AHashMap<EntityType, AHashMap<String, usize>> {
    let mut fields_map: AHashMap<EntityType, AHashMap<String, usize>> =
        AHashMap::new();
    items.iter().for_each(|item| {
        match &item.plan.item {
            PlanOrAmendmentRep::Plan(plan) => {
                fill_field_indexes!(
                    fields_map,
                    plan,
                    EntityType::Plan,
                    original_field_names
                );
            }
            PlanOrAmendmentRep::Amendment(amendment) => {
                fill_field_indexes!(
                    fields_map,
                    amendment,
                    EntityType::ContractAmendment,
                    original_field_names
                );
            }
        }

        if let Some(agenda) = &item.agenda {
            fill_field_indexes!(
                fields_map,
                agenda,
                EntityType::Agenda,
                original_field_names
            );
        }
        if let Some(agenda_item) = &item.agenda_item {
            fill_field_indexes!(
                fields_map,
                agenda_item,
                EntityType::AgendaItem,
                original_field_names
            );
        }

        if let Some(protocol) = &item.protocol {
            fill_field_indexes!(
                fields_map,
                protocol,
                EntityType::Protocol,
                original_field_names
            );
        }
        if let Some(protocol_item_calculated) = &item.protocol_item {
            let protocol_item = &protocol_item_calculated.item;
            fill_field_indexes!(
                fields_map,
                protocol_item,
                EntityType::ProtocolItem,
                original_field_names
            );
        }
    });
    fields_map
}

fn convert_plan_calculated_items_to_data_records(
    items: &[GetPlansCalculatedItem],
    original_field_names: &[String],
) -> Vec<DataRecord> {
    let mut data_records: Vec<DataRecord> = Vec::with_capacity(items.len());
    let field_indexes = get_field_indexes(items, original_field_names);

    items.iter().for_each(|item| match &item.plan.item {
        PlanOrAmendmentRep::Plan(plan) => {
            if let Some(plan_or_amendment_field_indexes) =
                field_indexes.get(&EntityType::Plan)
            {
                data_records.push(fill_data_record(
                    original_field_names,
                    item,
                    plan,
                    plan_or_amendment_field_indexes,
                    &field_indexes,
                ));
            }
        }
        PlanOrAmendmentRep::Amendment(amendment) => {
            if let Some(plan_or_amendment_field_indexes) =
                field_indexes.get(&EntityType::ContractAmendment)
            {
                data_records.push(fill_data_record(
                    original_field_names,
                    item,
                    amendment,
                    plan_or_amendment_field_indexes,
                    &field_indexes,
                ));
            }
        }
    });
    data_records
}

fn fill_data_record<T: DbAdaptorFieldsWithValues>(
    original_field_names: &[String],
    export_item: &GetPlansCalculatedItem,
    plan_or_amendment: &T,
    plan_or_amendment_field_indexes: &AHashMap<String, usize>,
    other_field_indexes: &AHashMap<EntityType, AHashMap<String, usize>>,
) -> DataRecord {
    let record: DataRecord = original_field_names
        .iter()
        .map(|field_name| {
            if let Some(index) = plan_or_amendment_field_indexes.get(field_name) {
                plan_or_amendment
                    .fields_with_values()
                    .get(*index)
                    .and_then(|value| value.value.clone())
                    .into()
            } else if let (Some(index), Some(protocol)) = (
                other_field_indexes
                    .get(&EntityType::Protocol)
                    .and_then(|map| map.get(field_name)),
                &export_item.protocol,
            ) {
                protocol
                    .fields_with_values()
                    .get(*index)
                    .and_then(|value| value.value.clone())
                    .into()
            } else if let (Some(index), Some(protocol_item)) = (
                other_field_indexes
                    .get(&EntityType::ProtocolItem)
                    .and_then(|map| map.get(field_name)),
                &export_item.protocol_item,
            ) {
                protocol_item
                    .item
                    .fields_with_values()
                    .get(*index)
                    .and_then(|value| value.value.clone())
                    .into()
            } else if let Some(value) =
                &export_item.protocol_item.as_ref().and_then(|item| {
                    let value =
                        &item.get_calculated_values_map().get(field_name).cloned();
                    value.clone()
                })
            {
                (*value).clone().into()
            } else if let (Some(index), Some(agenda)) = (
                other_field_indexes
                    .get(&EntityType::Agenda)
                    .and_then(|map| map.get(field_name)),
                &export_item.agenda,
            ) {
                agenda
                    .fields_with_values()
                    .get(*index)
                    .and_then(|value| value.value.clone())
                    .into()
            } else if let (Some(index), Some(agenda_item)) = (
                other_field_indexes
                    .get(&EntityType::AgendaItem)
                    .and_then(|map| map.get(field_name)),
                &export_item.agenda_item,
            ) {
                agenda_item
                    .fields_with_values()
                    .get(*index)
                    .and_then(|value| value.value.clone())
                    .into()
            } else if let Some(value) =
                export_item.plan.get_calculated_values_map().get(field_name)
            {
                value.to_owned().into()
            } else {
                TaggedValue::Null
            }
        })
        .collect::<Vec<TaggedValue>>();
    record
}

fn extract_data_records<A>(
    items: Vec<(A, CalculatedData)>,
    field_names: &[String],
) -> Vec<DataRecord>
where
    A: DbAdaptorFieldsWithValues,
{
    let mut data: Vec<DataRecord> = Vec::with_capacity(items.len());

    let indexes = items
        .get(0)
        .map(|item| build_field_index(&item.0.fields_with_values(), field_names))
        .unwrap_or_default();

    for (item, calculated_values) in &items {
        let fields = item.fields_with_values();
        let record: DataRecord = field_names
            .iter()
            .map(|field_name| {
                if let Some(index) = indexes.get(field_name) {
                    fields.get(*index).and_then(|value| value.value.clone()).into()
                } else if let Some(value) = calculated_values.as_ref() {
                    value.get(field_name).cloned().into()
                } else {
                    TaggedValue::Null
                }
            })
            .collect::<Vec<TaggedValue>>();
        data.push(record);
    }
    data
}

fn build_field_index(
    fields: &[Field],
    field_names: &[String],
) -> AHashMap<String, usize> {
    field_names
        .iter()
        .filter_map(|name| {
            fields
                .iter()
                .position(|field| field.field() == name)
                .map(|index| (name.to_string(), index))
        })
        .collect()
}
