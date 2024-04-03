use async_trait::async_trait;
use eyre::Result;

use crate::{log::*, types::Executor};

#[derive(Default, Debug)]
pub struct DryRunExecutor {}

#[async_trait]
impl Executor<String> for DryRunExecutor {
    async fn execute(&self, action: String) -> Result<()> {
        info!(target: Module::EXECUTOR, "Dry run: {}", action);
        Ok(())
    }
}
