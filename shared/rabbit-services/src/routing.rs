/// Queue name for the `Processing` service (general processing requests).
pub const PROCESSING_QUEUE: &str = "processing";
/// Queue for incoming PPZ (not DS) from the monolith.
pub const PROCESSING_PLAN_QUEUE: &str = "plans";
/// Queue for incoming DS (not PPZ) from the monolith.
pub const PROCESSING_AMEND_QUEUE: &str = "contract_amendment";

/// Queue name for the TCP (Technical-Commercial Proposal) service.
pub const TCP_QUEUE: &str = "tcp_action";
/// Queue name for the `Views` service — fetching fields allowed for a user.
pub const VIEWS_ALLOWED_FIELDS_QUEUE: &str = "allowed_fields";
/// Queue name for the `Views` service — session validation.
pub const VIEWS_CHECK_SESSION_QUEUE: &str = "check_session";
/// Queue name for the `Views` service — action permission check.
pub const VIEWS_CHECK_PERMISSION_QUEUE: &str = "check_permission";
/// Queue for receiving login/logout data from the monolith.
pub const VIEWS_UPDATE_USER_SESSIONS_QUEUE: &str = "user_sessions";
/// Queue for receiving user operation rights and visible sections.
pub const VIEWS_GET_USER_RIGHTS_AND_SECTIONS_QUEUE: &str =
    "user_rights_and_sections";
/// Queue responsible for notifications and their configuration.
pub const VIEWS_NOTIFICATION_STORAGE_QUEUE: &str = "notification_storage";

/// Queue name for the `NSI` (master data) service — fetching dictionaries.
pub const REQUEST_DICTIONARY_QUEUE: &str = "dictionary";
/// Queue name for the `NSI` (master data) service — performing actions.
pub const MASTER_DATA_ACTION_QUEUE: &str = "master_data_action";
/// Queue for auto-routing assignments.
pub const ROUTING_QUEUE: &str = "auto_routing";

/// Queue name for the `Notifications` service.
pub const NOTIFICATIONS_QUEUE: &str = "notifications";

/// Queue name for the `PrintDoc` service — document generation.
pub const REQUEST_PRINTDOC_QUEUE: &str = "print_doc";

/// Queue name for the `Integration` service — document storage/retrieval.
pub const DOCUMENTS_QUEUE: &str = "documents";
/// Queue name for the `Integration` service — SAP PI interaction.
pub const INTEGRATION_QUEUE: &str = "integration";

/// Queue name for the `log-storage` service.
pub const LOG_STORAGE_QUEUE: &str = "logs";

/// Queue on which planning publishes data for the Specialized Departments module.
pub const SPEC_DEPS_DATA_CONSUME_QUEUE: &str =
    "asez2.planning.specialized_departments.publish";

/// RPC API queue for the Specialized Departments module.
pub const SPEC_DEPS_RPC_QUEUE: &str = "asez2.specialized_departments.rpc";

/// Queue for publishing Specialized Departments results.
pub const SPEC_DEPS_RESULT_PUBLISH_QUEUE: &str =
    "asez2.specialized_departments_result.publish";

/// Queue for confirming Specialized Departments results.
pub const SPEC_DEPS_RESULT_CONFIRM_QUEUE: &str =
    "asez2.specialized_departments_result.confirm";

/// RPC API queue for the Scheduler calendar module.
pub const SCHEDULER_CALENDAR_RPC_QUEUE: &str = "asez2.calendar.rpc";

/// All possible routes to services in the system.
pub enum AsezRabbitRouting {
    /// Route to the `Processing` service — general processing requests.
    Processing,
    /// Route to the `TCP` (Technical-Commercial Proposal) service.
    TcpAction,
    /// Route to the `Views` service — user allowed fields.
    ViewsAllowedFields,
    /// Route to the `Views` service — session check.
    ViewsCheckSession,
    /// Route to the `Views` service — permission check.
    ViewsCheckPermission,
    /// Route to the `Views` service — update session authentication.
    ViewsUpdateUserSessions,
    /// Route to the `Views` service — user rights and section visibility.
    ViewsPricingSectionUser,
    /// Route to the `Views` service — notifications data.
    ViewsNotificationStorage,
    /// Route to the `NSI` service — fetch dictionaries.
    RequestDictionaries,
    /// Route to the Scheduler calendar module.
    SchedulerCalendar,
    /// Route to the `NSI` service — perform master data actions.
    MasterDataAction,
    /// Route to the auto-routing service.
    Routing,
    /// Route to the `Notifications` service.
    Notifications,
    /// Route to the `PrintDoc` service — document generation.
    RequestPrintDoc,
    /// Route to the `Integration` service — SAP PI.
    IntegrationCallPi,
    /// Route to the `Integration` service — document storage (OpenText).
    IntegrationOpenText,
    /// Route to the `log-storage` service.
    Log,
    /// Route for publishing Specialized Departments results.
    SpecDepsResult,
    /// RPC route for the Specialized Departments module.
    SpecDepsRPC,
}

impl AsezRabbitRouting {
    /// Returns the full routing tuple: (`exchange_name`, `queue_name`).
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
