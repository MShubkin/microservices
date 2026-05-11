//! This module contatins the rabbit related infrastructure, including the configuration,
//! queue spawner and holder.
use crate::common::rules::ProcessingRules;
use crate::common::{
    MonolithSender, ProcessingCtx, ProcessingError, RabbitConfig, Result,
};

use amqprs::channel::{BasicConsumeArguments, QueueDeclareArguments};
use broker::rabbit::RabbitConsumer;
use broker::BrokerAdapter;
use futures::future::BoxFuture;
use futures::StreamExt;
use sqlx::PgPool;

use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;
use std::fmt;
use std::ops::Deref;
use std::result::Result as StdResult;
use std::sync::Arc;

pub(crate) mod processes;

impl RabbitConfig {
    /// This function exists as we often use it in tests.
    #[tracing::instrument(skip_all)]
    pub(crate) async fn spawn_nest(
        &self,
        q: QueueSpec,
        db_pool: &Arc<PgPool>,
        monolith_service: &Arc<MonolithService<MonolithHttpDriver>>,
    ) -> Result<(ProcessingCtx, RabbitConsumer, RabbitRunner)> {
        tracing::info!("Processing: Creating queue {:?}", q.entrance_name);
        fn trace_queue<T: std::fmt::Display>(e: T, queue_name: &str) -> T {
            tracing::error!(
                kind = "broker",
                "Processing: Error registering queue on {}: {}",
                queue_name,
                e
            );
            e
        }

        let adaptor = self.get_rabbit().await?;

        let entrance_args = QueueDeclareArguments::default()
            .queue(q.entrance_name.to_owned())
            .durable(true)
            .finish();
        // Declare the main queues.
        adaptor
            .declare_queue(entrance_args)
            .await
            .map_err(|e| trace_queue(e, &q.entrance_name))?;

        // Declare the auxillary queues for sending to the plans server.
        for queue_name in q.extra_queues.iter() {
            let extra_q_args = QueueDeclareArguments::default()
                .queue(queue_name.to_owned())
                .durable(true)
                .finish();
            adaptor
                .declare_queue(extra_q_args)
                .await
                .map_err(|e| trace_queue(e, queue_name))?;
        }

        let tag = format!("{}-consumer", q.entrance_name);
        let args = BasicConsumeArguments::new(&q.entrance_name, &tag);
        let entrance = adaptor.register_consumer(args).await.map_err(|e| {
            tracing::error!(
                kind = "broker",
                "Processing: Error registering consumer on {}: {}",
                q.entrance_name,
                e
            );
            e
        })?;
        let rules = ProcessingRules::new(db_pool.deref()).await?;

        let pcxt = ProcessingCtx {
            entrance_queue_name: q.entrance_name,
            adaptor: adaptor.into(),
            db_pool: Arc::clone(db_pool),
            rules: rules.into(),
            monolith_service: Arc::clone(monolith_service),
            tracing_fields: None,
        };
        let runner = q.runner;
        Ok((pcxt, entrance, *runner))
    }

    /// This function creates:
    /// 1. The HTTP client.
    /// 2. The rabbit listener queues.
    /// 3. Launches the main loop.
    #[tracing::instrument(skip_all)]
    pub(crate) async fn dig_forever(
        self,
        queues: Vec<QueueSpec>,
        db_pool: PgPool,
        monolith_service: MonolithService<MonolithHttpDriver>,
    ) -> Result<()> {
        tracing::info!("Processing: Launching main loop");

        let monolith_sender =
            MonolithSender::new(&self, &db_pool).await?.add_silent_pool().await?;

        monolith_sender.run();

        let count = queues.len();
        let db_pool = Arc::new(db_pool);

        let monolith_service = Arc::new(monolith_service);

        let mut handles = vec![];
        for q in queues {
            let entrance_name = q.entrance_name.clone();
            let (nest, mut consumer, runner) =
                self.spawn_nest(q, &db_pool, &monolith_service).await?;

            handles.push(tokio::task::spawn(async move {
                tracing::info!(
                    "Processing: Spawned listener loop for {}",
                    entrance_name
                );
                loop {
                    // Tracing is done mostly in inner functions`.
                    // However we DO use a catch all error here just in case.
                    if let Err(error) = runner(nest.clone(), &mut consumer).await {
                        if error.is_broker_timeout() {
                            tracing::trace!(
                                kind = "broker",
                                %error,
                                "Таймаут получения запроса с кролика"
                            );
                        } else {
                            tracing::error!(
                                kind = "broker",
                                %error,
                                "Oшибка при обработке запроса с кролика"
                            );
                        }
                    }
                }
            }));
        }
        let mut live_handles =
            futures::stream::iter(handles).buffer_unordered(count);

        // This makes sure that we await all handles in whatever order they come before
        // we shut down.
        while live_handles.next().await.is_some() {}
        monolith_sender.stop().await;
        Ok(())
    }
}

pub(crate) type FutureAlias<'a> = BoxFuture<'a, StdResult<(), ProcessingError>>;
/// This type exists to make our life easier.
/// It describes the argument and outputs of a function that the RabbitHole
/// can run.
/// TODO: A better way of running the future that allows us to use an `async`
/// outer function.
pub(crate) type RabbitRunner =
    for<'a> fn(ProcessingCtx, &'a mut RabbitConsumer) -> FutureAlias<'a>;

/// When creating a RabbitNest we need the names of the entrance and exit queue
/// as well as the function that will be run in order to create the nest.
pub(crate) struct QueueSpec {
    /// "request" or "response" is to be prepended based on what the queue is doing.
    pub(crate) entrance_name: String,
    pub(crate) runner: Box<RabbitRunner>,
    pub(crate) extra_queues: Vec<String>,
}

impl fmt::Debug for QueueSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("QueueSpec")
            .field("entrance_name", &self.entrance_name)
            .finish()
    }
}

impl QueueSpec {
    pub(crate) fn new(
        entrance_name: &str,
        runner: RabbitRunner,
        queues: &[&str],
    ) -> Self {
        Self {
            entrance_name: entrance_name.to_string(),
            runner: Box::new(runner),
            extra_queues: queues.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
        }
    }

    /// This creates the specifications for the four default queues.
    /// More queues can be declared dynamically, but it is not clear
    /// why we would do so, or how we would use them.
    pub(crate) fn default_queues() -> Vec<QueueSpec> {
        const QUEUES: &[&str] = &[
            rabbit_services::routing::PROCESSING_PLAN_QUEUE,
            rabbit_services::routing::PROCESSING_AMEND_QUEUE,
        ];

        let plan = processes::process_plans_queue as RabbitRunner;
        let legacy = processes::process_legacy_queue as RabbitRunner;

        vec![
            QueueSpec::new(
                rabbit_services::routing::PROCESSING_QUEUE,
                plan,
                QUEUES,
            ),
            QueueSpec::new(
                rabbit_services::routing::PROCESSING_PLAN_QUEUE,
                legacy,
                &[],
            ),
        ]
    }
}
