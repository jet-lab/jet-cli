use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use clap::Subcommand;

use crate::config::ConfigOverride;

/// Margin program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum MarginCommand {}

/// The main entry point and handler for all margin
/// program interaction commands.
pub fn entry(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    _subcmd: &MarginCommand,
) -> Result<()> {
    let _cfg = overrides.transform(*program_id)?;
    unimplemented!();
}
