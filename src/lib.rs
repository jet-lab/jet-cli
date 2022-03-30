use anyhow::Result;
use clap::{AppSettings, Parser};

mod config;
mod staking;

use config::ConfigOverride;

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
