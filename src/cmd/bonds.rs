use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anyhow::Result;
use clap::Subcommand;
use jet_bonds::{accounts, instruction, UserAuthentication};

use crate::config::{Config, Overrides};
use crate::macros::*;
use crate::program::{create_program_client, send_with_approval};
use crate::pubkey::derive_auth_account;
use crate::terminal::{print_serialized, DisplayOptions};

/// Bonds program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum BondsCommand {}

/// The main entry point and handler for all bonds
/// program interaction commands.
pub fn entry(overrides: &Overrides, program_id: &Pubkey, subcmd: &BondsCommand) -> Result<()> {
    let _cfg = Config::new(overrides, *program_id)?;
    Ok(())
}
