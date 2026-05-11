pub(crate) mod monolith_sender;
pub(crate) mod number_range;
pub(crate) mod processing_context;
pub(crate) mod rabbit_config;
pub(crate) mod result;
pub(crate) mod rules;
pub(crate) mod status_rollback;
#[cfg(test)]
mod tests;

pub(crate) use monolith_sender::{MonolithSender, MonolithSenderObject};
pub(crate) use number_range::{op_with_numbers, EcObjectType, NumberRequest};
pub(crate) use processing_context::ProcessingCtx;
pub(crate) use rabbit_config::RabbitConfig;
pub(crate) use result::{ProcessingError, Result};

pub(crate) const NO_SEND_TO_PLANNING: &str = "PROCESSING_NO_SEND_TO_PLANNING";
