use crate::web3::{
    self,
    futures::{Future, Stream},
    transports::Http,
    types::{Block, BlockId, Transaction},
    Web3,
};
use futures::sync::mpsc::{self, UnboundedReceiver};
use std::{sync::Arc, time::Duration};

#[derive(DebugStub)]
pub struct EthereumWeb3BlockPoller {}

impl EthereumWeb3BlockPoller {
    pub fn create(
        client: Arc<Web3<Http>>,
        polling_wait_time: Duration,
    ) -> Result<UnboundedReceiver<Block<Transaction>>, web3::Error> {
        let filter = client.eth_filter();
        let filter = filter.create_blocks_filter().wait()?;

        info!(
            "Starting listener for Ethereum from block {} waiting for new blocks.",
            client
                .eth()
                .block_number()
                .wait()
                .expect("Could not get block height from web3 client")
        );

        let (state_sender, state_receiver) = mpsc::unbounded();

        let _ = filter
            .stream(polling_wait_time)
            .and_then(|block_hash| client.eth().block_with_txs(BlockId::from(block_hash)))
            .filter(Option::is_some)
            .map(Option::unwrap)
            .for_each(|block| {
                let _ = state_sender.unbounded_send(block);
                Ok(())
            });

        Ok(state_receiver)
    }
}
