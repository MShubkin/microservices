pub mod callbacks;

pub mod properties;

pub mod routing;
pub use routing::*;

pub mod services;
pub use services::*;

pub mod consume;
pub mod publish;

/// Content type used to exchange message via RabbitMQ.
const CONTENT_TYPE: &str = "application/json";
