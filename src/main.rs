pub mod app_config;
pub mod collectors;
pub mod engine;
pub mod executors;
pub mod log;
pub mod strategies;
pub mod types;
pub mod utilities;

use std::sync::Arc;

use anyhow::Result;
use app_config::{allocator_name, Cli, Web3MonitorConfig};
use clap::Parser;
use ethers::providers::{Provider, Ws};
use log::*;
use shadow_rs::shadow;

use crate::{collectors::block_collector::BlockCollector, engine::Engine, types::{Actions, CollectorMap, Events}};

shadow!(build);

#[tokio::main]
async fn main() -> Result<()> {
    init_log();
    let cli = Cli::parse();
    if cli.version {
        build::print_build_in();
        println!("allocator: {}", allocator_name());
        return Ok(());
    }

    let config = Web3MonitorConfig::new(cli.config.unwrap())?;
    info!(target: Module::APP, "web3 monitor config: {config:?}");

    let ws = Ws::connect(config.node.ws).await?;
    let provider = Arc::new(Provider::new(ws));

    let mut engine: Engine<Events, Actions> = Engine::default();

    let block_collector = Box::new(BlockCollector::new(provider.clone()));
    let block_collector = CollectorMap::new(block_collector, Events::NewBlock);
    engine.add_collector(Box::new(block_collector));

    Ok(())
}
