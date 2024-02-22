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
use ethers::{
    providers::{Provider, Ws},
    types::Filter,
};
use log::*;
use shadow_rs::shadow;

use crate::{
    collectors::{
        block_collector::BlockCollector, log_collector::LogCollector,
        mempool_collector::MempoolCollector,
    },
    engine::Engine,
    executors::dry_run::DryRunExecutor,
    strategies::{new_block::NewBlockStrategy, new_log::NewLogStrategy},
    types::{Actions, CollectorMap, Events, ExecutorMap},
};

shadow!(build);

const UNISWAPV2_ROUTER02: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

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

    let filter = Filter::new().address(vec![UNISWAPV2_ROUTER02.parse()?]);
    let log_collector = Box::new(LogCollector::new(provider.clone(), filter));
    let log_collector = CollectorMap::new(log_collector, Events::Log);
    engine.add_collector(Box::new(log_collector));

    let block_collector = Box::new(BlockCollector::new(provider.clone()));
    let block_collector = CollectorMap::new(block_collector, Events::NewBlock);
    engine.add_collector(Box::new(block_collector));

    let mempool_collector = Box::new(MempoolCollector::new(provider.clone()));
    let mempool_collector = CollectorMap::new(mempool_collector, Events::Transaction);
    engine.add_collector(Box::new(mempool_collector));

    let new_block_strategy = NewBlockStrategy::default();
    engine.add_strategy(Box::new(new_block_strategy));

    let new_log_strategy = NewLogStrategy::default();
    engine.add_strategy(Box::new(new_log_strategy));

    let dry_run_executor = Box::<DryRunExecutor>::default();
    let dry_run_executor = ExecutorMap::new(dry_run_executor, |action| match action {
        Actions::DryRun(action) => Some(action),
        _ => None,
    });
    engine.add_executor(Box::new(dry_run_executor));

    if let Ok(mut set) = engine.run().await {
        while let Some(res) = set.join_next().await {
            info!(target: Module::ENGINE, "res: {:?}", res);
        }
    }

    Ok(())
}
