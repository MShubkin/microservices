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

/// Service that extracts domain identifiers (IDs, UUIDs) from a request.
///
/// For GET requests, ID is extracted from the path parameter.
///
/// For POST requests, IDs and UUIDs are extracted from the request body, using these patterns:
///
/// ```ignore
/// { "id": XXX, "uuid": "YYYYYYYY-YYYY-YYYY-YYYYYYYYYYYY" }
/// ```
///
/// ```ignore
/// { "item_list": [{ "id": XXX, "uuid": "YYY"}]}
/// ```
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

/// Transformation for extracting domain identifiers from a request.
///
/// See [`DomainIDsService`](DomainIDsService) for more details.
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
