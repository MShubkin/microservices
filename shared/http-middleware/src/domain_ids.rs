use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_http::{h1::Payload, HttpMessage, Method};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::{Bytes, Path},
    Error, FromRequest,
};
use futures::future::LocalBoxFuture;
use igg_tracing::tracing_fields::*;
use serde_json::{Map, Value};
use tracing::field::display;
use tracing_actix_web::RootSpan;
use uuid::Uuid;

/// Middleware, которая извлекает числовые идентификаторы бизнес-объектов из запроса
/// и записывает их в `AsezTracingFieldsCollection` и в корневой span для аудита.
///
/// Для GET: id берётся из path-параметра (`/entity/{id}`).
/// Для POST с JSON: парсятся поля `id`/`plan_id`/`uuid` и массив `item_list`.
/// Двухуровневый парсинг нужен потому, что в системе два паттерна:
/// - одиночные операции — `{ "id": 123 }`
/// - групповые операции — `{ "item_list": [{ "id": 1 }, { "id": 2 }] }`
///
/// После чтения тела POST оно восстанавливается обратно в payload через `Payload::create`
/// + `unread_data` — иначе handler-у придёт пустое тело. Это стандартный трюк
/// в Actix когда middleware нужно заглянуть в body без его потребления.
pub struct DomainIDsService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for DomainIDsService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
        + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let (http_req, payload) = req.parts_mut();
            let (mut object_ids, mut object_uuids) = (vec![], vec![]);

            if http_req.method() == Method::GET {
                if let Ok(id) =
                    Path::<i64>::extract(http_req).await.map(|x| x.into_inner())
                {
                    object_ids.push(id);
                }
            } else if http_req.method() == Method::POST
                && &http_req.content_type().to_lowercase() == "application/json"
            {
                let bytes = Bytes::from_request(http_req, payload).await?;
                if let Ok(json) = serde_json::from_slice(&bytes) {
                    extract_ids(&json, &mut object_ids, &mut object_uuids);
                }
                let (_, mut payload) = Payload::create(true);
                payload.unread_data(bytes);
                req.set_payload(payload.into());
            };

            let object_ids = CommaList(object_ids);
            let object_uuids = CommaList(object_uuids);

            if let Some(root_span) = req.extensions().get::<RootSpan>() {
                root_span.record(OBJECT_IDS, &display(&object_ids));
                root_span.record(OBJECT_UUIDS, &display(&object_uuids));
            }

            if let Some(fields) =
                req.extensions_mut().get_mut::<AsezTracingFieldsCollection>()
            {
                fields.set_object_ids(object_ids).set_object_uuids(object_uuids);
            }

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

fn extract_ids(value: &Value, ids: &mut Vec<i64>, uuids: &mut Vec<Uuid>) {
    let Some(value) = value.as_object() else {
        return;
    };
    if let Some(item_list) = value.get("item_list") {
        extract_item_list(item_list, ids, uuids);
    };
    extract_id(value, ids);
    extract_uuid(value, uuids);
}

/// Ищет числовой id по двум именам: `id` — стандартное поле, `plan_id` — историческое
/// поле из старых endpoint-ов, которые ещё не переименованы. Берётся только первое совпадение.
fn extract_id(value: &Map<String, Value>, ids: &mut Vec<i64>) {
    for name in ["id", "plan_id"] {
        if let Some(id) = value.get(name).and_then(Value::as_i64) {
            ids.push(id);
            return;
        }
    }
}

fn extract_uuid(value: &Map<String, Value>, ids: &mut Vec<Uuid>) {
    if let Some(uuid) =
        value.get("uuid").and_then(Value::as_str).and_then(|x| x.parse().ok())
    {
        ids.push(uuid);
    }
}

fn extract_id_uuid(value: &Value, ids: &mut Vec<i64>, uuids: &mut Vec<Uuid>) {
    let Some(value) = value.as_object() else {
        return;
    };
    extract_id(value, ids);
    extract_uuid(value, uuids);
}

fn extract_item_list(item_list: &Value, ids: &mut Vec<i64>, uuids: &mut Vec<Uuid>) {
    let Some(item_list) = item_list.as_array() else {
        return;
    };
    for item in item_list {
        extract_id_uuid(item, ids, uuids);
    }
}

/// Маркер-тип для регистрации через `.wrap()`. Создаёт `DomainIDsService` для каждого worker-а.
pub struct DomainIDsTransform;

impl<S: 'static, B> Transform<S, ServiceRequest> for DomainIDsTransform
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = DomainIDsService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DomainIDsService {
            service: Rc::new(service),
        }))
    }
}
