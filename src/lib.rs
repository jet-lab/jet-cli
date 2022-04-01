use anyhow::Result;
use clap::{AppSettings, Parser};

mod cmd;
mod config;
mod macros;
mod terminal;

use cmd::*;
use config::ConfigOverride;

/// The top level command line options parser for the binary.
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
        Command::Staking { subcmd } => staking::entry(&opts.cfg, &subcmd),
    }
}
