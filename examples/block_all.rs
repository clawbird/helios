use std::error::Error;
use std::process;
use std::collections::HashSet;
use ethers::{
    core::types::{Block, BlockId, Transaction, TransactionReceipt, H256, Address},
    providers::{Http, Middleware, Provider},
    signers::Wallet,
    // trie::{MerklePatriciaTrie, Trie},
};
use env_logger::Env;

async fn fetch_block_data(provider: Provider<Http>, block_number: BlockId) -> Block<H256> {
    let block;
    match provider.get_block(block_number).await {
        Ok(x) => {
            if let Some(_block) = x {
                block = _block;
            } else {
                log::debug!("Received empty block");
                process::exit(1);
            }
        },
        Err(e) => {
            log::debug!("Unable to get block {:#?}", e);
            process::exit(1);
        },
    }
    // println!("block {:?}", block);

    return block;
}

async fn fetch_all_transactions(provider: Provider<Http>, block_number: BlockId) -> Vec<Transaction> {
    let block_with_txs;
    match provider.get_block_with_txs(block_number).await {
        Ok(x) => {
            if let Some(_block_with_txs) = x {
                block_with_txs = _block_with_txs;
            } else {
                log::debug!("Received empty block with transactions");
                process::exit(1);
            }
        },
        Err(e) => {
            log::debug!("Unable to get block with transactions {:#?}", e);
            process::exit(1);
        },
    }

    let transactions: Vec<Transaction> = block_with_txs.transactions;

    return transactions;
}

fn dedup(vs: &Vec<Address>) -> Vec<Address> {
    let hs = vs.iter().cloned().collect::<HashSet<Address>>();

    hs.into_iter().collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let provider_url = std::env::var("GOERLI_EXECUTION_RPC").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
    let provider = Provider::<Http>::try_from(provider_url)?;
    // log::debug!("provider {:#?}", provider);
    let block_number = 9751183.into(); // Replace with the desired block number
    // Fetch all transactions within the specified block
    let transactions = fetch_all_transactions(provider.clone(), block_number).await;
    log::info!("transactions {:#?}", transactions);

    // Fetch the block data, including the state root
    let block = fetch_block_data(provider.clone(), block_number).await;
    log::info!("block state root {:#?}", block.state_root);

    let addresses = transactions
        .iter()
        .map(|tx| {
            let from = tx.from;
            let to = tx.to;
            vec![from, to.unwrap_or_default()]
        })
        .flatten()
        .collect::<Vec<_>>();
    // Remove duplicate addresses
    let addresses_deduped = dedup(&addresses);
    log::info!("addresses {:#?}", addresses_deduped);

    Ok(())
}