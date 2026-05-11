use asez2_shared_db::{db_item::AsezTimestamp, DbAdaptor, DbItem};
use shared_essential::{
    domain::routes::{RouteCrit, RouteData, RouteHeader, RouteHeaderRep},
    presentation::dto::{
        master_data::{
            error::{MasterDataError, MasterDataResult},
            request::RouteCreateReq,
            response::RouteCreateResponse,
            routes::{
                try_from_route_criteria, RouteCriterion, TryFromRouteCriterionError,
            },
        },
        response_request::{
            ApiResponse, ApiResponseDataWrapper, BusinessMessage, Message, Messages,
        },
    },
};
use sqlx::{types::Json, PgPool};

#[derive(Debug, thiserror::Error)]
enum RouteCreateError {
    #[error("отсутствует тип маршрута")]
    NoType,
    #[error("ошибка преобразования критерия: {0}")]
    Conv(#[from] TryFromRouteCriterionError),
}

impl From<RouteCreateError> for MasterDataError {
    fn from(value: RouteCreateError) -> Self {
        MasterDataError::InternalError(format!("Ошибка создания маршрута: {value}"))
    }
}

#[derive(Debug)]
pub enum RouteCreateMessage {
    Ok,
}

impl BusinessMessage for RouteCreateMessage {
    type Entity = RouteHeader;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let id = entity.id;
        Message::success(format!(
            "Маршрут {id} сохранен\nДля запуска маршрута нажмите на кнопку \"Запустить маршрут\""
        ))
        .with_param_item(entity)
    }

    fn plural<T>(&self, _entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        todo!()
    }
}

const ROUTE_RETURN_FIELDS: &[&str] = &[RouteHeader::uuid, RouteHeader::id];

pub async fn route_create(
    dto: RouteCreateReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteCreateResponse, ()>> {
    let RouteCreateReq {
        user_id,
        header,
        criteria,
        data,
    } = dto;
    let mut messages = Messages::default();

    if header.type_id.is_none() {
        return Err(RouteCreateError::NoType.into());
    }

    let now = AsezTimestamp::now();

    let mut header = header.into_item()?;
    header.created_at = now;
    header.changed_at = now;
    header.created_by = user_id;
    header.changed_by = user_id;

    let mut tx = pool.begin().await?;

    let created = header.insert_returning(&mut tx).await?;

    let mut crits_to_create = criteria
        .into_iter()
        .map(|(field, crits)| from_crits(field, crits, &created))
        .collect::<Result<Vec<_>, _>>()?;
    RouteCrit::insert_vec(&mut crits_to_create, &mut tx).await?;

    let mut data = RouteData {
        route_uuid: created.uuid,
        data: Json(Some(data)),
        created_at: now,
        changed_at: now,
        created_by: user_id,
        changed_by: user_id,
    };
    data.insert(&mut tx).await?;

    RouteCreateMessage::Ok.checked_append(&mut messages, &[&created]);

    let created = ApiResponseDataWrapper::from(RouteHeaderRep::from_item(
        created,
        Some(ROUTE_RETURN_FIELDS),
    ));

    tx.commit().await?;

    Ok((created, messages).into())
}

fn from_crits(
    field_name: String,
    criteria: Vec<RouteCriterion>,
    header: &RouteHeader,
) -> Result<RouteCrit, RouteCreateError> {
    let predicate = try_from_route_criteria(&field_name, criteria).map(Json)?;
    Ok(RouteCrit {
        route_uuid: header.uuid,
        field_name,
        predicate,
        is_removed: false,
        created_at: header.created_at,
        changed_at: header.changed_at,
        created_by: header.created_by,
        changed_by: header.changed_by,
    })
}
