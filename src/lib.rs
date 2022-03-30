use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::Cluster;
use anyhow::{anyhow, Result};
use clap::{AppSettings, Parser};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

mod staking;

#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    Staking {
        #[clap(subcommand)]
        subcmd: staking::Command,
    },
}

/// Main handler function to root parse all commands and delegate
/// to the appropriate subcommand entrypoints.
pub fn run(opts: Opts) -> Result<()> {
    match opts.command {
        Command::Staking { subcmd } => staking::entry(&subcmd),
    }
}

/// Loads the keypair from the argued path location.
fn load_keypair(path: PathBuf) -> Result<Keypair> {
    if !path.exists() {
        return Err(anyhow!(
            "No keypair found at default location {}",
            path.to_str().unwrap()
        ));
    }

    let key = read_to_string(path)?;
    Keypair::from_bytes(key.as_bytes()).map_err(Into::into)
}

/// Returns the path of the system default keypair path that
/// is typically created by the Solana toolkit when creating
/// your first keypair.
fn default_keypair_path() -> Result<PathBuf> {
    match dirs::home_dir().as_mut() {
        Some(p) => {
            p.push(".config/solana/id.json");
            Ok(p.to_path_buf())
        }
        None => Err(anyhow!(
            "$HOME does not exist and cannot be used to find default keypair."
        )),
    }
}

/// Parse the argued url or shortcode for an RPC
/// cluster endpoint to create a client.
fn parse_cluster(url: &str) -> Result<Cluster> {
    match url {
        "m" | "mainnet-beta" => Ok(Cluster::Mainnet),
        "d" | "devnet" => Ok(Cluster::Devnet),
        "t" | "testnet" => Ok(Cluster::Testnet),
        "l" | "localnet" => Ok(Cluster::Localnet),
        _ => Cluster::from_str(url),
    }
}
