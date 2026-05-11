use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbItem;
use format_tools::numeric_format;
use shared_essential::{
    domain::routes::RouteHeader,
    presentation::dto::{
        master_data::{
            error::MasterDataResult, request::RouteStopReq,
            response::RouteStopResponse,
        },
        response_request::{ApiResponse, BusinessMessage, Message, Messages},
    },
};
use sqlx::PgPool;

use crate::application::routes::RoutesNotFound;

#[derive(Debug)]
pub enum RouteStopMessage {
    AlreadyInactive,
    Deactivated,
}

pub async fn route_stop(
    dto: RouteStopReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteStopResponse, ()>> {
    let mut tx = pool.begin().await?;

    let route_select = Select::with_fields([
        RouteHeader::id,
        RouteHeader::uuid,
        RouteHeader::type_id,
        RouteHeader::is_active,
    ])
    .in_any(RouteHeader::uuid, &dto);
    let routes = RouteHeader::select(&route_select, &mut tx).await?;

    let (will_be_deactivated, already_inactive, not_found) =
        routes.into_iter().fold(
            (Vec::new(), Vec::new(), RoutesNotFound::from_iter(dto)),
            |(mut will_be_deactivated, mut already_inactive, mut not_found),
             mut r| {
                not_found.found(&r.uuid);
                if r.is_active {
                    r.is_active = false;
                    will_be_deactivated.push(r);
                } else {
                    already_inactive.push(r);
                }
                (will_be_deactivated, already_inactive, not_found)
            },
        );
    RouteHeader::update_vec(
        &will_be_deactivated,
        Some(&[RouteHeader::is_active]),
        &mut tx,
    )
    .await?;

    tx.commit().await?;

    let mut messages = Messages::default();
    not_found.append(&mut messages);
    RouteStopMessage::AlreadyInactive
        .checked_append(&mut messages, &already_inactive);
    RouteStopMessage::Deactivated
        .checked_append(&mut messages, &will_be_deactivated);

    Ok(ApiResponse::default().with_messages(messages))
}

impl BusinessMessage for RouteStopMessage {
    type Entity = RouteHeader;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            RouteStopMessage::AlreadyInactive => {
                Message::error(format!("Маршрут {} уже остановлен", entity.id))
            }
            RouteStopMessage::Deactivated => {
                Message::success(format!("Маршрут {} остановлен", entity.id))
            }
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let count = entities.len();
        let msg = match self {
            RouteStopMessage::AlreadyInactive => Message::error(numeric_format!(
                "{@count} маршрут{@|а|ов} уже остановлен{@|ы}"
            )),
            RouteStopMessage::Deactivated => Message::success(numeric_format!(
                "{@count} маршрут{@|а|ов} остановлен{@|ы}"
            )),
        };

        msg.with_param_items(entities)
    }
}
