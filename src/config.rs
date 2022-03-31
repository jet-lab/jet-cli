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
    pub keypair_path: PathBuf,
    pub verbose: bool,
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
    #[clap(global = true, short = 'v', long)]
    verbose: bool,
}

impl ConfigOverride {
    pub fn transform(&self) -> Result<Config> {
        let normalized_path = if self.keypair_path.starts_with('~') {
            PathBuf::from(shellexpand::tilde(&self.keypair_path).to_string())
        } else {
            PathBuf::from(&self.keypair_path)
        };

        if !normalized_path.exists() {
            return Err(anyhow!("Provided keypair path was invalid"));
        }

        let data = read_to_string(&normalized_path)?;
        let bytes = serde_json::from_str::<Vec<u8>>(&data)?;
        let keypair = Keypair::from_bytes(&bytes)?;

        Ok(Config {
            cluster: self.url.clone(),
            keypair,
            keypair_path: normalized_path,
            verbose: self.verbose,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Cluster;
    use super::ConfigOverride;

    #[test]
    fn cfg_transforms_tilde() {
        let cfg = ConfigOverride {
            keypair_path: "~/.config/solana/id.json".into(),
            url: Cluster::Devnet,
            verbose: false,
        }
        .transform()
        .unwrap();

        assert!(!cfg.keypair_path.starts_with("~"));
    }

    #[test]
    fn cfg_persists_cluster() {
        let cfg = ConfigOverride {
            keypair_path: "~/.config/solana/id.json".into(),
            url: Cluster::Mainnet,
            verbose: false,
        }
        .transform()
        .unwrap();

        assert_eq!(cfg.cluster, Cluster::Mainnet);
    }

    #[test]
    fn cfg_read_keypair_bytes() {
        let cfg = ConfigOverride {
            keypair_path: "~/.config/solana/id.json".into(),
            url: Cluster::Devnet,
            verbose: false,
        }
        .transform()
        .unwrap();

        assert!(cfg.keypair.to_base58_string().len() >= 32);
    }
}
