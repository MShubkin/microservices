use std::error::Error;
use std::fmt::Display;

use crate::presentation::dto::master_data::error::MasterDataError;
use crate::presentation::dto::technical_commercial_proposal::TcpError;
use actix_web::ResponseError;
use serde::{Deserialize, Serialize};
use time::format_description::FormatItem;
use time::{macros::format_description, Duration, OffsetDateTime};

use super::estimated_commission::EcError;
use super::integration::IntegError;
use super::log_storage::LogStorageError;
use super::notification::NotificationError;
use super::price_analysis::PaError;
use super::print_docs::PrintDocError;
use super::processing::ProcessingError;
use super::response_request::ResponseMessage;
use super::scheduler::error::SchedulerError;
use super::specialized_departments::SpecDepsError;
use super::view_storage::ViewStorageError;

/// Базовый тип [`Result`] в проекте `Asez 2.0`, на основе которого
/// сервисы взаимодействуют. Это означает, что каждый
/// сервис возвращает [`AsezResult<T>`] другому сервису
pub type AsezResult<T> = Result<T, AsezError>;

/// Базовый формат временной метки в [`AsezError`]
pub static TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

/// Трейт, который описывает ошибку проекта Asez как "полную по Asez".
/// Вам просто нужно реализовать это для вашей enum-ошибки
/// и это подскажет вам, что вы должны реализовать дополнительно
pub trait AsezErrorComplete:
    Error + Display + ResponseError + ResponseMessage + ErrorLevel + Into<AsezErrorDict>
{
}

/// Основная ошибка в `Asez 2.0` ,
/// с которой могут взаимодействовать сервисы
#[derive(Debug, Serialize, Deserialize)]
pub struct AsezError {
    /// Возможная ошибка из любого сервиса
    error: AsezErrorDict,
    /// Временная метка ошибки
    timestamp: String,
}

/// Словарь ошибок, чтобы сделать возможным
/// сохранить все возможные ошибки в [`AsezError`]
#[derive(Debug, Serialize, Deserialize)]
pub enum AsezErrorDict {
    EstimatedCommission(EcError),
    Integration(IntegError),
    ViewStorage(ViewStorageError),
    LogStorage(LogStorageError),
    Notification(NotificationError),
    PriceAnalysis(PaError),
    Processing(ProcessingError),
    MasterData(MasterDataError),
    Tcp(TcpError),
    PrintDocs(PrintDocError),
    SpecDeps(SpecDepsError),
    Scheduler(SchedulerError),
}

impl AsezError {
    /// # Описание
    ///
    /// Конструктор ошибки, который автоматически проставит
    /// временную метку
    ///
    /// # Аргументы
    /// `error` - Любая ошибка, которая реализует [AsezErrorComplete]
    pub fn new<E>(error: E) -> Self
    where
        E: AsezErrorComplete,
    {
        let current_time = OffsetDateTime::now_utc()
            .checked_add(Duration::hours(3))
            .expect("Is it 2038 year?");
        AsezError {
            error: error.into(),
            timestamp: current_time
                .format(TIME_FORMAT)
                .expect("`TIME_FORMAT` некорректный"),
        }
    }

    /// Получение [источника](Source) ошибки
    pub fn source(&self) -> Source {
        match self.error {
            AsezErrorDict::EstimatedCommission(_) => Source::EstimatedCommission,
            AsezErrorDict::Integration(_) => Source::Integration,
            AsezErrorDict::ViewStorage(_) => Source::ViewStorage,
            AsezErrorDict::LogStorage(_) => Source::LogStorage,
            AsezErrorDict::Notification(_) => Source::Notification,
            AsezErrorDict::PriceAnalysis(_) => Source::PriceAnalysis,
            AsezErrorDict::Processing(_) => Source::Processing,
            AsezErrorDict::MasterData(_) => Source::MasterData,
            AsezErrorDict::Tcp(_) => Source::Processing,
            AsezErrorDict::PrintDocs(_) => Source::PrintDocs,
            AsezErrorDict::SpecDeps(_) => Source::SpecializedDepartments,
            AsezErrorDict::Scheduler(_) => Source::Scheduler,
        }
    }

    /// Получение внутренней ошибки определенного сервиса
    pub fn error(&self) -> &AsezErrorDict {
        &self.error
    }

    /// Получение временной метки ошибки
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

impl std::error::Error for AsezError {}

/// Описывает уровень ошибки для ее обработки
/// и выполнения некоторой операции в соответствии с уровнем
///
/// При желании уровень ошибки может быть реализован, например, следующим образом:
/// ```rust
/// use shared_essential::presentation::dto::error::Level;
/// use shared_essential::presentation::dto::error::ErrorLevel;
///
///  enum MyError {
///      Internal,    
///      InvalidBody,
///      DbStmt,
///      BusinessLogic,
///      DbConnection,
///  }
///
///  impl ErrorLevel for MyError {
///     fn error_level(&self) -> Level {
///        match self {
///             Self::Internal | Self::InvalidBody => Level::Stop,
///             Self::DbStmt => Level::Stop,
///             Self::BusinessLogic | Self::DbConnection => Level::Retry,
///         }
///     }
/// }
/// ```
pub trait ErrorLevel {
    fn error_level(&self) -> Level;
}

/// Уровень ошибки
#[derive(Debug)]
pub enum Level {
    /// Операция может быть проигнорирована
    Ignore,
    /// Операция может быть перевыполнена
    Retry,
    /// Операция не может быть прекращена и должна быть остановлена
    Stop,
}

impl ResponseError for AsezError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        self.error.error_response()
    }
}

impl ErrorLevel for AsezError {
    fn error_level(&self) -> Level {
        self.error.error_level()
    }
}

impl Display for AsezError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error in {} at {}: {}",
            self.source(),
            self.timestamp,
            self.error
        )
    }
}

impl ResponseError for AsezErrorDict {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            AsezErrorDict::EstimatedCommission(e) => e.error_response(),
            AsezErrorDict::Integration(e) => e.error_response(),
            AsezErrorDict::ViewStorage(e) => e.error_response(),
            AsezErrorDict::LogStorage(e) => e.error_response(),
            AsezErrorDict::Notification(e) => e.error_response(),
            AsezErrorDict::PriceAnalysis(e) => e.error_response(),
            AsezErrorDict::Processing(e) => e.error_response(),
            AsezErrorDict::MasterData(e) => e.error_response(),
            AsezErrorDict::Tcp(e) => e.error_response(),
            AsezErrorDict::PrintDocs(e) => e.error_response(),
            AsezErrorDict::SpecDeps(e) => e.error_response(),
            AsezErrorDict::Scheduler(e) => e.error_response(),
        }
    }
}

impl ErrorLevel for AsezErrorDict {
    fn error_level(&self) -> Level {
        match self {
            AsezErrorDict::EstimatedCommission(e) => e.error_level(),
            AsezErrorDict::Integration(e) => e.error_level(),
            AsezErrorDict::ViewStorage(e) => e.error_level(),
            AsezErrorDict::LogStorage(e) => e.error_level(),
            AsezErrorDict::Notification(e) => e.error_level(),
            AsezErrorDict::PriceAnalysis(e) => e.error_level(),
            AsezErrorDict::Processing(e) => e.error_level(),
            AsezErrorDict::MasterData(e) => e.error_level(),
            AsezErrorDict::Tcp(e) => e.error_level(),
            AsezErrorDict::PrintDocs(e) => e.error_level(),
            AsezErrorDict::SpecDeps(e) => e.error_level(),
            AsezErrorDict::Scheduler(e) => e.error_level(),
        }
    }
}

impl Display for AsezErrorDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsezErrorDict::EstimatedCommission(e) => write!(f, "{}", e),
            AsezErrorDict::Integration(e) => write!(f, "{}", e),
            AsezErrorDict::ViewStorage(e) => write!(f, "{}", e),
            AsezErrorDict::LogStorage(e) => write!(f, "{}", e),
            AsezErrorDict::Notification(e) => write!(f, "{}", e),
            AsezErrorDict::PriceAnalysis(e) => write!(f, "{}", e),
            AsezErrorDict::Processing(e) => write!(f, "{}", e),
            AsezErrorDict::MasterData(e) => write!(f, "{}", e),
            AsezErrorDict::Tcp(e) => write!(f, "{}", e),
            AsezErrorDict::PrintDocs(e) => write!(f, "{}", e),
            AsezErrorDict::SpecDeps(e) => write!(f, "{}", e),
            AsezErrorDict::Scheduler(e) => write!(f, "{}", e),
        }
    }
}

/// Источник ошибки
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[repr(i8)]
pub enum Source {
    /// Сервис `Price Analysis`
    PriceAnalysis = 1,
    /// Сервис `Technical Commercial Proposal`
    TechnicalCommercialProposal = 2,
    /// Сервис `Notification`
    Notification = 3,
    /// Сервис `Processing`
    Processing = 4,
    /// Сервис `Print Documents`
    PrintDocs = 5,
    /// Сервис `Estimated Commission`
    EstimatedCommission = 6,
    /// Сервис `Specialized Departments`
    SpecializedDepartments = 7,
    /// Сервис `Integration`
    Integration = 8,
    /// Сервис `View Storage`
    ViewStorage = 9,
    /// Сервис `Log Storage`
    LogStorage = 10,
    /// Сервис `Master Data`
    MasterData = 11,
    /// Сервис `Scheduler`
    Scheduler = 12,
}

impl AsRef<str> for Source {
    fn as_ref(&self) -> &str {
        match self {
            Source::EstimatedCommission => "estimated-commission",
            Source::PriceAnalysis => "price-analysis",
            Source::TechnicalCommercialProposal => "technical-commercial-proposal",
            Source::Notification => "notification",
            Source::Processing => "processing-plans",
            Source::PrintDocs => "print-docs",
            Source::SpecializedDepartments => "specialized-departments",
            Source::Integration => "integration",
            Source::ViewStorage => "view-storage",
            Source::LogStorage => "log-storage",
            Source::MasterData => "master-data",
            Source::Scheduler => "scheduler",
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[cfg(test)]
mod test_examples {
    use actix_web::ResponseError;

    use crate::presentation::dto::error::{AsezErrorDict, AsezResult};
    use crate::presentation::dto::notification::NotificationError;

    use super::{AsezError, ErrorLevel};

    #[test]
    fn error_roundtrip() {
        // Поток этого взаимодействия:
        // `pa` -> `ec` -> `notification` - Здесь произошла некоторая ошибка в `notification`, и нам нужно вернуть ее в `pa`...
        // `pa` <- `ec` <- `notification`

        // Сервис `notification` хочет вернуть эту ошибку в службу `ec`
        // Не забывайте, что службы обмениваются данными с помощью `AsezResult<T>`, где T - возвращаемый тип данных
        let notification_err: AsezError =
            AsezError::new(NotificationError::Internal(String::new()));
        let result: AsezResult<u64> = AsezResult::Err(notification_err);
        let ser_result = serde_json::to_string(&result).unwrap();

        // Здесь мы получаем AsezResult обращений в `ec` сервисе от сервиса `notification`
        let de_result: AsezResult<u64> = serde_json::from_str(&ser_result).unwrap();
        if let Err(err) = &de_result {
            println!("Уровень ошибки - {:?}", err.error_level());
            println!("Сама ошибка - {}", err);
            println!("Время ошибки - {}", err.timestamp());
            println!("Источник ошибки - {}", err.source());

            // Также мы может узнать точный тип ошибки
            if let AsezErrorDict::Notification(NotificationError::Internal(_)) =
                err.error()
            {
                println!("IT IS INTERNAL ERROR!!");
            }
        }
        // А затем мы хотим вернуть его в `pa` сервис
        let ser_result = serde_json::to_string(&de_result).unwrap();

        // Получние ответа в `pa`
        let de_result: AsezResult<u64> = serde_json::from_str(&ser_result).unwrap();
        // И теперь мы хотим вернуть пользователю сообщение об ошибке
        if let Err(err) = de_result {
            let err_http_response = err.error_response();
            println!("Ответ пользователю - {:?}", err_http_response);
        }
    }
}
