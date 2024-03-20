#![allow(unused)]
pub use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

pub struct Module;

impl Module {
    pub const APP: &'static str = "app";
    pub const REST: &'static str = "rest";
    pub const COLLECTOR: &'static str = "collector";
    pub const STRATEGY: &'static str = "strategy";
    pub const EXECUTOR: &'static str = "executor";
    pub const ENGINE: &'static str = "engine";
}

pub fn init_log() {
    let env_filter = EnvFilter::new("info,tokio_reactor=info,tower_http=info,app=info,rest=info");

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .compact()
        .json()
        .flatten_event(true)
        .init();
}
