//! Temporary TEMPORARY error types.
use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result as StdResult;

use env_setup::EnvError;
use serde_json::Error as JError;
use sqlx::migrate::MigrateError;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum SharedDbError {
    Serde(JError),
    Sqlx(SqlxError),
    SqlxMigrate(MigrateError),
    IoError(IoError),
    Env(EnvError),
    Other(String),
    Join(String),
    TaskJoin(JoinError),
    ValueError(String),
}

/// This is an alias. It needs to be public for the derive macros.
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
