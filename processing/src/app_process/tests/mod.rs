#![allow(clippy::inconsistent_digit_grouping)]

mod sections;

mod calls_legacy;
mod estimated_commission;
mod price_analysis;

use amqprs::channel::QueueDeclareArguments;
use sqlx::{
    types::{Json as SqlxJ, Uuid},
    FromRow, PgPool,
};
use std::sync::Arc;
use tables::processing::regulatory_deadline_price::RegulatoryDeadlinePrice;
use tokio::task::JoinHandle;

use asez2_shared_db::db_item::{Filter, FilterTree};
use asez2_shared_db::{db_item::Select, DbAdaptor, DbItem, Value};
use broker::rabbit::RabbitAdapter;
use broker::{BrokerAdapter, RetryArgs};
use rabbit_services::properties::AsezRabbitProperties;
use rabbit_services::PROCESSING_QUEUE;
use shared_essential::{
    domain::{
        maths::*,
        tables::processing::field_histories::{FieldChange, HistoryStatus},
        *,
    },
    presentation::dto::{
        general::{ObjectIdentifier, ObjectIdentifierWithStatusNote},
        processing::*,
        response_request::Message,
        Source,
    },
};

use crate::common::NO_SEND_TO_PLANNING;
use crate::common::{ProcessingCtx, ProcessingError, Result};
use rabbit_services::master_data::MasterDataService;

/// Таблицы процессинга, наполняемые в тестах.
///
/// Эти таблицы должны чиститься перед каждым вызовом тестов.
const PROCESSING_TRANSIENT_TABLES: &[&str] = &[
    ContractAmendment::TABLE,
    ContractAmendmentVersion::TABLE,
    ContractAmendmentItem::TABLE,
    ContractAmendmentItemVersion::TABLE,
    EcAgenda::TABLE,
    EcAgendaItem::TABLE,
    EsCommissionResult::TABLE,
    EcProtocol::TABLE,
    EcProtocolItem::TABLE,
    FieldChange::TABLE,
    Plan::TABLE,
    PlanItem::TABLE,
    PlanVersion::TABLE,
    PlanItemVersion::TABLE,
    PlanLegacy::TABLE,
    PlanItemLegacy::TABLE,
    RelAgendaProtocol::TABLE,
    RelAgendaProtocolItem::TABLE,
    Attachment::TABLE,
    EcPartner::TABLE,
    PartnerTypeCommission::TABLE,
    StatusHistory::TABLE,
    DocumentApprover::TABLE,
    RegulatoryDeadlinePrice::TABLE,
    PlanRetrospective::TABLE,
    PriceAnalysisUser::TABLE,
];

const USER1: i32 = 658;

impl asez2_tables::test_setup::TestSetupError for ProcessingError {}

/// THis nest has no valid rabbit
pub(crate) async fn mock_processing_context_inner(
    pool: Arc<PgPool>,
    rcfg: env_setup::RabbitCfg,
    monolith_cfg: env_setup::MonolithCfg,
) -> ProcessingCtx {
    // Do not use this rabbit.
    let args = amqprs::connection::OpenConnectionArguments::new(
        &rcfg.host, rcfg.port, &rcfg.user, &rcfg.pw,
    )
    .virtual_host(&rcfg.vhost)
    .finish();
    let retry_args = RetryArgs::new(rcfg.retries, rcfg.retry_interval_ms);
    let adaptor = RabbitAdapter::connect(args, retry_args).await.unwrap();

    // Auto-create queue if not defined
    let queue_args: QueueDeclareArguments =
        QueueDeclareArguments::new(PROCESSING_QUEUE).durable(true).finish();
    adaptor.declare_queue(queue_args).await.unwrap();

    let rules = crate::common::rules::ProcessingRules::new(&*pool).await.unwrap();

    let driver = MonolithHttpDriver::basic_driver(monolith_cfg.url)
        .expect("Ошибка при настройке http драйвера");
    let monolith_service = Arc::new(MonolithService::new(driver));

    ProcessingCtx {
        db_pool: pool,
        entrance_queue_name: String::new(),
        adaptor: adaptor.into(),
        rules: rules.into(),
        monolith_service,
        tracing_fields: None,
    }
}
/// This nest builds a rabbit with default settings.
pub(crate) async fn mock_processing_context(pool: Arc<PgPool>) -> ProcessingCtx {
    let rcfg = env_setup::RabbitCfg::from_env().unwrap();
    let monolith_cfg = env_setup::MonolithCfg::from_env().unwrap();
    mock_processing_context_inner(pool, rcfg, monolith_cfg).await
}

async fn mock_rabbit_adapter() -> RabbitAdapter {
    let config = env_setup::RabbitCfg::from_env().unwrap();
    shared_essential::infrastructure::rabbit::setup_rabbit_adapter(&config)
        .await
        .unwrap()
}

pub(crate) async fn master_data_service(pctx: &ProcessingCtx) -> MasterDataService {
    MasterDataService::new(
        pctx.adaptor.clone(),
        AsezRabbitProperties::default(),
        Source::Processing,
    )
}

/// This nest builds a rabbit with a manually assigned Vhost.
pub(crate) async fn mock_processing_context_with_vhost(
    pool: Arc<PgPool>,
    vhost: &str,
) -> ProcessingCtx {
    let mut rcfg = env_setup::RabbitCfg::from_env().unwrap();
    rcfg.vhost = vhost.to_owned();
    let monolith_cfg = env_setup::MonolithCfg::from_env().unwrap();
    mock_processing_context_inner(pool, rcfg, monolith_cfg).await
}

pub(crate) async fn run_db_test<F, FutFn>(
    extra_migs_files: &'static [&'static str],
    run: FutFn,
) where
    F: futures::Future<Output = ()>,
    FutFn: FnOnce(Arc<PgPool>) -> F + 'static,
{
    testing::BaseMigPath::MigrationsHome
        .run_test_with_migrations(
            "src/app_process/tests/extra_migrations",
            extra_migs_files,
            PROCESSING_TRANSIENT_TABLES,
            run,
        )
        .await
}

pub(crate) async fn run_db_rabbit_test<F, FutFn>(
    extra_migs_files: &'static [&'static str],
    run: FutFn,
) where
    F: futures::Future<Output = ()>,
    FutFn: FnOnce(Arc<PgPool>, Arc<RabbitAdapter>) -> F + 'static,
{
    let rabbit = mock_rabbit_adapter().await.into();
    let run = move |db_pool| run(db_pool, rabbit);
    testing::BaseMigPath::MigrationsHome
        .run_test_with_migrations(
            "src/app_process/tests/extra_migrations",
            extra_migs_files,
            PROCESSING_TRANSIENT_TABLES,
            run,
        )
        .await
}

pub(self) use monolith::*;
use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;

pub(super) mod monolith {
    use super::*;
    use crate::presentation::legacy_interaction::*;

    use amqprs::channel::{BasicConsumeArguments, BasicPublishArguments};
    use amqprs::BasicProperties;
    use broker::rabbit::RabbitConsumer;
    use broker::Consumer;
    use env_setup::RabbitCfg;
    use shared_essential::presentation::dto::response_request::Messages;

    async fn create_source_consumer(adaptor: &RabbitAdapter) -> RabbitConsumer {
        let extra_q_args = QueueDeclareArguments::default()
            .queue(SEND_TO_MONOLITH_QUEUE.to_owned())
            .durable(false)
            .finish();
        _ = adaptor.declare_queue(extra_q_args).await;

        let args = BasicConsumeArguments::new(
            SEND_TO_MONOLITH_QUEUE,
            "plans_source-consumer",
        );
        adaptor.register_consumer(args).await.unwrap()
    }

    async fn publish(adaptor: &RabbitAdapter, rsvp: &str, msg: Messages) {
        let args = BasicPublishArguments::new("", rsvp);
        let props = BasicProperties::default()
            .with_content_type("application/json")
            .with_persistence(true)
            .finish();

        let publisher = adaptor.register_publisher(props, args).await.unwrap();
        let expiration = std::time::Duration::from_millis(3_000);

        publisher.publish_with_expiration(&msg, expiration).await.unwrap();
    }

    /// This function has all the processes needed to send and receive items to the
    /// "planning" module, also known as the "monolith".
    pub(super) async fn launch_monolith_listener(
        n: &ProcessingCtx,
        mut comparator: Vec<ProcessingToLegacyReq>,
    ) -> JoinHandle<()> {
        if std::env::var(NO_SEND_TO_PLANNING).is_ok() {
            println!("No monolith sender");
            return tokio::task::spawn(async {});
        }
        let adaptor = n.adaptor.clone();
        let rabbit_cfg = RabbitCfg::from_env().unwrap().into();
        let monolith_sender =
            crate::common::MonolithSender::new(&rabbit_cfg, &n.db_pool)
                .await
                .unwrap();
        monolith_sender.run();

        tokio::task::spawn(async move {
            let mut source_consumer = create_source_consumer(&adaptor).await;
            let mut output_comp = Vec::new();

            while let Ok(mut msg) = source_consumer.consume().await {
                if let ProcessingToLegacyReq::UpdatePlans(ref mut p) = msg.content {
                    for plan in p.iter_mut() {
                        plan.header.changed_at = None;
                        plan.header.created_at = None;
                    }
                }
                if let ProcessingToLegacyReq::UpdateAmendments(ref mut a) =
                    msg.content
                {
                    for amendment in a.iter_mut() {
                        amendment.header.changed_at = None;
                        amendment.header.created_at = None;
                    }
                }
                output_comp.push(msg.content);

                let rsvp = msg.properties.reply_to().unwrap();
                publish(&adaptor, rsvp, Messages::default()).await;
                // We disable time fields on the plan, because it harms us.
                if comparator.len() == output_comp.len() {
                    monolith_sender.stop().await;
                    break;
                }
            }

            comparator.sort_by(|a, b| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            });
            output_comp.sort_by(|a, b| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            });

            assert_eq!(
                comparator, output_comp,
                "{:#?}\n{:#?}",
                comparator, output_comp
            )
        })
    }

    /// This function has all the processes needed to send and receive items to the
    /// "planning" module, also known as the "monolith".
    pub(super) async fn launch_monolith_listener_return_error(
        n: &ProcessingCtx,
        comparator: usize,
    ) -> JoinHandle<()> {
        if std::env::var(NO_SEND_TO_PLANNING).is_ok() {
            println!("No monolith sender");
            return tokio::task::spawn(async {});
        }
        let adaptor = n.adaptor.clone();
        let rabbit_cfg = RabbitCfg::from_env().unwrap().into();
        let monolith_sender =
            crate::common::MonolithSender::new(&rabbit_cfg, &n.db_pool)
                .await
                .unwrap();
        monolith_sender.run();

        tokio::task::spawn(async move {
            let mut source_consumer = create_source_consumer(&adaptor).await;
            let mut output_comp = 0;

            while let Ok(msg) = source_consumer.consume().await {
                let _: ProcessingToLegacyReq = msg.content;
                let rsvp = msg.properties.reply_to().unwrap();

                output_comp += 1;

                let messages = Messages::from(vec![Message::error("Oh no")]);

                publish(&adaptor, rsvp, messages).await;
                // We disable time fields on the plan, because it harms us.
                if comparator == output_comp {
                    monolith_sender.stop().await;
                    break;
                }
            }
        })
    }
}
