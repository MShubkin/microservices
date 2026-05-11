pub mod common;
pub mod common_lookup_cfg;
pub mod db_enum_cfg;
pub mod enrichment;
pub mod id_lookup_cfg;
pub mod master_data;
pub mod monolith;
pub mod planning_masterdata;
pub mod rest;

#[cfg(test)]
mod tests;

pub use common::{IntegrationError, IntegrationResult};
