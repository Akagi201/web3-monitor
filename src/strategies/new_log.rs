use anyhow::Result;
use async_trait::async_trait;
use ethers::types::Log;

use crate::types::{Actions, Events, Strategy};

#[derive(Default, Debug)]
pub struct NewLogStrategy {
    pub last_height: u64,
}

#[async_trait]
impl Strategy<Events, Actions> for NewLogStrategy {
    async fn sync_state(&mut self) -> Result<()> {
        Ok(())
    }
    async fn process_event(&mut self, event: Events) -> Vec<Actions> {
        match event {
            Events::Log(log) => self.handle_new_log(log),
            _ => vec![],
        }
    }
}

impl NewLogStrategy {
    fn handle_new_log(&mut self, data: Log) -> Vec<Actions> {
        if let Some(block_number) = data.block_number {
            if block_number.as_u64() > self.last_height {
                self.last_height = block_number.as_u64();
                return vec![Actions::DryRun(format!("New log: {:?}", data))];
            }
        }
        vec![]
    }
}
