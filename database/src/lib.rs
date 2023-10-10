use eyre::{Report, Result};

use common::types::CheckpointData;
use config::Config;

pub trait Database {
    fn new(_config: &Config) -> Result<Self> where Self: Sized;

    fn load_checkpoint(&self) -> Result<CheckpointData, Report>;

    fn save_checkpoint(&mut self, checkpoint: CheckpointData) -> Result<()>;
}
