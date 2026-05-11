use crate::app_process::estimated_commission::get_protocol_list_by_date::GetProtocolListByDateError;
use crate::app_process::price_analysis::update_plan::UpdatePlanCAError;
use crate::app_process::{UpdateAgendaError, UpdateProtocolError};
use crate::common::number_range::EcObjectType;

use asez2_shared_db::result::SharedDbError;
use rabbit_services::properties::TracingPropertiesError;
use shared_essential::application::external::IntegrationError;
use shared_essential::application::records;
use shared_essential::domain::maths::CurrencyError;
use shared_essential::presentation::dto::processing::ProcessingError as PError;
use shared_essential::presentation::dto::response_request::{
    ApiResponse, ApiResponseData, Message, Messages, Status,
};
use shared_essential::presentation::dto::AsezError;

use monolith_service::http::error::MonolithHttpError;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, ProcessingError>;

#[derive(Error, Debug)]
/// TODO: Any potentially forwards facing text should be in Russian.
pub(crate) enum ProcessingError {
    #[error("SerDe Error: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Broker error: {0}")]
    Broker(#[from] broker::error::BrokerError),
    #[error("Error from ASEZ2 DB backend: {0}")]
    DbBackendError(#[from] SharedDbError),
    #[error("Postgres error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Error setting up env variables: {0}")]
    EnvSetup(#[from] env_setup::EnvError),
    #[error("Local IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parsing error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Tokio lock error: {0}")]
    Tokio(#[from] tokio::sync::TryLockError),
    #[error("Tokio lock error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("Error setting up tracers: {0}")]
    TraceSetupError(#[from] trace_setup::TsError),
    #[error("Error extracting tracing fields: {0}")]
    TracingFieldsError(#[from] TracingPropertiesError),
    #[error(
        "Incoming error from {:?}: {:?} at {}",
        .0.source(),
        .0.error(),
        .0.timestamp()
    )]
    IncomingError(#[from] AsezError),
    #[error("Cannot convert Uuid: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("Rules check for {0} failed.")]
    RulesLawyer(String, Messages),
    #[error("Нарушение консистентности БД: {0}")]
    DbInconsistency(String),
    #[cfg(with_plan_db)]
    #[error("Could not push to SAP for {0}.")]
    SapPushFail(String, Messages),
    #[error("Could not complete update for {0}.")]
    UpdateFail(String, Messages),
    #[error("Нельзя удалить позиции повесток: {0}.")]
    AgendaItemsRemove(String),
    #[error("Не удается изменить форму: {0}.")]
    ChangeForm(String),
    #[error("Не удалось создать протокол: {0}.")]
    CreateProtocol(String),
    #[error("Не удалось утвердить протокол: {0}.")]
    ApproveProtocol(String),
    #[error("Ошибка секций: {0}.")]
    Section(String),
    #[error("Не удалось добавить ППЗ/ДС в Повестку СК: {0}")]
    AddPlansAgenda(String),
    #[error("Не удалось переместить ППЗ/ДС между Повестками СК: {0}")]
    TransferPlansAgenda(String),
    #[error("Не удалось добавить ППЗ/ДС в Протокол: {0}")]
    AddPlansProtocol(String),
    #[error("Ошибка при получении списка Повесток СК: {0}")]
    GetAgendaList(String),
    #[error("Ошибка при получении списка Повесток СК по дате: {0}")]
    GetAgendaListByDate(String),
    #[error("Ошибка при получении списка Протоколов СК по дате: {0}")]
    GetProtocolListByDate(GetProtocolListByDateError),
    #[error("Ошибка при получении списка Протоколов СК по Повестке СК: {0}")]
    GetProtocolListByAgenda(String),
    #[error("Ошибка при получении подробностей Протокола СК: {0}")]
    GetProtocolDetails(String),
    #[error("Ошибка при получении подробностей Повестки СК: {0}")]
    GetAgendaDetails(String),
    #[error("Ошибка при получении списка элементов: {0}")]
    GetItemList(String),
    #[error("Number range for objects of type \"{0:?}\" is full.")]
    NumberRangeOverflow(EcObjectType),
    #[error("Не удается назначить эксперта АЦ для ППЗ/ДС: {0}.")]
    AssignExpert(String),
    #[error("Не удается откатить статус (не консистентный БД): {0}.")]
    StatusRevert(String),
    #[error("Не удается обновить Повестку: {0}")]
    UpdateAgenda(#[from] UpdateAgendaError),
    #[error("Не удается обновить Протокол: {0}")]
    UpdateProtocol(#[from] UpdateProtocolError),
    #[error("Не удается обновить ППЗ/ДС: {0}")]
    UpdatePlanCA(#[from] UpdatePlanCAError),
    #[error("Ошибка внешнего сервиса: {0}.")]
    Integration(#[from] IntegrationError),
    #[error("Ошибка экспорта: {0}.")]
    Export(String),
    #[error("Ошибка импорта: {0}.")]
    Import(String),
    #[error("Ошибка валют: {0}.")]
    Currency(#[from] CurrencyError),
    #[error("Два заголовка ППЗ/ДС имеют одинаковый порядковый номер: {0}")]
    SrmHeaderImport(i64),
    #[error("Monolith error: {0}")]
    Monolith(#[from] MonolithHttpError),
    #[error("Внутренняя ошибка обработки: {0}")]
    #[allow(dead_code)]
    InternalError(String),
}

// NB: AsezError is the transmissible error that is sent along to other microservices
// and potentially to the user.
// It should contain enough information that:
// 1. The next service can decide whether the operation can be repeated or not.
// 2. The admin can easily find the relevant error logs and retrieve the error.
// 3. The user can easily transmit what they saw to the admin.
impl From<ProcessingError> for AsezError {
    fn from(x: ProcessingError) -> Self {
        use broker::BrokerError::{SendAck, SendNack, WaitingTooLong};
        use sqlx::Error::{Io as SqlxIo, PoolClosed, PoolTimedOut, WorkerCrashed};
        use ProcessingError::*;
        // First check for an inherited error.
        if let IncomingError(x) = x {
            return x;
        }

        // Then deal with other variants properly.
        let p: PError = match x {
            // Temporary broker errors
            Broker(WaitingTooLong) => {
                PError::TempBroker(WaitingTooLong.to_string())
            }
            Broker(SendAck) | Broker(SendNack) => {
                PError::TempBroker("Acknowledgement".to_string())
            }
            // Some DB backend errors may be temporary (acquire connection)
            Sqlx(PoolClosed) | DbBackendError(SharedDbError::Sqlx(PoolClosed)) => {
                PError::TempDb(PoolClosed.to_string())
            }
            Sqlx(PoolTimedOut)
            | DbBackendError(SharedDbError::Sqlx(PoolTimedOut)) => {
                PError::TempDb(PoolTimedOut.to_string())
            }
            Sqlx(WorkerCrashed)
            | DbBackendError(SharedDbError::Sqlx(WorkerCrashed)) => {
                PError::TempDb(WorkerCrashed.to_string())
            }
            Sqlx(x) | DbBackendError(SharedDbError::Sqlx(x))
                if matches!(x, SqlxIo(..)) =>
            {
                PError::TempDb(x.to_string())
            }
            // Other broker errors are permanent.
            Broker(x) => PError::Internal(x.to_string()),
            Io(io) => PError::Internal(format!("IO: {io}")),
            Parse(_) => PError::Internal("Received unparsable number".to_string()),
            DbInconsistency(msg) => PError::Internal(format!(
                "Нарушение консистентности базы данных: {}",
                msg
            )),
            // Db Serialization errors are still considered serialization errors.
            Sqlx(err) => PError::Serialization(format!("Sqlx DB error {:?}", err)),
            DbBackendError(SharedDbError::Serde(err)) => {
                PError::Serialization(format!("DB Serialization error {:?}", err))
            }
            // others are more serious and are described as internal/serialize.
            DbBackendError(x) => PError::Internal(format!("Database: {}", x)),
            EnvSetup(x) => PError::Internal(format!("Service startup {x}")),
            SerDe(_) => PError::Serialization("Invalid body".to_string()),
            Tokio(_) | TokioJoin(_) => PError::Internal("System".to_string()),
            TraceSetupError(x) => PError::Internal(format!("Logging: {x}")),
            TracingFieldsError(x) => {
                PError::Internal(format!("Tracing fields: {x}"))
            }
            RulesLawyer(t, m) => PError::Update(t, m),
            #[cfg(with_plan_db)]
            SapPushFail(t, m) => PError::Update(t, m),
            UpdateFail(t, m) => PError::Update(t, m),
            StatusRevert(x) => PError::Internal(x),
            Uuid(_) => PError::Internal("Malformed Uuid".to_string()),
            Section(x)
            | AgendaItemsRemove(x)
            | CreateProtocol(x)
            | GetAgendaList(x)
            | GetAgendaListByDate(x)
            | GetProtocolListByAgenda(x)
            | GetProtocolDetails(x)
            | GetAgendaDetails(x)
            | ChangeForm(x)
            | AddPlansAgenda(x)
            | TransferPlansAgenda(x)
            | AddPlansProtocol(x)
            | AssignExpert(x)
            | GetItemList(x)
            | ApproveProtocol(x) => {
                PError::Business(x) // This message is meaningful.
            }
            Integration(e) => PError::Business(e.to_string()),
            UpdateAgenda(ref err) => match err {
                UpdateAgendaError::MissingMeetingDate => {
                    PError::Business(err.to_string())
                }
                UpdateAgendaError::Messages(messages) => {
                    PError::BusinessMessages(messages.to_owned())
                }
                UpdateAgendaError::Items(_)
                | UpdateAgendaError::NoAgenda(_)
                | UpdateAgendaError::RemovedAgenda(_)
                | UpdateAgendaError::NoSource(_)
                | UpdateAgendaError::Db(_) => PError::Business(x.to_string()),
            },
            UpdateProtocol(ref err) => match err {
                UpdateProtocolError::MissingProtocolDate => {
                    PError::Business(err.to_string())
                }
                UpdateProtocolError::Items(_)
                | UpdateProtocolError::MissingField(_)
                | UpdateProtocolError::Db(_) => PError::Business(x.to_string()),
            },
            UpdatePlanCA(ref err) => match err {
                UpdatePlanCAError::OldCommissionDate { .. }
                | UpdatePlanCAError::UnableToUpdateCommissionDate { .. }
                | UpdatePlanCAError::InvalidEconomy => {
                    PError::Business(err.to_string())
                }
                UpdatePlanCAError::MissingRequiredField(_)
                | UpdatePlanCAError::NotFound(_)
                | UpdatePlanCAError::MissingItemUuid
                | UpdatePlanCAError::ItemNotFound(_) => {
                    PError::Business(x.to_string())
                }
            },
            GetProtocolListByDate(_) => PError::Business(x.to_string()),
            // We should never get here since `IncomingError` is inspected first.
            NumberRangeOverflow(_) => PError::Internal(x.to_string()), // This message is meaningful.
            IncomingError(_) => unreachable!("This is dealt with above."),
            Export(x) => PError::Internal(format!("Export: {x}")),
            Import(x) => PError::Internal(format!("Import: {x}")),
            Currency(x) => PError::Internal(x.to_string()),
            SrmHeaderImport(x) => PError::Internal(x.to_string()),
            Monolith(x) => PError::Monolith(x.to_string()),
            InternalError(x) => PError::Internal(x),
        };
        // This can now be transmitted via rabbit.
        AsezError::new(p)
    }
}

impl<T: ApiResponseData> From<&ProcessingError> for ApiResponse<T, ()> {
    fn from(f: &ProcessingError) -> Self {
        Self {
            status: Status::Error,
            messages: Messages::from(vec![f.into()]),
            ..Default::default()
        }
    }
}

impl From<&ProcessingError> for Message {
    fn from(f: &ProcessingError) -> Self {
        Message::stop(f.to_string())
    }
}

impl ProcessingError {
    pub fn is_broker_timeout(&self) -> bool {
        matches!(self, ProcessingError::Broker(broker::BrokerError::WaitingTooLong))
    }
}

impl From<records::Error> for ProcessingError {
    fn from(error: records::Error) -> Self {
        match error {
            records::Error::DbError(error) => {
                ProcessingError::DbBackendError(error)
            }
            records::Error::Rules(table, messages) => {
                ProcessingError::RulesLawyer(table.to_string(), messages)
            }
            records::Error::UpdateFailed(table, messages) => {
                ProcessingError::UpdateFail(table.to_string(), messages)
            }
            records::Error::StatusError(error) => match error.downcast() {
                Ok(err) => *err,
                Err(err) => ProcessingError::UpdateFail(
                    err.to_string(),
                    Messages::default(),
                ),
            },
        }
    }
}
