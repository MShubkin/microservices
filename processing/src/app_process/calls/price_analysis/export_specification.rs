use std::collections::HashMap;

use uuid::Uuid;

use asez2_shared_db::db_item::int_array::AsezArray;
use asez2_shared_db::db_item::Select;
use itertools::Itertools;
use shared_essential::{
    domain::{
        maths::*, ContractAmendment, ContractAmendmentItemRep, Plan,
        PlanItemFullRep,
    },
    presentation::dto::{
        general::{
            DataRecords, FloatPrecisionFormat, NullValueFormat, TaggedValue,
        },
        processing::{
            price_analysis::{ExportSpecificationField, ExportSpecificationReq},
            CompletePlansRequest, GetContractAmendmentDataRep, GetPlanDataRep,
        },
        response_request::{ApiResponse, EntityKind, Messages},
        AsezResult,
    },
};

use crate::app_process::external::planning_masterdata::{
    process_planning_multiple_request, MultipleRequest,
};
use crate::app_process::{get_complete_contract_amendments, get_complete_plans};
use crate::common::{ProcessingCtx, ProcessingError};

pub(crate) async fn export_specification(
    dto: ExportSpecificationReq,
    proc_ctx: ProcessingCtx,
) -> crate::common::Result<ApiResponse<DataRecords, ()>> {
    tracing::info!(
        kind = "get",
        "Получен запрос на экспорт спецификации: {req:?}\n",
        req = dto,
    );
    let mut response = ApiResponse::default();

    let item_uuid_vec =
        dto.item_id_list.iter().map(|value| value.uuid).collect_vec();

    match dto.object_identifier.object_type {
        EntityKind::Plan => {
            let (plan_data, messages) = get_plan_for_export(
                dto.object_identifier.uuid,
                &proc_ctx,
                &dto.field_configuration,
            )
            .await?;

            let item_list = plan_data
                .items
                .into_iter()
                .filter(|item| {
                    dto.item_id_list.is_empty()
                        || item_uuid_vec.contains(&item.uuid.unwrap_or_default())
                })
                .collect_vec();
            response.data = convert_ids_to_text(
                fill_export_records(
                    ExportItemsType::PlanItem(item_list),
                    &dto.field_configuration,
                )
                .await?,
                &dto,
            )
            .await?;
            response.messages = messages;
        }
        EntityKind::ContractAmendment => {
            let (contract_amendment_data, messages) =
                get_contract_amendment_for_export(
                    dto.object_identifier.uuid,
                    &proc_ctx,
                    &dto.field_configuration,
                )
                .await?;
            let item_list = contract_amendment_data
                .items
                .into_iter()
                .filter(|item| {
                    dto.item_id_list.is_empty()
                        || item_uuid_vec.contains(&item.uuid.unwrap_or_default())
                })
                .collect_vec();

            response.data = convert_ids_to_text(
                fill_export_records(
                    ExportItemsType::ContractAmendmentItem(item_list),
                    &dto.field_configuration,
                )
                .await?,
                &dto,
            )
            .await?;
            response.messages = messages;
        }
        other => {
            return Err(ProcessingError::Export(format!(
                "Для типа сущности {:?} экспорт не поддерживается",
                other
            )));
        }
    };
    Ok(response)
}

enum ExportItemsType {
    PlanItem(Vec<PlanItemFullRep>),
    ContractAmendmentItem(Vec<ContractAmendmentItemRep>),
}

#[derive(Clone, Debug, Default)]
struct ExportSpecificationItem {
    pub id: Option<i64>,
    pub description_internal: Option<String>,
    pub uuid: Option<Uuid>,
    pub quantity: Option<Quantity>,
    pub unit_id: Option<i16>,
    pub price: Option<CurrencyValue>,
    pub sum_excluded_vat: Option<CurrencyValue>,
    pub vat_id: Option<VatId>,
    pub sum_included_vat: Option<CurrencyValue>,
    pub currency_id: Option<i16>,
    pub pricing_quantity: Option<Quantity>,
    pub pricing_price: Option<CurrencyValue>,
    pub pricing_vat_id: Option<VatId>,
}

async fn fill_export_records(
    item_type: ExportItemsType,
    field_configuration: &[ExportSpecificationField],
) -> AsezResult<DataRecords> {
    let export_items: Vec<ExportSpecificationItem> = match item_type {
        ExportItemsType::PlanItem(items) => items
            .into_iter()
            .map(|item| ExportSpecificationItem {
                id: item.id,
                description_internal: item.description_internal.flatten(),
                uuid: item.uuid,
                quantity: item.quantity,
                unit_id: item.unit_id,
                price: item.price,
                sum_excluded_vat: item.sum_excluded_vat,
                vat_id: item.vat_id,
                sum_included_vat: item.sum_included_vat,
                currency_id: item.currency_id,
                pricing_quantity: item.pricing_quantity.flatten(),
                pricing_price: item.pricing_price.flatten(),
                pricing_vat_id: item.pricing_vat_id,
            })
            .collect(),
        ExportItemsType::ContractAmendmentItem(items) => items
            .into_iter()
            .map(|item| ExportSpecificationItem {
                id: item.id,
                description_internal: item.description_internal,
                uuid: item.uuid,
                quantity: item.quantity,
                unit_id: item.unit_id,
                price: item.price,
                sum_excluded_vat: item.sum_excluded_vat,
                vat_id: item.vat_id,
                sum_included_vat: item.sum_included_vat,
                currency_id: item.currency_id,
                pricing_quantity: item.pricing_quantity,
                pricing_price: item.pricing_price,
                pricing_vat_id: item.pricing_vat_id,
            })
            .collect(),
    };

    process_export_items(export_items, field_configuration).await
}

/// Заполнение структуры DataRecords
async fn process_export_items(
    mut item_list: Vec<ExportSpecificationItem>,
    field_configuration: &[ExportSpecificationField],
) -> AsezResult<DataRecords> {
    item_list.sort_by_key(|item| item.id);

    let mut records = DataRecords {
        captions: field_configuration
            .iter()
            .map(|value| value.header_name.clone())
            .collect(),
        field_list: field_configuration
            .iter()
            .map(|value| value.field_id.clone())
            .collect(),
        ..Default::default()
    };

    item_list.into_iter().enumerate().for_each(|item| {
        let mut data_record = Vec::with_capacity(records.field_list.len());

        for (index, item_field_id) in records.field_list.iter().enumerate() {
            let r = match item_field_id.as_str() {
                "number" => TaggedValue::Int((item.0 + 1) as i64),
                "id" => {
                    if let Some(id) = item.1.id {
                        TaggedValue::Int(id)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Int)
                    }
                }
                "description_internal" => {
                    if let Some(description_internal) = &item.1.description_internal
                    {
                        TaggedValue::String(description_internal.clone())
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::String)
                    }
                }
                "uuid" => {
                    if let Some(uuid) = item.1.uuid {
                        TaggedValue::Uuid(uuid)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Uuid)
                    }
                }
                "quantity" => {
                    if let Some(quantity) = item.1.quantity {
                        TaggedValue::Quantity(quantity)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Float(
                            FloatPrecisionFormat::Three,
                        ))
                    }
                }
                "unit_id" => {
                    if let Some(unit_id) = item.1.unit_id {
                        TaggedValue::Int(unit_id as i64)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Int)
                    }
                }
                "price" => {
                    if let Some(price) = item.1.price {
                        TaggedValue::CValue(price)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Float(
                            FloatPrecisionFormat::Double,
                        ))
                    }
                }
                "sum_excluded_vat" => {
                    if let Some(sum_excluded_vat) = item.1.sum_excluded_vat {
                        TaggedValue::CValue(sum_excluded_vat)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Float(
                            FloatPrecisionFormat::Double,
                        ))
                    }
                }
                "vat_id" => {
                    if let Some(vat_id) = item.1.vat_id {
                        TaggedValue::Int(vat_id as i64)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Int)
                    }
                }
                "sum_included_vat" => {
                    if let Some(sum_included_vat) = item.1.sum_included_vat {
                        TaggedValue::CValue(sum_included_vat)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Float(
                            FloatPrecisionFormat::Double,
                        ))
                    }
                }
                "currency_id" => {
                    if let Some(currency_id) = item.1.currency_id {
                        TaggedValue::Int(currency_id as i64)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Int)
                    }
                }
                "pricing_quantity" => {
                    if let Some(pricing_quantity) = item.1.pricing_quantity {
                        TaggedValue::Quantity(pricing_quantity)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Float(
                            FloatPrecisionFormat::Three,
                        ))
                    }
                }
                "pricing_price" => {
                    if let Some(pricing_price) = item.1.pricing_price {
                        TaggedValue::CValue(pricing_price)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Float(
                            FloatPrecisionFormat::Double,
                        ))
                    }
                }
                "pricing_vat_id" => {
                    if let Some(pricing_vat_id) = item.1.pricing_vat_id {
                        TaggedValue::Int(pricing_vat_id as i64)
                    } else {
                        TaggedValue::NullWithFormat(NullValueFormat::Int)
                    }
                }
                _ => continue,
            };
            data_record.insert(index, r);
        }
        records.data.push(data_record);
    });
    Ok(records)
}

/// Замена id-шников на текстовые значения
async fn convert_ids_to_text(
    mut records: DataRecords,
    dto: &ExportSpecificationReq,
) -> Result<DataRecords, ProcessingError> {
    // Запрос в монолит на выборку справочников
    let request = create_multiple_request(&records).await?;
    // Справочники от монолита
    let response =
        process_planning_multiple_request(request, dto.user_id, dto.token.as_str())
            .await?;

    // Ставки НДС
    let vat_map: HashMap<u8, String> = response
        .vats
        .iter()
        .filter_map(|value| {
            if value.text.is_empty() || value.text == "Составной" {
                None
            } else {
                Some((value.id, value.text.clone()))
            }
        })
        .collect();
    // Валюты
    let currency_map: HashMap<u16, String> = response
        .currencies
        .iter()
        .map(|value| (value.id, value.text.clone()))
        .collect();
    // Единицы измерения
    let unit_map: HashMap<u16, String> = response
        .units
        .iter()
        .map(|value| (value.id, value.text.clone()))
        .collect();

    for record in records.data.iter_mut() {
        for (index, field_id) in records.field_list.iter().enumerate() {
            if let Some(value) = record.get_mut(index) {
                match field_id.as_str() {
                    "vat_id" => {
                        if let TaggedValue::Int(id) = value {
                            let text = vat_map
                                .get(&(*id as u8))
                                .unwrap_or(&"".to_owned())
                                .clone();
                            *value = TaggedValue::String(text);
                        }
                    }
                    "pricing_vat_id" => {
                        if let TaggedValue::Int(id) = value {
                            let text = vat_map
                                .get(&(*id as u8))
                                .unwrap_or(&"".to_owned())
                                .clone();
                            *value = TaggedValue::RangeString((
                                text,
                                AsezArray(
                                    vat_map
                                        .iter()
                                        .map(|value| value.1.clone())
                                        .sorted()
                                        .collect_vec(),
                                ),
                            ));
                        } else {
                            *value = TaggedValue::NullWithFormat(
                                NullValueFormat::RangeString(
                                    vat_map
                                        .iter()
                                        .map(|value| value.1.clone())
                                        .sorted()
                                        .collect_vec(),
                                ),
                            );
                        }
                    }
                    "unit_id" => {
                        if let TaggedValue::Int(id) = value {
                            let text = unit_map
                                .get(&(*id as u16))
                                .unwrap_or(&"".to_owned())
                                .clone();
                            *value = TaggedValue::String(text);
                        }
                    }
                    "currency_id" => {
                        if let TaggedValue::Int(id) = value {
                            let text = currency_map
                                .get(&(*id as u16))
                                .unwrap_or(&"".to_owned())
                                .clone();
                            *value = TaggedValue::String(text);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(records)
}

/// Создание запроса в монолит на выборку справочников
async fn create_multiple_request(
    records: &DataRecords,
) -> AsezResult<MultipleRequest> {
    let mut request = MultipleRequest::default();

    //Ставки НДС загружаем все
    request.vat_ids.append(&mut (1..20u8).collect_vec());

    for record in records.data.iter() {
        for (index, field_id) in records.field_list.iter().enumerate() {
            if let Some(value) = record.get(index) {
                match field_id.as_str() {
                    "unit_id" => {
                        if let TaggedValue::Int(id) = value {
                            request.unit_ids.push(*id as u16);
                        }
                    }
                    "currency_id" => {
                        if let TaggedValue::Int(id) = value {
                            request.currency_ids.push(*id as u16);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    request.currency_ids =
        request.currency_ids.into_iter().unique().collect::<Vec<_>>();
    request.unit_ids = request.unit_ids.into_iter().unique().collect::<Vec<_>>();

    Ok(request)
}

async fn get_plan_for_export(
    plan_uuid: Uuid,
    proc_ctx: &ProcessingCtx,
    field_configuration: &[ExportSpecificationField],
) -> AsezResult<(GetPlanDataRep, Messages)> {
    let plan_request = CompletePlansRequest {
        section: Default::default(),
        select: Select::with_fields(["id", "uuid"])
            .eq(Plan::uuid, plan_uuid)
            .take_first(),
        item_fields: get_selected_field_ids(field_configuration),
        user_id: Default::default(),
    };

    let response =
        get_complete_plans(plan_request, proc_ctx.db_pool.clone()).await?;

    let data = response
        .data
        .item_list
        .get(0)
        .ok_or(ProcessingError::GetItemList(plan_uuid.to_string()))?
        .clone();

    Ok((data, response.messages))
}

async fn get_contract_amendment_for_export(
    contract_amendment_uuid: Uuid,
    proc_ctx: &ProcessingCtx,
    field_configuration: &[ExportSpecificationField],
) -> AsezResult<(GetContractAmendmentDataRep, Messages)> {
    let plan_request = CompletePlansRequest {
        section: Default::default(),
        select: Select::with_fields(["id", "uuid"])
            .eq(ContractAmendment::uuid, contract_amendment_uuid)
            .take_first(),
        item_fields: get_selected_field_ids(field_configuration),
        user_id: Default::default(),
    };

    let response =
        get_complete_contract_amendments(plan_request, proc_ctx.db_pool.clone())
            .await?;
    let data = response
        .data
        .item_list
        .get(0)
        .ok_or(ProcessingError::GetItemList(contract_amendment_uuid.to_string()))?
        .clone();

    Ok((data, response.messages))
}

fn get_selected_field_ids(
    field_configuration: &[ExportSpecificationField],
) -> Vec<String> {
    field_configuration
        .iter()
        .filter_map(|value| {
            if value.is_select_field {
                Some(value.field_id.to_string())
            } else {
                None
            }
        })
        .collect()
}
