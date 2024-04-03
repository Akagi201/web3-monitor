use async_trait::async_trait;
use ethers::types::{Address, Transaction};
use eyre::Result;

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
        let target_address: Address = "0x28C6c06298d514Db089934071355E5743bf21d60"
            .parse()
            .unwrap(); // Binance hot wallet
        if data.from == target_address {
            vec![Actions::DryRun(format!("New mempool tx: {:?}", data))]
        } else {
            vec![]
        }
    }
}
