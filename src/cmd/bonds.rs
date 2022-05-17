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

#![allow(unused)]

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anyhow::Result;
use clap::Subcommand;
use jet_bonds::{accounts, instruction};

use crate::config::{Config, Overrides};
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
