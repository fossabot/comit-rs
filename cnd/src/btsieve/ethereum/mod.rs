mod transaction_pattern;
mod web3_connector;

pub use self::{
    transaction_pattern::{Event, Topic, TransactionPattern},
    web3_connector::Web3Connector,
};
use crate::{
    btsieve::{BlockByHash, LatestBlock, Predates, ReceiptByHash},
    ethereum::{Transaction, TransactionAndReceipt, TransactionReceipt, H256, U256},
};
use anyhow;
use futures_core::compat::Future01CompatExt;
use genawaiter::{
    sync::{Co, Gen},
    GeneratorState,
};
use std::collections::HashSet;

type Hash = H256;
type Block = crate::ethereum::Block<Transaction>;

pub async fn matching_transaction<C>(
    connector: C,
    pattern: TransactionPattern,
    start_of_swap: Option<u32>,
) -> anyhow::Result<TransactionAndReceipt>
where
    C: LatestBlock<Block = Option<Block>>
        + BlockByHash<Block = Option<Block>, BlockHash = Hash>
        + ReceiptByHash<Receipt = Option<TransactionReceipt>, TransactionHash = Hash>
        + Clone,
{
    let mut block_generator = Gen::new({
        let connector = connector.clone();
        |co| async move { find_relevant_blocks(connector, &co, start_of_swap).await }
    });

    loop {
        match block_generator.async_resume().await {
            GeneratorState::Yielded(block) => {
                if let Some(transaction_and_receipt) =
                    check_block_against_pattern(connector.clone(), block, pattern.clone()).await?
                {
                    return Ok(transaction_and_receipt);
                } else {
                    continue;
                }
            }
            GeneratorState::Complete(Err(e)) => return Err(e),
            GeneratorState::Complete(Ok(infallible)) => match infallible {},
        }
    }
}

/// This function uses the `connector` to find blocks relevant to a swap.
///
/// It yields those blocks as part of the process.
async fn find_relevant_blocks<C>(
    mut connector: C,
    co: &Co<Block>,
    start_of_swap: Option<u32>,
) -> anyhow::Result<std::convert::Infallible>
where
    C: LatestBlock<Block = Option<Block>>
        + BlockByHash<Block = Option<Block>, BlockHash = Hash>
        + ReceiptByHash<Receipt = Option<TransactionReceipt>, TransactionHash = Hash>
        + Clone,
{
    let mut seen_blocks: HashSet<Hash> = HashSet::new();

    loop {
        let block = connector
            .latest_block()
            .compat()
            .await?
            .ok_or_else(|| anyhow::anyhow!("Connector returned nullable latest block"))?;
        co.yield_(block.clone()).await;

        let blockhash = block
            .hash
            .ok_or_else(|| anyhow::anyhow!("Connector returned latest block with nullable hash"))?;
        seen_blocks.insert(blockhash);

        if let Some(start_of_swap) = start_of_swap {
            if seen_blocks.len() == 1 && block.timestamp > U256::from(start_of_swap) {
                walk_back_until(
                    past_timestamp(start_of_swap),
                    connector.clone(),
                    co,
                    blockhash,
                )
                .await?;
            }
        }

        let parent_hash = block.parent_hash;
        if !seen_blocks.contains(&parent_hash) && seen_blocks.len() > 1 {
            walk_back_until(
                seen_block_or_past_seen_block(seen_blocks.clone(), start_of_swap.unwrap_or(0)),
                connector.clone(),
                co,
                parent_hash,
            )
            .await?;
        }

        // The duration of this timeout could/should depend on the network
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
}

/// Walks the blockchain backwards from the given hash until the predicate given
/// in `stop_condition` returns `true`.
///
/// This functions yields all blocks as part of its process.
async fn walk_back_until<C, P>(
    stop_condition: P,
    connector: C,
    co: &Co<Block>,
    starting_blockhash: Hash,
) -> anyhow::Result<()>
where
    C: BlockByHash<Block = Option<Block>, BlockHash = Hash>,
    P: Fn(&Block) -> anyhow::Result<bool>,
{
    let mut blockhash = starting_blockhash;

    loop {
        let block = connector
            .block_by_hash(blockhash)
            .compat()
            .await?
            .ok_or_else(|| anyhow::anyhow!("Could not fetch block with hash {}", blockhash))?;

        co.yield_(block.clone()).await;

        if stop_condition(&block)? {
            return Ok(());
        } else {
            blockhash = block.parent_hash
        }
    }
}

/// Constructs a predicate that returns `true` if the given block predates the
/// start of the swap.
fn past_timestamp(start_of_swap: u32) -> impl Fn(&Block) -> anyhow::Result<bool> {
    move |block| Ok(block.predates(start_of_swap))
}

/// Constructs a predicate that returns `true` if we have seen the given block
/// or the block is younger than the swap itself.
fn seen_block_or_past_seen_block(
    seen_blocks: HashSet<Hash>,
    start_of_swap: u32,
) -> impl Fn(&Block) -> anyhow::Result<bool> {
    move |block: &Block| {
        let have_seen_block = seen_blocks.contains(
            &block
                .hash
                .ok_or_else(|| anyhow::anyhow!("block without hash"))?,
        );
        let past_start_of_swap = past_timestamp(start_of_swap)(block)?;

        Ok(have_seen_block || past_start_of_swap)
    }
}

async fn check_block_against_pattern<C>(
    connector: C,
    block: Block,
    pattern: TransactionPattern,
) -> anyhow::Result<Option<TransactionAndReceipt>>
where
    C: ReceiptByHash<Receipt = Option<TransactionReceipt>, TransactionHash = Hash>,
{
    let needs_receipt = pattern.needs_receipts(&block);
    let block_hash = block
        .hash
        .ok_or_else(|| anyhow::anyhow!("block without hash"))?;

    if needs_receipt {
        log::debug!(
            "bloom-filter of block {:x} suggests to fetch receipts for {:?}",
            block_hash,
            pattern
        );
    } else {
        log::debug!(
            "bloom-filter of block {:x} suggests to not fetch receipts for {:?}",
            block_hash,
            pattern
        );
    }

    for transaction in block.transactions.into_iter() {
        let tx_hash = transaction.hash;

        let receipt = if needs_receipt {
            let receipt = connector
                .receipt_by_hash(tx_hash)
                .compat()
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not get transaction receipt for transaction {:x}",
                        tx_hash
                    )
                })?;

            Some(receipt)
        } else {
            None
        };

        let result = pattern.matches(&transaction, receipt.as_ref());

        log::debug!(
            "matching {:?} against transaction {:x} yielded {}",
            pattern,
            tx_hash,
            result
        );

        if result {
            let receipt = match receipt {
                Some(receipt) => receipt,
                None => connector
                    .receipt_by_hash(tx_hash)
                    .compat()
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Could not get transaction receipt for transaction {:x}",
                            tx_hash
                        )
                    })?,
            };

            return Ok(Some(TransactionAndReceipt {
                transaction,
                receipt,
            }));
        }
    }

    Ok(None)
}

impl Predates for Block {
    fn predates(&self, timestamp: u32) -> bool {
        self.timestamp < U256::from(timestamp)
    }
}
