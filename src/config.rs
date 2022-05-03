use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::Cluster;
use anyhow::{anyhow, Result};
use clap::{Parser, ValueHint};
use solana_cli_config::Config as SolanaConfig;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::rc::Rc;

/// The struct definition of the available global command
/// options that can be used to override or set standard behavior.
#[derive(Debug, Parser)]
pub struct Overrides {
    /// Auto-approve the signing and execution of the command transaction(s).
    #[clap(global = true, long)]
    auto_approve: bool,
    #[clap(global = true, long)]
    commitment: Option<CommitmentConfig>,
    /// Override of the path to the keypair to be used as signer.
    #[clap(
        global = true,
        long,
        value_hint = ValueHint::FilePath,
    )]
    keypair: Option<String>,
    /// Override of the cluster to use.
    #[clap(global = true, short = 'u', long)]
    url: Option<Cluster>,
    /// Enables logging verbosity for things like transaction signatures.
    #[clap(global = true, short = 'v', long)]
    verbose: bool,
}

/// Default implementation for the `Overrides` struct purposed for
/// quickly instantiating during the cargo test executions.
#[cfg(test)]
impl Default for Overrides {
    fn default() -> Self {
        Self {
            auto_approve: false,
            commitment: Some(CommitmentConfig::confirmed()),
            keypair: Some("~/.config/solana/id.json".into()),
            url: Some(Cluster::Devnet),
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
    pub program_id: Pubkey,
    pub verbose: bool,
}

impl Config {
    /// Create a new `Config` instance that processes appropriate file-based configurations
    /// and then applies any argument provided overrides discovered from the command.
    pub fn new(overrides: &Overrides, program_id: Pubkey) -> Result<Self> {
        let sol_cfg: SolanaConfig =
            SolanaConfig::load(solana_cli_config::CONFIG_FILE.as_ref().unwrap())?;

        let n_keypair = normalize_path_arg(
            "--keypair",
            overrides.keypair.as_ref().unwrap_or(&sol_cfg.keypair_path),
        )?;

        let keypair = {
            let data = read_to_string(&n_keypair)?;
            let bytes: Vec<u8> = serde_json::from_str(&data)?;
            Keypair::from_bytes(&bytes)
        }?;

        let cluster = overrides
            .url
            .as_ref()
            .unwrap_or(&Cluster::Custom(
                sol_cfg.json_rpc_url,
                sol_cfg.websocket_url,
            ))
            .to_owned();

        Ok(Self {
            auto_approved: overrides.auto_approve,
            cluster,
            keypair: Rc::new(keypair),
            program_id,
            verbose: overrides.verbose,
        })
    }

    /// Create a new instance of `Config` from another with a different program ID.
    pub fn clone_with_program(&self, program_id: Pubkey) -> Self {
        Self {
            auto_approved: self.auto_approved,
            cluster: self.cluster.clone(),
            keypair: self.keypair.clone(),
            program_id,
            verbose: self.verbose,
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
            program_id: Pubkey::default(),
            verbose: bool::default(),
        }
    }
}

/// Normalizes the argued filepath based string into a fully-qualified system path.
fn normalize_path_arg(name: &str, val: &str) -> Result<PathBuf> {
    let normalized = if val.starts_with('~') {
        PathBuf::from(shellexpand::tilde(&val).to_string())
    } else {
        PathBuf::from(&val)
    };

    if !normalized.exists() {
        return Err(anyhow!("provided file path for `{}` was invalid", name));
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::{Cluster, Config, Overrides};
    use anchor_client::solana_sdk::pubkey::Pubkey;

    #[test]
    fn cfg_persists_cluster() {
        let cfg = Config::new(&Overrides::default(), Pubkey::default()).unwrap();
        assert_eq!(cfg.cluster, Cluster::Devnet);
    }

    #[test]
    fn cfg_read_keypair_bytes() {
        let cfg = Config::new(&Overrides::default(), Pubkey::default()).unwrap();
        assert!(cfg.keypair.to_base58_string().len() >= 32);
    }
}
