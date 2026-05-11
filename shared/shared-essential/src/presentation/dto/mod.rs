//! Описание контрактов общения между сервисами
pub mod estimated_commission;
pub mod import;
pub mod integration;
pub mod log_storage;
pub mod master_data;
pub mod notification;
pub mod price_analysis;
pub mod print_docs;
pub mod processing;
pub mod response_request;
pub mod scheduler;
pub mod specialized_departments;
pub mod technical_commercial_proposal;
pub mod view_storage;

pub mod error;
pub use error::{AsezError, AsezErrorDict, AsezResult, Source};

pub mod export;
pub mod general;
pub mod value;
pub use value::UiValue;

#[cfg(test)]
mod tests;
