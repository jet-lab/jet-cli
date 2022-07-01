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

use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use clap::{AppSettings, Parser};

mod cmd;
mod config;
mod macros;
mod program;
mod pubkey;
mod terminal;

use cmd::*;
use config::Overrides;

/// Jet Protocol command line interface for interacting
/// with the various programs.
#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Opts {
    #[clap(flatten)]
    cfg: Overrides,
    #[clap(subcommand)]
    command: Command,
}

/// The parser for the first level of commands that should
/// be based on the Jet Protocol programs that a user can
/// interact with via the command line tool.
#[derive(Debug, Parser)]
enum Command {
    /// jet_rewards program commands for airdrops.
    Airdrop {
        /// Override of the `jet_rewards` program ID.
        #[clap(global = true, long, value_parser, default_value_t = jet_rewards::ID)]
        program: Pubkey,
        #[clap(subcommand)]
        subcmd: airdrop::AirdropCommand,
    },
    /// jet_auth program commands.
    Auth {
        /// Override of the `jet_auth` program ID.
        #[clap(global = true, long, value_parser, default_value_t = jet_auth::ID)]
        program: Pubkey,
        #[clap(subcommand)]
        subcmd: auth::AuthCommand,
    },
    /// jet_margin program commands.
    Margin {
        /// Override of the `jet_margin` program ID.
        #[clap(global = true, long, value_parser, default_value_t = jet_margin::ID)]
        program: Pubkey,
        #[clap(subcommand)]
        subcmd: margin::MarginCommand,
    },
    /// jet_margin_pool program commands.
    MarginPool {
        /// Override of the `jet_margin_pool` program ID.
        #[clap(global = true, long, value_parser, default_value_t = jet_margin_pool::ID)]
        program: Pubkey,
        #[clap(subcommand)]
        subcmd: margin_pool::MarginPoolCommand,
    },
    /// jet_staking program commands.
    Staking {
        /// Override of the `jet_staking` program ID.
        #[clap(global = true, long, value_parser, default_value_t = jet_staking::ID)]
        program: Pubkey,
        #[clap(subcommand)]
        subcmd: staking::StakingCommand,
    },
}

/// Main handler function to root parse all commands and delegate
/// to the appropriate subcommand entrypoints.
pub fn run(opts: Opts) -> Result<()> {
    match opts.command {
        Command::Airdrop { program, subcmd } => airdrop::entry(&opts.cfg, &program, &subcmd),
        Command::Auth { program, subcmd } => auth::entry(&opts.cfg, &program, &subcmd),
        Command::Margin { program, subcmd } => margin::entry(&opts.cfg, &program, &subcmd),
        Command::MarginPool { program, subcmd } => margin_pool::entry(&opts.cfg, &program, &subcmd),
        Command::Staking { program, subcmd } => staking::entry(&opts.cfg, &program, &subcmd),
    }
}
