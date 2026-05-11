use asez2_shared_db::db_item::{Select, SelectionKind};
use asez2_shared_db::result::SharedDbError;
use asez2_shared_db::DbItem;
use asez2_shared_db::Value;
use shared_essential::domain::{EcProtocol, EcProtocolStatus, ProtocolType};
use shared_essential::presentation::dto::response_request::{
    Message, Messages, Status,
};
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) mod action;
pub(crate) mod pre_request;

pub(crate) type InternalResponseType<A> = (Status, A, Messages);

pub(crate) const PROTOCOL_FIELDS: [&str; 6] = [
    EcProtocol::uuid,
    EcProtocol::id,
    EcProtocol::protocol_type_id,
    EcProtocol::registration_number,
    EcProtocol::status_id,
    EcProtocol::protocol_date,
];
pub(crate) const RETURN_FIELDS: &[&str] = &[
    EcProtocol::uuid,
    "protocol_id",
    EcProtocol::registration_number,
    "protocol_status_id",
    EcProtocol::protocol_date,
];

pub(crate) async fn select_protocol_list(
    uuids: &[Uuid],
    protocol_type_id: ProtocolType,
    db_pool: &PgPool,
) -> Result<Vec<EcProtocol>, SharedDbError> {
    if uuids.is_empty() {
        return Ok(Vec::new());
    }
    let uuid_values: Vec<Value> = uuids.iter().map(Value::from).collect();
    let select = Select::with_fields(PROTOCOL_FIELDS)
        .add_expand_filter("uuid", SelectionKind::In, uuid_values)
        .add_expand_filter(
            "protocol_type_id",
            SelectionKind::Equals,
            Some(protocol_type_id as i64),
        );
    EcProtocol::select(&select, db_pool).await
}

pub(crate) fn collect_status_errors(
    protocol_list: &[EcProtocol],
) -> impl Iterator<Item = Message> + '_ {
    protocol_list.iter().filter_map(|protocol| {
        match protocol.status_id {
            EcProtocolStatus::AgreementPending |
            EcProtocolStatus::Confirmed |
            EcProtocolStatus::Deleted => {
                let text = format!(r#"Перевести Протокол {} на статус "На согласовании" невозможно. Текущий статус Протокола "{}""#,
                                   protocol.id, protocol.status_id);
                Some(Message::error(text).with_param_item(protocol))
            }
            _ => None,
        }
    })
}

pub(crate) fn collect_status_warnings(
    protocol_list: &[EcProtocol],
) -> impl Iterator<Item = Message> + '_ {
    protocol_list.iter().filter_map(|protocol| {
        match protocol.status_id {
            EcProtocolStatus::SignaturePending => {
                let text = format!("Текущий статус Протокола {} - \"На подписании\". Вы хотите перевести Протокол на статус \"На согласовании\"",
                                   protocol.id);
                Some(Message::warn(text))
            }
            _ => None,
        }
    })
}
