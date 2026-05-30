//! Имена очередей и маршруты ко всем сервисам АСЭЗ 2.0.
//!
//! Строковые константы здесь -- это физические имена очередей в брокере.
//! Менять их надо синхронно с конфигурацией принимающего сервиса; расхождение
//! имён -- самая частая причина молчащего RPC (сообщение уходит, ответ не приходит).

// Очереди сервиса Processing

/// Очередь общих запросов на обработку (процессинг).
pub const PROCESSING_QUEUE: &str = "processing";
/// Очередь входящих ППЗ (не ДС) от монолита.
pub const PROCESSING_PLAN_QUEUE: &str = "plans";
/// Очередь входящих ДС (не ППЗ) от монолита.
pub const PROCESSING_AMEND_QUEUE: &str = "contract_amendment";

// Очереди сервиса Views

/// Очередь TCP (технико-коммерческое предложение).
pub const TCP_QUEUE: &str = "tcp_action";
/// Очередь для получения полей, доступных пользователю.
pub const VIEWS_ALLOWED_FIELDS_QUEUE: &str = "allowed_fields";
/// Очередь валидации сессии пользователя.
pub const VIEWS_CHECK_SESSION_QUEUE: &str = "check_session";
/// Очередь проверки прав на действие.
pub const VIEWS_CHECK_PERMISSION_QUEUE: &str = "check_permission";
/// Очередь обновления сессий (вход/выход пользователей от монолита).
pub const VIEWS_UPDATE_USER_SESSIONS_QUEUE: &str = "user_sessions";
/// Очередь для получения прав пользователя и видимых разделов.
pub const VIEWS_GET_USER_RIGHTS_AND_SECTIONS_QUEUE: &str =
    "user_rights_and_sections";
/// Очередь хранилища уведомлений и их конфигурации.
pub const VIEWS_NOTIFICATION_STORAGE_QUEUE: &str = "notification_storage";

// Очереди сервиса НСИ (master data)

/// Очередь запросов на получение справочников НСИ.
pub const REQUEST_DICTIONARY_QUEUE: &str = "dictionary";
/// Очередь действий над НСИ (маршруты, орг.структура и т.д.).
pub const MASTER_DATA_ACTION_QUEUE: &str = "master_data_action";
/// Очередь сервиса авторазбора (автоматическое назначение маршрутов).
pub const ROUTING_QUEUE: &str = "auto_routing";

/// Очередь сервиса уведомлений.
pub const NOTIFICATIONS_QUEUE: &str = "notifications";

/// Очередь генерации документов (PrintDoc).
pub const REQUEST_PRINTDOC_QUEUE: &str = "print_doc";

// Очереди сервиса Integration

/// Очередь хранения/получения документов (OpenText).
pub const DOCUMENTS_QUEUE: &str = "documents";
/// Очередь взаимодействия с SAP PI.
pub const INTEGRATION_QUEUE: &str = "integration";

/// Очередь сервиса хранения логов.
pub const LOG_STORAGE_QUEUE: &str = "logs";

// Очереди модуля Спецотделов

/// Очередь, в которую планирование публикует данные для модуля Спецотделов.
pub const SPEC_DEPS_DATA_CONSUME_QUEUE: &str =
    "asez2.planning.specialized_departments.publish";

/// RPC-очередь модуля Спецотделов.
pub const SPEC_DEPS_RPC_QUEUE: &str = "asez2.specialized_departments.rpc";

/// Очередь для публикации результатов Спецотделов.
pub const SPEC_DEPS_RESULT_PUBLISH_QUEUE: &str =
    "asez2.specialized_departments_result.publish";

/// Очередь подтверждения результатов Спецотделов.
pub const SPEC_DEPS_RESULT_CONFIRM_QUEUE: &str =
    "asez2.specialized_departments_result.confirm";

/// RPC-очередь модуля календаря Планировщика.
pub const SCHEDULER_CALENDAR_RPC_QUEUE: &str = "asez2.calendar.rpc";

/// Все возможные маршруты к сервисам системы.
///
/// Используется как аргумент при вызове [`AsezRabbitService::service_request`].
/// Значение однозначно определяет пару (exchange, queue), в которую будет опубликовано сообщение.
/// Пустой exchange означает использование стандартного direct-обменника RabbitMQ.
pub enum AsezRabbitRouting {
    /// Маршрут к сервису Processing (общие запросы на обработку).
    Processing,
    /// Маршрут к сервису TCP (технико-коммерческое предложение).
    TcpAction,
    /// Маршрут к сервису Views -- поля, доступные пользователю.
    ViewsAllowedFields,
    /// Маршрут к сервису Views -- проверка сессии.
    ViewsCheckSession,
    /// Маршрут к сервису Views -- проверка прав на действие.
    ViewsCheckPermission,
    /// Маршрут к сервису Views -- обновление сессий аутентификации.
    ViewsUpdateUserSessions,
    /// Маршрут к сервису Views -- права пользователя и видимые разделы.
    ViewsPricingSectionUser,
    /// Маршрут к сервису Views -- данные уведомлений.
    ViewsNotificationStorage,
    /// Маршрут к сервису НСИ -- получение справочников.
    RequestDictionaries,
    /// Маршрут к модулю календаря Планировщика.
    SchedulerCalendar,
    /// Маршрут к сервису НСИ -- действия над мастер-данными.
    MasterDataAction,
    /// Маршрут к сервису авторазбора.
    Routing,
    /// Маршрут к сервису уведомлений.
    Notifications,
    /// Маршрут к сервису генерации документов.
    RequestPrintDoc,
    /// Маршрут к сервису Integration -- SAP PI.
    IntegrationCallPi,
    /// Маршрут к сервису Integration -- хранение документов (OpenText).
    IntegrationOpenText,
    /// Маршрут к сервису хранения логов.
    Log,
    /// Маршрут для публикации результатов Спецотделов.
    SpecDepsResult,
    /// RPC-маршрут модуля Спецотделов.
    SpecDepsRPC,
}

impl AsezRabbitRouting {
    /// Возвращает пару `(exchange, queue)` для публикации сообщения.
    ///
    /// Пустой exchange означает прямую маршрутизацию по имени очереди (default exchange).
    pub fn as_full_routing(&self) -> (&'static str, &'static str) {
        use AsezRabbitRouting::*;
        match self {
            Processing => ("", PROCESSING_QUEUE),
            TcpAction => ("", TCP_QUEUE),
            ViewsAllowedFields => ("", VIEWS_ALLOWED_FIELDS_QUEUE),
            ViewsCheckSession => ("", VIEWS_CHECK_SESSION_QUEUE),
            ViewsCheckPermission => ("", VIEWS_CHECK_PERMISSION_QUEUE),
            ViewsUpdateUserSessions => ("", VIEWS_UPDATE_USER_SESSIONS_QUEUE),
            ViewsNotificationStorage => ("", VIEWS_NOTIFICATION_STORAGE_QUEUE),
            RequestDictionaries => ("", REQUEST_DICTIONARY_QUEUE),
            MasterDataAction => ("", MASTER_DATA_ACTION_QUEUE),
            Routing => ("", ROUTING_QUEUE),
            Notifications => ("", NOTIFICATIONS_QUEUE),
            RequestPrintDoc => ("", REQUEST_PRINTDOC_QUEUE),
            IntegrationCallPi => ("", INTEGRATION_QUEUE),
            IntegrationOpenText => ("", DOCUMENTS_QUEUE),
            ViewsPricingSectionUser => {
                ("", VIEWS_GET_USER_RIGHTS_AND_SECTIONS_QUEUE)
            }
            Log => ("", LOG_STORAGE_QUEUE),
            SpecDepsResult => ("", SPEC_DEPS_RESULT_PUBLISH_QUEUE),
            SpecDepsRPC => ("", SPEC_DEPS_RPC_QUEUE),
            SchedulerCalendar => ("", SCHEDULER_CALENDAR_RPC_QUEUE),
        }
    }
}
