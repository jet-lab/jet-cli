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
use config::ConfigOverride;

/// Jet Protocol command line interface for interacting
/// with the various programs.
#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Opts {
    #[clap(flatten)]
    cfg: ConfigOverride,
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
        #[clap(global = true, long, default_value_t = jet_rewards::ID)]
        program: Pubkey,
        #[clap(subcommand)]
        subcmd: airdrop::AirdropCommand,
    },
    /// jet_auth program commands.
    Auth {
        /// Override of the `jet_auth` program ID.
        #[clap(global = true, long, default_value_t = jet_auth::ID)]
        program: Pubkey,
        #[clap(subcommand)]
        subcmd: auth::AuthCommand,
    },
    /// jet_margin program commands.
    // Margin {
    //     /// Override of the `jet_margin` program ID.
    //     #[clap(global = true, long, default_value_t = jet_margin::ID)]
    //     program: Pubkey,
    //     #[clap(subcommand)]
    //     subcmd: margin::MarginCommand,
    // },
    /// jet_staking program commands.
    Staking {
        /// Override of the `jet_staking` program ID.
        #[clap(global = true, long, default_value_t = jet_staking::ID)]
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
        // Command::Margin { program, subcmd } => margin::entry(&opts.cfg, &program, &subcmd),
        Command::Staking { program, subcmd } => staking::entry(&opts.cfg, &program, &subcmd),
    }
}
