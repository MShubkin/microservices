//! Модуль для работы со справочником "Причины аннулирования".
//! Содержит всю логику: от взаимодействия с БД до формирования ответов для API.

use ahash::{AHashMap, AHashSet};
use asez2_shared_db::DbAdaptor;
use broker::rabbit::RabbitAdapter;
use format_tools::numeric_format;
use itertools::Itertools;
use monolith_service::http::MonolithHttpService;
use sqlx::PgPool;
use std::sync::Arc;

use asez2_shared_db::db_item::{
    int_array::AsezArray, joined::JoinTo, update_fields_helper, AsezTimestamp,
    DbAdaptorFieldsWithValues, DbItem, DbUpdateByFilter, DbUpsert, Filter,
    FilterTree, Select,
};

use shared_essential::{
    domain::plan_reasons_cancel::{
        CheckReason, JoinedPlanReasonsCancel, JoinedPlanReasonsCancelSelector,
        PlanReasonCancelCustomer, PlanReasonCancelHeader,
        PlanReasonCancelHeaderRep,
    },
    presentation::dto::{
        master_data::{
            error::{MasterDataError, MasterDataResult},
            plan_reasons_cancel::{
                PlanReasonCancel,
                PlanReasonCancelCustomer as PlanReasonCancelCustomerDto,
            },
            request::{
                CreatePlanReasonsCancelReq, SearchByIdReq, SearchByUserInput,
                SearchPlanReasonCancelReq, SearchPlanReasonsCancelRabbitReq,
                UpdatePlanReasonsCancelReq,
            },
            response::{
                PlanReasonCancelDeleteResponse,
                PlanReasonCancelDeleteRestoreResponse,
                PlanReasonCancelSearchResponse, SearchResultValue,
            },
        },
        response_request::{
            BusinessMessage, Message, MessageKind, Messages, PaginatedData,
        },
    },
    replacement,
};

use rabbit_services::{
    print_doc::PrintDocService, properties::AsezRabbitProperties,
};
use shared_essential::presentation::dto::{
    export::ReplacementConfig,
    general::{DataRecord, DataRecords, InternalExportReq, TaggedValue},
    print_docs::{common::TemplateFormat, Content, PrintReq},
    response_request::EntityKind,
    Source,
};

use crate::presentation::dto::ExportRequest;
use asez2_shared_db::Value;
use shared_essential::presentation::dto::general::UiSelect;
use shared_essential::presentation::dto::response_request::ApiResponse;
pub mod repository {
    use super::*;

    pub(crate) async fn search(
        search_request: &SearchPlanReasonCancelReq,
        pool: &PgPool,
    ) -> MasterDataResult<Vec<JoinedPlanReasonsCancel>> {
        let search_text = &search_request.search.search;
        let header = &search_request.header;

        let mut select = Select::full::<PlanReasonCancelHeader>()
            .eq_maybe(PlanReasonCancelHeader::is_removed, header.is_removed)
            .eq_maybe(
                PlanReasonCancelHeader::is_reason_fill_type,
                header.is_reason_fill_type,
            )
            .eq_maybe(PlanReasonCancelHeader::is_new_plan, header.is_new_plan)
            .eq_maybe(
                PlanReasonCancelHeader::check_reason_id,
                header.check_reason_id,
            )
            .take_n(search_request.search.quantity as usize);

        if let Some(functionality_ids) = &header.functionality_id_list {
            if !functionality_ids.0.is_empty() {
                select = select.array_overlaps(
                    PlanReasonCancelHeader::functionality_id_list,
                    Value::Vec16(functionality_ids.clone()),
                );
            }
        }

        if !search_text.is_empty() {
            select = select.fields_containing(
                [PlanReasonCancelHeader::text],
                format!("(?i){}", search_text),
            );
        }

        assemble_aggregates(&select, pool).await
    }

    pub(crate) async fn get_by_id(
        id: i32,
        pool: &PgPool,
    ) -> MasterDataResult<JoinedPlanReasonsCancel> {
        let select = Select::full::<PlanReasonCancelHeader>()
            .eq(PlanReasonCancelHeader::id, id);

        let aggregates = assemble_aggregates(&select, pool).await?;

        aggregates.into_iter().next().ok_or_else(|| {
            MasterDataError::Business(Messages::default().with_message(
                Message::error(format!(
                    "Причина аннулирования с id: {} не найдена",
                    id
                )),
            ))
        })
    }

    pub(crate) async fn get_by_ids(
        ids: &[i32],
        pool: &PgPool,
    ) -> MasterDataResult<(Messages, Vec<JoinedPlanReasonsCancel>)> {
        let mut messages = Messages::default();
        if ids.is_empty() {
            return Ok((Messages::default(), Vec::new()));
        }

        let select = Select::full::<PlanReasonCancelHeader>()
            .in_any(PlanReasonCancelHeader::id, ids);

        let aggregates = assemble_aggregates(&select, pool).await?;

        if aggregates.len() < ids.len() {
            let found_ids: AHashSet<i32> =
                aggregates.iter().map(|agg| agg.header.id).collect();
            for id in ids {
                if !found_ids.contains(id) {
                    messages.add_message(
                        MessageKind::Error,
                        format!("Запись с id: {id} не найдена"),
                    );
                }
            }
        }

        Ok((messages, aggregates))
    }

    pub(crate) async fn get_by_select(
        select_req: UiSelect,
        pool: &PgPool,
    ) -> MasterDataResult<Vec<JoinedPlanReasonsCancel>> {
        let full_select = Select::try_from(select_req)?;
        get_by_select_internal(full_select, pool).await
    }

    pub(crate) async fn get_by_select_internal(
        select: Select,
        pool: &PgPool,
    ) -> MasterDataResult<Vec<JoinedPlanReasonsCancel>> {
        let Select { offset, take_n, .. } = select.clone();
        let (mut header_select, mut customer_select) =
            select.split_for::<PlanReasonCancelHeaderRep>();

        header_select = header_select.offset_maybe(offset).take_n_maybe(take_n);

        if !customer_select.filter_list.is_empty() {
            customer_select.field_list =
                vec![PlanReasonCancelCustomer::plan_reason_cancel_id.to_string()];
            customer_select =
                customer_select.eq(PlanReasonCancelCustomer::is_removed, false);

            let customers =
                PlanReasonCancelCustomer::select(&customer_select, pool).await?;

            let header_ids: AHashSet<i32> =
                customers.into_iter().map(|c| c.plan_reason_cancel_id).collect();

            if header_ids.is_empty() {
                return Ok(Vec::new());
            }

            header_select =
                header_select.in_any(PlanReasonCancelHeader::id, header_ids);
        }

        let aggregates = assemble_aggregates(&header_select, pool).await?;

        Ok(aggregates)
    }

    pub(crate) async fn create(
        header_rep: PlanReasonCancelHeaderRep,
        customer_ids: AHashSet<i32>,
        user_id: i32,
        pool: &PgPool,
    ) -> MasterDataResult<(JoinedPlanReasonsCancel, Messages)> {
        let now = AsezTimestamp::now();
        let mut tx = pool.begin().await?;

        let mut rep = header_rep;
        rep.created_at = Some(now);
        rep.created_by = Some(user_id);
        rep.changed_at = Some(now);
        rep.changed_by = Some(user_id);
        rep.is_removed = Some(false);
        let mut header = rep.into_item()?;

        let created_header = header.insert_returning(&mut *tx).await?;
        let id = created_header.id;

        let mut customers_to_insert: Vec<PlanReasonCancelCustomer> = customer_ids
            .into_iter()
            .map(|customer_id| PlanReasonCancelCustomer {
                id: 0,
                plan_reason_cancel_id: id,
                customer_id,
                is_removed: false,
                changed_at: now,
                changed_by: user_id,
                created_at: now,
                created_by: user_id,
            })
            .collect();

        let mut created_customers = Vec::new();
        if !customers_to_insert.is_empty() {
            created_customers = PlanReasonCancelCustomer::insert_vec_returning(
                &mut customers_to_insert,
                &mut tx,
            )
            .await?;
        }

        tx.commit().await?;

        let aggregate = JoinedPlanReasonsCancel {
            header: created_header,
            customers: created_customers,
        };

        let mut messages = Messages::default();
        PlanReasonCancelOpMessage::Created
            .checked_append(&mut messages, &[&aggregate]);

        Ok((aggregate, messages))
    }

    pub(crate) async fn update(
        header_rep: PlanReasonCancelHeaderRep,
        new_customer_ids: AHashSet<i32>,
        user_id: i32,
        pool: &PgPool,
    ) -> MasterDataResult<(JoinedPlanReasonsCancel, Messages)> {
        let now = AsezTimestamp::now();
        let id = header_rep.id.ok_or_else(|| {
            MasterDataError::Business(Messages::default().with_message(
                Message::error(
                    "В запросе на обновление отсутствует обязательное поле 'id'.",
                ),
            ))
        })?;

        let mut tx = pool.begin().await?;

        let select = Select::full::<PlanReasonCancelHeader>()
            .eq(PlanReasonCancelHeader::id, id);
        let old_header = PlanReasonCancelHeader::select_option(&select, &mut tx)
            .await?
            .ok_or_else(|| {
                MasterDataError::Business(Messages::default().with_message(
                    Message::error(format!("Запись с id: {id} не найдена")),
                ))
            })?;

        let mut rep = header_rep;
        rep.changed_at = Some(now);
        rep.changed_by = Some(user_id);

        if rep.check_reason_id.is_none() {
            rep.check_reason_id = Some(0);
        }

        let mask =
            PlanReasonCancelHeaderRep::create_strict_bind_mask(&[rep.clone()])?;
        let update_fields = update_fields_helper::<PlanReasonCancelHeader>(&mask);

        let updated_header = rep.into_item_merged(old_header)?;
        updated_header.update(Some(&update_fields), &mut *tx).await?;

        let customer_update_values = PlanReasonCancelCustomer {
            is_removed: true,
            changed_at: now,
            changed_by: user_id,
            ..Default::default()
        };
        customer_update_values
            .update_by_filter(
                &[
                    PlanReasonCancelCustomer::is_removed,
                    PlanReasonCancelCustomer::changed_at,
                    PlanReasonCancelCustomer::changed_by,
                ],
                &FilterTree::from(Filter::eq(
                    PlanReasonCancelCustomer::plan_reason_cancel_id,
                    id,
                )),
                &mut *tx,
            )
            .await?;

        let mut customers_to_upsert: Vec<PlanReasonCancelCustomer> =
            new_customer_ids
                .into_iter()
                .map(|customer_id| PlanReasonCancelCustomer {
                    id: 0,
                    plan_reason_cancel_id: id,
                    customer_id,
                    is_removed: false,
                    created_at: now,
                    created_by: user_id,
                    changed_at: now,
                    changed_by: user_id,
                })
                .collect();

        let mut upserted_customers = Vec::new();
        if !customers_to_upsert.is_empty() {
            upserted_customers = PlanReasonCancelCustomer::upsert_returning(
                &mut customers_to_upsert,
                Some(&[
                    PlanReasonCancelCustomer::is_removed,
                    PlanReasonCancelCustomer::changed_at,
                    PlanReasonCancelCustomer::changed_by,
                ]),
                &mut tx,
            )
            .await?;
        }

        tx.commit().await?;

        let aggregate = JoinedPlanReasonsCancel {
            header: updated_header,
            customers: upserted_customers,
        };

        let mut messages = Messages::default();
        PlanReasonCancelOpMessage::Updated
            .checked_append(&mut messages, &[&aggregate]);

        Ok((aggregate, messages))
    }

    pub(crate) async fn delete(
        ids: &[i32],
        user_id: i32,
        pool: &PgPool,
    ) -> MasterDataResult<(Vec<JoinedPlanReasonsCancel>, Messages)> {
        if ids.is_empty() {
            return Ok((vec![], Default::default()));
        }
        let mut tx = pool.begin().await?;
        let now = AsezTimestamp::now();

        let header_update_values = PlanReasonCancelHeader {
            is_removed: true,
            changed_at: now,
            changed_by: user_id,
            ..Default::default()
        };

        let filter = FilterTree::and_from_list([
            Filter::in_any(PlanReasonCancelHeader::id, ids.iter().copied()),
            Filter::eq(PlanReasonCancelHeader::is_removed, false),
        ]);

        header_update_values
            .update_by_filter(
                &[
                    PlanReasonCancelHeader::is_removed,
                    PlanReasonCancelHeader::changed_at,
                    PlanReasonCancelHeader::changed_by,
                ],
                &filter,
                &mut *tx,
            )
            .await?;

        tx.commit().await?;

        let (messages, aggregates) = get_by_ids(ids, pool).await?;
        Ok((aggregates, messages))
    }

    pub(crate) async fn restore(
        ids: &[i32],
        user_id: i32,
        pool: &PgPool,
    ) -> MasterDataResult<(Vec<JoinedPlanReasonsCancel>, Messages)> {
        if ids.is_empty() {
            return Ok((vec![], Default::default()));
        }
        let mut tx = pool.begin().await?;
        let now = AsezTimestamp::now();

        let header_update_values = PlanReasonCancelHeader {
            is_removed: false,
            changed_at: now,
            changed_by: user_id,
            ..Default::default()
        };

        let filter = FilterTree::and_from_list([
            Filter::in_any(PlanReasonCancelHeader::id, ids.iter().copied()),
            Filter::eq(PlanReasonCancelHeader::is_removed, true),
        ]);

        header_update_values
            .update_by_filter(
                &[
                    PlanReasonCancelHeader::is_removed,
                    PlanReasonCancelHeader::changed_at,
                    PlanReasonCancelHeader::changed_by,
                ],
                &filter,
                &mut *tx,
            )
            .await?;

        tx.commit().await?;

        let (messages, aggregates) = get_by_ids(ids, pool).await?;
        Ok((aggregates, messages))
    }

    async fn assemble_aggregates(
        header_select: &Select,
        pool: &PgPool,
    ) -> MasterDataResult<Vec<JoinedPlanReasonsCancel>> {
        let customer_select = Select::full::<PlanReasonCancelCustomer>()
            .eq(PlanReasonCancelCustomer::is_removed, false);

        let customer_join_data =
            PlanReasonCancelCustomer::join_default().selecting(customer_select);

        let joined_results =
            JoinedPlanReasonsCancelSelector::new_with_order(header_select.clone())
                .set_customers(customer_join_data)
                .get(pool)
                .await?;

        Ok(joined_results)
    }
}

/// API /rest/dictionary/v1/plan_reason_cancel/get/item_list/
pub async fn get_item_list(
    select_req: UiSelect,
    monolith_service: &MonolithHttpService,
    token: String,
    pool: &PgPool,
) -> MasterDataResult<(PaginatedData<PlanReasonCancel>, Messages)> {
    let field_list = select_req.field_list.clone();
    let aggregates = repository::get_by_select(select_req, pool).await?;
    let customer_map = fetch_customers_map(monolith_service, token).await?;

    let mut messages = Messages::default();
    if aggregates.is_empty() {
        let empty_vec: Vec<&JoinedPlanReasonsCancel> = Vec::new();
        PlanReasonCancelOpMessage::NoData.checked_append(&mut messages, &empty_vec);
    } else {
        PlanReasonCancelOpMessage::Listed
            .checked_append(&mut messages, &aggregates);
    }

    let item_list: Vec<_> = aggregates
        .into_iter()
        .map(|item| {
            to_plan_reason_cancel_response(item, &customer_map, Some(&field_list))
        })
        .collect();

    let paginated_data = PaginatedData::from(item_list);

    Ok((paginated_data, messages))
}

/// API /rest/dictionary/v1/plan_reason_cancel/search/
pub async fn search(
    search_request: &SearchPlanReasonCancelReq,
    monolith_service: &MonolithHttpService,
    token: String,
    pool: &PgPool,
) -> MasterDataResult<(PlanReasonCancelSearchResponse, Messages)> {
    let aggregates = repository::search(search_request, pool).await?;
    let customer_map = fetch_customers_map(monolith_service, token).await?;
    let value = aggregates
        .into_iter()
        .map(|item| to_plan_reason_cancel_response(item, &customer_map, None))
        .collect();

    Ok((SearchResultValue { value }, Messages::default()))
}

/// API /rest/dictionary/v1/plan_reason_cancel/search_by_id/
pub async fn search_by_id(
    search_request: &SearchByIdReq,
    monolith_service: &MonolithHttpService,
    token: String,
    pool: &PgPool,
) -> MasterDataResult<(PlanReasonCancelSearchResponse, Messages)> {
    let ids: Vec<i32> = search_request.iter().copied().collect();
    let (messages, aggregates) = repository::get_by_ids(&ids, pool).await?;
    let customer_map = fetch_customers_map(monolith_service, token).await?;
    let value = aggregates
        .into_iter()
        .map(|item| to_plan_reason_cancel_response(item, &customer_map, None))
        .collect();

    Ok((SearchResultValue { value }, messages))
}

/// API /rest/dictionary/v1/plan_reason_cancel/get/detail/
pub async fn get_detail(
    id: i32,
    monolith_service: &MonolithHttpService,
    token: String,
    pool: &PgPool,
) -> MasterDataResult<(PlanReasonCancel, Messages)> {
    let aggregate = repository::get_by_id(id, pool).await?;
    let customer_map = fetch_customers_map(monolith_service, token).await?;
    let dto = to_plan_reason_cancel_response(aggregate, &customer_map, None);
    Ok((dto, Messages::default()))
}

/// API /rest/dictionary/v1/plan_reason_cancel/create/item/
pub async fn create_item(
    create_request: CreatePlanReasonsCancelReq,
    user_id: i32,
    monolith_service: &MonolithHttpService,
    token: String,
    pool: &PgPool,
) -> MasterDataResult<(PlanReasonCancel, Messages)> {
    let validation_messages =
        validate_create_request(&create_request, pool).await?;

    if !validation_messages.is_empty() {
        return Err(MasterDataError::Business(validation_messages));
    }

    let customer_map = fetch_customers_map(monolith_service, token.clone()).await?;
    let customer_ids =
        prepare_customer_ids(&customer_map, &create_request.customers);

    let (aggregate, messages) =
        repository::create(create_request.header, customer_ids, user_id, pool)
            .await?;

    let response_dto =
        to_plan_reason_cancel_response(aggregate, &customer_map, None);
    Ok((response_dto, messages))
}

/// API /rest/dictionary/v1/plan_reason_cancel/update/item/
pub async fn update_item(
    update_request: UpdatePlanReasonsCancelReq,
    user_id: i32,
    monolith_service: &MonolithHttpService,
    token: String,
    pool: &PgPool,
) -> MasterDataResult<(PlanReasonCancel, Messages)> {
    let validation_messages =
        validate_update_request(&update_request, pool).await?;

    if !validation_messages.is_empty() {
        return Err(MasterDataError::Business(validation_messages));
    }

    let customer_map = fetch_customers_map(monolith_service, token.clone()).await?;
    let customer_ids =
        prepare_customer_ids(&customer_map, &update_request.customers);

    let (aggregate, messages) =
        repository::update(update_request.header, customer_ids, user_id, pool)
            .await?;

    let response_dto =
        to_plan_reason_cancel_response(aggregate, &customer_map, None);
    Ok((response_dto, messages))
}

/// API /rest/dictionary/v1/plan_reason_cancel/delete/item/
pub async fn delete_items(
    ids: &[i32],
    user_id: i32,
    pool: &PgPool,
) -> MasterDataResult<(PlanReasonCancelDeleteRestoreResponse, Messages)> {
    let (aggregates, mut messages) = repository::delete(ids, user_id, pool).await?;

    let processed_ids: AHashSet<i32> =
        aggregates.iter().map(|agg| agg.header.id).collect();

    PlanReasonCancelOpMessage::Deleted.checked_append(&mut messages, &aggregates);

    let item_list = ids
        .iter()
        .map(|&id| PlanReasonCancelDeleteResponse {
            id,
            is_success: processed_ids.contains(&id),
        })
        .collect();

    Ok((PlanReasonCancelDeleteRestoreResponse { item_list }, messages))
}

/// API /rest/dictionary/v1/plan_reason_cancel/restore/item/
pub async fn restore_items(
    ids: &[i32],
    user_id: i32,
    pool: &PgPool,
) -> MasterDataResult<(PlanReasonCancelDeleteRestoreResponse, Messages)> {
    let (aggregates, mut messages) =
        repository::restore(ids, user_id, pool).await?;

    let processed_ids: AHashSet<i32> =
        aggregates.iter().map(|agg| agg.header.id).collect();

    PlanReasonCancelOpMessage::Restored.checked_append(&mut messages, &aggregates);

    let item_list = ids
        .iter()
        .map(|&id| PlanReasonCancelDeleteResponse {
            id,
            is_success: processed_ids.contains(&id),
        })
        .collect();

    Ok((PlanReasonCancelDeleteRestoreResponse { item_list }, messages))
}

/// API /rest/dictionary/v1/plan_reason_cancel/export/
pub async fn export(
    req: ExportRequest,
    user_id: i32,
    token: String,
    broker_adapter: Arc<RabbitAdapter>,
    pool: &PgPool,
) -> MasterDataResult<Option<(Vec<u8>, String)>> {
    export_logic(req, user_id, token, broker_adapter, pool).await
}

macro_rules! validate {
    ($messages:ident, $request:ident, {
        $( $field:ident : $predicate:expr => $field_name:literal ),*
        $(,)?
    }) => {
        $(
            if $predicate(&$request.$field) {
                $messages.add_message(
                    MessageKind::Error,
                    format!("Поле '{}' обязательно для заполнения", $field_name),
                );
            }
        )*
    };
}

/// Проверяет, является ли `Option<String>` пустым или `None`.
fn is_empty_opt_str(opt_str: &Option<String>) -> bool {
    opt_str.as_ref().map_or(true, String::is_empty)
}

/// Проверяет, является ли `Option<AsezArray<T>>` (обертка над `Vec<T>`) пустым или `None`.
fn is_empty_opt_vec<T>(opt_vec: &Option<AsezArray<T>>) -> bool {
    opt_vec.as_ref().map_or(true, |v| v.0.is_empty())
}

/// Валидирует запрос на создание причины аннулирования.
async fn validate_create_request(
    request: &CreatePlanReasonsCancelReq,
    pool: &PgPool,
) -> MasterDataResult<Messages> {
    let mut messages = Messages::default();
    let header = &request.header;

    validate!(messages, header, {
        text: is_empty_opt_str => "Наименование причины",
        functionality_id_list: is_empty_opt_vec => "Функциональность",
        impact_area_id: Option::is_none => "Основание аннулирования",
        is_objective_reason: Option::is_none => "Объективная причина",
        is_new_plan: Option::is_none => "Новая ППЗ/ДС",
        is_reason_fill_type: Option::is_none => "Автоматическое заполнение причины",
    });

    validate!(messages, request, {
        customers: <[_]>::is_empty => "Заказчик",
    });

    if let Some(check_reason_id) = header.check_reason_id {
        check_unique_check_reason_id(None, check_reason_id, pool, &mut messages)
            .await?;
    }

    Ok(messages)
}

/// Валидирует запрос на обновление причины аннулирования.
async fn validate_update_request(
    request: &UpdatePlanReasonsCancelReq,
    pool: &PgPool,
) -> MasterDataResult<Messages> {
    let mut messages = Messages::default();
    let header = &request.header;

    validate!(messages, header, {
        text: is_empty_opt_str => "Наименование причины",
        functionality_id_list: is_empty_opt_vec => "Функциональность",
        impact_area_id: Option::is_none => "Основание аннулирования",
        is_objective_reason: Option::is_none => "Объективная причина",
        is_new_plan: Option::is_none => "Новая ППЗ/ДС",
        is_reason_fill_type: Option::is_none => "Автоматическое заполнение причины",
    });

    validate!(messages, request, {
        customers: <[_]>::is_empty => "Заказчик",
    });

    if let Some(check_reason_id) = header.check_reason_id {
        check_unique_check_reason_id(
            header.id,
            check_reason_id,
            pool,
            &mut messages,
        )
        .await?;
    }

    Ok(messages)
}

async fn check_unique_check_reason_id(
    current_id: Option<i32>,
    check_reason_id: i16,
    pool: &PgPool,
    messages: &mut Messages,
) -> MasterDataResult<()> {
    if CheckReason::from(check_reason_id) != CheckReason::Protocol {
        return Ok(());
    }

    let search_request = SearchPlanReasonCancelReq {
        search: SearchByUserInput {
            from: 0,
            quantity: 1,
            search: String::new(),
        },
        header: PlanReasonCancelHeaderRep {
            check_reason_id: Some(check_reason_id),
            is_removed: Some(false),
            ..Default::default()
        },
    };

    let aggregates = repository::search(&search_request, pool).await?;

    let has_duplicate = if let Some(current_id) = current_id {
        aggregates.iter().any(|agg| agg.header.id != current_id)
    } else {
        !aggregates.is_empty()
    };

    if has_duplicate {
        messages.add_message(
            MessageKind::Error,
            "Причина аннулирования с проверкой для протокола очной СК уже существует. Выберите другое или удалите значение в поле 'Проверка для ППЗ'".to_string()
        );
    }

    Ok(())
}

fn to_plan_reason_cancel_response(
    domain_model: JoinedPlanReasonsCancel,
    all_customers_map: &AHashMap<i32, String>,
    field_list: Option<&[String]>,
) -> PlanReasonCancel {
    let header = domain_model.header;
    let customers = domain_model.customers;

    let active_customer_ids: ahash::AHashSet<i32> = customers
        .iter()
        .filter(|c| !c.is_removed)
        .map(|c| c.customer_id)
        .collect();

    let customer_response = {
        let all_customer_ids_from_map: ahash::AHashSet<i32> =
            all_customers_map.keys().copied().collect();

        if !all_customer_ids_from_map.is_empty()
            && active_customer_ids.len() >= all_customer_ids_from_map.len()
        {
            PlanReasonCancelCustomerDto::All
        } else {
            PlanReasonCancelCustomerDto::In {
                filter_values: active_customer_ids.into_iter().sorted().collect(),
            }
        }
    };

    let final_field_list = match field_list {
        Some(fl) if !fl.is_empty() => {
            let mut fields = fl.to_vec();
            fields.push(PlanReasonCancelHeader::is_removed.to_string());
            Some(fields)
        }
        _ => None,
    };

    let mut header_rep =
        PlanReasonCancelHeaderRep::from_item(header, final_field_list.as_deref());

    if header_rep.check_reason_id == Some(0) {
        header_rep.check_reason_id = None;
    }

    PlanReasonCancel {
        header: header_rep,
        customers: vec![customer_response],
    }
}

pub fn to_plan_reason_cancel_response_simple(
    domain_model: JoinedPlanReasonsCancel,
) -> PlanReasonCancel {
    let header =
        PlanReasonCancelHeaderRep::from_item::<&str>(domain_model.header, None);
    let customer_ids: Vec<i32> = domain_model
        .customers
        .into_iter()
        .filter(|c| !c.is_removed)
        .map(|c| c.customer_id)
        .collect();

    let customers = vec![PlanReasonCancelCustomerDto::In {
        filter_values: customer_ids,
    }];

    PlanReasonCancel { header, customers }
}

/// Функция только для получения мапы из монолита
async fn fetch_customers_map(
    monolith_service: &MonolithHttpService,
    token: String,
) -> MasterDataResult<AHashMap<i32, String>> {
    let customers = monolith_service.get_customer_updates(token).await?;

    if customers.is_empty() {
        return Err(MasterDataError::InternalError(
            "Монолит вернул пустой список заказчиков. Это не ожидаемое поведение."
                .to_string(),
        ));
    }

    Ok(customers.into_iter().map(|c| (c.id, c.text)).collect())
}

/// Подготавливает набор ID заказчиков на основе запроса.
/// Фронтом  используется общий формат через массив объектов
/// Ситуации передачи с несколькими строками не будет
fn prepare_customer_ids(
    customer_map: &AHashMap<i32, String>,
    customers_req: &[PlanReasonCancelCustomerDto],
) -> AHashSet<i32> {
    match customers_req.first() {
        Some(PlanReasonCancelCustomerDto::All) => {
            customer_map.keys().copied().collect()
        }
        Some(PlanReasonCancelCustomerDto::In { filter_values }) => {
            filter_values.iter().cloned().collect()
        }
        None => AHashSet::new(),
    }
}

pub async fn search_internal(
    dto: SearchPlanReasonsCancelRabbitReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<Vec<PlanReasonCancel>, ()>> {
    let select = Select::full::<PlanReasonCancelHeader>()
        .eq(PlanReasonCancelHeader::is_removed, false)
        .in_any_maybe(PlanReasonCancelHeader::id, dto.ids)
        .eq_maybe(PlanReasonCancelHeader::check_reason_id, dto.check_reason_id);

    let aggregates = repository::get_by_select_internal(select, pool).await?;

    let item_list = aggregates
        .into_iter()
        .map(to_plan_reason_cancel_response_simple)
        .collect();

    Ok(ApiResponse::default().with_data(item_list))
}

async fn export_logic(
    req: ExportRequest,
    user_id: i32,
    token: String,
    broker_adapter: Arc<RabbitAdapter>,
    pool: &PgPool,
) -> MasterDataResult<Option<(Vec<u8>, String)>> {
    let field_list = req.select.field_list.clone();

    let print_doc_service = PrintDocService::new(
        broker_adapter,
        AsezRabbitProperties::default(),
        Source::MasterData,
    );

    let aggregates = match repository::get_by_select(req.select, pool).await {
        Ok(aggregates) => aggregates,
        Err(e) => {
            tracing::error!(
                "Ошибка получения списка причин аннулирования для экспорта: {:?}",
                e
            );
            return Err(e);
        }
    };

    if aggregates.is_empty() {
        return Ok(None);
    }

    let data_records =
        transform_aggregates_to_raw_datarecords(aggregates, &field_list);

    let replacements = get_export_replacements();
    let format = req.format.unwrap_or(TemplateFormat::Xlsx);

    let content = Content {
        extension: format,
        confidentially: false,
        input_content: PrintReq::General(InternalExportReq {
            format: Some(format),
            template: Some("export_table".to_owned()),
            user_id,
            monolith_token: token,
            data: data_records,
            replacements,
        }),
    };

    let response = match print_doc_service.create_document(&content).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Ошибка вызова сервиса print-doc: {}", e);
            return Err(MasterDataError::InternalError(format!(
                "Ошибка генерации файла: {}",
                e
            )));
        }
    };

    let compressed_bytes = response.buf.ok_or_else(|| {
        MasterDataError::InternalError(
            "Сервис генерации не вернул файл".to_string(),
        )
    })?;

    match shared_essential::common::compression::decompress_bzip(&compressed_bytes)
    {
        Ok(file_bytes) => {
            Ok(Some((file_bytes, "plan_reasons_cancel_export.xlsx".to_string())))
        }
        Err(e) => {
            tracing::error!("Ошибка распаковки файла от print-doc: {}", e);
            Err(MasterDataError::InternalError(
                "Ошибка обработки сгенерированного файла".to_string(),
            ))
        }
    }
}

fn get_export_replacements() -> ReplacementConfig {
    [
        replacement!(impact_area_id: nsi_dict(PlanReasonCancelImpactArea) as Text),
        replacement!(
            functionality_id_list: nsi_dict(PlanReasonCancelFunctionality) as Text
        ),
        replacement!(
            check_reason_id: nsi_dict(PlanReasonCancelCheckReason) as Text
        ),
        replacement!(customer_id: planning_common_dict(Customer) as Text),
        replacement!(created_by: planning_dict(Users) as User),
        replacement!(changed_by: planning_dict(Users) as User),
        replacement!(created_at: Date),
        replacement!(changed_at: Date),
        replacement!(is_objective_reason: boolean("Объективная", "Не объективная")),
        replacement!(is_reason_fill_type: boolean("Автоматическое", "Ручное")),
        replacement!(is_new_plan: boolean("Да", "Нет")),
        replacement!(is_removed: boolean("Нет", "Да")), // is_removed: false -> "Да" (актуально), true -> "Нет" (не актуально)
    ]
    .into_iter()
    .collect::<ReplacementConfig>()
}

/// Вспомогательная функция для динамической трансформации агрегатов в `DataRecords`.
fn transform_aggregates_to_raw_datarecords(
    aggregates: Vec<JoinedPlanReasonsCancel>,
    requested_field_list: &[String],
) -> DataRecords {
    let final_field_list: Vec<String> = if requested_field_list.is_empty() {
        [
            PlanReasonCancelHeader::id,
            PlanReasonCancelHeader::text,
            PlanReasonCancelHeader::impact_area_id,
            PlanReasonCancelHeader::is_objective_reason,
            PlanReasonCancelHeader::is_new_plan,
            PlanReasonCancelHeader::is_removed,
            PlanReasonCancelHeader::is_reason_fill_type,
            PlanReasonCancelCustomer::customer_id,
            PlanReasonCancelHeader::functionality_id_list,
            PlanReasonCancelHeader::created_by,
            PlanReasonCancelHeader::created_at,
            PlanReasonCancelHeader::changed_by,
            PlanReasonCancelHeader::changed_at,
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    } else {
        requested_field_list.to_vec()
    };

    let captions: Vec<String> = final_field_list
        .iter()
        .map(|field| {
            match field.as_str() {
                PlanReasonCancelHeader::id => "Код причины",
                PlanReasonCancelHeader::text => "Наименование причины",
                PlanReasonCancelHeader::impact_area_id => "Сфера влияния",
                PlanReasonCancelHeader::is_objective_reason => {
                    "Объективная / не объективная причина"
                }
                PlanReasonCancelHeader::is_new_plan => "Новая ППЗ/ДС",
                PlanReasonCancelHeader::is_reason_fill_type => {
                    "Автоматическое / ручное заполнение причины"
                }
                PlanReasonCancelCustomer::customer_id => "Заказчик",
                PlanReasonCancelHeader::functionality_id_list => "Бизнес-функция",
                PlanReasonCancelHeader::is_removed => "Актуальность",
                PlanReasonCancelHeader::created_by => "Автор создания",
                PlanReasonCancelHeader::created_at => "Дата создания",
                PlanReasonCancelHeader::changed_by => "Автор изменения",
                PlanReasonCancelHeader::changed_at => "Дата изменения",
                PlanReasonCancelHeader::check_reason_id => "Проверки для ППЗ",
                _ => field,
            }
            .to_string()
        })
        .collect();

    let aggregates_len = aggregates.len();

    let data: Vec<DataRecord> = aggregates
        .into_iter()
        .map(|aggregate| {
            let header_rep = PlanReasonCancelHeaderRep::from_item::<&str>(
                aggregate.header,
                None,
            );
            let header_values: AHashMap<String, TaggedValue> = header_rep
                .fields_with_values()
                .into_iter()
                .filter_map(|f| {
                    let field_name = f.field().to_string();
                    f.value.map(|v| (field_name, TaggedValue::from(v)))
                })
                .collect();

            let customer_ids_value = {
                let customer_ids: Vec<i64> = aggregate
                    .customers
                    .iter()
                    .filter(|c| !c.is_removed)
                    .map(|c| c.customer_id as i64)
                    .collect();
                TaggedValue::Vec64(AsezArray(customer_ids))
            };

            final_field_list
                .iter()
                .map(|field_name| {
                    if field_name == PlanReasonCancelCustomer::customer_id {
                        customer_ids_value.clone()
                    } else {
                        header_values
                            .get(field_name)
                            .cloned()
                            .unwrap_or(TaggedValue::Null)
                    }
                })
                .collect::<DataRecord>()
        })
        .collect();

    DataRecords {
        captions,
        field_list: final_field_list,
        data,
        entity_kind: vec![EntityKind::PlanReasonCancel; aggregates_len],
    }
}

#[derive(Debug)]
enum PlanReasonCancelOpMessage {
    Created,
    Updated,
    Deleted,
    Restored,
    Listed,
    NoData,
}

impl BusinessMessage for PlanReasonCancelOpMessage {
    type Entity = JoinedPlanReasonsCancel;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let text = &entity.header.text;
        let msg_text = match self {
            Self::Created => {
                format!("Причина аннулирования '{text}' успешно создана")
            }
            Self::Updated => {
                format!("Причина аннулирования '{text}' успешно изменена")
            }
            Self::Deleted => format!("Причина аннулирования '{text}' удалена"),
            Self::Restored => {
                format!("Причина аннулирования '{text}' восстановлена")
            }
            Self::Listed => "Список причин аннулирования загружен".to_string(),
            Self::NoData => "По указанным критериям записи не найдены".to_string(),
        };
        Message::success(msg_text)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let count = entities.len();

        let msg_text = match self {
            Self::Created => {
                numeric_format!(
                    "Успешно создан{@а|ы} {@count} причин{@а|ы|} аннулирования"
                )
            }
            Self::Updated => {
                numeric_format!(
                    "Успешно изменен{@а|ы} {@count} причин{@а|ы|} аннулирования"
                )
            }
            Self::Deleted => {
                numeric_format!("Удален{@а|ы} {@count} причин{@а|ы|} аннулирования")
            }
            Self::Restored => {
                numeric_format!(
                    "Восстановлен{@а|ы} {@count} причин{@а|ы|} аннулирования"
                )
            }
            Self::Listed => {
                numeric_format!("Найдено {@count} причин{@а|ы|} аннулирования")
            }
            Self::NoData => "По указанным критериям записи не найдены".to_string(),
        };
        Message::success(msg_text)
    }
}
