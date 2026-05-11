use serde::{Deserialize, Serialize};

/// Подтверждение доставки сообщения
#[derive(Debug, Serialize, Deserialize)]
pub struct CommercialOfferResponseConfirmation {
    /// Идентификатор сообщения
    pub request_id: String,
    /// Cтатус обработки (success/error)
    pub status: String,
    /// Результат обработки
    pub message: String,
    /// Ошибки
    pub errors: Option<Errors>,
    /// Идентификатор объекта
    #[serde(rename = "TCPID")]
    pub tcp_id: i32,
}

/// Ошибки
#[derive(Debug, Serialize, Deserialize)]
pub struct Errors {
    pub error: Vec<Error>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    /// Код ошибки
    pub code: String,
    /// Описание ошибки
    pub message: String,
    /// Детальное описание ошибки
    pub details: Details,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Details {
    /// Описание
    pub field: String,
}

impl CommercialOfferResponseConfirmation {
    pub fn new(
        status: impl Into<String>,
        message: impl Into<String>,
        errors: Option<Errors>,
        tcp_id: i32,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            status: status.into(),
            message: message.into(),
            errors,
            tcp_id,
        }
    }

    pub fn success(tcp_id: i32, request_id: impl Into<String>) -> Self {
        Self::new(
            "success",
            "Ценовая информация успешно обработана",
            None,
            tcp_id,
            request_id,
        )
    }

    pub fn internal_error(tcp_id: i32, request_id: impl Into<String>) -> Self {
        Self::new(
            "error",
            "Внутренняя ошибка",
            Some(Errors {
                error: vec![Error {
                    code: "02".into(),
                    message: "Произошла внутренняя ошибка обработки сообщения"
                        .into(),
                    details: Details { field: "".into() },
                }],
            }),
            tcp_id,
            request_id,
        )
    }

    pub fn validation_error(
        tcp_id: i32,
        errors: Vec<Error>,
        request_id: impl Into<String>,
    ) -> Self {
        Self::new(
            "error",
            "Ошибка валидации данных",
            Some(Errors { error: errors }),
            tcp_id,
            request_id,
        )
    }
}
