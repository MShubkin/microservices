use crate::application::routes::fetch_routes;
use asez2_shared_db::db_item::{AsezTimestamp, Select};
use asez2_shared_db::DbItem;
use shared_essential::domain::routes::{RouteCrit, RouteFull, RouteHeader};
use shared_essential::presentation::dto::master_data::error::MasterDataError;
use shared_essential::presentation::dto::master_data::request::RouteCopyReq;
use shared_essential::presentation::dto::master_data::response::RouteCopyResponse;
use shared_essential::presentation::dto::response_request::{Message, Messages};
use shared_essential::presentation::dto::{
    master_data::error::MasterDataResult, response_request::ApiResponse,
};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

pub async fn route_copy(
    RouteCopyReq {
        uuid,
        name_short,
        user_id,
    }: RouteCopyReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteCopyResponse, ()>> {
    let select = Select::full::<RouteHeader>().eq(RouteHeader::uuid, uuid);
    let mut routes = fetch_routes(select, pool).await?;

    let mut messages = Messages::default();
    let Some(route) = routes.pop() else {
        RouteCopyMessage::NotFound(uuid).checked_append(&mut messages);
        return Ok(ApiResponse::default().with_messages(messages));
    };
    let now = AsezTimestamp::now();
    let RouteFull {
        mut route,
        mut crits,
        mut data,
    } = route;

    route.uuid = Uuid::new_v4();
    route.created_at = now;
    route.changed_at = now;
    route.created_by = user_id;
    route.changed_by = user_id;
    route.name_short = Some(name_short);
    route.is_active = false;

    let mut tx = pool.begin().await?;

    let new_header = route.insert_returning(&mut tx).await?;
    if !crits.is_empty() {
        crits.iter_mut().for_each(|crit| {
            crit.route_uuid = new_header.uuid;
            crit.created_at = now;
            crit.changed_at = now;
            crit.created_by = user_id;
            crit.changed_by = user_id;
        });
        RouteCrit::insert_vec(&mut crits, &mut tx).await?;
    }

    data.route_uuid = new_header.uuid;
    data.created_at = now;
    data.changed_at = now;
    data.created_by = user_id;
    data.changed_by = user_id;
    data.insert(&mut tx).await?;

    tx.commit().await?;

    RouteCopyMessage::Ok(new_header.id).checked_append(&mut messages);

    let response = ApiResponse::<Option<i64>, ()> {
        data: Some(new_header.id),
        messages,
        ..Default::default()
    };

    Ok(response)
}
/// Сообщения об ошибках при копировании маршрута
#[derive(Error, Debug)]
pub enum RouteCopyMessage {
    #[error("Маршрут с uuid : {0} не найден")]
    NotFound(Uuid),
    #[error("Операция copy не вернула заголовок")]
    CopyFailed,
    #[error("Маршрут скопирован. Номер созданного маршрута {0}")]
    Ok(i64),
}

impl RouteCopyMessage {
    pub fn checked_append(&self, messages: &mut Messages) {
        match self {
            RouteCopyMessage::NotFound(_) | RouteCopyMessage::CopyFailed => {
                messages.add_prepared_message(Message::error(self.to_string()));
            }
            RouteCopyMessage::Ok(_) => {
                messages.add_prepared_message(Message::success(self.to_string()));
            }
        }
    }
}

impl From<RouteCopyMessage> for MasterDataError {
    fn from(value: RouteCopyMessage) -> Self {
        MasterDataError::InternalError(format!(
            "Ошибка копирования маршрута: {value}"
        ))
    }
}
