use crate::app_process::records::ProcessingRulesChecker;
use crate::common::rules::ProcessingRules;
use broker::rabbit::RabbitAdapter;
use igg_tracing::tracing_fields::AsezTracingFieldsCollection;
use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;
use shared_essential::application::records::RecordCtx;
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;

/// This structure contains all necessary information for working with an
/// entrance and exit queue.
#[derive(Clone)]
pub(crate) struct ProcessingCtx {
    pub(crate) entrance_queue_name: String,
    pub(crate) adaptor: Arc<RabbitAdapter>,
    pub(crate) db_pool: Arc<PgPool>,
    pub(crate) rules: Arc<ProcessingRules>,
    pub(crate) monolith_service: Arc<MonolithService<MonolithHttpDriver>>,
    pub(crate) tracing_fields: Option<AsezTracingFieldsCollection>,
}

impl ProcessingCtx {
    /// Создает контекст фиксации изменений.
    pub(crate) fn create_record_context(&self) -> RecordCtx {
        RecordCtx::new(0, self.db_pool.clone())
    }

    /// Создает контекст для принятия внешних данных (например, из монолита).
    pub(crate) fn create_external_context(&self) -> RecordCtx {
        RecordCtx::new(0, self.db_pool.clone()).with_external_ctx(true)
    }

    /// Создает обработчик правил переходов статусов.
    pub(crate) fn create_rules_checker(&self) -> ProcessingRulesChecker {
        ProcessingRulesChecker::new(self.rules.clone(), self.db_pool.clone())
    }
}

impl fmt::Debug for ProcessingCtx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RabbitNests")
            .field("entrance_queue_name", &self.entrance_queue_name)
            .field("adaptor", &self.adaptor)
            .field("db_pool", &self.db_pool)
            .field("monolith_service", &self.monolith_service)
            .finish()
    }
}
