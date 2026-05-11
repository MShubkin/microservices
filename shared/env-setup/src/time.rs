use std::str::FromStr;

use thiserror::Error;
use time::{format_description::FormatItem, macros::format_description};
use time::{Time, UtcOffset};

use crate::{var, EnvError};

const TIME_FORMAT: &[FormatItem<'_>] =
    format_description!("[hour]:[minute]:[second]");
const TZ_FORMAT: &[FormatItem<'_>] =
    format_description!("[offset_hour sign:mandatory]");

#[derive(Debug, Clone, Copy)]
pub struct TimeWithOffset {
    pub time: Time,
    pub offset: UtcOffset,
}

#[derive(Error, Debug)]
pub enum TimeParseError {
    #[error("Невалидный формат, ожидается {0}")]
    InvalidFormat(String),
    #[error("Невалидный формат времени: {0}")]
    InvalidTime(time::error::Parse),
    #[error("Невалидный формат часового пояса: {0}")]
    InvalidTz(time::error::Parse),
}

impl TimeWithOffset {
    pub fn from_env(env: &str) -> Result<Self, EnvError> {
        var(env).and_then(|s| Self::from_str(s.as_str()).map_err(Into::into))
    }

    pub fn from_env_with_default(
        env: &str,
        default: TimeWithOffset,
    ) -> Result<Self, EnvError> {
        TimeWithOffset::from_env(env).or_else(|err| match err {
            EnvError::MissingVar(_) => Ok(default),
            _ => Err(err),
        })
    }
}

impl FromStr for TimeWithOffset {
    type Err = TimeParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (time, offset) = s.split_once(' ').ok_or_else(|| {
            TimeParseError::InvalidFormat(String::from(
                "[hour]:[minute]:[second] +[offset_hour]",
            ))
        })?;
        let time =
            Time::parse(time, TIME_FORMAT).map_err(TimeParseError::InvalidTime)?;
        let offset = UtcOffset::parse(offset, TZ_FORMAT)
            .map_err(TimeParseError::InvalidTz)?;

        Ok(Self { offset, time })
    }
}
