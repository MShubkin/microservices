use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BrokerError>;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum BrokerError {
    /// Что-то пошло не так при работе с сервером брокера
    #[error("Внутренняя ошибка: `{0}`")]
    Internal(String),
    #[error("Слишком долгое ожидание сообщения")]
    WaitingTooLong,
    #[error("Слишком долгое ожидание ответа на сообщение")]
    WaitingTooLongForReply,
    #[error("Нет отправителей, от которых можно было бы получать сообщения")]
    NoSenders,
    #[error("Нет получателей для получения сообщения")]
    NoReceivers,
    #[error("Ошибка при десериализации получаемых данных: `{0}`")]
    InvalidReceivedMessage(String),
    #[error("Ошибка при сериализации отправляемых данных: `{0}`")]
    InvalidSentMessage(String),
    #[error("Не удается отправить подтверждение")]
    SendAck,
    #[error("Не удается отправить отрицательное подтверждение")]
    SendNack,
    #[error("Не удается закрыть канал")]
    CloseChannel,
}

impl From<amqprs::error::Error> for BrokerError {
    fn from(err: amqprs::error::Error) -> Self {
        Self::Internal(err.to_string())
    }
}
