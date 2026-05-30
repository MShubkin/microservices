//! Локальная замена библиотеки `gas_tracing` для формирования SIEM-совместимых логов.
//!
//! ## Зачем отдельная библиотека?
//!
//! Оригинальная `gas_tracing` не поддерживалась и была недоступна на всех окружениях,
//! поэтому был написан собственный аналог с предсказуемым поведением.
//!
//! ## Формат CEF (Common Event Format)
//!
//! Каждая строка лога выглядит так:
//! ```text
//! May 11 12:00:00 192.168.1.1 CEF:0|Vendor|ServiceName|1.0|category|event_id|severity|key=value ...
//! ```
//! Поля разделяются символом `|`. Спецсимволы экранируются по-разному в заголовке
//! (`|` → `\|`) и в расширении (`=` → `\=`). Детали — в `cef::escape_cef_value`.
//!
//! ## Архитектура
//!
//! ```text
//! tracing::Event
//!     └── CEFTracingLayer::on_event()          (tracing_subscriber::Layer)
//!             ├── CEFVisitorEvent::record_str() (извлекает kind, id, user_code)
//!             ├── ищет ReqBaseData в родительских span-ах
//!             └── CEFTracingLayer::write_event() → NonBlocking writer → файл/stdout
//! ```
//!
//! Для Actix-Web интеграции используется `ServiceRootSpanBuilder` из модуля `with_actix`,
//! который записывает `ReqBaseData` в span extensions при каждом входящем HTTP-запросе.
//!
//! ## Фильтрация событий по `kind`
//!
//! CEF-слой логирует только события, у которых поле `kind` входит в заданный список.
//! Пустой список означает «логировать все» (используется для SIEM-аудита).
//! Пример: `tracing::info!(kind = "siem", "User logged in")`.
//!
//! ## Поддержка нескольких форматов
//!
//! Для добавления нового формата достаточно добавить метод `write_event_as_X`
//! в `CEFTracingLayer` и переключатель в `write_event`. Библиотека не зависит
//! жёстко от Actix — она совместима с любым `tracing_subscriber::Subscriber`.
//!
//! TODO: Decide whether we want to keep the &str references or use something
//! a little bit more sensible.
use std::net;
use std::{fmt::Debug, net::Ipv4Addr};

use tracing::{
    field::{Field, Visit},
    Level,
};
pub use tracing_subscriber::Registry;

pub mod tracing_fields;

pub mod cef;
// TODO: resolve features in /shared
#[cfg(test)]
mod test;
#[cfg(feature = "with-actix-web")]
pub mod with_actix;

pub use cef::*;
#[cfg(feature = "with-actix-web")]
pub use with_actix::*;

/// Инициализирует логгер для локальной разработки с форматированным выводом в stdout.
///
/// Не забудьте установить переменную окружения `RUST_LOG` для управления уровнем
/// (например, `RUST_LOG=info,broker=debug`).
pub fn setup_dev_logger() {
    setup_loggers!(tracing_subscriber::fmt::layer())
}

/// Инициализирует глобальный subscriber из произвольного набора слоёв.
///
/// В продакшне SRM использует четыре слоя:
/// 1. `fmt::layer()` — читаемый текстовый вывод для локальных логов
/// 2. `json::layer()` — JSON для агрегаторов (ELK, Loki)
/// 3. CEF-слой аутентификации — аудит входов/выходов
/// 4. CEF-слой безопасности (SIEM) — подозрительные события
///
/// Макрос позволяет передать любой набор слоёв через запятую.
/// Фильтрация по `RUST_LOG` применяется ко всем слоям сразу через `EnvFilter`.
///
/// # Пример
/// ```rust,ignore
/// setup_loggers!(
///     tracing_subscriber::fmt::layer(),
///     cef_auth_layer,
///     cef_siem_layer,
/// );
/// ```
#[macro_export]
macro_rules! setup_loggers {
    ($($extra_layer:expr),*$(,)?) => {{
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        $crate::Registry::default()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            $(.with($extra_layer))*
            .init()
    }}
}

/// Категория события для CEF-поля `SignatureID`.
///
/// Числовые значения соответствуют типам событий в оригинальной `gas_tracing`.
/// Используется при построении CEF-строки: поле `category` в заголовке.
#[derive(Debug, Clone, Copy)]
pub enum TracingKind {
    User = 1,
    Document = 2,
    Expert = 3,
    /// Категория из оригинальной библиотеки с неустановленной семантикой.
    Something = 4,
}

/// Данные HTTP-запроса, которые CEF-слой записывает в каждую строку лога.
///
/// Хранится в `span.extensions()` — `CEFTracingLayer::on_new_span` помещает
/// экземпляр сюда, когда span содержит поле `user_code`. Все дочерние события
/// этого span-а автоматически получат эти данные при вызове `on_event`.
///
/// Поля соответствуют CEF Extension-полям:
/// - `category`   → заголовочное поле `SignatureID`
/// - `src_ip`     → `src`  (CEF Extension)
/// - `src_port`   → `spt`  (CEF Extension)
/// - `user_agent` → `requestClientApplication` (CEF Extension)
/// - `user_code`  → `suser` (CEF Extension)
///
/// `src_ip` хранится как `String`, а не `net::IpAddr` — для совместимости
/// с оригинальной `gas_tracing`, где это поле было строковым.
///
/// Используйте `Default::default()` для создания — конструктор `new` не нужен,
/// так как заполнение происходит через трейт `Visit`.
#[derive(PartialEq, Debug, Clone)]
pub struct ReqBaseData {
    pub category: String,
    pub uri: String,
    pub src_ip: String,
    /// CEF Extension поле `spt`.
    pub src_port: u16,
    /// CEF Extension поле `requestClientApplication`.
    pub user_agent: String,
    /// CEF Extension поле `suser` — код пользователя из сессии АСЭЗ.
    pub user_code: String,
    pub request_id: String,
    pub trace_id: String,
}

impl Default for ReqBaseData {
    fn default() -> Self {
        Self {
            category: "default".to_string(),
            uri: Default::default(),
            src_ip: Ipv4Addr::UNSPECIFIED.to_string(),
            src_port: Default::default(),
            user_agent: Default::default(),
            user_code: "unknown".to_string(),
            request_id: Default::default(),
            trace_id: Default::default(),
        }
    }
}

/// This exists to make creating a `CEFTracingLayer` less ugly.
#[derive(Debug, Clone)]
pub struct ServiceDescription {
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub host: net::IpAddr,
}

impl ReqBaseData {
    /// Обновляет одно поле по его имени из tracing span.
    ///
    /// Пустые значения **не перезаписывают** существующие данные — это намеренное
    /// поведение: middleware может вызываться в произвольном порядке, и более ранний
    /// middleware не должен затирать данные, установленные позже.
    ///
    /// `request_id` нормализуется: дефисы удаляются (`"uuid-v4"` → `"uuidv4"`),
    /// так как это требование формата SIEM-системы.
    pub fn record(&mut self, field: &str, v: &str) {
        if v.is_empty() {
            return;
        }
        match field {
            "category" => self.category = v.to_owned(),
            "uri" => self.uri = v.to_owned(),
            "source_ip" => self.src_ip = v.to_owned(),
            "user_agent" => self.user_agent = v.to_owned(),
            "user_code" => self.user_code = v.to_owned(),
            "trace_id" => self.trace_id = v.to_owned(),
            "request_id" => self.request_id = v.replace('-', ""),
            _ => {}
        }
    }

    /// Обновляет `src_port` по числовому значению.
    ///
    /// Существует отдельно от `record` для тестируемости — `record_u64` (реализация
    /// трейта `Visit`) приводит `u64` к `u16`, что не очевидно без явного теста.
    /// Нулевой порт не перезаписывает уже установленный (аналогично `record`).
    pub fn record_u16(&mut self, field: &str, v: u16) {
        if v != 0 && field == "source_port" {
            self.src_port = v;
        }
    }
}

/// Реализация `Visit` позволяет `ReqBaseData` накапливать данные прямо из
/// `tracing`-событий и span-атрибутов без промежуточных структур.
///
/// `tracing_subscriber` вызывает методы этого трейта для каждого поля события.
/// Специализированные методы (`record_str`, `record_u64`) имеют приоритет над
/// `record_debug` — это важно для `src_port`, который является числом.
impl Visit for ReqBaseData {
    /// Fallback для полей, не имеющих специализированного типа.
    /// Debug-форматирование включает кавычки (`"value"`), что приемлемо для CEF.
    fn record_debug(&mut self, field: &Field, v: &dyn Debug) {
        let fname = field.name();
        self.record(fname, &format!("{:?}", v))
    }

    /// `src_port` приходит как `u64` через tracing span attributes.
    /// Приведение к `u16` небезопасно в общем случае, но для порта значения
    /// выше 65535 физически невозможны — это инвариант сетевого стека.
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_u16(field.name(), value as u16)
    }

    fn record_str(&mut self, field: &Field, v: &str) {
        self.record(field.name(), v)
    }
}

impl ServiceDescription {
    pub fn new(vendor: &str, name: &str, version: &str, host: net::IpAddr) -> Self {
        Self {
            vendor: vendor.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
            host,
        }
    }
}

/// Преобразует уровень `tracing` в числовой severity CEF (0–10).
///
/// CEF severity: 0–3 = Low, 4–6 = Medium, 7–8 = High, 9–10 = Very High.
/// Пропуск значений 4 и 6–7 — намеренный: оставляет место для будущих уровней.
/// `tracing::Level` имеет размер 8 байт, поэтому принимается по ссылке.
pub fn event_severity(lvl: &Level) -> u8 {
    match *lvl {
        Level::TRACE => 1,
        Level::DEBUG => 2,
        Level::INFO => 3,
        Level::WARN => 5,
        Level::ERROR => 8,
    }
}
