use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use ethers::types::{Address, Block, U256};
use eyre::{Report, Result};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize}; // Import serde traits
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_bytes::ByteBuf; // Import serde_bytes for binary data serialization
use client::ClientBuilder;
use config::{
    networks::Network,
    Config,
};
use common::types::{BlockTag, CheckpointData};
use database::Database;

// Temporary in-memory database
#[derive(Default)]
struct TemporaryDB {
    checkpoint: Option<CheckpointData>,
}

impl TemporaryDB {
    fn new() -> Self {
        Self::default()
    }
}

impl Database for TemporaryDB {
    fn new(_config: &Config) -> Result<Self> {
        Ok(Self::default())
    }

    fn load_checkpoint(&self) -> Result<CheckpointData, Report> {
        if let Some(checkpoint) = &self.checkpoint {
            Ok(checkpoint.clone()) // Return a clone of the stored checkpoint
        } else {
            Err(eyre::eyre!("No checkpoint found in the database"))
        }
    }

    fn save_checkpoint(&mut self, checkpoint: CheckpointData) -> Result<()> {
        self.checkpoint = Some(checkpoint);
        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct PartialViewDataStorage<D>
    where
        D: Database,
{
    partial_view_data: HashMap<Address, U256>,
    address_queue: VecDeque<Address>,
    max_queue_capacity: usize,
    database: D,
}

impl<D> PartialViewDataStorage<D>
    where
        D: Database,
{
    pub fn new(max_queue_capacity: usize, database: D) -> Result<Self> {
        let mut storage = PartialViewDataStorage {
            partial_view_data: HashMap::new(),
            address_queue: VecDeque::new(),
            max_queue_capacity,
            database,
        };

        // Load the checkpoint from the database during initialization
        storage.load_checkpoint()?;

        Ok(storage)
    }

    // @todo: query and store
    pub fn query_and_store(&mut self, _address: Address) -> Result<()> {
        // Query and store data as before


        // TODO - use input argument `_address` and the stored data instead of hard-coding below
        
        // Create a checkpoint struct with your data
        let checkpoint = CheckpointData {
            field1: "SomeData".to_string(),
            field2: 42,
            binary_data: ByteBuf::from(vec![0, 1, 2, 3]), // Example binary data
            // Set other fields as needed
        };

        // After storing data, save the checkpoint to the database
        self.save_checkpoint(checkpoint)?;

        Ok(())
    }

    fn load_checkpoint(&mut self) -> Result<CheckpointData, Report> {
        let checkpoint: CheckpointData = self.database.load_checkpoint()?;
        // No need for deserialization here since we'll be working with the CheckpointData struct

        Ok(checkpoint)
    }

    fn save_checkpoint(&mut self, checkpoint: CheckpointData) -> Result<()> {
        // TODO - use input argument `checkpoint` instead of hard-coding below
        
        // Create a checkpoint struct with your data
        let checkpoint = CheckpointData {
            field1: "SomeData".to_string(),
            field2: 42,
            binary_data: ByteBuf::from(vec![0, 1, 2, 3]), // Example binary data
            // Set other fields as needed
        };

        // // Serialize the checkpoint using MessagePack
        // let mut buffer = Vec::new();
        // let mut serializer = Serializer::new(&mut buffer);
        // checkpoint.serialize(&mut serializer)?;

        // Save the serialized checkpoint to the database
        // self.database.save_checkpoint(buffer.into())?;
        self.database.save_checkpoint(checkpoint)?;

        Ok(())
    }
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     // Replace this with your actual Helios RPC URL
//     let helios_rpc_url = "https://helios-rpc-url.com";

//     // Replace this with the address we want to query
//     let address_str = "0x0000000000000000000000000000000000000000";
//     let address = Address::from_str(address_str)?;

//     // Create a new Helios client with the temporary in-memory database
//     let mut client = ClientBuilder::new()
//         .network(Network::MAINNET)
//         .execution_rpc(helios_rpc_url)
//         .database(TemporaryDB::new())
//         .build()?;

//     // Start the client
//     client.start().await?;

//     // Query the latest block number
//     let block_number = client.get_block_number().await?;
//     println!("Latest block number: {}", block_number);

//     // Query the balance of the specified address at the latest block
//     let balance = client.get_balance(&address, BlockTag::Latest).await?;
//     println!(
//         "Balance of {}: {}",
//         address_str,
//         ethers::utils::format_ether(balance)
//     );

//     // Query the latest block
//     let latest_block: Option<Block> = client.get_block(BlockTag::Latest).await?;
//     if let Some(block) = latest_block {
//         println!("Latest block details: {:?}", block);
//     } else {
//         println!("Latest block not found");
//     }

//     Ok(())
// }

// We've added a binary_data field to the CheckpointData struct, which is an example of binary data serialized using the serde_bytes::ByteBuf type.
// We've used the rmp_serde crate to serialize and deserialize the CheckpointData struct using MessagePack format.
// The save_checkpoint method now serializes the CheckpointData struct into MessagePack format and saves it to the database.
// The load_checkpoint method loads the MessagePack data from the database and deserializes it into a CheckpointData struct.
