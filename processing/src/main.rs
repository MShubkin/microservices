//! Архитектура : 'FE->Http->Микросервисы->rabbitemq->Микросервисы'
//! По словам Вадима Савитского:
//! "Создай мне Протокол(FE)->http->
//!  Сметная комиссия (BE)->rabbitem->
//!  Processing-rabbitemq->Сметная комиссия (BE)->
//!  http->FE"
//!
//! Т.Е. у нас архитектура у Processing "rabbitmq->Processing->rabbitmq"
pub(crate) mod app_process;
pub(crate) mod common;
pub(crate) mod infrastructure;
pub(crate) mod presentation;

use crate::common::Result;
use crate::infrastructure as infra;

const MIN_THREADS: u8 = 4;

/// In order to be able to configure thread count on launch we use this
/// approach instead of the more usual `#[tokio::main]`.
fn main() -> Result<()> {
    println!("Reading Processing config from env...");
    let config = infra::ProcessingConfig::from_env()?;
    // We never use less than 4 threads.
    let num_threads = std::cmp::max(config.server_thread_count, MIN_THREADS);

    println!("Initialising Processing with {} async threads...", num_threads);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(usize::from(num_threads))
        .enable_all()
        .build()?
        .block_on(inner_main(config))
}

/// This is the inner function to main. It is also its own function to
/// allow easier testing
pub(crate) async fn inner_main(cfg: infra::ProcessingConfig) -> Result<()> {
    println!("Initializing tracing..");
    let (_app_guard, span) = cfg.initiate_log()?;
    let _guard = span.enter();

    let queues = infra::QueueSpec::default_queues();

    // Полная структура cfg более не логируется: даже с redacted-Debug это утечка
    // топологии (хосты, порты, лимиты) в SIEM. Конкретные поля логируются по месту,
    // где они используются.
    tracing::info!(kind = "infra", "Processing config loaded");
    println!("About to launch Processing service loop.");

    // NB: Selecting between either process here means that the whole service
    // dies when one dies. This way we do not have a health-check running once
    // the main service dies or a phantom service running if the health-check dies.
    let (host, port) = (cfg.host.clone(), cfg.port);
    tokio::select! {
        x = cfg.launch(queues) => trace_end(x, "Processing health-check"),
        x = infrastructure::start_http_server(&host, port) => trace_end(x, "Processing"),
    }
}

fn trace_end<T>(inp: Result<T>, name: &str) -> Result<T> {
    inp.map_err(|e| {
        tracing::error!(
            kind = "siem",
            "{process} ended with error: {e}",
            process = name,
            e = e
        );
        e
    })
    .map(|x| {
        tracing::info!(kind = "siem", "{} has ended", name);
        x
    })
}
