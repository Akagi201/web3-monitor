use anyhow::Result;
use async_trait::async_trait;

use crate::{
    collectors::block_collector::NewBlock,
    types::{Actions, Events, Strategy},
};

#[derive(Default, Debug)]
pub struct NewBlockStrategy {
    pub last_height: u64,
}

#[async_trait]
impl Strategy<Events, Actions> for NewBlockStrategy {
    async fn sync_state(&mut self) -> Result<()> {
        Ok(())
    }
    async fn process_event(&mut self, event: Events) -> Vec<Actions> {
        match event {
            Events::NewBlock(block) => self.handle_new_block(block),
            _ => vec![],
        }
    }
}

impl NewBlockStrategy {
    fn handle_new_block(&mut self, data: NewBlock) -> Vec<Actions> {
        if data.number.as_u64() > self.last_height {
            self.last_height = data.number.as_u64();
            vec![Actions::DryRun(format!("New block: {:?}", data))]
        } else {
            vec![]
        }
    }
}
