use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::Cluster;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::rc::Rc;

/// The struct definition of the available global command
/// options that can be used to override or set standard behavior.
#[derive(Debug, Parser)]
pub struct ConfigOverride {
    /// Auto-approve the signing and execution of the command transaction(s).
    #[clap(global = true, long)]
    auto_approve: bool,
    /// Override of the path to the keypair to be used as signer.
    #[clap(
        global = true,
        long = "keypair",
        default_value = "~/.config/solana/id.json"
    )]
    keypair_path: String,
    /// Override of the cluster to use.
    #[clap(global = true, short = 'u', long, default_value_t = Cluster::Localnet)]
    url: Cluster,
    /// Enables logging verbosity for things like transaction signatures.
    #[clap(global = true, short = 'v', long)]
    verbose: bool,
}

impl ConfigOverride {
    /// Converts the provided and default command line global options
    /// into the standard configuration for the executed command.
    pub fn transform(&self, program_id: Pubkey) -> Result<Config> {
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
            auto_approved: self.auto_approve,
            cluster: self.url.clone(),
            keypair: Rc::new(keypair),
            keypair_path: normalized_path,
            program_id,
            verbose: self.verbose,
        })
    }
}

/// Default implementation for the `ConfigOverride` struct purposed for
/// quickly instantiating during the cargo test executions.
#[cfg(test)]
impl Default for ConfigOverride {
    fn default() -> Self {
        Self {
            auto_approve: false,
            keypair_path: "~/.config/solana/id.json".into(),
            url: Cluster::Devnet,
            verbose: false,
        }
    }
}

/// The struct definitions of the options that are transformed
/// by the global CLI overrides for all commands.
#[derive(Debug)]
pub struct Config {
    pub auto_approved: bool,
    pub cluster: Cluster,
    pub keypair: Rc<Keypair>,
    pub keypair_path: PathBuf,
    pub program_id: Pubkey,
    pub verbose: bool,
}

impl Config {
    /// Create a new instance of `Config` from another with a different program ID.
    pub fn from_with_program(other: &Config, program_id: Pubkey) -> Self {
        Self {
            auto_approved: other.auto_approved,
            cluster: other.cluster.clone(),
            keypair: other.keypair.clone(),
            keypair_path: other.keypair_path.clone(),
            program_id,
            verbose: other.verbose,
        }
    }
}

/// Default implementation for the `Config` struct purposed for
/// quickly instantiating during the cargo test executions.
#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            auto_approved: bool::default(),
            cluster: Cluster::default(),
            keypair: Rc::new(Keypair::new()),
            keypair_path: PathBuf::default(),
            program_id: Pubkey::default(),
            verbose: bool::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cluster;
    use super::ConfigOverride;
    use anchor_client::solana_sdk::pubkey::Pubkey;

    #[test]
    fn cfg_transforms_tilde() {
        let cfg = ConfigOverride {
            auto_approve: false,
            keypair_path: "~/.config/solana/id.json".into(),
            url: Cluster::Devnet,
            verbose: false,
        }
        .transform(Pubkey::default())
        .unwrap();

        assert!(!cfg.keypair_path.starts_with("~"));
    }

    #[test]
    fn cfg_persists_cluster() {
        let cfg = ConfigOverride {
            auto_approve: false,
            keypair_path: "~/.config/solana/id.json".into(),
            url: Cluster::Mainnet,
            verbose: false,
        }
        .transform(Pubkey::default())
        .unwrap();

        assert_eq!(cfg.cluster, Cluster::Mainnet);
    }

    #[test]
    fn cfg_read_keypair_bytes() {
        let cfg = ConfigOverride {
            auto_approve: false,
            keypair_path: "~/.config/solana/id.json".into(),
            url: Cluster::Devnet,
            verbose: false,
        }
        .transform(Pubkey::default())
        .unwrap();

        assert!(cfg.keypair.to_base58_string().len() >= 32);
    }
}
