use std::{net::AddrParseError, num::ParseIntError};

use amqprs::{BasicProperties, FieldTable, FieldValue, LongStr, ShortStr};
use igg_tracing::tracing_fields::*;
use time::OffsetDateTime;
use tracing::Span;
use uuid::Uuid;

// Хедеры запроса
/// Хедер с айди пользователя, который инициировал запрос
pub const REQUEST_USER_ID_HEADER: &str = "request_user_id";
/// Хедер с айди запроса в системе
pub const REQUEST_ID_HEADER: &str = "request_id";

/// Базовые свойства для взаимодействия с RabbitMQ
/// в рамказ АСЭЗ 2.0
#[derive(Debug, Clone)]
pub struct AsezRabbitProperties {
    basic_props: BasicProperties,
    headers: FieldTable,
}

impl AsezRabbitProperties {
    pub fn new(basic_props: BasicProperties) -> Self {
        Self {
            basic_props,
            headers: FieldTable::new(),
        }
    }

    /// Сеттер хедера [`REQUEST_USER_ID_HEADER`]
    pub fn with_user_id(mut self, user_id: i32) -> Self {
        self.set_user_id(user_id);
        self
    }

    /// Сеттер хедера [`REQUEST_USER_ID_HEADER`]
    pub fn set_user_id(&mut self, user_id: i32) {
        self.headers.insert(
            ShortStr::try_from(REQUEST_USER_ID_HEADER)
                .expect("Длина хедера всегда меньше 255"),
            FieldValue::S(
                user_id
                    .to_string()
                    .try_into()
                    .expect("Длина числа всегда меньше 2^32"),
            ),
        );
    }

    /// Сеттер хедера [`REQUEST_ID_HEADER`]
    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.headers.insert(
            ShortStr::try_from(REQUEST_ID_HEADER)
                .expect("Длина хедера всегда меньше 255"),
            FieldValue::S(
                LongStr::try_from(request_id.to_string())
                    .expect("Длина Uuid всегда меньше u32::MAX"),
            ),
        );
        self
    }

    /// Геттер хедера [`REQUEST_USER_ID_HEADER`]
    pub fn user_id(&self) -> Option<&str> {
        let user_id = self.headers.get(
            &ShortStr::try_from(REQUEST_USER_ID_HEADER)
                .expect("Длина хедера всегда меньше 255"),
        )?;
        match user_id {
            FieldValue::S(str) => Some(str.as_ref()),
            _ => None,
        }
    }

    /// Геттер хедера [`REQUEST_ID_HEADER`]
    pub fn request_id(&self) -> Option<Uuid> {
        let request_id = self.headers.get(
            &ShortStr::try_from(REQUEST_ID_HEADER)
                .expect("Длина хедера всегда меньше 255"),
        )?;
        match request_id {
            FieldValue::S(str) => Uuid::parse_str(str.as_ref()).ok(),
            _ => None,
        }
    }

    /// Добавляет поля для трассировки для передачи через адаптер
    /// вызываемому серверу.
    pub fn add_tracing_fields(&mut self, fields: &AsezTracingFieldsCollection) {
        self.headers.insert(
            field_short_str(REQUEST_ID),
            fields.request_id.to_string().into(),
        );
        self.headers.insert(field_short_str(URI), fields.uri.clone().into());
        self.headers.insert(
            field_short_str(SOURCE_IP),
            fields.source.ip().to_string().into(),
        );
        self.headers.insert(
            field_short_str(SOURCE_PORT),
            fields.source.port().to_string().into(),
        );
        self.headers
            .insert(field_short_str(USER_AGENT), fields.user_agent.clone().into());
        if let Some(user_id) = &fields.user_id {
            self.headers
                .insert(field_short_str(USER_ID), user_id.to_string().into());
        }
        if let Some(user_name) = &fields.user_name {
            self.headers
                .insert(field_short_str(USER_NAME), user_name.clone().into());
        }
        if !fields.object_ids.is_empty() {
            self.headers.insert(
                field_short_str(OBJECT_IDS),
                fields.object_ids.to_string().into(),
            );
        }
        if !fields.object_uuids.is_empty() {
            self.headers.insert(
                field_short_str(OBJECT_UUIDS),
                fields.object_uuids.to_string().into(),
            );
        }
        self.headers.insert(
            field_short_str(TIMESTAMP),
            FieldValue::T(fields.timestamp.unix_timestamp() as u64),
        );
    }

    pub fn finish(mut self) -> BasicProperties {
        self.basic_props.with_headers(self.headers);
        self.basic_props
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TracingPropertiesError {
    #[error("Нет обязательных полей")]
    NoMandatoryFields,
    #[error("Ошибка сериализации поля: {0}")]
    Serde(#[from] amqp_serde::Error),
    #[error(transparent)]
    IntParse(#[from] ParseIntError),
    #[error(transparent)]
    UuidParse(#[from] uuid::Error),
    #[error(transparent)]
    AddrParse(#[from] AddrParseError),
    #[error("Неверный тип поля {0}: {1}")]
    InvaldFieldValueType(&'static str, String),
    #[error("Невалидный unix timestamp: {0}")]
    InvalidTimestamp(#[from] time::error::ComponentRange),
}

fn field_short_str(field: &str) -> ShortStr {
    field.try_into().expect("Длина хедера всегда меньше 255")
}

impl Default for AsezRabbitProperties {
    fn default() -> Self {
        Self {
            basic_props: BasicProperties::default()
                .with_content_type("application/json")
                .with_persistence(true)
                .finish(),
            headers: FieldTable::new(),
        }
    }
}

impl From<BasicProperties> for AsezRabbitProperties {
    fn from(basic_props: BasicProperties) -> Self {
        let headers = basic_props.headers().cloned().unwrap_or_default();
        Self {
            basic_props,
            headers,
        }
    }
}

/// Creates a tracing span with ASEZ common fields filled
/// basing on the provided Rabbit field table.
pub fn get_rabbit_span(properties: &BasicProperties) -> Span {
    use igg_tracing::tracing_fields::*;
    use tracing::info_span;

    fn field_value<'a>(name: &str, fields: &'a FieldTable) -> &'a str {
        let name = field_short_str(name);
        fields.get(&name).and_then(|v| v.try_into().ok()).unwrap_or("")
    }

    if let Some(fields) = properties.headers() {
        info_span!(
            "rabbit_span",
            "asez.user_id"      = %field_value(REQUEST_USER_ID_HEADER, fields),
            "asez.user_name"    = %field_value(USER_NAME, fields),
            "asez.request_id"   = %field_value(REQUEST_ID, fields),
            "asez.rabbit_request_id"   = %field_value(REQUEST_ID_HEADER, fields),
            "asez.object_ids"   = %field_value(OBJECT_IDS, fields),
            "asez.object_uuids" = %field_value(OBJECT_UUIDS, fields),
            "asez.uri"          = %field_value(URI, fields),
            "asez.user_agent"   = %field_value(USER_AGENT, fields),
            "asez.source_ip"    = %field_value(SOURCE_IP, fields),
            "asez.source_port"  = %field_value(SOURCE_PORT, fields),
        )
    } else {
        info_span!(
            "rabbit_span",
            "asez.user_id" = tracing::field::Empty,
            "asez.user_name" = tracing::field::Empty,
            "asez.request_id" = tracing::field::Empty,
            "asez.object_ids" = tracing::field::Empty,
            "asez.object_uuids" = tracing::field::Empty,
            "asez.category" = tracing::field::Empty,
            "asez.uri" = tracing::field::Empty,
            "asez.user_agent" = tracing::field::Empty,
            "asez.source_ip" = tracing::field::Empty,
            "asez.source_port" = tracing::field::Empty,
        )
    }
}

/// По свойствам сообщения строит набор полей для трассировки.
pub fn basic_properties_to_tracing_fields(
    properties: &BasicProperties,
) -> Result<Option<AsezTracingFieldsCollection>, TracingPropertiesError> {
    let Some(headers) = properties.headers() else {
        return Ok(None);
    };
    let request_id = headers
        .get(&field_short_str(REQUEST_ID))
        .map(TryInto::<&String>::try_into)
        .transpose()?;
    let request_id = request_id.map(|x| x.parse()).transpose()?;
    let uri = headers
        .get(&field_short_str(URI))
        .map(TryInto::<&String>::try_into)
        .transpose()?
        .cloned();
    let source_ip = headers
        .get(&field_short_str(SOURCE_IP))
        .map(TryInto::<&String>::try_into)
        .transpose()?;
    let source_port = headers
        .get(&field_short_str(SOURCE_PORT))
        .map(TryInto::<&String>::try_into)
        .transpose()?;
    let user_agent = headers
        .get(&field_short_str(USER_AGENT))
        .map(TryInto::<&String>::try_into)
        .transpose()?
        .cloned();

    if let (
        Some(request_id),
        Some(uri),
        Some(source_ip),
        Some(source_port),
        Some(user_agent),
    ) = (request_id, uri, source_ip, source_port, user_agent)
    {
        let source = format!("{source_ip}:{source_port}").parse()?;
        let user_id = headers
            .get(&field_short_str(USER_ID))
            .map(TryInto::<&String>::try_into)
            .transpose()?;
        let user_id = user_id.map(|x| x.parse()).transpose()?;
        let user_name = headers
            .get(&field_short_str(USER_NAME))
            .map(TryInto::<&String>::try_into)
            .transpose()?
            .cloned();
        let object_ids = headers
            .get(&field_short_str(OBJECT_IDS))
            .map(TryInto::<&String>::try_into)
            .transpose()?;
        let object_ids =
            object_ids.map(|x| x.parse()).transpose()?.unwrap_or_default();
        let object_uuids = headers
            .get(&field_short_str(OBJECT_UUIDS))
            .map(TryInto::<&String>::try_into)
            .transpose()?;
        let object_uuids =
            object_uuids.map(|x| x.parse()).transpose()?.unwrap_or_default();
        let timestamp = headers
            .get(&field_short_str(TIMESTAMP))
            .map(|x| {
                if let FieldValue::T(v) = x {
                    Ok(OffsetDateTime::from_unix_timestamp(*v as i64)?)
                } else {
                    Err(TracingPropertiesError::InvaldFieldValueType(
                        TIMESTAMP,
                        format!("{x:?}"),
                    ))
                }
            })
            .transpose()?
            .unwrap_or(OffsetDateTime::now_utc());
        Ok(Some(AsezTracingFieldsCollection {
            request_id,
            uri,
            source,
            user_agent,
            user_id,
            user_name,
            object_ids,
            object_uuids,
            timestamp,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use igg_tracing::tracing_fields::{AsezTracingFieldsCollection, CommaList};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::properties::AsezRabbitProperties;

    #[test]
    fn tracing_fields_roundtrip() {
        let mut fields = AsezTracingFieldsCollection {
            request_id: Uuid::new_v4(),
            uri: "/some/uri/?query=123".to_string(),
            source: "8.8.8.8:88".parse().unwrap(),
            user_agent: "mosaic".to_string(),
            user_id: Some(666),
            user_name: Some("Конечно, Вася".to_string()),
            object_ids: CommaList(vec![1, 2, 3]),
            object_uuids: CommaList(vec![
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
            ]),
            timestamp: OffsetDateTime::now_utc(),
        };
        let mut props = AsezRabbitProperties::default();
        props.add_tracing_fields(&fields);
        let basic_props = props.finish();
        let res = super::basic_properties_to_tracing_fields(&basic_props);
        assert!(matches!(res, Ok(Some(_))), "${res:#?}");
        // nanoseconds do not make it thrhough rabbit properties...
        fields.timestamp = fields.timestamp.replace_nanosecond(0).unwrap();
        assert_eq!(res.unwrap().unwrap(), fields);
    }
}
