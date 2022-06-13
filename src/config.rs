// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::Cluster;
use anyhow::{anyhow, Result};
use clap::{Parser, PossibleValue, ValueHint};
use solana_cli_config::Config as SolanaConfig;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::rc::Rc;

/// The struct definition of the available global command
/// options that can be used to override or set standard behavior.
#[derive(Debug, Parser)]
pub struct Overrides {
    /// Auto-approve the signing and execution of the command transaction(s).
    #[clap(global = true, long, value_parser)]
    auto_approve: bool,
    /// Override of the commitment level used for the RPC client.
    #[clap(global = true, long, value_parser = [PossibleValue::new("confirmed"), PossibleValue::new("finalized"), PossibleValue::new("processed")])]
    commitment: Option<CommitmentConfig>,
    /// Override of the path to the keypair to be used as signer.
    #[clap(global = true, long, value_parser, value_hint = ValueHint::FilePath)]
    keypair: Option<String>,
    /// Override of the cluster or RPC URL to use (or their first letter): ["mainnet-beta", "devnet", "testnet", "localnet"].
    #[clap(global = true, short = 'u', long, value_parser)]
    url: Option<Cluster>,
    /// Enables logging verbosity for things like transaction signatures.
    #[clap(global = true, short = 'v', long, value_parser)]
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
    use anchor_client::solana_sdk::pubkey::Pubkey;
    use std::path::PathBuf;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn path_normalization() {
        let good_res = normalize_path_arg("--test", "/etc");
        assert!(good_res.is_ok());
        assert_eq!(good_res.unwrap(), PathBuf::from_str("/etc").unwrap());

        let good_tilde_res = normalize_path_arg("--test", "~/.config/solana/id.json");
        assert!(good_tilde_res.is_ok());
        assert!(good_tilde_res.unwrap().starts_with("/"));

        let bad_res = normalize_path_arg("--test", "/does/not/exist");
        assert!(bad_res.is_err());
        assert!(bad_res.unwrap_err().to_string().contains("invalid"));
    }

    #[test]
    fn cfg_clone_with_new_program() {
        let cfg = Config::new(&Overrides::default(), Pubkey::default()).unwrap();
        let new_cfg = cfg.clone_with_program(Pubkey::new_from_array([5; 32]));
        assert_ne!(new_cfg.program_id, Pubkey::default());
    }

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
