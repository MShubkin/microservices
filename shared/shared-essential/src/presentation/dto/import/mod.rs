use actix_multipart::Multipart;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::presentation::dto::general::ObjectIdentifier;
use crate::presentation::dto::response_request::EntityKind;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct UploadMultipartData {
    pub object_identifier: ObjectIdentifier,
    pub file: Vec<u8>,
    pub file_name: String,
    pub token: String,
    pub is_registered_by_d647: Option<bool>,
}

impl UploadMultipartData {
    pub async fn try_from(mut request: Multipart) -> Result<Self, MultipartError> {
        let mut multipart_data = UploadMultipartData::default();

        while let Some(item) = request.next().await {
            let mut field =
                item.map_err(|error| MultipartError::Internal(error.to_string()))?;

            while let Some(chunk) = field.next().await {
                let field_name = field.name();
                let field_value_bytes = &chunk
                    .map_err(|error| {
                        MultipartError::IncorrectParameter(
                            field_name.to_owned(),
                            error.to_string(),
                        )
                    })?
                    .to_vec();
                match field_name {
                    "id" => {
                        let field_value_str = std::str::from_utf8(
                            field_value_bytes,
                        )
                        .map_err(|error| {
                            MultipartError::IncorrectParameter(
                                field_name.to_owned(),
                                error.to_string(),
                            )
                        })?;
                        multipart_data.object_identifier.id =
                            field_value_str.parse::<i64>().unwrap_or_default();
                    }
                    "uuid" => {
                        let field_value_str = std::str::from_utf8(
                            field_value_bytes,
                        )
                        .map_err(|error| {
                            MultipartError::IncorrectParameter(
                                field_name.to_owned(),
                                error.to_string(),
                            )
                        })?;
                        multipart_data.object_identifier.uuid =
                            Uuid::parse_str(field_value_str).map_err(|error| {
                                MultipartError::IncorrectParameter(
                                    field_name.to_owned(),
                                    error.to_string(),
                                )
                            })?;
                    }
                    "object_type" => {
                        let field_value_str = std::str::from_utf8(
                            field_value_bytes,
                        )
                        .map_err(|error| {
                            MultipartError::IncorrectParameter(
                                field_name.to_owned(),
                                error.to_string(),
                            )
                        })?;
                        multipart_data.object_identifier.object_type =
                            EntityKind::from(field_value_str);
                    }
                    "file" => {
                        multipart_data.file = field_value_bytes.clone();
                        multipart_data.file_name = field
                            .content_disposition()
                            .get_filename()
                            .unwrap_or("import.xlsx")
                            .to_owned();
                    }
                    "token" => {
                        let field_value_str = std::str::from_utf8(
                            field_value_bytes,
                        )
                        .map_err(|error| {
                            MultipartError::IncorrectParameter(
                                field_name.to_owned(),
                                error.to_string(),
                            )
                        })?;
                        multipart_data.token = field_value_str.to_owned();
                    }
                    "is_registered_by_d647" => {
                        let field_value_str = std::str::from_utf8(
                            field_value_bytes,
                        )
                        .map_err(|error| {
                            MultipartError::IncorrectParameter(
                                field_name.to_owned(),
                                error.to_string(),
                            )
                        })?;
                        multipart_data.is_registered_by_d647 =
                            Some(field_value_str == "true");
                    }
                    _ => {}
                }
            }
        }
        Ok(multipart_data)
    }
}

#[derive(Error, Debug)]
pub enum MultipartError {
    #[error("Ошибка обработки тела multi-part запроса: `{0}`")]
    Internal(String),
    #[error("Ошибка обработки multi-part параметра: `{0}`, `{1}`")]
    IncorrectParameter(String, String),
}
