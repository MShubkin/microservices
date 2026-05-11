use monolith_service::http::MonolithHttpService;
use sqlx::PgPool;

/// ! Infrastructure is a layer that works with third–party
/// ! libraries, frameworks, and so on.
use broker::rabbit::RabbitAdapter;
pub use env::Env;
pub use shared_essential::infrastructure::rabbit::setup_rabbit_adapter;

use std::sync::Arc;

mod env;
pub mod rabbit;
pub mod web;

#[derive(Clone)]
pub struct GlobalConfig {
    pub env: Env,
    pub broker_adapter: Arc<RabbitAdapter>,
    pub db_pool: Arc<PgPool>,
    pub monolith: MonolithHttpService,
}

impl GlobalConfig {
    pub fn new(
        env: Env,
        broker_adapter: Arc<RabbitAdapter>,
        db_pool: Arc<PgPool>,
        monolith: MonolithHttpService,
    ) -> Self {
        Self {
            env,
            broker_adapter,
            db_pool,
            monolith,
        }
    }
}
