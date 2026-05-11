use asez2_shared_db::{db_item::Select, DbAdaptor};
use shared_essential::{
    domain::routes::{RouteFull, RouteHeader, RouteHeaderRep},
    presentation::dto::{
        master_data::{
            error::MasterDataResult, request::RouteDetailsReq, response::*,
            routes::TryFromRouteCritError,
        },
        response_request::{ApiResponse, Message, Messages},
    },
};
use sqlx::PgPool;

use crate::application::routes::fetch_routes;

const ROUTE_HEADER_FIELDS: &[&str] = &[
    RouteHeader::type_id,
    RouteHeader::uuid,
    RouteHeader::id,
    RouteHeader::name_short,
    RouteHeader::is_exception,
    RouteHeader::is_removed,
    RouteHeader::is_active,
    RouteHeader::created_at,
    RouteHeader::created_by,
    RouteHeader::changed_at,
    RouteHeader::changed_by,
];

/// Сообщения об ошибках при обработке маршрутов
#[derive(Debug)]
pub enum RouteListMessage {
    NotFound(i64),
    Conversion(TryFromRouteCritError),
}

impl RouteListMessage {
    pub fn checked_append(&self, messages: &mut Messages) {
        messages.add_prepared_message(self.to_message());
    }

    pub fn to_message(&self) -> Message {
        match self {
            RouteListMessage::NotFound(id) => {
                Message::error(format!("Маршрут с идентификатором {id} не найден"))
            }
            RouteListMessage::Conversion(error) => Message::error(format!(
                "Ошибка преобразования критерия маршрута: {error}"
            )),
        }
    }
}

/// Функция получение маршрутов согласования в разрезе Департамента
///
/// # Аргументы
/// - `dto`: Параметры запроса, включая секцию, пользователя и фильтры.
/// - `pool`: Пул соединений с БД.
///
/// # Возвращает
/// - `ApiResponse` с данными о маршрутах или сообщением об ошибке.
///
/// TODO: добавить фильтрацию и сортировку
pub async fn get_route_details(
    dto: RouteDetailsReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteDetailsResponse, ()>> {
    let RouteDetailsReq { route_id } = dto;

    let select = prepare_select(route_id);
    let mut routes = fetch_routes(select, pool).await?;

    let mut messages = Messages::default();
    let Some(route) = routes.pop() else {
        RouteListMessage::NotFound(route_id).checked_append(&mut messages);
        return Ok(ApiResponse::default().with_messages(messages));
    };

    let data = into_route_item(route, &mut messages)?.unwrap_or_default();

    Ok((data, messages).into())
}

fn into_route_item(
    route: RouteFull,
    messages: &mut Messages,
) -> MasterDataResult<Option<RouteDetailsResponse>> {
    let RouteFull { route, crits, data } = route;
    let route = RouteHeaderRep::from_item(route, Some(ROUTE_HEADER_FIELDS));
    let data = data.data.0;
    Ok(match RouteDetailsResponse::try_from((route, crits, data)) {
        Ok(v) => Some(v),
        Err(err) => {
            RouteListMessage::Conversion(err).checked_append(messages);
            None
        }
    })
}

fn prepare_select(route_id: i64) -> Select {
    let mut select = Select::default().eq(RouteHeader::id, route_id);
    select.field_list.retain(|x| ROUTE_HEADER_FIELDS.contains(&x.as_str()));
    select
}
