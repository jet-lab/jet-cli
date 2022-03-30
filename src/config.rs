use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::Cluster;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub cluster: Cluster,
    pub keypair: Keypair,
}

#[derive(Debug, Parser)]
pub struct ConfigOverride {
    #[clap(
        global = true,
        short = 'k',
        long = "keypair",
        default_value = "~/.config/solana/id.json"
    )]
    keypair_path: String,
    #[clap(global = true, short = 'u', long, default_value_t = Cluster::Devnet)]
    url: Cluster,
}

impl ConfigOverride {
    pub fn transform(&self) -> Result<Config> {
        let normalized_path = if self.keypair_path.starts_with("~") {
            PathBuf::from(shellexpand::tilde(&self.keypair_path).to_string())
        } else {
            PathBuf::from(&self.keypair_path)
        };

        if !normalized_path.exists() {
            return Err(anyhow!("Provided keypair path was invalid"));
        }

        let data = read_to_string(normalized_path)?;
        let bytes = serde_json::from_str::<Vec<u8>>(&data)?;
        let keypair = Keypair::from_bytes(&bytes)?;

        Ok(Config {
            cluster: self.url.clone(),
            keypair,
        })
    }
}
