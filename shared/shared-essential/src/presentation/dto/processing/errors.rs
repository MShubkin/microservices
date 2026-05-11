use super::*;
use monolith_service::http::error::MonolithHttpError;

/// Basic `Error` enum for the Processing service
#[derive(Error, Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum ProcessingError {
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
    #[error("Oшибка : {0}")]
    Business(String),
    #[error("Бизнес ошибки (см. сообщения)")]
    BusinessMessages(Messages),
    #[error("Invalid body was passed: `{0}`")]
    Serialization(String),
    #[error("Temporary Db Error: {0}")]
    TempDb(String),
    #[error("Temporary Broker Error: {0}")]
    TempBroker(String),
    #[error("Невалидное тело запроса: `{0}`")]
    InvalidBody(String),
    #[error("Проблема обновления таблицы \"{0}\", смотрите список ошибок.")]
    Update(String, Messages),
    #[error("Monolith Http Error: {0}")]
    Monolith(String),
}
impl AsezErrorComplete for ProcessingError {}

impl ResponseMessage for ProcessingError {
    fn message_response(&self) -> Vec<Message> {
        use ProcessingError::*;
        match self {
            Business(ref x) => {
                vec![Message::error(x.to_string())]
            }
            BusinessMessages(ref x) => x.messages.clone(),
            Internal(ref x) | Serialization(ref x) => {
                vec![Message::stop(format!(
                    "Внутренняя ошибка. Доложите администратору:\n{}",
                    x
                ))]
            }
            TempDb(ref x) | TempBroker(ref x) => {
                vec![Message::stop(format!("Внутренняя ошибка инфраструктуры. Доложите администратору или повторите операцию позднее:\n{}", x))]
            }
            Update(_, ref m) => m.messages.to_vec(),
            InvalidBody(err) => vec![Message::stop(err.to_owned())],
            Monolith(err) => {
                vec![Message::stop(format!(
                    "Ошибка обращения к монолиту:\n{}",
                    err
                ))]
            }
        }
    }
}

impl ResponseError for ProcessingError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let messages = self.message_response();
        let response: ApiResponse<(), ()> = ((), messages).into();

        // Absolute failsafe.
        let body = match serde_json::to_string(&response) {
            Err(_) => {
                let msg = format!(
                    "Error processing server error with code: {}",
                    self.status_code()
                );
                return HttpResponse::InternalServerError().body(msg);
            }
            Ok(r) => r,
        };
        // Build final response.
        let mut builder = match self {
            ProcessingError::Business(_) => HttpResponse::Ok(),
            ProcessingError::BusinessMessages(m)
            | ProcessingError::Update(_, m)
                if !m.is_stop() =>
            {
                HttpResponse::Ok()
            }
            ProcessingError::BusinessMessages(_)
            | ProcessingError::Update(..)
            | ProcessingError::Internal(_) => HttpResponse::InternalServerError(),
            ProcessingError::Serialization(_) | ProcessingError::InvalidBody(_) => {
                HttpResponse::UnprocessableEntity()
            }
            ProcessingError::TempDb(_)
            | ProcessingError::TempBroker(_)
            | ProcessingError::Monolith(_) => HttpResponse::RequestTimeout(),
        };
        builder.content_type(ContentType::json()).body(body)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ProcessingError::Business(_) => StatusCode::OK,
            ProcessingError::BusinessMessages(m)
            | ProcessingError::Update(_, m)
                if !m.is_stop() =>
            {
                StatusCode::OK
            }
            ProcessingError::BusinessMessages(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ProcessingError::Update(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ProcessingError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProcessingError::Serialization(_) | ProcessingError::InvalidBody(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            ProcessingError::TempDb(_)
            | ProcessingError::TempBroker(_)
            | ProcessingError::Monolith(_) => StatusCode::REQUEST_TIMEOUT,
        }
    }
}

impl ErrorLevel for ProcessingError {
    fn error_level(&self) -> Level {
        match self {
            ProcessingError::TempDb(_) | ProcessingError::TempBroker(_) => {
                Level::Retry
            }
            _ => Level::Stop,
        }
    }
}

impl From<ProcessingError> for AsezErrorDict {
    fn from(p: ProcessingError) -> Self {
        Self::Processing(p)
    }
}

impl From<ProcessingError> for AsezError {
    fn from(err: ProcessingError) -> Self {
        AsezError::new(err)
    }
}

impl From<BrokerError> for ProcessingError {
    fn from(err: BrokerError) -> Self {
        Self::TempBroker(err.to_string())
    }
}
impl From<MonolithHttpError> for ProcessingError {
    fn from(err: MonolithHttpError) -> Self {
        Self::Monolith(err.to_string())
    }
}
