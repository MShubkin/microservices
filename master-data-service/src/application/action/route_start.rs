use format_tools::numeric_format;
use sqlx::{PgPool, Postgres, Transaction};

use asez2_shared_db::{db_item::Select, DbItem};
use shared_essential::{
    domain::routes::{RouteApprType, RouteHeader},
    presentation::dto::{
        master_data::{
            error::MasterDataResult as Result, request::RouteStartReq,
            response::RouteStartResponse,
        },
        response_request::{ApiResponse, BusinessMessage, Message, Messages},
    },
};

use crate::application::routes::{
    check_duplicated_routes, params_from_routes, CoincidentRoute, RoutesNotFound,
};

pub(crate) async fn route_start(
    dto: RouteStartReq,
    pool: &PgPool,
) -> Result<ApiResponse<RouteStartResponse, ()>> {
    match dto.type_id {
        RouteApprType::SpecializedDepartments => route_start_pd(dto, pool).await,
        _ => route_start_acsk(dto, pool).await,
    }
}

async fn route_start_acsk(
    dto: RouteStartReq,
    pool: &PgPool,
) -> Result<ApiResponse<RouteStartResponse, ()>> {
    let mut tx = pool.begin().await?;
    let mut messages = Messages::default();

    let mut will_be_activated =
        get_and_split_routes(dto, &mut messages, &mut tx).await?;

    will_be_activated.iter_mut().for_each(|i| {
        i.is_active = true;
    });
    RouteHeader::update_vec(
        &will_be_activated,
        Some(&[RouteHeader::is_active]),
        &mut tx,
    )
    .await?;

    tx.commit().await?;

    RouteStartMessage::Activated.checked_append(&mut messages, &will_be_activated);

    Ok(messages.into())
}

async fn route_start_pd(
    dto: RouteStartReq,
    pool: &PgPool,
) -> Result<ApiResponse<RouteStartResponse, ()>> {
    let mut tx = pool.begin().await?;
    let mut messages = Messages::default();

    let to_activate = get_and_split_routes(dto, &mut messages, &mut tx).await?;

    let (mut will_be_activated, duplicated_routes, coincident_routes) =
        check_duplicated_routes(to_activate.iter().map(|i| i.uuid), true, &mut tx)
            .await?;
    RouteStartMessage::SimilarActiveFound(coincident_routes)
        .checked_append(&mut messages, &duplicated_routes);

    will_be_activated.iter_mut().for_each(|i| {
        i.is_active = true;
    });
    RouteHeader::update_vec(
        &will_be_activated,
        Some(&[RouteHeader::is_active]),
        &mut tx,
    )
    .await?;

    tx.commit().await?;

    RouteStartMessage::Activated.checked_append(&mut messages, &will_be_activated);

    Ok(messages.into())
}

// Заранее генерируем сообщения
// Если уже есть активные маршруты, которые пользователь хочет опять активировать,
// то надо высветить предупреждение об этом
async fn get_and_split_routes(
    dto: RouteStartReq,
    messages: &mut Messages,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<RouteHeader>> {
    let route_select = Select::with_fields([
        RouteHeader::id,
        RouteHeader::uuid,
        RouteHeader::type_id,
        RouteHeader::is_active,
    ])
    .in_any(RouteHeader::uuid, &dto.items)
    .eq(RouteHeader::is_removed, false)
    .eq(RouteHeader::type_id, dto.type_id);
    let routes = RouteHeader::select(&route_select, tx).await?;

    // Заранее генерируем сообщения
    // Если уже есть активные маршруты, которые пользователь хочет опять активировать,
    // то надо высветить предупреждение об этом
    let (to_activate, already_active, not_found) = routes.into_iter().fold(
        (Vec::new(), Vec::new(), RoutesNotFound::from_iter(dto.items)),
        |(mut to_activate, mut already_active, mut not_found), r| {
            not_found.found(&r.uuid);
            if !r.is_active {
                to_activate.push(r)
            } else {
                already_active.push(r)
            }

            (to_activate, already_active, not_found)
        },
    );

    not_found.append(messages);
    RouteStartMessage::AlreadyActive.checked_append(messages, &already_active);

    Ok(to_activate)
}

#[derive(Debug)]
pub(crate) enum RouteStartMessage {
    Activated,
    AlreadyActive,
    /// Кто то придумал так делать:
    /// "​<системный номер запускаемого маршрута1> - <системный номер совпавшего маршрута1>"
    SimilarActiveFound(Vec<CoincidentRoute>),
}

impl BusinessMessage for RouteStartMessage {
    type Entity = RouteHeader;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let id = entity.id;
        match self {
            Self::SimilarActiveFound(routes) => {
                let params = params_from_routes(routes);
                // Как минимум один совпадающий элемент должен быть, иначе
                // сообщение не имеет смысла
                let text = match &routes.get(0).map(|lonely| &lonely.others[..]) {
                    Some([single]) => {
                        format!("Маршрут {id} не запущен. Найден совпадающий маршрут: {}", single)
                    }
                    _ => format!(
                        "Маршрут {id} не запущен. Найдены совпадающие маршруты"
                    ),
                };
                Message::error(text).with_parameters(params)
            }
            Self::AlreadyActive => {
                Message::error(format!("Маршрут {id} уже запущен"))
                    .with_param_item(entity)
            }
            Self::Activated => Message::success(format!("Маршрут {id} запущен"))
                .with_param_item(entity),
        }
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let count = entities.len();
        match self {
            Self::SimilarActiveFound(routes) => {
                let params = params_from_routes(routes);

                Message::error(numeric_format!(
                    "{@count} маршрут{@|а|ов} не запущен{@|ы}. Найдены совпадающие маршруты"
                ))
                .with_parameters(params)
            }
            Self::AlreadyActive => Message::error(numeric_format!(
                "{@count} маршрут{@|а|ов} уже запущен{@|ы}"
            ))
            .with_param_items(entities),
            Self::Activated => Message::success(numeric_format!(
                "{@count} маршрут{@|а|ов} запущен{@|ы}"
            ))
            .with_param_items(entities),
        }
    }
}
