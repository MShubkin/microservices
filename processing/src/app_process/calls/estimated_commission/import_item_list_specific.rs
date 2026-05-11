use asez2_shared_db::db_item::{AsezDate, AsezTimestamp};

use crate::app_process::estimated_commission::get_protocol_details;
use crate::app_process::estimated_commission::get_protocol_details::get_protocol;
use crate::app_process::estimated_commission::get_protocol_items_by_id_range::get_protocol_items_by_id_range_inner;
use crate::app_process::{get_agenda_details, get_agenda_items_by_id_range};
use crate::common::{ProcessingCtx, ProcessingError, Result};
use ahash::AHashMap;
use itertools::Itertools;
use shared_essential::domain::legacy::plans::PlanStatus;
use shared_essential::domain::{
    maths::CurrencyValue, EcAgendaItemRep, EcProtocolItem, PlanOrAmendment,
    ProtocolDetails, ProtocolType, ResultId,
};
use shared_essential::presentation::dto::general::{
    DataRecords, ObjectIdentifier, TaggedValue,
};
use shared_essential::presentation::dto::processing::price_analysis::ImportReq;
use shared_essential::presentation::dto::processing::{
    GetAgendaDetailsReq, GetAgendaDetailsRes, GetAgendaItemsByIdRangeReq,
    GetProtocolItemsByIdRangeReq, ImportItemListSpecificResponseData,
    MergedAgendaItem, MergedAgendaOrProtocolItem, ProtocolDetailsItem,
};
use shared_essential::presentation::dto::response_request::{
    ApiResponse, EntityKind, Messages,
};
use std::collections::HashMap;
use std::ops::SubAssign;

use std::time::Duration;
use uuid::Uuid;

/// Файл "Реестр СК Текущий". Индекс столбца "Номер ППЗ/ДС"
pub const CURRENT_REESTR_PLAN_NUMBER_COLUMN_INDEX: usize = 1;
/// Файл "Реестр СК Текущий". Индекс столбца "Стоимость (без НДС)"
pub const CURRENT_REESTR_SUM_EXCLUDED_VAT_COLUMN_INDEX: usize = 6;
/// Файл "Реестр СК Текущий". Индекс столбца "Стоимость АЦ (без НДС)"
pub const CURRENT_REESTR_PRICING_SUM_EXCLUDED_VAT_COLUMN_INDEX: usize = 7;
/// Файл "Реестр СК Текущий". Индекс столбца "Стоимость СК (без НДС)"
pub const CURRENT_REESTR_COMMISSION_SUM_EXCLUDED_VAT_COLUMN_INDEX: usize = 8;
/// Файл "Реестр СК Текущий". Индекс столбца "Столбец Время проведения"
pub const CURRENT_REESTR_REVIEWED_AT_COLUMN_INDEX: usize = 12;

/// Файл "Реестр Д647 как приложение". Индекс столбца "Номер ППЗ/ДС"
pub const ATTACHMENT_REESTR_PLAN_NUMBER_COLUMN_INDEX: usize = 1;
/// Файл "Реестр Д647 как приложение". Индекс столбца "Стоимость (без НДС)"
pub const ATTACHMENT_REESTR_SUM_EXCLUDED_VAT_COLUMN_INDEX: usize = 8;
/// Файл "Реестр Д647 как приложение". Индекс столбца "Стоимость АЦ (без НДС)"
pub const ATTACHMENT_REESTR_PRICING_SUM_EXCLUDED_VAT_COLUMN_INDEX: usize = 7;
/// Файл "Реестр Д647 как приложение". Индекс столбца "Стоимость СК (без НДС)"
pub const ATTACHMENT_REESTR_COMMISSION_SUM_EXCLUDED_VAT_COLUMN_INDEX: usize = 7;

macro_rules! insert_field {
    ($idx:expr, $src:ident, $dest:ident, $field:ident) => {
        let v = match $src.get($idx) {
            Some(TaggedValue::Float(v)) => Some(CurrencyValue::from_f64(*v)?),
            Some(TaggedValue::Int(v)) => Some(CurrencyValue::from_i64(*v)?),
            Some(TaggedValue::CValue(v)) => Some(*v),
            _ => $dest.$field,
        };
        $dest.$field = v;
    };
}

/// Импорт на Фронт(Без сохранения в БД) Списка ППЗ/ДС в повестку/protocol
pub(crate) async fn import_item_list_specific(
    request: ImportReq,
    proc_ctx: ProcessingCtx,
) -> Result<ApiResponse<ImportItemListSpecificResponseData, ()>> {
    tracing::info!(
        kind = "get",
        "Получен запрос на импорт на фронт списка ппз/дс: {req:?}\n",
        req = request,
    );

    let mut response = ApiResponse::default();

    match request.object_identifier.object_type {
        EntityKind::Agenda => {
            let agenda_details = get_agenda_details(
                GetAgendaDetailsReq {
                    id: request.object_identifier.id,
                },
                proc_ctx.db_pool.clone(),
            )
            .await?;
            if agenda_details.messages.is_error() {
                response.messages.add_messages(agenda_details.messages);
                return Ok(response);
            }
            let (agenda_items, messages) =
                import_agenda_items(&request, &agenda_details.data, &proc_ctx)
                    .await?;
            response.messages.add_messages(messages);
            response.data = ImportItemListSpecificResponseData {
                item_list: MergedAgendaOrProtocolItem::AgendaItems(agenda_items),
            };
        }
        EntityKind::Protocol => {
            let protocol_details_inner =
                get_protocol(request.object_identifier.id, &proc_ctx.db_pool)
                    .await?;

            let (protocol_items, messages) =
                import_protocol_items(&request, protocol_details_inner, &proc_ctx)
                    .await?;
            response.messages.add_messages(messages);
            response.data = ImportItemListSpecificResponseData {
                item_list: MergedAgendaOrProtocolItem::ProtocolItems(
                    protocol_items,
                ),
            };
        }
        _ => {}
    }

    Ok(response)
}

/// Импорт на фронт позиций повестки
async fn import_agenda_items(
    request: &ImportReq,
    agenda_details: &GetAgendaDetailsRes,
    proc_ctx: &ProcessingCtx,
) -> Result<(Vec<MergedAgendaItem>, Messages)> {
    let import_agenda_items = convert_data_records_to_import_items(
        request,
        agenda_details.agenda.meeting_date,
    )?;

    let import_plan_id_with_index: AHashMap<i64, usize> = import_agenda_items
        .iter()
        .enumerate()
        .map(|(position, item)| (item.plan_id, position))
        .collect();

    let mut current_agenda_items;
    if request.is_registered_by_d647.unwrap_or_default() {
        current_agenda_items = agenda_details.agenda_item_d647_list.clone();
    } else {
        current_agenda_items = agenda_details.agenda_item_list.clone();
    }

    update_current_agenda_items(&mut current_agenda_items, &import_agenda_items)
        .await?;
    let messages = add_new_agenda_items(
        request,
        &mut current_agenda_items,
        &import_agenda_items,
        proc_ctx,
    )
    .await?;

    current_agenda_items.retain(|item| {
        import_plan_id_with_index
            .contains_key(&item.plan.plan_id().unwrap_or_default())
    });

    current_agenda_items.sort_by(|a, b| {
        import_plan_id_with_index
            .get(&a.plan.plan_id().unwrap_or_default())
            .cmp(
                &import_plan_id_with_index
                    .get(&b.plan.plan_id().unwrap_or_default()),
            )
    });

    Ok((current_agenda_items, messages))
}

/// Импорт на фронт позиций протокола
async fn import_protocol_items(
    request: &ImportReq,
    protocol_details: ProtocolDetails,
    proc_ctx: &ProcessingCtx,
) -> Result<(Vec<ProtocolDetailsItem>, Messages)> {
    let import_protocol_items =
        convert_data_records_to_import_items(request, None)?;

    let import_plan_id_with_index: AHashMap<i64, usize> = import_protocol_items
        .iter()
        .enumerate()
        .map(|(position, item)| (item.plan_id, position))
        .collect();

    let ProtocolDetails {
        mut items,
        plans,
        amendments,
        ..
    } = protocol_details;

    items = items.into_iter().unique_by(|value| value.uuid).collect();
    items.retain(|item| {
        item.is_registered_by_d647
            == request.is_registered_by_d647.unwrap_or_default()
    });
    let mut plan_or_amendment_map =
        PlanOrAmendment::collect_map_by_uuid(plans, amendments);

    let mut protocol_items_details = update_current_protocol_items(
        &mut items,
        &mut plan_or_amendment_map,
        &import_protocol_items,
    )
    .await?;

    let messages = add_new_protocol_items(
        request,
        &mut protocol_items_details,
        &import_protocol_items,
        proc_ctx,
    )
    .await?;

    protocol_items_details.retain(|item| {
        import_plan_id_with_index
            .contains_key(&item.plan.plan_id().unwrap_or_default())
    });

    // Автозаполнение поля "Решение СК"(при условии protocol_type_id=1, is_registered_by_d647 = false)
    if request.is_registered_by_d647.map_or(false, |b| !b) {
        protocol_items_details.iter_mut().for_each(|item| {
            if *item.plan.status_id() == Some(PlanStatus::PlanCancelled) {
                item.protocol_item.item.result_id = Some(ResultId::Cancel);
            } else {
                let pricing_sum_excluded_vat =
                    item.protocol_item.item.pricing_sum_excluded_vat.flatten();
                let commission_sum_excluded_vat =
                    item.protocol_item.item.commission_sum_excluded_vat.flatten();
                if commission_sum_excluded_vat.is_none() {
                    item.protocol_item.item.result_id = Some(ResultId::NotAgreed);
                } else if pricing_sum_excluded_vat == commission_sum_excluded_vat {
                    item.protocol_item.item.result_id = Some(ResultId::Approved);
                } else {
                    item.protocol_item.item.result_id =
                        Some(ResultId::AgreedWithPriceCorrection);
                }
            }
        });
    }

    protocol_items_details.sort_by(|a, b| {
        import_plan_id_with_index
            .get(&a.plan.plan_id().unwrap_or_default())
            .cmp(
                &import_plan_id_with_index
                    .get(&b.plan.plan_id().unwrap_or_default()),
            )
    });

    Ok((protocol_items_details, messages))
}

/// Обновление текущих позиций повестки данными из импорта
async fn update_current_agenda_items(
    current_items: &mut Vec<MergedAgendaItem>,
    import_items: &[ImportItem],
) -> Result<()> {
    let import_items_map: HashMap<_, _> =
        import_items.iter().map(|item| (item.plan_id, item)).collect();
    for current_item in current_items {
        if let Some(import_item) =
            import_items_map.get(&current_item.plan.plan_id().unwrap_or_default())
        {
            current_item.agenda_item.sum_excluded_vat =
                Some(import_item.sum_excluded_vat);
            current_item.agenda_item.pricing_sum_excluded_vat =
                Some(import_item.pricing_sum_excluded_vat);
            current_item.agenda_item.reviewed_at = Some(import_item.reviewed_at);
        }
    }
    Ok(())
}

/// Добавление новых позиций в повестку
async fn add_new_agenda_items(
    request: &ImportReq,
    current_items: &mut Vec<MergedAgendaItem>,
    import_items: &[ImportItem],
    proc_ctx: &ProcessingCtx,
) -> Result<Messages> {
    let import_items_map: HashMap<_, _> =
        import_items.iter().map(|item| (item.plan_id, item)).collect();
    // Import item plan ids
    let import_plan_ids =
        import_items.iter().map(|value| value.plan_id).collect_vec();
    // Current item plan ids
    let current_plan_ids = current_items
        .iter()
        .map(|item| item.plan.plan_id().unwrap_or_default())
        .collect_vec();
    // New agenda item plan ids
    let new_plan_ids: Vec<_> = import_plan_ids
        .iter()
        .filter(|&x| !current_plan_ids.contains(x))
        .cloned()
        .collect();

    if !new_plan_ids.is_empty() {
        let get_agenda_items_by_id_range_req = GetAgendaItemsByIdRangeReq {
            agenda_id: request.object_identifier.id,
            is_registered_by_d647: request
                .is_registered_by_d647
                .unwrap_or_default(),
            uuid: request.object_identifier.uuid,
            item_list: new_plan_ids.iter().map(|item| vec![*item]).collect(),
        };
        let response = get_agenda_items_by_id_range(
            get_agenda_items_by_id_range_req,
            proc_ctx.db_pool.clone(),
        )
        .await?;
        if response.messages.is_error() {
            return Ok(response.messages);
        }
        let new_plan_amendments_list = response.data.item_list;
        for new_plan_amendment in new_plan_amendments_list {
            if let Some(import_item) = import_items_map
                .get(&new_plan_amendment.plan_id().unwrap_or_default())
            {
                let mut merge_item = MergedAgendaItem {
                    agenda_item: EcAgendaItemRep {
                        uuid: Some(Default::default()),
                        agenda_uuid: Some(request.object_identifier.uuid),
                        source_uuid: *new_plan_amendment.uuid(),
                        sum_excluded_vat: Some(import_item.sum_excluded_vat),
                        pricing_sum_excluded_vat: Some(
                            import_item.pricing_sum_excluded_vat,
                        ),
                        is_registered_by_d647: Some(
                            import_item.is_registered_by_d647,
                        ),
                        ..Default::default()
                    },
                    plan: new_plan_amendment.clone(),
                };

                if import_item.reviewed_at.is_some() {
                    merge_item.agenda_item.reviewed_at =
                        Some(import_item.reviewed_at);
                }
                current_items.push(merge_item);
            }
        }
    }
    Ok(Messages::default())
}

/// Добавление новых позиций в протокол
async fn add_new_protocol_items(
    request: &ImportReq,
    current_items: &mut Vec<ProtocolDetailsItem>,
    import_items: &[ImportItem],
    proc_ctx: &ProcessingCtx,
) -> Result<Messages> {
    let import_items_map: HashMap<_, _> =
        import_items.iter().map(|item| (item.plan_id, item)).collect();
    // Import item plan ids
    let import_plan_ids =
        import_items.iter().map(|value| value.plan_id).collect_vec();

    // Current item plan ids
    let current_plan_ids = current_items
        .iter()
        .map(|item| item.plan.plan_id().unwrap_or_default())
        .collect_vec();
    // New agenda item plan ids
    let new_plan_ids: Vec<_> = import_plan_ids
        .iter()
        .filter(|&x| !current_plan_ids.contains(x))
        .cloned()
        .collect();

    if !new_plan_ids.is_empty() {
        let get_protocol_items_by_id_range_req = GetProtocolItemsByIdRangeReq {
            protocol_id: request.object_identifier.id,
            is_registered_by_d647: request
                .is_registered_by_d647
                .unwrap_or_default(),
            uuid: request.object_identifier.uuid,
            item_list: new_plan_ids.iter().map(|item| vec![*item]).collect_vec(),
            protocol_type_id: ProtocolType::InPersonMeeting,
        };
        let (new_plan_amendments_list, messages) =
            get_protocol_items_by_id_range_inner(
                get_protocol_items_by_id_range_req,
                &proc_ctx.db_pool,
            )
            .await?;

        if messages.is_error() {
            return Ok(messages);
        }

        for new_plan_amendment in new_plan_amendments_list {
            if let Some(import_item) = import_items_map.get(new_plan_amendment.id())
            {
                let ec_protocol_item = EcProtocolItem {
                    protocol_uuid: request.object_identifier.uuid,
                    source_uuid: *new_plan_amendment.uuid(),
                    sum_excluded_vat: import_item.sum_excluded_vat,
                    pricing_sum_excluded_vat: import_item.pricing_sum_excluded_vat,
                    is_registered_by_d647: import_item.is_registered_by_d647,
                    commission_sum_excluded_vat: import_item
                        .commission_sum_excluded_vat,
                    ..Default::default()
                };
                let merge_item = get_protocol_details::convert_inner(
                    new_plan_amendment,
                    ec_protocol_item,
                )?;
                current_items.push(merge_item);
            }
        }
    }
    Ok(Messages::default())
}

/// Обновление текущих позиций протокола данными из импорта
async fn update_current_protocol_items(
    ec_items: &mut Vec<EcProtocolItem>,
    plan_or_amendment_map: &mut HashMap<Uuid, PlanOrAmendment>,
    import_items: &[ImportItem],
) -> Result<Vec<ProtocolDetailsItem>> {
    let mut protocol_details_items = Vec::new();

    let import_items_map: HashMap<_, _> =
        import_items.iter().map(|item| (item.plan_id, item)).collect();

    for ec_item in ec_items {
        if let Some(plan_or_amendment) =
            plan_or_amendment_map.get_mut(&ec_item.source_uuid)
        {
            if let Some(import_item) = import_items_map.get(plan_or_amendment.id())
            {
                ec_item.sum_excluded_vat = import_item.sum_excluded_vat;
                ec_item.pricing_sum_excluded_vat =
                    import_item.pricing_sum_excluded_vat;
                ec_item.is_registered_by_d647 = import_item.is_registered_by_d647;
                ec_item.commission_sum_excluded_vat =
                    import_item.commission_sum_excluded_vat;
            }
            let merge_item = get_protocol_details::convert_inner(
                plan_or_amendment.clone(),
                ec_item.clone(),
            )?;
            protocol_details_items.push(merge_item);
        }
    }
    Ok(protocol_details_items)
}

fn convert_data_records_to_import_items(
    request: &ImportReq,
    agenda_or_protocol_meeting_date: Option<AsezDate>,
) -> Result<Vec<ImportItem>> {
    let ImportReq {
        object_identifier: ObjectIdentifier { .. },
        data_records: DataRecords { data, .. },
        is_registered_by_d647,
        ..
    } = request;

    let mut items = Vec::with_capacity(data.len());

    // Файл "Реестр СК Текущий"
    let mut plan_number_column_index = CURRENT_REESTR_PLAN_NUMBER_COLUMN_INDEX;
    let mut sum_excluded_vat_column_index =
        CURRENT_REESTR_SUM_EXCLUDED_VAT_COLUMN_INDEX;
    let mut pricing_sum_excluded_vat_column_index =
        CURRENT_REESTR_PRICING_SUM_EXCLUDED_VAT_COLUMN_INDEX;
    let mut commission_sum_excluded_vat_column_index =
        CURRENT_REESTR_COMMISSION_SUM_EXCLUDED_VAT_COLUMN_INDEX;
    let mut reviewed_at_column_index =
        Some(CURRENT_REESTR_REVIEWED_AT_COLUMN_INDEX);

    // Файл "Реестр Д647 как приложение"
    if is_registered_by_d647.unwrap_or_default() {
        plan_number_column_index = ATTACHMENT_REESTR_PLAN_NUMBER_COLUMN_INDEX;
        sum_excluded_vat_column_index =
            ATTACHMENT_REESTR_SUM_EXCLUDED_VAT_COLUMN_INDEX;
        pricing_sum_excluded_vat_column_index =
            ATTACHMENT_REESTR_PRICING_SUM_EXCLUDED_VAT_COLUMN_INDEX;
        commission_sum_excluded_vat_column_index =
            ATTACHMENT_REESTR_COMMISSION_SUM_EXCLUDED_VAT_COLUMN_INDEX;
        reviewed_at_column_index = None;
    }

    for data_record in data {
        let mut import_item = ImportItem {
            is_registered_by_d647: is_registered_by_d647.unwrap_or_default(),
            ..Default::default()
        };
        // Номер ППЗ/ДС
        if let Some(data) = data_record.get(plan_number_column_index) {
            import_item.plan_id =
                data.get_string_value().parse::<i64>().map_err(|error| {
                    get_processing_field_error("Номер ППЗ/ДС", error.to_string())
                })?;
        }
        // Стоимость (без НДС)
        insert_field!(
            sum_excluded_vat_column_index,
            data_record,
            import_item,
            sum_excluded_vat
        );
        // Стоимость АЦ (без НДС)
        insert_field!(
            pricing_sum_excluded_vat_column_index,
            data_record,
            import_item,
            pricing_sum_excluded_vat
        );
        // Стоимость СК (без НДС)
        insert_field!(
            commission_sum_excluded_vat_column_index,
            data_record,
            import_item,
            commission_sum_excluded_vat
        );

        // Время проведения
        if let Some(index) = reviewed_at_column_index {
            if let (Some(tagged_value), Some(meeting_date)) =
                (data_record.get(index), agenda_or_protocol_meeting_date)
            {
                match tagged_value {
                    TaggedValue::DateTime(asez_timestamp) => {
                        let mut reviewed_at = meeting_date
                            .0
                            .try_with_hms(
                                asez_timestamp.time().hour(),
                                asez_timestamp.time().minute(),
                                0,
                            )
                            .map_err(|error| {
                                get_processing_field_error(
                                    "Время проведения",
                                    error.to_string(),
                                )
                            })?;

                        reviewed_at.sub_assign(Duration::new(3 * 3600, 0));
                        import_item.reviewed_at = Some(AsezTimestamp(reviewed_at));
                    }
                    TaggedValue::String(time_str) => {
                        if let Some((hour, minute)) =
                            time_str.split_once([':', ' '])
                        {
                            let mut reviewed_at = meeting_date
                                .0
                                .try_with_hms(
                                    hour.trim().parse::<u8>().unwrap_or_default(),
                                    minute.trim().parse::<u8>().unwrap_or_default(),
                                    00,
                                )
                                .map_err(|error| {
                                    get_processing_field_error(
                                        "Время проведения",
                                        error.to_string(),
                                    )
                                })?;
                            reviewed_at.sub_assign(Duration::new(3 * 3600, 0));
                            import_item.reviewed_at =
                                Some(AsezTimestamp(reviewed_at));
                        }
                    }
                    TaggedValue::Null | TaggedValue::NullWithFormat(_) => {}
                    _ => {
                        return Err(get_processing_field_error(
                            "Время проведения",
                            "Некорректный формат ячейки".to_string(),
                        ));
                    }
                }
            }
        }
        items.push(import_item);
    }
    Ok(items)
}

fn get_processing_field_error(field: &str, error: String) -> ProcessingError {
    ProcessingError::Import(format!("Ошибка обработки поля {}: {}", field, error))
}

#[derive(Default, Debug)]
struct ImportItem {
    pub plan_id: i64,
    pub sum_excluded_vat: Option<CurrencyValue>,
    pub pricing_sum_excluded_vat: Option<CurrencyValue>,
    pub commission_sum_excluded_vat: Option<CurrencyValue>,
    pub reviewed_at: Option<AsezTimestamp>,
    pub is_registered_by_d647: bool,
}
