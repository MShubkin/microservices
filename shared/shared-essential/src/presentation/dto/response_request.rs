//! Здесь хранится обобщенная форма ответов API.
//! Однако она, вероятно, неполная и может быть изменена.
use crate::presentation::dto::general::ObjectIdentifier;
use ahash::AHashMap;
use asez2_shared_db::db_item::{AsezDate, Paginated, Select};

use asez2_tables::master_data::routes::RouteApprType;
use asez2_tables::{
    DocumentApprover, DocumentApproverRep, PlanOrAmendmentRep, AMENDMENT_ID_RANGE,
    PLAN_ID_RANGE,
};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use uuid::Uuid;

use crate::domain::PlanOrAmendment;

/// Описывает преобразование в сообщения для конечного пользователя (frontend)
pub trait ResponseMessage {
    fn message_response(&self) -> Vec<Message>;
}

/// Currently a marker trait for valid ApiResponse Data.
pub trait ApiResponseData:
    Debug + Default + Serialize + for<'a> Deserialize<'a>
{
    fn skip(&self) -> bool {
        false
    }
}

/// Обобщенный тип для возвращения конечному пользователю (frontend)
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(bound = "")]
pub struct ApiResponse<S: ApiResponseData, O>
where
    O: Serialize + for<'d> Deserialize<'d> + Default + PartialEq,
{
    /// Статус ответа
    pub status: Status,
    #[serde(default)]
    #[serde(skip_serializing_if = "ApiResponseData::skip")]
    /// Возвращаемые данные
    pub data: S,
    /// Cообщения для конечного пользователя
    #[serde(default)]
    pub messages: Messages,
    /// Объекты с дополнительный информацией, которая может пригодиться
    /// конечному пользователю
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub objects: Vec<O>,
}

impl<S: ApiResponseData, O> ApiResponse<S, O>
where
    O: Serialize + for<'d> Deserialize<'d> + Default + PartialEq,
{
    pub fn with_messages(mut self, messages: Messages) -> Self {
        self.status = messages.kind.into();
        self.messages = messages;
        self
    }

    pub fn with_data(mut self, data: S) -> Self {
        self.data = data;
        self
    }
}

impl<S, M, O, D> From<(D, M)> for ApiResponse<S, O>
where
    S: ApiResponseData,
    M: Into<Messages>,
    O: Serialize + for<'d> Deserialize<'d> + Default + PartialEq,
    D: Into<S>,
{
    fn from((data, messages): (D, M)) -> Self {
        let messages = messages.into();
        Self {
            status: messages.kind.into(),
            data: data.into(),
            messages,
            objects: vec![],
        }
    }
}

impl<S, M, O> From<M> for ApiResponse<S, O>
where
    S: ApiResponseData,
    M: Into<Messages>,
    O: Serialize + for<'d> Deserialize<'d> + Default + PartialEq,
{
    fn from(messages: M) -> Self {
        let messages = messages.into();
        Self {
            status: messages.kind.into(),
            messages,
            ..Default::default()
        }
    }
}

/// Сообщение для интерфейса, которое может содержать сообщение об ошибке,
/// информацию об успешном действии и т.д.
#[derive(
    Serialize, Deserialize, Clone, PartialOrd, Ord, Eq, Debug, Default, PartialEq,
)]
pub struct Messages {
    /// Список сообщений
    pub messages: Vec<Message>,
    /// Общий тип сообщения
    pub kind: MessageKind,
}

/// Интерфейс для создания бизнес сообщений
/// в множественном и единственном числе
pub trait BusinessMessage {
    /// Сущность, которая определяет, будет ли сообщение
    /// в множественном или единственном числе
    type Entity;

    /// Присоединение нового сообщения к остальным сообщениям
    ///
    /// Включает в себя проверку на результат работы [`BusinessMessage::resolve`]
    fn checked_append<T>(&self, messages: &mut Messages, entities: &[T])
    where
        T: AsRef<Self::Entity>,
    {
        if let Some(message) = self.resolve(entities) {
            messages.add_prepared_message(message);
        }
    }

    /// Само определит по количеству [`BusinessMessage::Entity`], какой
    /// тип сообщения будет сгенерирован
    ///
    /// Если `entities` является пустым, то будет возвращено [`Option::None`]
    fn resolve<T>(&self, entities: &[T]) -> Option<Message>
    where
        T: AsRef<Self::Entity>,
    {
        match entities.len() {
            0 => None,
            1 => self.singular(entities[0].as_ref()).into(),
            _ => self.plural(entities).into(),
        }
    }

    /// Формирование сообщения в единственном числе
    fn singular(&self, entity: &Self::Entity) -> Message;

    /// Формирование сообщения в множественном числе
    ///
    /// Пользователь должен обеспечить условие, что entities
    /// не является пустым
    fn plural<T>(&self, entities: &[T]) -> Message
    where
        T: AsRef<Self::Entity>;
}

impl Messages {
    /// Добавление "сырого" сообщения  к уже существующим в буфере
    pub fn add_message(&mut self, kind: MessageKind, text: String) {
        self.add_prepared_message(Message {
            kind,
            text,
            parameters: Params::default(),
            fields: Default::default(),
        });
    }

    /// Добавление сообщения к уже существующим в буфере
    /// и установка нового типа сообщения, если он > предыдущего
    pub fn add_prepared_message(&mut self, message: Message) {
        if message.kind > self.kind {
            self.kind = message.kind;
        }
        self.messages.push(message);
    }

    /// Добавление сообщений к уже существующим в буфере
    pub fn add_messages(&mut self, messages: Messages) {
        for message in messages.messages {
            self.add_prepared_message(message);
        }
    }

    /// Добавление сообщений к уже существующим в буфере
    pub fn with_message<T>(mut self, message: T) -> Self
    where
        T: Into<Message>,
    {
        self.add_prepared_message(message.into());
        self
    }

    /// Добавление сообщений к уже существующим в буфере
    pub fn with_messages<T>(mut self, messages: Vec<T>) -> Self
    where
        T: Into<Message>,
    {
        messages
            .into_iter()
            .for_each(|msg| self.add_prepared_message(msg.into()));
        self
    }

    /// Проверяет общий уровень, является ли ошибкой
    pub fn is_error(&self) -> bool {
        self.kind >= MessageKind::Error
    }

    /// Проверяет общий уровень, является ли ошибкой
    pub fn is_stop(&self) -> bool {
        self.kind == MessageKind::Stop
    }

    pub fn is_warn(&self) -> bool {
        self.kind == MessageKind::Warning
    }

    /// Проверка, есть ли сообщения в буфере
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Очистка сообщений
    pub fn clear(&mut self) {
        self.kind = MessageKind::Success;
        self.messages.clear();
    }

    /// Проверка на эквивалентность [`Messages`], пренебрегая порядком сообщений
    /// и сравнивая сами сообщения с помощью [`Message::eq_unordered`]
    // TODO: Если в одном списке есть два одинаковых сообщения, а в другом одно, то все может не корректно отработать.
    pub fn eq_unordered(&self, rhs: &Self) -> (bool, Option<String>) {
        if self.kind != rhs.kind {
            return (false, Some("Kind not equal".to_string()));
        } else if self.messages.len() != rhs.messages.len() {
            return (false, Some("Count of messages not equal".to_string()));
        }

        let mut rmsg_iter = rhs.messages.iter();
        for msg in &self.messages {
            let mut found = false;
            for rmsg in rmsg_iter.by_ref() {
                let (eq, err) = msg.eq_unordered(rmsg);
                if eq {
                    found = true;
                    break;
                } else if let Some(err) = err {
                    return (false, Some(err));
                }
            }
            if !found {
                return (false, Some("No matching message found".to_string()));
            }
        }
        (true, None)
    }
}

impl From<Vec<Message>> for Messages {
    fn from(message: Vec<Message>) -> Self {
        let mut output = Self::default();
        output.messages.reserve(message.len());
        for m in message {
            output.add_prepared_message(m);
        }
        output
    }
}

impl From<Message> for Messages {
    fn from(value: Message) -> Self {
        vec![value].into()
    }
}

/// Сообщение для интерфейса, которое может содержать сообщение об ошибке,
/// информацию об успешном действии и т.д.
#[derive(
    Serialize, Deserialize, Clone, PartialOrd, Ord, Eq, Debug, Default, PartialEq,
)]
pub struct Message {
    /// Тип сообщения
    pub kind: MessageKind,
    /// Контент сообщения
    pub text: String,
    /// Параметры
    #[serde(skip_serializing_if = "Params::is_empty", default)]
    pub parameters: Params,
    /// Ссылки на поля
    #[serde(default)]
    pub fields: Vec<String>,
}

impl Message {
    /// Создание сообщения с kind=[MessageKind::Error]
    pub fn error<T: ToString>(text: T) -> Self {
        Message {
            kind: MessageKind::Error,
            text: text.to_string(),
            parameters: Params::default(),
            fields: Default::default(),
        }
    }

    /// Создание сообщения с kind=[MessageKind::Stop]
    pub fn stop<T: ToString>(text: T) -> Self {
        Message {
            kind: MessageKind::Stop,
            text: text.to_string(),
            parameters: Params::default(),
            fields: Default::default(),
        }
    }

    /// Создание сообщения с kind=[MessageKind::Warning]
    pub fn warn(text: String) -> Self {
        Message {
            kind: MessageKind::Warning,
            text,
            parameters: Params::default(),
            fields: Default::default(),
        }
    }

    /// Создание сообщения с kind=[MessageKind::Information]
    pub fn info<T: ToString>(text: T) -> Self {
        Message {
            kind: MessageKind::Information,
            text: text.to_string(),
            parameters: Params::default(),
            fields: Default::default(),
        }
    }

    /// Создание сообщения с kind=[MessageKind::Success]
    pub fn success<T: ToString>(text: T) -> Self {
        Message {
            kind: MessageKind::Success,
            text: text.to_string(),
            parameters: Params::default(),
            fields: Default::default(),
        }
    }

    pub fn with_parameters(mut self, parameters: Vec<ParamItem>) -> Self {
        self.parameters.item_list = parameters;
        self
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = fields;
        self
    }

    pub fn with_param_description<T: Display>(mut self, description: T) -> Self {
        self.parameters.description = description.to_string();
        self
    }

    pub fn with_param_item<I>(mut self, item: I) -> Self
    where
        I: Into<ParamItem>,
    {
        self.parameters.item_list.push(item.into());
        self
    }

    pub fn with_param_items<T, I, E>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: AsRef<E>,
        ParamItem: for<'a> From<&'a E>,
    {
        let params = params
            .into_iter()
            .map(|p| ParamItem::from(p.as_ref()))
            .collect::<Vec<_>>();
        self.parameters.item_list.extend(params);
        self
    }

    /// Проверка на эквивалентность сообщений, которая пренебрегает порядком
    /// [`ParamItem`] в [`Params`]
    // TODO: Если в одном списке есть два одинаковых сообщения, а в другом одно, то все может не корректно отработать.
    pub fn eq_unordered(&self, rhs: &Self) -> (bool, Option<String>) {
        if self.kind != rhs.kind {
            return (
                false,
                Some(format!("Kind not equal: {:?} vs {:?}", self.kind, rhs.kind)),
            );
        }
        if self.text != rhs.text {
            return (
                false,
                Some(format!("Text not equal: {:?} vs {:?}", self.text, rhs.text)),
            );
        }
        if self.parameters.item_list.len() != rhs.parameters.item_list.len() {
            return (
                false,
                Some(format!(
                    "Count of parameters not equal: {:?} vs {:?}",
                    self.parameters.item_list.len(),
                    rhs.parameters.item_list.len()
                )),
            );
        }
        if self.kind != rhs.kind {
            return (
                false,
                Some(format!("Kind not equal: {:?} vs {:?}", self.kind, rhs.kind)),
            );
        }

        let are_all_params_available = self
            .parameters
            .item_list
            .iter()
            .all(|param| rhs.parameters.contains(param));

        if !are_all_params_available {
            return (
                false,
                Some(format!(
                    "Parameters not equal: {:?} vs {:?}",
                    self.parameters, rhs.parameters
                )),
            );
        }

        (
            self.kind == rhs.kind
                && self.text == rhs.text
                && self.parameters.item_list.len()
                    == rhs.parameters.item_list.len()
                && are_all_params_available,
            None,
        )
    }
}

/// Статус ответа
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[serde(rename = "s")]
    #[default]
    Ok,
    #[serde(rename = "e")]
    Error,
}

/// Тип пользовательского сообщения
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum MessageKind {
    #[default]
    Success,
    Information,
    Warning,
    Error,
    Stop,
    None, // Monolith compatibility
}

impl From<MessageKind> for Status {
    fn from(msg_kind: MessageKind) -> Self {
        match msg_kind {
            MessageKind::Stop => Status::Error,
            _ => Status::Ok,
        }
    }
}

// Структура переноса данных в ФЕ.
#[derive(Debug, Default, Serialize, Deserialize, derive_more::Deref)]
pub struct PaginatedData<S> {
    /// Число записей которые запрос мог бы вернуть (до пагинации).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    /// Сами записи которые возвращаем.
    #[deref]
    pub item_list: Vec<S>,
}

impl<S> From<Vec<S>> for PaginatedData<S>
where
    S: Serialize + for<'a> Deserialize<'a>,
{
    fn from(item_list: Vec<S>) -> Self {
        Self {
            total: Some(item_list.len()),
            item_list,
        }
    }
}

impl<S> From<Paginated<S>> for PaginatedData<S> {
    fn from(value: Paginated<S>) -> Self {
        let Paginated { items, total } = value;
        PaginatedData {
            total: total.map(|total| total as usize),
            item_list: items,
        }
    }
}

impl<S> FromIterator<S> for PaginatedData<S> {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let item_list: Vec<_> = iter.into_iter().collect();
        let total = Some(item_list.len());
        PaginatedData { total, item_list }
    }
}

impl<S> PaginatedData<S> {
    pub fn new(select: &Select, data: Vec<S>) -> Self {
        let len = data.len();
        let total = select.count_total.unwrap_or(false).then_some(len);
        let start = select.offset.unwrap_or(0);
        let count = select.take_n.unwrap_or(len);
        let item_list = data.into_iter().skip(start).take(count).collect();
        Self { total, item_list }
    }

    pub fn is_empty(&self) -> bool {
        self.total.map_or(self.item_list.is_empty(), |total| total == 0)
    }

    pub fn map<T, F>(self, f: F) -> PaginatedData<T>
    where
        F: Fn(S) -> T,
    {
        PaginatedData {
            total: self.total,
            item_list: self.item_list.into_iter().map(f).collect(),
        }
    }

    pub fn try_map<T, E, F>(self, f: F) -> Result<PaginatedData<T>, E>
    where
        F: FnMut(S) -> Result<T, E>,
    {
        Ok(PaginatedData {
            total: self.total,
            item_list: self
                .item_list
                .into_iter()
                .map(f)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<T> ApiResponseData for PaginatedData<T> where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Default
{
}

impl ApiResponseData for () {
    fn skip(&self) -> bool {
        true
    }
}

impl<T> ApiResponseData for Option<T> where
    T: Debug + Serialize + for<'a> Deserialize<'a>
{
}

impl<T> ApiResponseData for Vec<T> where
    T: Debug + Serialize + for<'a> Deserialize<'a>
{
}

impl<K, V> ApiResponseData for AHashMap<K, V>
where
    K: Debug + Eq + std::hash::Hash + Serialize + for<'a> Deserialize<'a>,
    V: Debug + Serialize + for<'a> Deserialize<'a>,
{
}

impl<T> ApiResponseData for HashMap<Uuid, Vec<T>> where
    T: Debug + Default + Serialize + for<'a> Deserialize<'a>
{
}

impl ApiResponseData for String {}

impl ApiResponseData for DocumentApproverRep {}

impl ApiResponseData for serde_json::Value {}

#[derive(
    Debug, Clone, Copy, Eq, PartialOrd, Ord, Serialize, Deserialize, PartialEq,
)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Plan,
    ContractAmendment,
    Agenda,
    Protocol,
    PricingRoute,
    SpecializedDepartmentRoute,
    RequestPriceInfo,
    PlanReasonCancel,
    RequestItem,
    RequestList,
    Unknown,
}

impl From<&str> for EntityKind {
    fn from(value: &str) -> Self {
        match value {
            "plan" => EntityKind::Plan,
            "contract_amendment" => EntityKind::ContractAmendment,
            "agenda" => EntityKind::Agenda,
            "protocol" => EntityKind::Protocol,
            "pricing_route" => EntityKind::PricingRoute,
            "specialized_department_route" => {
                EntityKind::SpecializedDepartmentRoute
            }
            "request_price_info" => EntityKind::RequestPriceInfo,
            "request_item" => EntityKind::RequestItem,
            "request_list" => EntityKind::RequestList,
            &_ => EntityKind::Unknown,
        }
    }
}

impl Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Plan => write!(f, "ППЗ"),
            EntityKind::ContractAmendment => write!(f, "ДС"),
            EntityKind::Agenda => write!(f, "Повестка"),
            EntityKind::Protocol => write!(f, "Протокол"),
            EntityKind::PricingRoute => write!(f, "Маршрут АЦ"),
            EntityKind::SpecializedDepartmentRoute => write!(f, "Маршрут ПД"),
            EntityKind::RequestPriceInfo => write!(f, "Запрос информации о цене"),
            EntityKind::PlanReasonCancel => {
                write!(f, "Справочник «Причины аннулирования закупки»")
            }
            EntityKind::RequestItem => write!(f, "Позиция ЗЦИ"),
            EntityKind::RequestList => write!(f, "Список ЗЦИ"),
            EntityKind::Unknown => write!(f, "неизвестно"),
        }
    }
}

impl Default for EntityKind {
    fn default() -> Self {
        Self::Unknown
    }
}

impl EntityKind {
    pub(crate) fn undefined(&self) -> bool {
        matches!(*self, Self::Unknown)
    }

    /// Возвращет:
    /// 1. Если входит в [`PLAN_ID_RANGE`] => [`EntityKind::Plan`]
    /// 2. Если входит в [`AMENDMENT_ID_RANGE`] => [`EntityKind::ContractAmendment`]
    /// 3. Иначе => [`EntityKind::Unknown`]
    pub fn from_plan_id(id: i64) -> Self {
        if PLAN_ID_RANGE.contains(&id) {
            Self::Plan
        } else if AMENDMENT_ID_RANGE.contains(&id) {
            Self::ContractAmendment
        } else {
            Self::Unknown
        }
    }
}

impl From<RouteApprType> for EntityKind {
    fn from(value: RouteApprType) -> Self {
        match value {
            RouteApprType::SpecializedDepartments => {
                EntityKind::SpecializedDepartmentRoute
            }
            RouteApprType::PriceAnalysis => EntityKind::PricingRoute,
            _ => EntityKind::Unknown,
        }
    }
}

#[derive(
    Clone, Debug, Default, PartialOrd, Ord, Eq, Serialize, Deserialize, PartialEq,
)]
#[serde(default)]
pub struct Params {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub item_list: Vec<ParamItem>,
}

impl Params {
    /// Convenience function for tests.
    pub fn contains(&self, x: &ParamItem) -> bool {
        self.item_list.contains(x)
    }

    fn is_empty(&self) -> bool {
        self.description.is_empty() && self.item_list.is_empty()
    }
}

/// Параметры которые возвращаем на ФЕ.
/// Могут включать
/// - "id"  записи.
/// - "type" тип объекта по которому сообщение формируется.
/// - "username" десигнация пользователя.
/// - "date" какая-то дата на которую ссылаемся.
#[derive(
    Clone, Debug, Default, Eq, PartialOrd, Ord, Serialize, Deserialize, PartialEq,
)]
#[serde(default)] // Technically id is probably obligatory..
pub struct ParamItem {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// `type` is a really really bad field name.
    #[serde(rename = "type", skip_serializing_if = "EntityKind::undefined")]
    pub kind: EntityKind,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<AsezDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
}

impl ParamItem {
    pub fn new(
        id: String,
        kind: EntityKind,
        username: String,
        text: Option<String>,
        date: Option<AsezDate>,
        uuid: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            kind,
            username,
            date,
            uuid,
            text,
        }
    }

    pub fn from_id<T: ToString>(id: T) -> Self {
        Self {
            id: id.to_string(),
            ..Default::default()
        }
    }

    pub fn from_name<T: ToString>(user_name: T) -> Self {
        Self {
            username: user_name.to_string(),
            ..Default::default()
        }
    }

    pub fn with_user<T: ToString>(mut self, username: T) -> Self {
        self.username = username.to_string();
        self
    }

    pub fn with_date(mut self, date: AsezDate) -> Self {
        self.date = Some(date);
        self
    }

    pub fn with_uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    pub fn with_type(mut self, kind: EntityKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }
}

macro_rules! params_from_id {
    ($t:ty,$kind:ident) => {
        impl From<$t> for ParamItem {
            fn from(x: $t) -> Self {
                Self::from_id(x.id).with_type(EntityKind::$kind)
            }
        }
    };
}

macro_rules! params_from_id_uuid {
    ($t:ty,$kind:ident) => {
        impl From<$t> for ParamItem {
            fn from(x: $t) -> Self {
                Self::from_id(x.id).with_uuid(x.uuid).with_type(EntityKind::$kind)
            }
        }
    };
}

macro_rules! params_from_rep_id {
    ($t:ty,$kind:ident) => {
        impl From<$t> for ParamItem {
            fn from(x: $t) -> Self {
                Self::from_id(x.id.unwrap_or(-1)).with_type(EntityKind::$kind)
            }
        }
    };
}

impl From<&PlanOrAmendment> for ParamItem {
    fn from(p: &PlanOrAmendment) -> Self {
        let id = p.id();
        let kind = match p {
            PlanOrAmendment::Plan(_) => EntityKind::Plan,
            PlanOrAmendment::Amendment(_) => EntityKind::ContractAmendment,
        };

        Self::from_id(id).with_type(kind)
    }
}

impl From<&PlanOrAmendmentRep> for ParamItem {
    fn from(p: &PlanOrAmendmentRep) -> Self {
        // TODO: что делать в случае, если в репрезентации
        // нет id
        let id = p.id().unwrap_or(0);
        let kind = match p {
            PlanOrAmendmentRep::Plan(_) => EntityKind::Plan,
            PlanOrAmendmentRep::Amendment(_) => EntityKind::ContractAmendment,
        };

        Self::from_id(id).with_type(kind)
    }
}

impl From<&DocumentApprover> for ParamItem {
    fn from(p: &DocumentApprover) -> Self {
        ParamItem::from_id(p.plan_id).with_type(EntityKind::Plan)
    }
}

params_from_id_uuid!(&crate::domain::Plan, Plan);
params_from_id_uuid!(&crate::domain::PlanVersion, Plan);
params_from_rep_id!(&crate::domain::PlanRep, Plan);
params_from_rep_id!(&mut crate::domain::PlanRep, Plan);
params_from_id_uuid!(&crate::domain::ContractAmendment, ContractAmendment);
params_from_id_uuid!(&crate::domain::ContractAmendmentVersion, ContractAmendment);
params_from_id!(&crate::domain::EcAgenda, Agenda);
params_from_id!(&crate::domain::EcProtocol, Protocol);
params_from_id!(&crate::domain::tcp::RequestHeader, RequestPriceInfo);

impl From<&crate::domain::routes::RouteHeader> for ParamItem {
    fn from(x: &crate::domain::routes::RouteHeader) -> Self {
        use asez2_tables::master_data::routes::RouteApprType::*;
        let kind = match x.type_id {
            SpecializedDepartments => EntityKind::SpecializedDepartmentRoute,
            PriceAnalysis => EntityKind::PricingRoute,
            _ => EntityKind::Unknown,
        };
        Self::from_id(x.id).with_uuid(x.uuid).with_type(kind)
    }
}

impl From<&ObjectIdentifier> for ParamItem {
    fn from(x: &ObjectIdentifier) -> Self {
        Self::from_id(x.id).with_uuid(x.uuid).with_type(x.object_type)
    }
}

impl AsRef<ParamItem> for ParamItem {
    fn as_ref(&self) -> &ParamItem {
        self
    }
}

impl From<&ParamItem> for ParamItem {
    fn from(val: &ParamItem) -> ParamItem {
        val.to_owned()
    }
}

impl From<monolith_service::dto::Messages> for Messages {
    fn from(value: monolith_service::dto::Messages) -> Self {
        Self {
            kind: value.kind.into(),
            messages: value.messages.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<monolith_service::dto::MessageKind> for MessageKind {
    fn from(value: monolith_service::dto::MessageKind) -> Self {
        use monolith_service::dto::MessageKind::*;
        match value {
            Success => Self::Success,
            Information => Self::Information,
            Warning => Self::Warning,
            Error => Self::Error,
            Stop => Self::Stop,
            None => Self::None,
        }
    }
}

impl From<monolith_service::dto::Message> for Message {
    fn from(value: monolith_service::dto::Message) -> Self {
        Self {
            kind: value.kind.into(),
            text: value.text,
            parameters: value.parameters.into(),
            fields: Default::default(),
        }
    }
}

impl From<monolith_service::dto::Params> for Params {
    fn from(value: monolith_service::dto::Params) -> Self {
        Self {
            description: value.description,
            item_list: value.item_list.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<monolith_service::dto::ParamItem> for ParamItem {
    fn from(value: monolith_service::dto::ParamItem) -> Self {
        Self {
            date: value.date,
            id: value.id.to_string(),
            username: value.username,
            uuid: value.uuid,
            kind: value.kind.into(),
            text: None,
        }
    }
}

impl From<monolith_service::dto::EntityKind> for EntityKind {
    fn from(value: monolith_service::dto::EntityKind) -> Self {
        use monolith_service::dto::EntityKind::*;
        match value {
            Plan => Self::Plan,
            ContractAmendment => Self::ContractAmendment,
            Unknown => Self::Unknown,
        }
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, derive_more::Deref, derive_more::From,
)]
pub struct ApiResponseDataWrapper<T>(pub T);

impl<T> ApiResponseData for ApiResponseDataWrapper<T> where
    T: Debug + Default + Serialize + for<'a> Deserialize<'a>
{
}
