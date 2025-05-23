use std::sync::Arc;

use graph::{prelude::MetricsRegistry, prometheus::Registry};

#[macro_use]
extern crate diesel;

pub mod chain;
pub mod config;
pub mod dev;
pub mod launcher;
pub mod manager;
pub mod network_setup;
pub mod opt;
pub mod store_builder;
pub struct MetricsContext {
    pub prometheus: Arc<Registry>,
    pub registry: Arc<MetricsRegistry>,
    pub prometheus_host: Option<String>,
    pub job_name: Option<String>,
}
