use serde::Serialize;

/// Справочник «Производственный календарь»
pub mod scheduler_update_catalog_request;

#[derive(Serialize, Debug)]
pub struct ProductionDirectoryRequest {
    pub get_directory: bool,
}
