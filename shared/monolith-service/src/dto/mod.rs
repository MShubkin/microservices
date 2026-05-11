pub mod attachment;
pub mod category;
pub mod currency;
pub mod customer;
pub mod dictionary;
pub mod files_count;
pub mod okpd;
pub mod okved;
pub mod organization;
pub mod purchasing_trend;
pub mod time;
pub mod unit;
pub mod user;
pub mod vat;

use std::fmt::Display;

use crate::dto::category::Category;
use crate::dto::currency::Currency;
use crate::dto::customer::MonolithCustomer;
use crate::dto::purchasing_trend::PurchasingTrend;
use crate::dto::unit::Unit;
use asez2_shared_db::db_item::AsezDate;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Обобщенный ответ от монолита
#[derive(Debug, Serialize, Deserialize)]
pub struct MonolithResponse<S> {
    /// Статус ответа
    pub status: Status,
    /// Возвращаемые данные
    pub data: S,
    /// Cообщения для конечного пользователя
    /// (НБ: В монолите тут для некоторых функций нестандартная сериализация)
    #[serde(default)]
    pub messages: Messages,
}

/// Статус ответа
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "s")]
    Ok,
    #[serde(rename = "e")]
    Error,
}

/// Сообщение для интерфейса, которое может содержать сообщение об ошибке,
/// информацию об успешном действии и т.д.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Messages {
    /// Список сообщений
    pub messages: Vec<Message>,
    /// Общий тип сообщения
    pub kind: MessageKind,
}

impl Messages {
    /// Проверка, есть ли сообщения в буфере
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Тип пользовательского сообщения
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub enum MessageKind {
    Success,
    Information,
    Warning,
    Error,
    Stop,
    #[default]
    None, // Monolith compatibility
}

/// Сообщение для интерфейса, которое может содержать сообщение об ошибке,
/// информацию об успешном действии и т.д.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    /// Тип сообщения
    pub kind: MessageKind,
    /// Контент сообщения
    pub text: String,
    /// Параметры
    #[serde(skip_serializing_if = "Params::is_empty")]
    pub parameters: Params,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub item_list: Vec<ParamItem>,
}

impl Params {
    fn is_empty(&self) -> bool {
        self.item_list.is_empty()
    }
}

/// Параметры которые возвращаем на ФЕ.
/// Могут включать
/// - "id"  записи.
/// - "type" тип объекта по которому сообщение формируется.
/// - "username" десигнация пользователя.
/// - "date" какая-то дата на которую ссылаемся.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParamItem {
    pub id: ParamId,
    /// `type` is a really really bad field name.
    #[serde(rename = "type", skip_serializing_if = "EntityKind::unknown")]
    pub kind: EntityKind,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<AsezDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Plan,
    ContractAmendment,
    Unknown,
}

impl EntityKind {
    fn unknown(&self) -> bool {
        matches!(self, EntityKind::Unknown)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ParamId {
    String(String),
    Number(i64),
}

impl Display for ParamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamId::String(s) => write!(f, "{}", s),
            ParamId::Number(n) => write!(f, "{}", n),
        }
    }
}

impl<'de> Deserialize<'de> for ParamId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::String(s) => Ok(ParamId::String(s)),
            Value::Number(num) => {
                if let Some(n) = num.as_i64() {
                    Ok(ParamId::Number(n))
                } else {
                    Err(serde::de::Error::custom("Expected a i64"))
                }
            }
            _ => Err(serde::de::Error::custom("Expected a string or number")),
        }
    }
}

#[derive(Debug, Default)]
pub struct CommonDictionaries {
    pub customers: Vec<MonolithCustomer>,
    pub units: Vec<Unit>,
    pub currencies: Vec<Currency>,
    pub purchasing_trends: Vec<PurchasingTrend>,
    pub categories: Vec<Category>,
}
