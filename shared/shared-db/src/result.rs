//! Типы ошибок для shared-db.
//!
//! Все асинхронные функции трейта [`DbItem`] и смежного кода возвращают
//! [`Result<T>`], который является псевдонимом `std::result::Result<T, SharedDbError>`.
use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result as StdResult;

use env_setup::EnvError;
use serde_json::Error as JError;
use sqlx::migrate::MigrateError;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use tokio::task::JoinError;

/// Перечень ошибок, которые могут возникнуть при работе с БД.
///
/// Каждый вариант оборачивает ошибку от соответствующей библиотеки,
/// чтобы не терять контекст при конвертации через `?`.
#[derive(Debug)]
pub enum SharedDbError {
    /// Ошибка десериализации JSON (конфиги, JSON-поля в БД).
    Serde(JError),
    /// Ошибка sqlx при выполнении запроса или подключении.
    Sqlx(SqlxError),
    /// Ошибка применения миграций sqlx.
    SqlxMigrate(MigrateError),
    /// Ошибка ввода-вывода (чтение файла конфига и т.п.).
    IoError(IoError),
    /// Ошибка чтения переменных окружения.
    Env(EnvError),
    /// Произвольная строковая ошибка.
    Other(String),
    /// Ошибка при соединении нескольких результатов (join логика).
    Join(String),
    /// Ошибка завершения Tokio-задачи (`JoinHandle::await`).
    TaskJoin(JoinError),
    /// Ошибка преобразования значения между типами [`Value`].
    ValueError(String),
}

/// Псевдоним результата для всех функций этого крейта.
/// Публичен, потому что генерируемый макросами код ссылается на него напрямую.
pub type Result<T> = std::result::Result<T, SharedDbError>;

impl std::error::Error for SharedDbError {}

impl Display for SharedDbError {
    fn fmt(&self, f: &mut Formatter) -> StdResult<(), Error> {
        match self {
            Self::Sqlx(e) => write!(f, "{}", e),
            Self::SqlxMigrate(e) => write!(f, "{}", e),
            Self::Serde(e) => write!(f, "{}", e),
            Self::IoError(e) => write!(f, "{}", e),
            Self::Env(e) => write!(f, "{}", e),
            Self::TaskJoin(e) => write!(f, "{}", e),
            Self::ValueError(e) | Self::Other(e) | Self::Join(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

/// Макрос-сокращение: генерирует `From<E> for SharedDbError` для каждой
/// внешней ошибки, сворачивая её в нужный вариант.
macro_rules! error {
    ($e:ident, $var:ident) => {
        impl From<$e> for SharedDbError {
            fn from(e: $e) -> Self {
                Self::$var(e)
            }
        }
    };
}

impl From<&str> for SharedDbError {
    fn from(e: &str) -> Self {
        Self::Other(e.to_string())
    }
}
error!(EnvError, Env);
error!(SqlxError, Sqlx);
error!(MigrateError, SqlxMigrate);
error!(JError, Serde);
error!(IoError, IoError);
error!(String, Other);
error!(JoinError, TaskJoin);
