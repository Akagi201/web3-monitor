use std::path::PathBuf;

use clap::Parser;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Clone, Parser)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    #[arg(short, long, default_value = "false")]
    pub version: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Web3MonitorConfig {
    pub node: NodeConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    pub rpc: String,
    pub ws: String,
}

impl Web3MonitorConfig {
    pub fn new(config: PathBuf) -> Result<Self, ConfigError> {
        let c = Config::builder()
            .add_source(File::from(config))
            .add_source(Environment::with_prefix("WEB3MONITOR"))
            .build()?;
        c.try_deserialize()
    }
}

pub fn allocator_name() -> &'static str {
    if cfg!(feature = "jemalloc") {
        "jemalloc"
    } else {
        "system"
    }
}
