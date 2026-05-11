use asez2_shared_db::{
    db_item::{AsezTimestamp, Select},
    DbAdaptor, DbItem,
};
use shared_essential::{
    domain::routes::{RouteCrit, RouteData, RouteHeader, RouteHeaderRep},
    presentation::dto::{
        master_data::{
            error::{MasterDataError, MasterDataResult},
            request::RouteUpdateReq,
            response::RouteUpdateResponse,
            routes::{
                try_from_route_criteria, RouteCriterion, TryFromRouteCriterionError,
            },
        },
        response_request::{ApiResponse, BusinessMessage, Message, Messages},
    },
};
use sqlx::{types::Json, PgPool};

use crate::application::routes::{
    check_duplicated_routes, params_from_routes, CoincidentRoute,
};

#[derive(Debug, thiserror::Error)]
enum RouteUpdateError {
    #[error("ошибка преобразования критерия маршрута: {0}")]
    Conv(#[from] TryFromRouteCriterionError),
}

impl From<RouteUpdateError> for MasterDataError {
    fn from(value: RouteUpdateError) -> Self {
        MasterDataError::InternalError(format!(
            "ошибка обновления маршрута: {value}"
        ))
    }
}

#[derive(Debug)]
pub(crate) enum RouteUpdateMessage {
    SimilarActiveFound(CoincidentRoute),
    Ok,
}

impl BusinessMessage for RouteUpdateMessage {
    type Entity = RouteHeader;

    fn singular(&self, entity: &Self::Entity) -> Message {
        let id = entity.id;

        match self {
            Self::SimilarActiveFound(route) => {
                let params = params_from_routes(&[route.clone()]);
                // Как минимум один совпадающий элемент должен быть, иначе
                // сообщение не имеет смысла
                let text = match route.others[..] {
                    [single] => {
                        format!("Маршрут {id} не обновлен. Найден совпадающий маршрут: {}", single)
                    }
                    _ => format!(
                        "Маршрут {id} не обновлен. Найдены совпадающие маршруты"
                    ),
                };
                Message::error(text).with_parameters(params)
            }
            Self::Ok => Message::success(format!("Маршрут {id} успешно сохранен"))
                .with_param_item(entity),
        }
    }

    fn plural<T>(&self, _entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>,
    {
        unreachable!("only single route is updated")
    }
}

const ROUTE_HEADER_UPDATE_FIELDS: &[&str] = &[
    RouteHeader::name_short,
    RouteHeader::is_active,
    RouteHeader::changed_by,
    RouteHeader::changed_at,
];
const ROUTE_UPDATE_RETURN_FIELDS: &[&str] =
    &[RouteHeader::uuid, RouteHeader::id, RouteHeader::type_id];

const ROUTE_CRIT_UPDATE_FIELDS: &[&str] = &[
    RouteCrit::route_uuid,
    RouteCrit::field_name,
    RouteCrit::predicate,
    RouteCrit::is_removed,
    RouteCrit::changed_at,
    RouteCrit::changed_by,
];
const ROUTE_DATA_UPDATE_FIELDS: &[&str] = &[
    RouteData::route_uuid,
    RouteData::data,
    RouteData::changed_at,
    RouteData::changed_by,
];

const ROUTE_RETURN_FIELDS: &[&str] = &[RouteHeader::uuid, RouteHeader::id];

pub async fn route_update(
    dto: RouteUpdateReq,
    pool: &PgPool,
) -> MasterDataResult<ApiResponse<RouteUpdateResponse, ()>> {
    let RouteUpdateReq {
        user_id,
        header,
        criteria,
        data,
    } = dto;
    let now = AsezTimestamp::now();

    let mut header = header.into_item()?;
    let route_uuid = header.uuid;

    header.changed_at = now;
    header.changed_by = user_id;

    let data = RouteData {
        route_uuid,
        data: Json(Some(data)),
        changed_at: header.changed_at,
        changed_by: header.changed_by,
        ..Default::default()
    };

    let mut tx = pool.begin().await?;
    let mut messages = Messages::default();

    // При неактивном маршруте не должно быть проверки на совпадающие
    if header.is_active {
        let (_, duplicated_route, mut coincident_routes) =
            check_duplicated_routes([route_uuid].into_iter(), false, &mut tx)
                .await?;
        // Обновляется только один маршрут, соответсвенно одна запись в coincident_routes
        if let (Some(r), Some(c)) =
            (duplicated_route.first(), coincident_routes.pop())
        {
            messages.add_prepared_message(
                RouteUpdateMessage::SimilarActiveFound(c).singular(r),
            )
        };
    }

    if messages.is_error() {
        return Err(MasterDataError::Business(messages));
    }

    let mut crits_to_remove = RouteCrit::select(
        &Select::with_fields(ROUTE_CRIT_UPDATE_FIELDS)
            .eq(RouteCrit::route_uuid, header.uuid)
            .not_in_any(RouteCrit::field_name, criteria.keys()),
        &mut tx,
    )
    .await?;

    let crits_to_update = criteria
        .into_iter()
        .map(|(field, crits)| from_crits(field, crits, &header))
        .collect::<Result<Vec<_>, _>>()?;
    crits_to_remove.iter_mut().for_each(|c| {
        c.is_removed = true;
        c.changed_at = header.changed_at;
        c.changed_by = header.changed_by
    });

    let updated = header
        .update_returning(
            Some(ROUTE_HEADER_UPDATE_FIELDS),
            Some(ROUTE_UPDATE_RETURN_FIELDS),
            &mut tx,
        )
        .await?;

    let merged_crits =
        crits_to_remove.into_iter().chain(crits_to_update).collect::<Vec<_>>();
    RouteCrit::update_vec(&merged_crits, Some(ROUTE_CRIT_UPDATE_FIELDS), &mut tx)
        .await?;
    data.update(Some(ROUTE_DATA_UPDATE_FIELDS), &mut tx).await?;

    RouteUpdateMessage::Ok.checked_append(&mut messages, &[&updated]);

    let updated = RouteHeaderRep::from_item(updated, Some(ROUTE_RETURN_FIELDS));

    tx.commit().await?;

    Ok((RouteUpdateResponse::from(updated), messages).into())
}

fn from_crits(
    field_name: String,
    criteria: Vec<RouteCriterion>,
    header: &RouteHeader,
) -> Result<RouteCrit, RouteUpdateError> {
    let predicate = try_from_route_criteria(&field_name, criteria).map(Json)?;
    Ok(RouteCrit {
        route_uuid: header.uuid,
        field_name,
        predicate,
        is_removed: false,
        changed_at: header.changed_at,
        changed_by: header.changed_by,
        ..Default::default()
    })
}
