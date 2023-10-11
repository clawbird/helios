use std::net::IpAddr;
use std::{
    path::PathBuf,
    process::exit,
    str::FromStr,
    sync::{Arc, Mutex},
};

use clap::Parser;
use common::utils::hex_str_to_bytes;
use dirs::home_dir;
use eyre::{eyre, Result};
use ethers::{
    core::types::{Block, BlockId, Transaction, TransactionReceipt, H256, Address},
    providers::{Http, Middleware, Provider},
    signers::Wallet,
    // trie::{MerklePatriciaTrie, Trie},
};
use futures::executor::block_on;
use tracing::{error, info};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::FmtSubscriber;

use client::{Client, ClientBuilder};
use config::{CliConfig, Config};

use partial_view::PartialViewDataStorage;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .expect("invalid env filter");

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("subsriber set failed");

    let config = get_config();

    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
//sogol addded: 
    // Initialize the Ethereum provider URL and address public key from environment variables
    // let provider_url = std::env::var("PROVIDER_URI").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
    // let public_key = std::env::var("ETHEREUM_ADDRESS_PUBLIC_KEY").expect("ETHEREUM_ADDRESS_PUBLIC_KEY not set");
    // Initialize the Ethereum provider URL from environment variable or use default
    // let provider = Provider::<Http>::try_from(provider_url)?;

    // let block_number = 9751182; // Replace with the desired block number

    // let example_str_addr: &str = "0xe7cf7C3BA875Dd3884Ed6a9082d342cb4FBb1f1b";
    // let example_addr: Address = example_str_addr.parse().unwrap();

    // // Fetch all transactions within the specified block
    // let transactions = fetch_all_transactions(&provider, block_number).await?;

    // let addresses = transactions.iter()
    //     .filter_map(|tx| {
    //         let from = tx.from;
    //         let to = tx.to.unwrap_or_default();
    //         if to.is_contract() {
    //             Some(to)
    //         } else {
    //             Some(from)
    //         }
    //     })
    //     .collect::<Vec<_>>();
    // println!(
    //     "Addresses: {:?}, Number of Transactions: {}",
    //     addresses,
    //     transactions.len()
    // );

    // let provider = Provider::<Http>::try_from(
    //     provider_url,
    // )?;

    // let block_number = 9751182;
    // let block = provider
    //     .get_block_with_txs(BlockId::Number(block_number.into()))
    //     .await?;

    // let block = match block {
    //     Some(block) => block,
    //     None => return Err(eyre!("Block not found")),
    // };

    // let addresses = block
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
    //     "Addresses: {:?}, State root: {:?}",
    //     addresses, block.state_root
    // );
   
    // TODO - we shouldnt need this any, as we pass the addresses as optional flags in the cli
    // Define your target addresses here
    // let target_addresses = vec![
    //     Address::from_str("0xYourTargetAddress1").unwrap(),
    //     Address::from_str("0xYourTargetAddress2").unwrap(),
    // ];

    // Create the Helios client with the specified target addresses
    let mut client = match ClientBuilder::new()
        .config(config)
        // .target_addresses(target_addresses.clone()) // Pass target addresses here
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            error!(target: "helios::runner", error = %err);
            exit(1);
        }
    };

    if let Err(err) = client.start().await {
        error!(target: "helios::runner", error = %err);
        exit(1);
    }

    register_shutdown_handler(client);
    std::future::pending().await
}

fn register_shutdown_handler(client: Client) {
    let client = Arc::new(client);
    let shutdown_counter = Arc::new(Mutex::new(0));

    ctrlc::set_handler(move || {
        let mut counter = shutdown_counter.lock().unwrap();
        *counter += 1;

        let counter_value = *counter;

        if counter_value == 3 {
            info!(target: "helios::runner", "forced shutdown");
            exit(0);
        }

        info!(
            target: "helios::runner",
            "shutting down... press ctrl-c {} more times to force quit",
            3 - counter_value
        );

        if counter_value == 1 {
            let client = client.clone();
            std::thread::spawn(move || {
                block_on(client.shutdown());
                exit(0);
            });
        }
    })
    .expect("could not register shutdown handler");
}

fn get_config() -> Config {
    let cli = Cli::parse();

    let config_path = home_dir().unwrap().join(".helios/helios.toml");

    let cli_config = cli.as_cli_config();

    Config::from_file(&config_path, &cli.network, &cli_config)
}

#[derive(Parser)]
#[clap(version, about)]
/// Helios is a fast, secure, and portable light client for Ethereum
struct Cli {
    #[clap(short, long, default_value = "mainnet")]
    network: String,
    #[clap(short = 'b', long, env)]
    rpc_bind_ip: Option<IpAddr>,
    #[clap(short = 'p', long, env)]
    rpc_port: Option<u16>,
    #[clap(short = 'w', long, env)]
    checkpoint: Option<String>,
    #[clap(short, long, env)]
    execution_rpc: Option<String>,
    #[clap(short, long, env)]
    consensus_rpc: Option<String>,
    #[clap(short, long, env)]
    data_dir: Option<String>,
    #[clap(short = 'f', long, env)]
    fallback: Option<String>,
    #[clap(short = 'l', long, env)]
    load_external_fallback: bool,
    #[clap(short = 's', long, env)]
    strict_checkpoint_age: bool,
    #[clap(short = 'a', long, env)]
    target_addresses: Option<Vec<String>>,
}

impl Cli {
    fn as_cli_config(&self) -> CliConfig {
        let checkpoint = self
            .checkpoint
            .as_ref()
            .map(|c| hex_str_to_bytes(c).expect("invalid checkpoint"));

        let target_addresses: Option<String> = self.target_addresses.as_ref().map(|addresses| {
            addresses.iter().map(|address| address.to_string()).collect()
        });

        CliConfig {
            checkpoint,
            execution_rpc: self.execution_rpc.clone(),
            consensus_rpc: self.consensus_rpc.clone(),
            data_dir: self.get_data_dir(),
            rpc_bind_ip: self.rpc_bind_ip,
            rpc_port: self.rpc_port,
            fallback: self.fallback.clone(),
            load_external_fallback: self.load_external_fallback,
            strict_checkpoint_age: self.strict_checkpoint_age,
            target_addresses: self.target_addresses.clone(),
        }
    }

    fn get_data_dir(&self) -> PathBuf {
        if let Some(dir) = &self.data_dir {
            PathBuf::from_str(dir).expect("cannot find data dir")
        } else {
            home_dir()
                .unwrap()
                .join(format!(".helios/data/{}", self.network))
        }
    }
}
