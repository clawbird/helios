use std::{path::PathBuf, str::FromStr};


use env_logger::Env;
use ethers::{types::Address, utils};
use dirs::home_dir;
use eyre::Result;
use helios::{config::networks::Network, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let untrusted_execution_rpc_url = std::env::var("GOERLI_EXECUTION_RPC")?;
    log::info!("Using untrusted RPC URL [REDACTED]");

    let consensus_rpc_url = std::env::var("GOERLI_CONSENSUS_RPC")?;
    log::info!("Using consensus RPC URL: {}", consensus_rpc_url);

    let goerli_data_dir_ext = std::env::var("GOERLI_DATA_DIR_EXT")?;
    let data_path = home_dir().unwrap().join(goerli_data_dir_ext);

    let str_addr: &str = "0xe7cf7C3BA875Dd3884Ed6a9082d342cb4FBb1f1b";
    let target_address: Address = str_addr.parse().unwrap();
    let target_addresses: Vec<Address> = vec![target_address];

    let mut client: Client = ClientBuilder::new()
        .network(Network::GOERLI)
        .consensus_rpc(&consensus_rpc_url)
        .execution_rpc(&untrusted_execution_rpc_url)
        .target_addresses(target_addresses)
        // .load_external_fallback()
        .data_dir(PathBuf::from(data_path))
        .build()?;

    log::info!(
        "Built client on network \"{}\" with external checkpoint fallbacks",
        Network::GOERLI
    );

    client.start().await?;
    log::info!("syncing status: {:#?}", client.syncing().await?);

    let head_block_num = client.get_block_number().await?;
    // note: 0xe7cf7C3BA875Dd3884Ed6a9082d342cb4FBb1f1b is a random ethereum address that i found
    let addr = Address::from_str("0xe7cf7C3BA875Dd3884Ed6a9082d342cb4FBb1f1b")?;
    let block = BlockTag::Latest;
    // RPC get_block_by_number
    let balance = client.get_balance(&addr, block).await?;
    log::info!("block {}", block);
    log::info!("synced up to block: {}", head_block_num);
    log::info!(
        "balance of address: {}",
        utils::format_ether(balance)
    );
    


//     let addresses = block
//     .transactions
//     .iter()
//     .map(|tx| {
//         let from = tx.from;
//         let to = tx.to;
//         vec![from, to.unwrap_or_default()]
//     })
//     .flatten()
//     .collect::<Vec<_>>();

// println!(
//         "Addresses: {:?}, State root: {:?}",
//         addresses, block.state_root
//     );

//sogol added:
// Fetch the block data, including the state root
// let block_number_to_fetch = 12345; // Replace with the desired block number
// let block_data = fetch_block_data(&provider_url, block_number_to_fetch).await?;

// // Iterate through accounts and fetch their state roots
// for account_address in get_all_accounts(&block_data.state_root) {
//     let state_root = fetch_state_root(&block_data.state_root, &account_address).await?;
//     // Process or store the state root as needed
//     println!("Account: {:?}, State Root: {:?}", account_address, state_root);
// }

    Ok(())
}
