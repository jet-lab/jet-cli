use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use clap::Subcommand;

use crate::config::ConfigOverride;

/// Rewards program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum RewardsCommand {}

/// The main entry point and handler for all rewards
/// program interaction commands.
pub fn entry(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    _subcmd: &RewardsCommand,
) -> Result<()> {
    let _cfg = overrides.transform(*program_id)?;

    // TODO:

    unimplemented!()
}
