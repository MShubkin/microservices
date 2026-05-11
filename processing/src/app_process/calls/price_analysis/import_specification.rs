use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use itertools::Itertools;
use uuid::Uuid;

use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbAdaptor;
use shared_essential::domain::plan_amendment::PlanOrAmendmentItemsRep;
use shared_essential::domain::{
    maths::*, ContractAmendment, ContractAmendmentItemRep, ContractAmendmentRep,
    Plan, PlanItemFullRep, PlanRep,
};
use shared_essential::presentation::dto::{
    general::{DataRecords, TaggedValue},
    processing::{
        price_analysis::{
            ImportReq, ImportSpecificationResponseData, UpdateContractAmendmentReq,
            UpdatePlanReq,
        },
        CompletePlansRequest, GetContractAmendmentDataRep, GetPlanDataRep,
        UserIdWrapper,
    },
    response_request::{ApiResponse, EntityKind, Messages},
};

use crate::app_process::external::planning_masterdata::{
    process_planning_multiple_request, MultipleRequest,
};
use crate::app_process::price_analysis::update_plan::{
    UpdatePlanCAError, CONTRACT_AMENDMENT_DTO_FIELDS, PLAN_DTO_FIELDS,
};
use crate::app_process::{
    get_complete_contract_amendments, get_complete_plans,
    pa_update_contract_amendment, pa_update_plan,
};
use crate::common::{ProcessingCtx, ProcessingError, Result};

pub(crate) async fn import_specification(
    mut request: ImportReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ImportSpecificationResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Получен запрос на импорт спецификации: {req:?}\n",
        req = request,
    );

    let mut response = ApiResponse::default();

    match request.object_identifier.object_type {
        EntityKind::Plan => {
            response.messages = update_plan(&mut request, proc_ctx.clone()).await?;
        }
        EntityKind::ContractAmendment => {
            response.messages =
                update_contract_amendment(&mut request, proc_ctx.clone()).await?;
        }
        _ => {
            return Err(ProcessingError::Import(format!(
                "Тип сущности {:?} не поддерживается",
                request.object_identifier.object_type
            )));
        }
    }
    let response_data = create_import_response(&request, &proc_ctx).await?;

    response.data = response_data.0;
    response.messages.add_messages(response_data.1);

    Ok(response)
}

/// Обновление позиций ППЗ
async fn update_plan(
    request: &mut ImportReq,
    proc_ctx: ProcessingCtx,
) -> Result<Messages> {
    convert_text_to_ids(request).await?;

    let plan_dto_fields = PLAN_DTO_FIELDS
        .iter()
        .chain(std::iter::once(&Plan::sum_excluded_vat))
        .collect_vec();

    let plan = PlanRep::select_maybe(
        &Select::with_fields(plan_dto_fields)
            .eq(Plan::uuid, request.object_identifier.uuid),
        &*proc_ctx.db_pool,
    )
    .await?
    .ok_or(UpdatePlanCAError::NotFound(request.object_identifier.id))?;

    let update_plan_request = UpdatePlanReq {
        plan,
        item_list: get_plan_items(&request.data_records)?,
        ..Default::default()
    };
    let user_id_wrapper = UserIdWrapper {
        user_id: request.user_id,
        dto: update_plan_request,
    };
    let messages = pa_update_plan(user_id_wrapper, proc_ctx).await?.messages;

    Ok(messages)
}

/// Обновление позиций ДС
async fn update_contract_amendment(
    request: &mut ImportReq,
    proc_ctx: ProcessingCtx,
) -> Result<Messages> {
    convert_text_to_ids(request).await?;

    let contract_dto_fields = CONTRACT_AMENDMENT_DTO_FIELDS
        .iter()
        .chain(std::iter::once(&ContractAmendment::sum_excluded_vat))
        .collect_vec();

    let contract_amendment = ContractAmendmentRep::select_maybe(
        &Select::with_fields(contract_dto_fields)
            .eq(ContractAmendment::uuid, request.object_identifier.uuid),
        &*proc_ctx.db_pool,
    )
    .await?
    .ok_or(UpdatePlanCAError::NotFound(request.object_identifier.id))?;

    let update_contract_amendment_request = UpdateContractAmendmentReq {
        contract_amendment,
        item_list: get_contract_amendment_items(&request.data_records)?,
        ..Default::default()
    };
    let user_id_wrapper = UserIdWrapper {
        user_id: request.user_id,
        dto: update_contract_amendment_request,
    };

    let messages =
        pa_update_contract_amendment(user_id_wrapper, proc_ctx).await?.messages;

    Ok(messages)
}

#[derive(Clone, Debug, Default)]
struct ImportSpecificationItem {
    pub id: Option<i64>,
    pub uuid: Option<Uuid>,
    pub pricing_quantity: Option<Quantity>,
    pub pricing_price: Option<CurrencyValue>,
    pub pricing_vat_id: Option<VatId>,
}

fn convert_data_record_to_import_items(
    data_records: &DataRecords,
) -> Result<Vec<ImportSpecificationItem>> {
    let mut items = Vec::with_capacity(data_records.data.len());

    for data_record in data_records.data.iter() {
        let mut item = ImportSpecificationItem::default();

        for (field_index, field_id) in data_records.field_list.iter().enumerate() {
            let value = data_record.get(field_index).unwrap_or(&TaggedValue::Null);

            match (field_id.as_str(), value) {
                ("id", TaggedValue::Int(x)) => item.id = Some(*x),
                ("id", value) => {
                    let v = value.get_string_value();
                    item.id = Some(parse::<i64>(&v, field_id)?);
                }
                ("uuid", TaggedValue::Uuid(x)) => item.uuid = Some(*x),
                ("uuid", value) => {
                    let v = value.get_string_value();
                    item.uuid = Some(Uuid::parse_str(v.as_str()).map_err(|e| {
                        make_field_error(field_id.as_str(), e.to_string())
                    })?);
                }
                ("pricing_quantity", TaggedValue::Quantity(x)) => {
                    item.pricing_quantity = Some(*x)
                }
                ("pricing_quantity", value) => {
                    let v = value.get_string_value();
                    let pricing_quantity = parse::<f64>(&v, field_id)?;
                    let pricing_quantity = Quantity::from_f64(pricing_quantity)?;
                    item.pricing_quantity = Some(pricing_quantity);
                }
                ("pricing_price", TaggedValue::CValue(x)) => {
                    item.pricing_price = Some(*x)
                }
                ("pricing_price", value) => {
                    let v = value.get_string_value();
                    let pricing_price = parse::<f64>(&v, field_id)?;
                    let pricing_price = CurrencyValue::from_f64(pricing_price)?;
                    item.pricing_price = Some(pricing_price);
                }
                ("pricing_vat_id", value) => {
                    let v = value.get_string_value();
                    let pricing_vat_id = parse::<VatId>(&v, field_id)?;
                    item.pricing_vat_id = Some(pricing_vat_id);
                }
                _ => {}
            }
        }
        items.push(item);
    }
    Ok(items)
}

fn parse<V: FromStr>(v: &str, field: &str) -> Result<V>
where
    <V as FromStr>::Err: Display,
{
    v.parse::<V>().map_err(|e| make_field_error(field, e.to_string()))
}

fn make_field_error(field: &str, error: String) -> ProcessingError {
    ProcessingError::Import(format!("Ошибка обработки поля {}: {}", field, error))
}

fn get_plan_items(data_records: &DataRecords) -> Result<Vec<PlanItemFullRep>> {
    let import_items = convert_data_record_to_import_items(data_records)?;
    let plan_items = import_items
        .iter()
        .map(|item| PlanItemFullRep {
            uuid: item.uuid,
            id: item.id,
            pricing_quantity: Some(item.pricing_quantity),
            pricing_price: Some(item.pricing_price),
            pricing_vat_id: item.pricing_vat_id,
            ..Default::default()
        })
        .collect_vec();

    Ok(plan_items)
}

fn get_contract_amendment_items(
    data_records: &DataRecords,
) -> Result<Vec<ContractAmendmentItemRep>> {
    let import_items = convert_data_record_to_import_items(data_records)?;
    let plan_items = import_items
        .iter()
        .map(|item| ContractAmendmentItemRep {
            uuid: item.uuid,
            id: item.id,
            pricing_quantity: item.pricing_quantity,
            pricing_price: item.pricing_price,
            pricing_vat_id: item.pricing_vat_id,
            ..Default::default()
        })
        .collect_vec();
    Ok(plan_items)
}

/// Замена текстовых значений на id
/// Выполняется только для полей, подлежащих обновлению
async fn convert_text_to_ids(dto: &mut ImportReq) -> Result<()> {
    // Запрос в монолит на выборку справочников
    let request = create_multiple_request().await?;
    // Справочники от монолита
    let response =
        process_planning_multiple_request(request, dto.user_id, dto.token.as_str())
            .await?;
    // Ставки НДС
    let vat_map: HashMap<String, u8> = response
        .vats
        .iter()
        .filter_map(|value| {
            if value.text.is_empty() {
                None
            } else {
                Some((value.text.clone(), value.id))
            }
        })
        .collect();
    for record in dto.data_records.data.iter_mut() {
        for (index, field_id) in dto.data_records.field_list.iter().enumerate() {
            if let Some(value) = record.get_mut(index) {
                if field_id.as_str() == "pricing_vat_id" {
                    if let TaggedValue::String(vat_text) = value {
                        let id = vat_map.get(vat_text).ok_or(
                            ProcessingError::Export(format!(
                                "Некорректное значение ставки НДС {:?}",
                                value
                            )),
                        )?;
                        *value = TaggedValue::Int(*id as i64);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Создание запроса в монолит на выборку справочников
/// Извлекаем только Ставки НДС
async fn create_multiple_request() -> Result<MultipleRequest> {
    Ok(MultipleRequest {
        vat_ids: (1..20u8).collect_vec(),
        ..Default::default()
    })
}

/// Создание ответа по результатам импорта
async fn create_import_response(
    request: &ImportReq,
    proc_ctx: &ProcessingCtx,
) -> Result<(ImportSpecificationResponseData, Messages)> {
    match request.object_identifier.object_type {
        EntityKind::Plan => {
            let data =
                get_plan_data(request.object_identifier.uuid, proc_ctx).await?;
            Ok((
                ImportSpecificationResponseData {
                    item_list: PlanOrAmendmentItemsRep::PlanItems(data.0.items),
                },
                data.1,
            ))
        }
        EntityKind::ContractAmendment => {
            let data = get_contract_amendment_data(
                request.object_identifier.uuid,
                proc_ctx,
            )
            .await?;
            Ok((
                ImportSpecificationResponseData {
                    item_list: PlanOrAmendmentItemsRep::ContractAmendmentItems(
                        data.0.items,
                    ),
                },
                data.1,
            ))
        }
        _ => Err(ProcessingError::Export(format!(
            "Тип сущности {:?} не поддерживается",
            request.object_identifier.object_type
        ))),
    }
}

/// Получение позиций ППЗ по результатам импорта
async fn get_plan_data(
    plan_uuid: Uuid,
    proc_ctx: &ProcessingCtx,
) -> Result<(GetPlanDataRep, Messages)> {
    let plan_request = CompletePlansRequest {
        section: Default::default(),
        select: Select::with_fields(["id", "uuid"])
            .eq(Plan::uuid, plan_uuid)
            .take_first(),
        item_fields: RESPONSE_FIELDS
            .iter()
            .map(|value| (*value).to_owned())
            .collect_vec(),
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

/// Получение позиций ДС по результатам импорта
async fn get_contract_amendment_data(
    contract_amendment_uuid: Uuid,
    proc_ctx: &ProcessingCtx,
) -> Result<(GetContractAmendmentDataRep, Messages)> {
    let plan_request = CompletePlansRequest {
        section: Default::default(),
        select: Select::with_fields(["id", "uuid"])
            .eq(ContractAmendment::uuid, contract_amendment_uuid)
            .take_first(),
        item_fields: RESPONSE_FIELDS
            .iter()
            .map(|value| (*value).to_owned())
            .collect_vec(),
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

/// Список полей, отправляемый на фронт по результатам импорта
const RESPONSE_FIELDS: &[&str] = &[
    "id",
    "uuid",
    "pricing_quantity",
    "pricing_price",
    "pricing_vat_id",
    "pricing_sum_excluded_vat",
    "pricing_sum_included_vat",
];
