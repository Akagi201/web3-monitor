use anyhow::Result;
use async_trait::async_trait;
use ethers::types::Transaction;

use crate::types::{Actions, Events, Strategy};

#[derive(Default, Debug)]
pub struct NewMempoolTxStrategy {
    pub last_height: u64,
}

#[async_trait]
impl Strategy<Events, Actions> for NewMempoolTxStrategy {
    async fn sync_state(&mut self) -> Result<()> {
        Ok(())
    }
    async fn process_event(&mut self, event: Events) -> Vec<Actions> {
        match event {
            Events::Transaction(tx) => self.handle_new_tx(tx),
            _ => vec![],
        }
    }
}

impl NewMempoolTxStrategy {
    fn handle_new_tx(&mut self, data: Transaction) -> Vec<Actions> {
        if let Some(block_number) = data.block_number {
            if block_number.as_u64() > self.last_height {
                self.last_height = block_number.as_u64();
                return vec![Actions::DryRun(format!("New mempool tx: {:?}", data))];
            }
        }
        vec![]
    }
}