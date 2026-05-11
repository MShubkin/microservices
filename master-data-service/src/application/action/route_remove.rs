use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbItem;
use shared_essential::{
    domain::routes::RouteHeader,
    presentation::dto::{
        master_data::{
            error::MasterDataResult, request::RouteRemoveReq,
            response::RouteRemoveResponse,
        },
        response_request::{ApiResponse, BusinessMessage, Message, Messages},
    },
};
use sqlx::PgPool;

use crate::application::routes::RoutesNotFound;

#[derive(Debug)]
pub enum RouteRemoveMessage {
    IsActive,
    AlreadyRemoved,
    Success,
}

pub async fn route_remove(
    dto: RouteRemoveReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteRemoveResponse, ()>> {
    let mut tx = pool.begin().await?;

    let route_select = Select::with_fields([
        RouteHeader::id,
        RouteHeader::uuid,
        RouteHeader::type_id,
        RouteHeader::is_active,
        RouteHeader::is_removed,
    ])
    .in_any(RouteHeader::uuid, &dto);
    let routes = RouteHeader::select(&route_select, &mut tx).await?;

    let (to_remove, active, removed, not_found) = routes.into_iter().fold(
        (Vec::new(), Vec::new(), Vec::new(), RoutesNotFound::from_iter(dto)),
        |(mut to_remove, mut active, mut removed, mut not_found), mut r| {
            not_found.found(&r.uuid);
            match (r.is_removed, r.is_active) {
                (true, _) => removed.push(r),
                (false, true) => active.push(r),
                (false, false) => {
                    r.is_removed = true;
                    to_remove.push(r)
                }
            }
            (to_remove, active, removed, not_found)
        },
    );

    RouteHeader::update_vec(&to_remove, Some(&[RouteHeader::is_removed]), &mut tx)
        .await?;

    tx.commit().await?;

    let mut messages = Messages::default();
    not_found.append(&mut messages);
    RouteRemoveMessage::IsActive.checked_append(&mut messages, &active);
    RouteRemoveMessage::AlreadyRemoved.checked_append(&mut messages, &removed);

    RouteRemoveMessage::Success.checked_append(&mut messages, &to_remove);

    Ok(ApiResponse::default().with_messages(messages))
}

impl BusinessMessage for RouteRemoveMessage {
    type Entity = RouteHeader;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let msg = match self {
            Self::IsActive => {
                Message::error(String::from("Нельзя удалить запущенный маршрут"))
            }
            Self::AlreadyRemoved => {
                Message::error(format!("Маршрут {} уже удален", entity.id))
            }
            Self::Success => {
                Message::success(format!("Маршрут {} удален", entity.id))
            }
        };

        msg.with_param_item(entity)
    }

    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        let case = match entities.len() {
            ..=4 => "маршрута",
            _ => "маршрутов",
        };
        let msg = match self {
            Self::IsActive => {
                Message::error(String::from("Нельзя удалить запущенные маршруты"))
            }
            Self::AlreadyRemoved => {
                Message::error(format!("{} {} уже удалено", entities.len(), case))
            }
            Self::Success => {
                Message::success(format!("{} {} удалено", entities.len(), case))
            }
        };

        msg.with_param_items(entities)
    }
}
