use actix_web::web::{Json, Query};
use actix_web::{http::header::ContentType, HttpResponse, ResponseError};

use actix_web::{web, Responder};
use monolith_service::http::monolith_token::MonolithToken;
use shared_essential::domain::OrganizationalStructureRep;
use shared_essential::presentation::dto::error::AsezError;
use shared_essential::presentation::dto::master_data::response::{
    DirectoryRecordResponse, SearchResultValue,
};
use shared_essential::{
    domain::enums::master_data::DirectoryType,
    presentation::dto::{
        general::UserId,
        master_data::{
            error::MasterDataError,
            favorites::{FavoriteItemData, FavoriteListData},
            request::{
                CreatePlanReasonsCancelReq, DeleteRestoreIdReq, SearchById,
                SearchByIdReq, SearchByUserInput, SearchPlanReasonCancelReq,
                UpdatePlanReasonsCancelReq,
            },
            response::{
                PlanReasonCancelDeleteRestoreResponse, PlanReasonCancelResponse,
                PlanReasonCancelSearchResponse,
            },
        },
        response_request::{ApiResponse, MessageKind, Messages, PaginatedData},
        AsezResult,
    },
};

use tracing::info;

use crate::application;
use crate::application::action::route_find;
use crate::application::master_data::{
    base::plan_reasons_cancel, org_user_assignment_search, organizational_structure,
};
use crate::presentation::dto::ExportRequest;
use crate::presentation::dto::{
    DicrionaryHierarchyResData, NSIHttpRespose, OrgUserAssigngnmentByIdReqBody,
    OrgUserAssigngnmentReqBody, OrgUserAssignmentResData,
    OrganizationalStructureSearchReqBody, OrganizationalStructureSearchResData,
    RouteFindResData, TestRoutes,
};
use crate::{
    application::{
        directory_get_updates::directory_get_updates,
        directory_search_by_id, directory_search_by_user_input,
        master_data::{favorites, get_master_data},
    },
    infrastructure::GlobalConfig,
};
use shared_essential::presentation::dto::general::UiSelect;
use shared_essential::presentation::dto::master_data::{
    plan_reasons_cancel::PlanReasonCancel, updates::MasterDataUpdates,
};

pub async fn index_root() -> impl Responder {
    "Master Data Service Root. Use '/rest/dictionary/v1' prefix."
}

pub async fn index() -> impl Responder {
    "Master Data Service"
}

pub async fn find_by_id(
    path: web::Path<(String, i32)>,
) -> AsezResult<Json<ApiResponse<DirectoryRecordResponse, ()>>> {
    info!(kind = "infra", "path {:?}", &path);

    let (directory, id) = &path.into_inner();
    let id_array: Vec<i32> = vec![*id];

    do_search_by_ids(directory, &id_array).await
}

pub async fn search_by_ids(
    path: web::Path<String>,
    data: Json<Vec<SearchById>>,
) -> AsezResult<Json<ApiResponse<DirectoryRecordResponse, ()>>> {
    info!(kind = "infra", "data {:?}", &data);

    let directory = &path.into_inner();
    let id_array: Vec<i32> = data.iter().map(|el| el.id).collect();

    do_search_by_ids(directory, &id_array).await
}

async fn do_search_by_ids(
    directory: &str,
    id_array: &[i32],
) -> AsezResult<Json<ApiResponse<DirectoryRecordResponse, ()>>> {
    let mut response = ApiResponse::<_, ()>::default();
    let directory_result = DirectoryType::try_from(directory);
    let master_data = get_master_data()?;

    match directory_result {
        Ok(directory) => {
            let search_result =
                directory_search_by_id(master_data, directory, id_array).await;
            match search_result {
                Ok((messages, record)) => {
                    response.data = record;
                    response.messages = messages;
                }
                Err(error) => {
                    add_messages_from_error(&mut response.messages, &error);
                }
            }
        }
        Err(error) => {
            add_messages_from_error(&mut response.messages, &error);
        }
    }

    Ok(Json(response))
}

pub async fn search(
    path: web::Path<String>,
    data: Json<SearchByUserInput>,
) -> AsezResult<Json<ApiResponse<DirectoryRecordResponse, ()>>> {
    info!(kind = "infra", "data {:?}", &data);

    let directory = &path.into_inner();

    let mut response = ApiResponse::<_, ()>::default();
    let directory_result = DirectoryType::try_from(directory.as_str());
    let master_data = get_master_data()?;

    match directory_result {
        Ok(directory) => {
            let search_result = directory_search_by_user_input(
                master_data,
                directory,
                &data.into_inner(),
            )
            .await;
            match search_result {
                Ok((messages, record)) => {
                    response.data = record;
                    response.messages = messages;
                }
                Err(error) => {
                    add_messages_from_error(&mut response.messages, &error);
                }
            }
        }
        Err(error) => {
            add_messages_from_error(&mut response.messages, &error);
        }
    }
    Ok(Json(response))
}

pub(crate) async fn get_updates(
    path: web::Path<i64>,
    token: MonolithToken,
    config: web::Data<GlobalConfig>,
) -> AsezResult<Json<ApiResponse<MasterDataUpdates, ()>>> {
    info!(kind = "master_data", "get_updates. path {:?}", &path);
    // выдавать только те справочники, что указаны в Swagger
    let res = directory_get_updates(path.into_inner()).await?;
    // используем get_updates как первый сервис, который дергается от имени пользователя
    // чтобы обновить кеши.
    org_user_assignment_search::run_refresh_organizational_user_assignment(
        token.into_inner(),
        &config,
    )
    .await;
    Ok(Json(ApiResponse::default().with_data(res)))
}

fn add_messages_from_error(messages: &mut Messages, error: &MasterDataError) {
    messages.add_message(MessageKind::Error, format!("{}", error));
}

pub(crate) async fn get_favorite_list(
    user_id: Query<UserId>,
    config: web::Data<GlobalConfig>,
) -> AsezResult<Json<ApiResponse<FavoriteListData, ()>>> {
    let pool = &config.db_pool;
    let res = favorites::get_favorite_list(user_id.user_id, pool).await?;
    Ok(Json(ApiResponse::default().with_data(res)))
}

pub(crate) async fn create_favorite_item(
    user_id: Query<UserId>,
    data: Json<FavoriteItemData>,
    config: web::Data<GlobalConfig>,
) -> AsezResult<Json<ApiResponse<(), ()>>> {
    let pool = &config.db_pool;
    favorites::create_favorite_item(user_id.user_id, data.into_inner(), pool)
        .await?;
    Ok(Json(ApiResponse::default()))
}

pub(crate) async fn delete_favorite_item(
    user_id: Query<UserId>,
    data: Json<FavoriteItemData>,
    config: web::Data<GlobalConfig>,
) -> AsezResult<Json<ApiResponse<Option<FavoriteItemData>, ()>>> {
    let pool = &config.db_pool;
    let res =
        favorites::delete_favorite_item(user_id.user_id, data.into_inner(), pool)
            .await
            .map(Some)?;
    Ok(Json(ApiResponse::default().with_data(res)))
}

pub async fn search_organizational_user_assignment_by_id(
    Json(data): Json<OrgUserAssigngnmentByIdReqBody>,
) -> AsezResult<Json<ApiResponse<OrgUserAssignmentResData, ()>>> {
    let res = org_user_assignment_search::organizational_user_assignment_by_id(
        data.iter(),
    )
    .await?;
    Ok(Json(ApiResponse::default().with_data(res)))
}

pub(crate) async fn search_organizational_user_assignment(
    Query(UserId { user_id }): Query<UserId>,
    Json(data): Json<OrgUserAssigngnmentReqBody>,
) -> AsezResult<Json<ApiResponse<OrgUserAssignmentResData, ()>>> {
    let res = org_user_assignment_search::organizational_user_assignment(
        user_id,
        &data.search,
        data.organization_structure_id,
        data.from as usize,
        data.quantity as usize,
    )
    .await?;
    Ok(Json(ApiResponse::default().with_data(res)))
}

pub(crate) async fn organizational_structure_search(
    data: web::Json<OrganizationalStructureSearchReqBody>,
    user_id: Query<UserId>,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<OrganizationalStructureSearchResData> {
    let value = organizational_structure::organizational_structure_search(
        user_id.into_inner().user_id,
        data.into_inner(),
        &config.db_pool,
    )
    .await?;
    Ok(web::Json(ApiResponse::default().with_data(value)))
}

pub(crate) async fn organization_structure_search_by_id(
    data: web::Json<SearchByIdReq>,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<SearchResultValue<OrganizationalStructureRep>> {
    let value =
        organizational_structure::search_by_id(data.iter(), &config.db_pool)
            .await?;
    Ok(web::Json(ApiResponse::default().with_data(SearchResultValue { value })))
}

pub(crate) async fn plan_reasons_cancel_get_item_list(
    data: web::Json<UiSelect>,
    token: MonolithToken,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<PaginatedData<PlanReasonCancel>> {
    let response = plan_reasons_cancel::get_item_list(
        data.into_inner(),
        &config.monolith,
        token.into_inner(),
        &config.db_pool,
    )
    .await?;
    Ok(web::Json(response.into()))
}

pub(crate) async fn plan_reasons_cancel_search(
    data: web::Json<SearchPlanReasonCancelReq>,
    token: MonolithToken,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<PlanReasonCancelSearchResponse> {
    let response = plan_reasons_cancel::search(
        &data.into_inner(),
        &config.monolith,
        token.into_inner(),
        &config.db_pool,
    )
    .await?;
    Ok(web::Json(response.into()))
}

pub(crate) async fn plan_reasons_cancel_search_by_id(
    data: web::Json<SearchByIdReq>,
    token: MonolithToken,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<PlanReasonCancelSearchResponse> {
    let response = plan_reasons_cancel::search_by_id(
        &data.into_inner(),
        &config.monolith,
        token.into_inner(),
        &config.db_pool,
    )
    .await?;
    Ok(web::Json(response.into()))
}

pub(crate) async fn plan_reasons_cancel_get_detail(
    data: web::Json<SearchById>,
    token: MonolithToken,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<PlanReasonCancelResponse> {
    let response = plan_reasons_cancel::get_detail(
        data.id,
        &config.monolith,
        token.into_inner(),
        &config.db_pool,
    )
    .await?;
    Ok(web::Json(response.into()))
}

pub(crate) async fn plan_reasons_cancel_create(
    data: web::Json<CreatePlanReasonsCancelReq>,
    user_id: Query<UserId>,
    config: web::Data<GlobalConfig>,
    token: MonolithToken,
) -> NSIHttpRespose<PlanReasonCancelResponse> {
    let response = plan_reasons_cancel::create_item(
        data.into_inner(),
        user_id.user_id,
        &config.monolith,
        token.into_inner(),
        &config.db_pool,
    )
    .await?;
    Ok(web::Json(response.into()))
}

pub(crate) async fn plan_reasons_cancel_update(
    data: web::Json<UpdatePlanReasonsCancelReq>,
    user_id: Query<UserId>,
    config: web::Data<GlobalConfig>,
    token: MonolithToken,
) -> NSIHttpRespose<PlanReasonCancelResponse> {
    let response = plan_reasons_cancel::update_item(
        data.into_inner(),
        user_id.user_id,
        &config.monolith,
        token.into_inner(),
        &config.db_pool,
    )
    .await?;
    Ok(web::Json(response.into()))
}

pub(crate) async fn plan_reasons_cancel_delete(
    data: web::Json<DeleteRestoreIdReq>,
    user_id: Query<UserId>,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<PlanReasonCancelDeleteRestoreResponse> {
    let ids: Vec<i32> = data.item_list.iter().map(|i| i.id).collect();

    let response =
        plan_reasons_cancel::delete_items(&ids, user_id.user_id, &config.db_pool)
            .await?;

    Ok(web::Json(response.into()))
}

pub(crate) async fn plan_reasons_cancel_restore(
    data: web::Json<SearchById>,
    user_id: Query<UserId>,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<PlanReasonCancelDeleteRestoreResponse> {
    let response = plan_reasons_cancel::restore_items(
        &[data.id],
        user_id.user_id,
        &config.db_pool,
    )
    .await?;

    Ok(web::Json(response.into()))
}

/// Экспорт справочника "Причины аннулирования" в XLSX.
pub(crate) async fn export_plan_reasons_cancel(
    config: web::Data<GlobalConfig>,
    user_id: Query<UserId>,
    token: MonolithToken,
    req: Json<ExportRequest>,
) -> HttpResponse {
    let result = plan_reasons_cancel::export(
        req.into_inner(),
        user_id.user_id,
        token.into_inner(),
        config.broker_adapter.clone(),
        &config.db_pool,
    )
    .await;

    match result {
        Ok(Some((file_bytes, file_name))) => HttpResponse::Ok()
            .content_type(ContentType::octet_stream())
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", file_name),
            ))
            .body(file_bytes),
        Ok(None) => HttpResponse::NoContent().finish(),
        Err(e) => {
            let asez_error: AsezError = e.into();
            asez_error.error_response()
        }
    }
}

/// API для иерархии многоуровневых справочников.
pub(crate) async fn get_hierarchy(
    path: web::Path<String>,
) -> NSIHttpRespose<DicrionaryHierarchyResData> {
    let data =
        application::hierarchical_values::get_hierarchy(&path.into_inner()).await?;
    Ok(web::Json(ApiResponse::default().with_data(data)))
}

pub(crate) async fn test_routes(
    Json(body): web::Json<TestRoutes>,
    config: web::Data<GlobalConfig>,
) -> NSIHttpRespose<RouteFindResData> {
    let result = route_find(body, &config.db_pool).await?;
    Ok(web::Json(result))
}
